#!/bin/sh
set -eu

[ "${RAR_PREAUTH_BUILD_CONTAINER-}" = rar-preauth-closure-v3 ] || {
    echo "refusing outside the pinned Prompt 7A build container" >&2
    exit 73
}
[ "${RAR_TARGET_EXECUTION-}" = prohibited ] || exit 73

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
output=${1-}
case "$output" in
    "$root"/out/r0/preauth/build/*) ;;
    *) echo "output must be under out/r0/preauth/build" >&2; exit 2 ;;
esac
[ ! -L "$output" ] || exit 2
mkdir -p "$output"

source_epoch=${SOURCE_DATE_EPOCH-}
case "$source_epoch" in *[!0-9]* | '') exit 2 ;; esac

/usr/bin/as --64 --fatal-warnings \
    -o "$output/preauth_entry.o" \
    "$root/nucleus/arch/x86_64/preauth_entry.S"
/usr/bin/ld.lld-19 \
    --build-id=none \
    --fatal-warnings \
    --no-dynamic-linker \
    --nostdlib \
    --static \
    -T "$root/nucleus/arch/x86_64/preauth.ld" \
    -o "$output/rar-r0-x86_64-preauth.elf" \
    "$output/preauth_entry.o"

/usr/bin/sha256sum \
    "$output/preauth_entry.o" \
    "$output/rar-r0-x86_64-preauth.elf" > "$output/SHA256SUMS"
printf '%s\n' \
    'artifact_execution=not-attempted' \
    'target_execution=not-attempted' \
    'qemu_execution=not-attempted' \
    'emulator_execution=not-attempted' \
    'vm_execution=not-attempted' > "$output/non-execution.evidence"
