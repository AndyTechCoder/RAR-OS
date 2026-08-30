#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
expected_images=$root/tools/rar-lab/images
images=${1-$expected_images}
[ "$images" = "$expected_images" ] || exit 1
[ -d "$images" ] && [ ! -L "$images" ] || exit 1
[ "$(CDPATH= cd -- "$images" && pwd -P)" = "$expected_images" ] || exit 1
expected='README.md
build.Containerfile
image-inputs-v1.env
launch-base.Containerfile
launch.Containerfile'
actual=$(find "$images" -mindepth 1 -maxdepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$images/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || exit 1
find "$images" -type l -print | /usr/bin/grep -q . && exit 1

build=$images/build.Containerfile
launch_base=$images/launch-base.Containerfile
launch=$images/launch.Containerfile
/bin/sh "$root/tools/ci/check-containerfile-static-policy.sh" "$build" "$launch_base" "$launch" >/dev/null
[ "$(/usr/bin/grep -c '^FROM \${BUILD_BASE}$' "$build")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -c '^FROM \${LAUNCH_BASE}$' "$launch_base")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -c '^FROM \${BUILD_IMAGE} AS qmp-builder$' "$launch")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -c '^FROM \${LAUNCH_BASE_IMAGE}$' "$launch")" -eq 1 ] || exit 1
for file in "$build" "$launch_base" "$launch"; do
    /usr/bin/grep -Fq 'USER 65532:65532' "$file" || exit 1
done
/usr/bin/grep -Fq 'install -d -m 0700 /bootstrap' "$build" || exit 1
for digest in RUST_MUSL_SHA256 RUST_NONE_SHA256 RUST_UEFI_SHA256; do
    /usr/bin/grep -Fq "\$$digest" "$build" || exit 1
done
[ "$(/usr/bin/grep -c 'sha256sum --check --strict' "$build")" -eq 3 ] || exit 1
/usr/bin/grep -Fq 'test ! -e /opt/rar-reference' "$build" || exit 1
! /usr/bin/grep -Fq '/opt/rar-reference/bin' "$build" || exit 1
/usr/bin/grep -Fq '"qemu-system-x86=$QEMU_VERSION" "ovmf=$OVMF_VERSION"' "$launch_base" || exit 1
/usr/bin/grep -Fq 'snapshot.debian.org/archive/debian/%s' "$launch_base" || exit 1
/usr/bin/grep -Fq -- '--target=x86_64-unknown-linux-musl' "$launch" || exit 1
/usr/bin/grep -Fq 'for output in a b; do' "$launch" || exit 1
/usr/bin/grep -Fq 'cmp /build/rar-qmp-client-a /build/rar-qmp-client-b' "$launch" || exit 1
/usr/bin/grep -Fq 'mkdir -p /evidence' "$launch" || exit 1
! /usr/bin/grep -Fq '/workspace' "$launch" || exit 1
for source in README.md build-plan.v1 json.rs main.rs; do
    [ "$(/usr/bin/grep -Fxc "COPY --chown=65532:65532 tools/rar-lab/qmp-client/$source /controller/tools/rar-lab/qmp-client/$source" "$launch")" -eq 1 ] || exit 1
done
[ "$(/usr/bin/grep -c '^COPY --chown=65532:65532 tools/rar-lab/qmp-client/' "$launch")" -eq 4 ] || exit 1
! /usr/bin/grep -Fqx 'COPY --chown=65532:65532 tools/rar-lab/qmp-client /controller/tools/rar-lab/qmp-client' "$launch" || exit 1

/bin/sh "$root/tools/ci/check-development-image-inputs.sh" "$images/image-inputs-v1.env" >/dev/null
printf '%s\n' 'Development image source policy passed'
