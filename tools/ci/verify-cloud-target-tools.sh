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
verify_file qemu "${RAR_QEMU_PATH-}" "${RAR_QEMU_SHA256-}"
verify_file firmware "${RAR_FIRMWARE_PATH-}" "${RAR_FIRMWARE_SHA256-}"
verify_file machine-profile "${RAR_MACHINE_PROFILE_PATH-}" "${RAR_MACHINE_PROFILE_SHA256-}"

driver=${1-}
[ -f "$driver" ] && [ ! -L "$driver" ] || fail "probe driver unavailable"
/bin/sh -eu "$driver"
