#!/bin/sh
set -eu

fail() {
    echo "cloud-target-tools: $1" >&2
    exit 73
}

verify_file() {
    label=$1
    path=$2
    expected=$3

    [ -f "$path" ] && [ ! -L "$path" ] || fail "$label is not a regular non-symlink file"
    resolved=$(/usr/bin/readlink -f -- "$path") || fail "$label path cannot be resolved"
    [ "$resolved" = "$path" ] || fail "$label path is not canonical"
    actual=$(/usr/bin/sha256sum "$path") || fail "$label cannot be hashed"
    actual=${actual%% *}
    [ "$actual" = "$expected" ] || fail "$label digest mismatch"
}

[ "${RAR_DEVELOPMENT_LAB-}" = cloud-v1 ] || fail "Development Lab boundary missing"
[ "$(id -u)" = "${RAR_CONTAINER_UID-}" ] || fail "container UID mismatch"
[ "$(id -g)" = "${RAR_CONTAINER_GID-}" ] || fail "container GID mismatch"

verify_file compiler "${RAR_COMPILER_PATH-}" "${RAR_COMPILER_SHA256-}"
verify_file linker "${RAR_LINKER_PATH-}" "${RAR_LINKER_SHA256-}"
for forbidden in RAR_REFERENCE_1_PATH RAR_REFERENCE_1_SHA256 RAR_REFERENCE_2_PATH RAR_REFERENCE_2_SHA256; do
    eval "value=\${$forbidden-}"
    [ -z "$value" ] || fail 'reference authority leaked into untrusted build role'
done

driver=${1-}
[ -f "$driver" ] && [ ! -L "$driver" ] || fail "probe driver unavailable"
/bin/sh -eu "$driver"

artifact=/build/rar-os-alpha.img
[ -f "$artifact" ] && [ ! -L "$artifact" ] || fail "build driver did not produce the fixed artifact"
size=$(/usr/bin/stat -c %s "$artifact") || fail "artifact size unavailable"
case "$size" in '' | *[!0-9]*) fail "artifact size malformed" ;; esac
[ "$size" -gt 0 ] || fail "artifact is empty"
[ "$size" -le $((64 * 1024 * 1024)) ] || fail "artifact exceeds the fixed bound"
/usr/bin/sha256sum "$artifact"
