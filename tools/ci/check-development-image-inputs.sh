#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
canonical=$root/tools/rar-lab/images/image-inputs-v1.env
fixtures=$root/spec/alpha/lab/fixtures/development-image-policy
inputs=${1-$canonical}
mode=${2-}
case "$inputs" in
    "$canonical") expected_parent=$root/tools/rar-lab/images ;;
    "$fixtures/inputs-ready.env"|"$fixtures/aliased-bases.env"|"$fixtures/populated-output.env") expected_parent=$fixtures ;;
    *) exit 1 ;;
esac
[ "$(CDPATH= cd -- "$(dirname -- "$inputs")" && pwd -P)" = "$expected_parent" ] || exit 1
[ -f "$inputs" ] && [ ! -L "$inputs" ] || exit 1
size=$(/usr/bin/stat -c %s "$inputs" 2>/dev/null || /usr/bin/stat -f %z "$inputs")
[ "$size" -le 32768 ] || exit 1
/usr/bin/awk 'length($0) > 4096 { exit 1 }' "$inputs" || exit 1
[ "$(/usr/bin/wc -l < "$inputs" | /usr/bin/tr -d ' ')" -eq 28 ] || exit 1
if /usr/bin/grep -Ev '^(schema|state|buildkit_image|build_base|launch_base|debian_snapshot|rust_version|rust_musl_url|rust_musl_sha256|rust_none_url|rust_none_sha256|rust_uefi_url|rust_uefi_sha256|openssl_url|openssl_source_sha256|libsodium_url|libsodium_source_sha256|qemu_version|ovmf_version|source_date_epoch|build_image|launch_base_image|launch_image|compiler_sha256|linker_sha256|qemu_sha256|firmware_sha256|qmp_binary_sha256)=[A-Za-z0-9._:/@+-]+$' "$inputs" | /usr/bin/grep -q .; then exit 1; fi
for key in schema state buildkit_image build_base launch_base debian_snapshot rust_version rust_musl_url rust_musl_sha256 rust_none_url rust_none_sha256 rust_uefi_url rust_uefi_sha256 openssl_url openssl_source_sha256 libsodium_url libsodium_source_sha256 qemu_version ovmf_version source_date_epoch build_image launch_base_image launch_image compiler_sha256 linker_sha256 qemu_sha256 firmware_sha256 qmp_binary_sha256; do
    [ "$(/usr/bin/grep -c "^$key=" "$inputs")" -eq 1 ] || exit 1
done
# shellcheck disable=SC1090
. "$inputs"
[ "$schema" = rar-development-image-inputs-v1 ] || exit 1
[ "$rust_version" = 1.95.0 ] || exit 1
[ "$rust_musl_url" = https://static.rust-lang.org/dist/rust-std-1.95.0-x86_64-unknown-linux-musl.tar.xz ] || exit 1
[ "$rust_none_url" = https://static.rust-lang.org/dist/rust-std-1.95.0-x86_64-unknown-none.tar.xz ] || exit 1
[ "$rust_uefi_url" = https://static.rust-lang.org/dist/rust-std-1.95.0-x86_64-unknown-uefi.tar.xz ] || exit 1
[ "$openssl_url" = https://github.com/openssl/openssl/releases/download/openssl-3.0.13/openssl-3.0.13.tar.gz ] || exit 1
[ "$libsodium_url" = https://github.com/jedisct1/libsodium/releases/download/1.0.19-RELEASE/libsodium-1.0.19.tar.gz ] || exit 1

outputs='build_image launch_base_image launch_image compiler_sha256 linker_sha256 qemu_sha256 firmware_sha256 qmp_binary_sha256'
validate_digest() {
    [ "${#1}" -eq 64 ] || exit 1
    case "$1" in *[!0-9a-f]*) exit 1 ;; esac
}
validate_image() {
    case "$1" in *@sha256:????????????????????????????????????????????????????????????????) ;;
        *) exit 1 ;;
    esac
    digest=${1##*@sha256:}
    validate_digest "$digest"
}
[ "$state" = decision-blocked ] || exit 1
validate_image "$buildkit_image"
validate_image "$build_base"
validate_image "$launch_base"
[ "${build_base##*@sha256:}" != "${launch_base##*@sha256:}" ] || exit 1
case "$debian_snapshot" in 20[0-9][0-9][01][0-9][0-3][0-9]T[0-2][0-9][0-5][0-9][0-5][0-9]Z) ;; *) exit 1 ;; esac
for digest in "$rust_musl_sha256" "$rust_none_sha256" "$rust_uefi_sha256" "$openssl_source_sha256" "$libsodium_source_sha256"; do validate_digest "$digest"; done
case "$qemu_version:$ovmf_version" in *[!A-Za-z0-9.+:~_-]*) exit 1 ;; esac
case "$source_date_epoch" in '' | *[!0-9]*) exit 1 ;; esac
[ "$source_date_epoch" -ge 1 ] || exit 1
for key in $outputs; do eval "value=\${$key}"; [ "$value" = unavailable ] || exit 1; done
case "$mode" in
    '') ;;
    --require-decision-blocked) ;;
    *) exit 1 ;;
esac
printf 'Development image input validation passed: state=%s\n' "$state"
