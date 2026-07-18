#!/bin/sh
set -eu

[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || {
    echo "acquisition requires the approved immutable OCI base" >&2
    exit 73
}
[ "${RAR_PREAUTH_ACQUISITION-}" = signed-snapshot-only ] || exit 73
case "${RAR_OUTPUT_UID-}:${RAR_OUTPUT_GID-}" in
    *[!0-9:]* | :* | *:) echo "invalid repository output ownership" >&2; exit 73 ;;
esac
[ "$(id -u)" = "$RAR_OUTPUT_UID" ] && [ "$(id -g)" = "$RAR_OUTPUT_GID" ] || {
    echo "acquisition must run as the invoking repository owner" >&2
    exit 73
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
case "$root" in /workspace | /workspace/*) ;; *) echo "unexpected container workspace" >&2; exit 73 ;; esac
output=$root/out/r0/preauth/acquisition
approved_packages=$root/spec/lab/preauth/locks/r0-x86_64-preauth-packages.v2
[ -f "$approved_packages" ] && [ ! -L "$approved_packages" ] || {
    echo "approved package manifest is absent or indirect" >&2
    exit 73
}
for path in "$root/out" "$root/out/r0" "$root/out/r0/preauth"; do
    [ ! -e "$path" ] || { [ -d "$path" ] && [ ! -L "$path" ]; } || {
        echo "unsafe acquisition ancestor: $path" >&2
        exit 73
    }
done
[ -d "$output" ] && [ ! -L "$output" ] || {
    echo "acquisition output must be a direct pre-created directory" >&2
    exit 73
}
for path in "$output" "$output/apt-state" "$output/apt-state/lists" \
    "$output/apt-state/lists/partial" "$output/apt-cache" \
    "$output/apt-cache/archives" "$output/apt-cache/archives/partial" \
    "$output/debs" "$output/licenses" "$output/derived-context" \
    "$output/derived-context/rootfs" "$output/derived-build" \
    "$output/derived-build/one" "$output/derived-build/two" "$output/host-tools"; do
    [ -d "$path" ] && [ ! -L "$path" ] || { echo "unsafe acquisition output skeleton" >&2; exit 73; }
    [ "$(/usr/bin/stat -c '%u:%g:%a' "$path")" = "$RAR_OUTPUT_UID:$RAR_OUTPUT_GID:755" ] || {
        echo "acquisition output ownership or mode mismatch" >&2
        exit 73
    }
done
mkdir -p "$output/apt-state/lists/partial" "$output/apt-cache/archives/partial" "$output/debs" "$output/licenses" "$output/derived-context/rootfs"

snapshot=20260630T000000Z
sources=$output/sources.list
cat > "$sources" <<EOF
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$snapshot trixie main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$snapshot trixie-updates main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$snapshot trixie-proposed-updates main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/$snapshot trixie-security main
EOF

apt_options="-o Dir::Etc::sourcelist=$sources -o Dir::Etc::sourceparts=- -o Dir::State=$output/apt-state -o Dir::Cache=$output/apt-cache -o APT::Install-Recommends=false -o Acquire::Check-Valid-Until=false -o Debug::NoLocking=1"
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
[ "$deb_count" -eq 36 ] || { echo "acquired package closure is not the approved 36-package set" >&2; exit 1; }
if find "$output" -type l | grep -q .; then
    echo "symlink in acquisition output" >&2
    exit 1
fi

package_manifest=$output/packages.v2
license_manifest=$output/licenses.v2
: > "$package_manifest"
: > "$license_manifest"
printf '%s\n' 'schema=rar-preauth-package-manifest-v2' >> "$package_manifest"
printf '%s\n' 'schema=rar-preauth-license-manifest-v2' >> "$license_manifest"
for deb in "$output"/debs/*.deb; do
    before=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$deb")
    sha=$(/usr/bin/sha256sum "$deb" | /usr/bin/cut -d ' ' -f 1)
    name=$(/usr/bin/dpkg-deb -f "$deb" Package)
    version=$(/usr/bin/dpkg-deb -f "$deb" Version)
    architecture=$(/usr/bin/dpkg-deb -f "$deb" Architecture)
    source_field=$(/usr/bin/dpkg-deb -f "$deb" Source 2>/dev/null || true)
    case "$source_field" in
        *' ('*')')
            source_name=${source_field%% *}
            source_version=${source_field#* (}
            source_version=${source_version%)}
            ;;
        '')
            source_name=$name
            source_version=$version
            ;;
        *)
            source_name=$source_field
            source_version=$version
            ;;
    esac
    size=$(/usr/bin/stat -c '%s' "$deb")
    approved_row=$(/usr/bin/grep -F "package|$name|" "$approved_packages")
    [ "$(printf '%s\n' "$approved_row" | /usr/bin/wc -l | /usr/bin/tr -d ' ')" -eq 1 ] || { echo "package identity absent or duplicated" >&2; exit 1; }
    license_sha=$(printf '%s\n' "$approved_row" | /usr/bin/cut -d '|' -f 8)
    after=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$deb")
    [ "$before" = "$after" ] || { echo "same-inode mutation detected for $deb" >&2; exit 1; }
    printf 'package|%s|%s|%s|%s|%s|%s|%s|%s|%s\n' \
        "$name" "$version" "$architecture" "$(basename "$deb")" "$size" "$sha" "$license_sha" \
        "$source_name" "$source_version" >> "$package_manifest"
    printf 'license|%s|%s\n' "$name" "$license_sha" >> "$license_manifest"
done
LC_ALL=C /usr/bin/sort -o "$package_manifest.sorted" "$package_manifest"
{
    printf '%s\n' 'schema=rar-preauth-package-manifest-v2'
    /usr/bin/grep '^package|' "$package_manifest.sorted"
} > "$package_manifest.canonical"
mv "$package_manifest.canonical" "$package_manifest"
LC_ALL=C /usr/bin/sort -o "$license_manifest.sorted" "$license_manifest"
{
    printf '%s\n' 'schema=rar-preauth-license-manifest-v2'
    /usr/bin/grep '^license|' "$license_manifest.sorted"
} > "$license_manifest.canonical"
mv "$license_manifest.canonical" "$license_manifest"

# Package bytes, binary identity, signed-snapshot source provenance and license
# digests must match the approved immutable manifest before any package is used
# to construct the derived root filesystem.
/usr/bin/cmp -s "$package_manifest" "$approved_packages" || {
    echo "observed package closure differs from the approved manifest" >&2
    /usr/bin/diff -u "$approved_packages" "$package_manifest" >&2 || true
    exit 1
}

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
inrelease_manifest=$output/debian-inrelease.sha256
security_inrelease_manifest=$output/debian-security-inrelease.sha256
: > "$lists_manifest"
: > "$inrelease_manifest"
: > "$security_inrelease_manifest"
for metadata in "$output"/apt-state/lists/*; do
    [ -f "$metadata" ] || continue
    metadata_sha=$(/usr/bin/sha256sum "$metadata" | /usr/bin/cut -d ' ' -f 1)
    metadata_name=$(basename "$metadata")
    printf '%s  %s\n' "$metadata_sha" "$metadata_name" >> "$lists_manifest"
    case "$metadata_name" in
        *debian-security*InRelease) printf '%s  %s\n' "$metadata_sha" "$metadata_name" >> "$security_inrelease_manifest" ;;
        *InRelease) printf '%s  %s\n' "$metadata_sha" "$metadata_name" >> "$inrelease_manifest" ;;
    esac
done
LC_ALL=C /usr/bin/sort -o "$lists_manifest" "$lists_manifest"
LC_ALL=C /usr/bin/sort -o "$inrelease_manifest" "$inrelease_manifest"
LC_ALL=C /usr/bin/sort -o "$security_inrelease_manifest" "$security_inrelease_manifest"
[ "$(/usr/bin/grep -c 'InRelease' "$lists_manifest")" -ge 4 ] || { echo "signed InRelease evidence absent" >&2; exit 1; }
[ "$(/usr/bin/wc -l < "$inrelease_manifest" | /usr/bin/tr -d ' ')" -eq 3 ] || { echo "Debian InRelease set is incomplete" >&2; exit 1; }
[ "$(/usr/bin/wc -l < "$security_inrelease_manifest" | /usr/bin/tr -d ' ')" -eq 1 ] || { echo "Debian security InRelease set is incomplete" >&2; exit 1; }

lock_value() {
    key=$1
    value=$(/usr/bin/sed -n "s/^$key=//p" "$root/spec/lab/preauth/locks/r0-x86_64-preauth-v3.lock")
    [ -n "$value" ] && [ "$(/usr/bin/grep -c "^$key=" "$root/spec/lab/preauth/locks/r0-x86_64-preauth-v3.lock")" -eq 1 ] || {
        echo "closure lock field missing or duplicated: $key" >&2
        exit 1
    }
    printf '%s\n' "$value"
}
assert_lock() {
    key=$1
    observed=$2
    [ "$(lock_value "$key")" = "$observed" ] || {
        echo "closure evidence mismatch: $key" >&2
        exit 1
    }
}
[ "$(/usr/bin/wc -l < "$root/spec/lab/preauth/locks/r0-x86_64-preauth-v3.lock" | /usr/bin/tr -d ' ')" -eq 32 ] || {
    echo "closure lock field count mismatch" >&2
    exit 1
}
assert_lock base_oci_index_sha256 "${RAR_CI_BOOTSTRAP_IMAGE#sha256:}"
assert_lock schema rar-preauth-closure-v3
assert_lock debian_archive snapshot.debian.org/archive/debian
assert_lock debian_snapshot "$snapshot"
assert_lock debian_suite trixie
assert_lock debian_security_archive snapshot.debian.org/archive/debian-security
assert_lock debian_security_snapshot "$snapshot"
assert_lock debian_security_suite trixie-security
assert_lock package_manifest_sha256 "$(/usr/bin/sha256sum "$package_manifest" | /usr/bin/cut -d ' ' -f 1)"
assert_lock license_manifest_sha256 "$(/usr/bin/sha256sum "$license_manifest" | /usr/bin/cut -d ' ' -f 1)"
assert_lock debian_archive_keyring_sha256 "$(/usr/bin/sha256sum /usr/share/keyrings/debian-archive-keyring.gpg | /usr/bin/cut -d ' ' -f 1)"
assert_lock inrelease_sha256 "$(/usr/bin/sha256sum "$inrelease_manifest" | /usr/bin/cut -d ' ' -f 1)"
assert_lock security_inrelease_sha256 "$(/usr/bin/sha256sum "$security_inrelease_manifest" | /usr/bin/cut -d ' ' -f 1)"
assert_lock acquisition_policy_sha256 "$(/usr/bin/sha256sum "$root/tools/toolchain/acquire-preauth-closure.sh" | /usr/bin/cut -d ' ' -f 1)"
assert_lock lld_path /usr/bin/ld.lld-19
assert_lock qemu_path /usr/bin/qemu-system-x86_64
assert_lock ovmf_code_path /usr/share/OVMF/OVMF_CODE_4M.fd
assert_lock ovmf_vars_path /usr/share/OVMF/OVMF_VARS_4M.fd
assert_lock target_linked_dependencies none
assert_lock certifiable true

# Build and validate the complete package archive plan before any extraction. Package bytes are
# copied once into a private container staging directory under content-addressed names, verified
# again, and only those immutable staged bytes are consumed. No mutable repository path is
# reopened by an extractor after validation.
private_stage=/tmp/rar-preauth-private-stage
[ ! -e "$private_stage" ] || { echo "private acquisition stage already exists" >&2; exit 73; }
/usr/bin/mkdir -m 700 "$private_stage" "$private_stage/debs" "$private_stage/rootfs"
trap '/bin/rm -rf "$private_stage"' EXIT HUP INT TERM
archive_plan=$private_stage/archive.plan
: > "$archive_plan"
for deb in "$output"/debs/*.deb; do
    name=$(/usr/bin/dpkg-deb -f "$deb" Package)
    expected_sha=$(/usr/bin/awk -F '|' -v name="$name" '$1 == "package" && $2 == name { print $7 }' "$approved_packages")
    observed_sha=$(/usr/bin/sha256sum "$deb" | /usr/bin/cut -d ' ' -f 1)
    [ "$expected_sha" = "$observed_sha" ] || { echo "package changed before immutable staging" >&2; exit 73; }
    staged=$private_stage/debs/$observed_sha.deb
    /usr/bin/cp --reflink=never "$deb" "$staged"
    [ "$(/usr/bin/stat -c '%h:%a' "$staged")" = '1:600' ] || /usr/bin/chmod 600 "$staged"
    [ "$(/usr/bin/sha256sum "$staged" | /usr/bin/cut -d ' ' -f 1)" = "$observed_sha" ] || { echo "staged package mutation" >&2; exit 73; }
    /usr/bin/dpkg-deb -c "$staged" | /usr/bin/awk '
        function safe(path) { return path != "" && path !~ /^\// && path !~ /(^|\/)\.\.?(\/|$)/ && path !~ /\\/ && path !~ /:/ }
        function safe_link(path, target, base,n,i,p,depth) {
          if (target == "" || target ~ /^\// || target ~ /\\/ || target ~ /:/) return 0;
          base=path; sub(/\/[^/]*$/, "", base); n=split(base "/" target, p, "/"); depth=0;
          for (i=1;i<=n;i++) { if (p[i]=="" || p[i]==".") continue; if (p[i]=="..") { if (depth==0) return 0; depth-- } else depth++ }
          return 1
        }
        NF < 6 { exit 73 }
        { path=$6; sub(/^\.\//, "", path); type=substr($1,1,1); if (type !~ /[-dl]/) exit 73;
          if (path == "" && type == "d") next; if (!safe(path)) exit 73;
          if (type == "l") { if ($7 != "->" || !safe_link(path,$8)) exit 73; print path "|" type "|" $8 }
          else print path "|" type }
    ' >> "$archive_plan" || { echo "unsafe package archive plan" >&2; exit 73; }
done
[ "$(/usr/bin/wc -l < "$archive_plan" | /usr/bin/tr -d ' ')" -le 200000 ] || { echo "package archive member bound exceeded" >&2; exit 73; }
/usr/bin/awk -F '|' '{ if (($1 in kind) && (kind[$1] != "d" || $2 != "d")) exit 73; kind[$1]=$2 }' "$archive_plan" || { echo "package archive destination collision" >&2; exit 73; }

for staged in "$private_stage"/debs/*.deb; do /usr/bin/dpkg-deb -x "$staged" "$private_stage/rootfs"; done
for row in $(/usr/bin/grep '^package|' "$approved_packages" | /usr/bin/cut -d '|' -f 2,8); do
    name=${row%|*}; expected=${row#*|}; copyright=$private_stage/rootfs/usr/share/doc/$name/copyright
    resolved=$(/usr/bin/realpath -e "$copyright") || { echo "license missing for $name" >&2; exit 73; }
    case "$resolved" in "$private_stage"/rootfs/usr/share/doc/*) ;; *) echo "license path escape" >&2; exit 73 ;; esac
    [ -f "$resolved" ] && [ "$(/usr/bin/sha256sum "$resolved" | /usr/bin/cut -d ' ' -f 1)" = "$expected" ] || { echo "license digest mismatch for $name" >&2; exit 73; }
done
for binding in \
    "/usr/bin/ld.lld-19:lld_sha256" \
    "/usr/bin/qemu-system-x86_64:qemu_sha256" \
    "/usr/share/OVMF/OVMF_CODE_4M.fd:ovmf_code_sha256" \
    "/usr/share/OVMF/OVMF_VARS_4M.fd:ovmf_vars_sha256"; do
    path=${binding%:*}; key=${binding#*:}; staged=$private_stage/rootfs$path
    [ -f "$staged" ] && [ "$(/usr/bin/sha256sum "$staged" | /usr/bin/cut -d ' ' -f 1)" = "$(lock_value "$key")" ] || { echo "derived tool byte mismatch: $key" >&2; exit 73; }
done

/usr/bin/cp -a "$private_stage/rootfs/." "$output/derived-context/rootfs/"
/usr/bin/find "$output/derived-context/rootfs" ! -type d \
    -exec /usr/bin/touch -h -d '@1784332800' {} +
/usr/bin/find "$output/derived-context/rootfs" -depth -type d \
    -exec /usr/bin/touch -d '@1784332800' {} +
cat > "$output/derived-context/Dockerfile" <<'EOF'
FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
ARG SOURCE_DATE_EPOCH=1784332800
COPY rootfs/ /
ENV RAR_PREAUTH_BUILD_CONTAINER=rar-preauth-closure-v3
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
        "debian_archive_keyring_sha256=$(/usr/bin/sha256sum /usr/share/keyrings/debian-archive-keyring.gpg | /usr/bin/cut -d ' ' -f 1)" \
        "inrelease_sha256=$(/usr/bin/sha256sum "$inrelease_manifest" | /usr/bin/cut -d ' ' -f 1)" \
        "security_inrelease_sha256=$(/usr/bin/sha256sum "$security_inrelease_manifest" | /usr/bin/cut -d ' ' -f 1)" \
        "license_manifest_sha256=$(/usr/bin/sha256sum "$license_manifest" | /usr/bin/cut -d ' ' -f 1)" \
        "acquisition_policy_sha256=$(/usr/bin/sha256sum "$root/tools/toolchain/acquire-preauth-closure.sh" | /usr/bin/cut -d ' ' -f 1)" \
        'signature_verification=apt-secure-passed' \
        'target_execution=not-attempted' \
        'qemu_execution=not-attempted' \
        'emulator_execution=not-attempted' \
        'vm_execution=not-attempted'
} > "$output/discovery.evidence"

# apt-secure, download-only APT, dpkg metadata reads, and dpkg-deb extraction
# operate entirely in the pre-created repository-local state directories. None
# requires host or container root, so every output inode retains runner ownership.
