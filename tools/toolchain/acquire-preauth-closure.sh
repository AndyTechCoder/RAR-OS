#!/bin/sh
set -eu

[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || {
    echo "acquisition requires the approved immutable OCI base" >&2
    exit 73
}
[ "${RAR_PREAUTH_ACQUISITION-}" = signed-snapshot-only ] || exit 73
[ "$(id -u)" -eq 0 ] || {
    echo "acquisition runs only inside the disposable OCI container" >&2
    exit 73
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
case "$root" in /workspace | /workspace/*) ;; *) echo "unexpected container workspace" >&2; exit 73 ;; esac
output=$root/out/r0/preauth/acquisition
for path in "$root/out" "$root/out/r0" "$root/out/r0/preauth" "$output"; do
    [ ! -L "$path" ] || { echo "symlink output refused: $path" >&2; exit 2; }
done
mkdir -p "$output/apt-state/lists/partial" "$output/apt-cache/archives/partial" "$output/debs" "$output/licenses" "$output/derived-context/debs"

snapshot=20260630T000000Z
sources=$output/sources.list
cat > "$sources" <<EOF
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$snapshot trixie main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$snapshot trixie-updates main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$snapshot trixie-proposed-updates main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/$snapshot trixie-security main
EOF

apt_options="-o Dir::Etc::sourcelist=$sources -o Dir::Etc::sourceparts=- -o Dir::State=$output/apt-state -o Dir::Cache=$output/apt-cache -o APT::Install-Recommends=false -o Acquire::Check-Valid-Until=false"
# shellcheck disable=SC2086
/usr/bin/apt-get $apt_options update
# apt verifies InRelease signatures and Packages checksums before this download-only transaction.
# shellcheck disable=SC2086
/usr/bin/apt-get $apt_options --yes --download-only --no-install-recommends install \
    'lld-19=1:19.1.7-3+b1' \
    'qemu-system-x86=1:10.0.8+ds-0+deb13u1+b2' \
    'ovmf=2025.02-8+deb13u1'

find "$output/apt-cache/archives" -maxdepth 1 -type f -name '*.deb' -exec cp {} "$output/debs/" \;
deb_count=$(find "$output/debs" -maxdepth 1 -type f -name '*.deb' | wc -l | tr -d ' ')
[ "$deb_count" -ge 3 ] || { echo "incomplete acquired package set" >&2; exit 1; }
if find "$output" -type l | grep -q .; then
    echo "symlink in acquisition output" >&2
    exit 1
fi

package_manifest=$output/packages.v2
: > "$package_manifest"
printf '%s\n' 'schema=rar-preauth-package-manifest-v2' >> "$package_manifest"
license_root=$output/licenses/root
mkdir -p "$license_root"
for deb in "$output"/debs/*.deb; do
    /usr/bin/dpkg-deb -x "$deb" "$license_root"
done
for deb in "$output"/debs/*.deb; do
    before=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$deb")
    sha=$(/usr/bin/sha256sum "$deb" | /usr/bin/cut -d ' ' -f 1)
    name=$(/usr/bin/dpkg-deb -f "$deb" Package)
    version=$(/usr/bin/dpkg-deb -f "$deb" Version)
    architecture=$(/usr/bin/dpkg-deb -f "$deb" Architecture)
    size=$(/usr/bin/stat -c '%s' "$deb")
    copyright=$license_root/usr/share/doc/$name/copyright
    resolved_copyright=$(/usr/bin/realpath -e "$copyright") || { echo "license missing for $name" >&2; exit 1; }
    case "$resolved_copyright" in "$license_root"/usr/share/doc/*) ;; *) echo "license path escape for $name" >&2; exit 1 ;; esac
    [ -f "$resolved_copyright" ] || { echo "license target is not regular for $name" >&2; exit 1; }
    license_sha=$(/usr/bin/sha256sum "$resolved_copyright" | /usr/bin/cut -d ' ' -f 1)
    after=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$deb")
    [ "$before" = "$after" ] || { echo "same-inode mutation detected for $deb" >&2; exit 1; }
    printf 'package|%s|%s|%s|%s|%s|%s|%s\n' \
        "$name" "$version" "$architecture" "$(basename "$deb")" "$size" "$sha" "$license_sha" >> "$package_manifest"
done
LC_ALL=C /usr/bin/sort -o "$package_manifest.sorted" "$package_manifest"
{
    printf '%s\n' 'schema=rar-preauth-package-manifest-v2'
    /usr/bin/grep '^package|' "$package_manifest.sorted"
} > "$package_manifest.canonical"
mv "$package_manifest.canonical" "$package_manifest"

for required in \
    'lld-19|1:19.1.7-3+b1|' \
    'qemu-system-x86|1:10.0.8+ds-0+deb13u1+b2|' \
    'ovmf|2025.02-8+deb13u1|'; do
    /usr/bin/grep -Fq "package|$required" "$package_manifest" || {
        echo "required exact package absent: $required" >&2
        exit 1
    }
done

base_manifest=$output/base-installed.v1
/usr/bin/dpkg-query -W -f='${binary:Package}|${Version}|${Architecture}\n' | LC_ALL=C /usr/bin/sort > "$base_manifest"
lists_manifest=$output/signed-metadata.sha256
find "$output/apt-state/lists" -maxdepth 1 -type f -exec /usr/bin/sha256sum {} \; | LC_ALL=C /usr/bin/sort > "$lists_manifest"
[ "$(/usr/bin/grep -c 'InRelease' "$lists_manifest")" -ge 4 ] || { echo "signed InRelease evidence absent" >&2; exit 1; }

cp "$output"/debs/*.deb "$output/derived-context/debs/"
cat > "$output/derived-context/Dockerfile" <<'EOF'
FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
COPY debs/ /rar-closure/
RUN printf '#!/bin/sh\nexit 101\n' > /usr/sbin/policy-rc.d && chmod 0755 /usr/sbin/policy-rc.d && dpkg -i /rar-closure/*.deb
ENV RAR_PREAUTH_BUILD_CONTAINER=rar-preauth-closure-v2
ENV RAR_TARGET_EXECUTION=prohibited
EOF

{
    printf '%s\n' \
        'schema=rar-preauth-closure-discovery-v2' \
        'base_oci_index_sha256=f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
        "debian_snapshot=$snapshot" \
        'debian_suite=trixie' \
        "package_count=$deb_count" \
        "package_manifest_sha256=$(/usr/bin/sha256sum "$package_manifest" | /usr/bin/cut -d ' ' -f 1)" \
        "base_manifest_sha256=$(/usr/bin/sha256sum "$base_manifest" | /usr/bin/cut -d ' ' -f 1)" \
        "signed_metadata_manifest_sha256=$(/usr/bin/sha256sum "$lists_manifest" | /usr/bin/cut -d ' ' -f 1)" \
        'signature_verification=apt-secure-passed' \
        'target_execution=not-attempted' \
        'qemu_execution=not-attempted' \
        'emulator_execution=not-attempted' \
        'vm_execution=not-attempted'
} > "$output/discovery.evidence"
