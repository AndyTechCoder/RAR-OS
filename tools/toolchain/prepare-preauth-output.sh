#!/bin/sh
set -eu

# Create the complete Prompt 7A repository output skeleton as the invoking
# runner. Containers may populate these directories, but may not create or
# change ownership of repository ancestors.

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
case "${1-}" in
    '') output_root=$root ;;
    --test-root)
        [ "${RAR_OUTPUT_GUARD_TESTING-}" = 1 ] || { echo "output_guard error=test-root-disabled" >&2; exit 73; }
        [ -n "${2-}" ] && [ "$#" -eq 2 ] || { echo "output_guard error=invalid-test-root" >&2; exit 73; }
        case "$2" in "$root"/out/r0/preauth/ownership-tests/*) ;; *) echo "output_guard error=test-root-outside-allowlist" >&2; exit 73 ;; esac
        output_root=$2
        ;;
    *) echo "output_guard error=usage" >&2; exit 73 ;;
esac

expected_uid=$(/usr/bin/id -u)
expected_gid=$(/usr/bin/id -g)
if [ "${RAR_OUTPUT_GUARD_TESTING-}" = 1 ] && [ -n "${RAR_OUTPUT_GUARD_EXPECTED_UID-}" ]; then
    case "$RAR_OUTPUT_GUARD_EXPECTED_UID" in *[!0-9]*|'') exit 73 ;; esac
    expected_uid=$RAR_OUTPUT_GUARD_EXPECTED_UID
fi
if [ "${RAR_OUTPUT_GUARD_TESTING-}" = 1 ] && [ -n "${RAR_OUTPUT_GUARD_EXPECTED_GID-}" ]; then
    case "$RAR_OUTPUT_GUARD_EXPECTED_GID" in *[!0-9]*|'') exit 73 ;; esac
    expected_gid=$RAR_OUTPUT_GUARD_EXPECTED_GID
fi

case "$output_root" in /*) ;; *) echo "output_guard error=nonabsolute-root" >&2; exit 73 ;; esac
[ ! -L "$output_root" ] || { echo "output_guard error=symlink-root" >&2; exit 73; }

directories='out
out/r0
out/r0/preauth
out/r0/preauth/acquisition
out/r0/preauth/acquisition/apt-state
out/r0/preauth/acquisition/apt-state/lists
out/r0/preauth/acquisition/apt-state/lists/partial
out/r0/preauth/acquisition/apt-cache
out/r0/preauth/acquisition/apt-cache/archives
out/r0/preauth/acquisition/apt-cache/archives/partial
out/r0/preauth/acquisition/debs
out/r0/preauth/acquisition/licenses
out/r0/preauth/acquisition/derived-context
out/r0/preauth/acquisition/derived-context/rootfs
out/r0/preauth/acquisition/derived-build
out/r0/preauth/acquisition/derived-build/one
out/r0/preauth/acquisition/derived-build/two
out/r0/preauth/acquisition/host-tools
out/r0/preauth/build
out/r0/preauth/build/one
out/r0/preauth/build/two
out/r0/artifacts
out/r0/artifacts/x86_64
out/r0/vm
out/r0/vm/x86_64'

approved_relative() {
    case "$1" in
        out|out/r0|out/r0/preauth|out/r0/preauth/acquisition|\
        out/r0/preauth/acquisition/apt-state|out/r0/preauth/acquisition/apt-state/lists|\
        out/r0/preauth/acquisition/apt-state/lists/partial|out/r0/preauth/acquisition/apt-cache|\
        out/r0/preauth/acquisition/apt-cache/archives|out/r0/preauth/acquisition/apt-cache/archives/partial|\
        out/r0/preauth/acquisition/debs|out/r0/preauth/acquisition/licenses|\
        out/r0/preauth/acquisition/derived-context|out/r0/preauth/acquisition/derived-context/rootfs|\
        out/r0/preauth/acquisition/derived-build|out/r0/preauth/acquisition/derived-build/one|\
        out/r0/preauth/acquisition/derived-build/two|out/r0/preauth/acquisition/host-tools|\
        out/r0/preauth/build|out/r0/preauth/build/one|out/r0/preauth/build/two|\
        out/r0/artifacts|out/r0/artifacts/x86_64|out/r0/vm|out/r0/vm/x86_64) return 0 ;;
        *) return 1 ;;
    esac
}

umask 022
if [ -e "$output_root/out" ] || [ -L "$output_root/out" ]; then
    /usr/bin/find -P "$output_root/out" -mindepth 1 -print | while IFS= read -r existing; do
        relative=${existing#"$output_root/"}
        approved_relative "$relative" || { echo "output_guard error=unexpected-node path=$relative" >&2; exit 73; }
        [ -d "$existing" ] && [ ! -L "$existing" ] || {
            echo "output_guard error=unexpected-node path=$relative" >&2
            exit 73
        }
    done
fi
printf '%s\n' "$directories" | while IFS= read -r relative; do
    approved_relative "$relative" || { echo "output_guard error=unallocated-path path=$relative" >&2; exit 73; }
    case "/$relative/" in */../*|*/./*|*//* ) echo "output_guard error=ambiguous-path path=$relative" >&2; exit 73 ;; esac
    path=$output_root/$relative
    [ ! -L "$path" ] || { echo "output_guard error=symlink path=$relative" >&2; exit 73; }
    if [ -e "$path" ]; then
        [ -d "$path" ] || { echo "output_guard error=unexpected-node path=$relative" >&2; exit 73; }
    else
        /usr/bin/mkdir "$path"
        /usr/bin/chmod 0755 "$path"
    fi
    actual=$(/usr/bin/stat -c '%u:%g:%a:%F' "$path")
    expected=$expected_uid:$expected_gid:755:directory
    [ "$actual" = "$expected" ] || {
        safe_actual=$(printf '%s' "$actual" | /usr/bin/cut -d: -f1-3)
        echo "output_guard error=metadata path=$relative expected=$expected_uid:$expected_gid:755 actual=$safe_actual" >&2
        exit 73
    }
    printf 'output_owner path=%s creator=host-prepare uid=%s gid=%s mode=0755 type=directory\n' \
        "$relative" "$expected_uid" "$expected_gid"
done
