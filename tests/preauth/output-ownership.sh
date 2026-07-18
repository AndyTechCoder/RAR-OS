#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
guard=$root/tools/toolchain/prepare-preauth-output.sh
tests=$root/out/r0/preauth/ownership-tests
for parent in "$root/out" "$root/out/r0" "$root/out/r0/preauth"; do
    [ ! -L "$parent" ] || { echo "ownership test parent is indirect" >&2; exit 73; }
    if [ ! -e "$parent" ]; then
        /usr/bin/mkdir "$parent"
        /usr/bin/chmod 0755 "$parent"
    fi
    [ -d "$parent" ] || { echo "ownership test parent is not a directory" >&2; exit 73; }
done
[ ! -e "$tests" ] && [ ! -L "$tests" ] || { echo "ownership test output already exists" >&2; exit 73; }
/usr/bin/mkdir "$tests"

prepare_case() {
    name=$1
    path=$tests/$name
    /usr/bin/mkdir "$path"
    RAR_OUTPUT_GUARD_TESTING=1 "$guard" --test-root "$path"
}
expect_refusal() {
    name=$1
    shift
    set +e
    "$@" > "$tests/$name.stdout" 2> "$tests/$name.stderr"
    status=$?
    set -e
    [ "$status" -eq 73 ] || { echo "ownership refusal failed: $name" >&2; exit 1; }
}

prepare_case success-one > "$tests/success-one.evidence"
prepare_case success-two > "$tests/success-two.evidence"
/usr/bin/sed 's#success-one#case#g' "$tests/success-one.evidence" > "$tests/success-one.normalized"
/usr/bin/sed 's#success-two#case#g' "$tests/success-two.evidence" > "$tests/success-two.normalized"
/usr/bin/cmp "$tests/success-one.normalized" "$tests/success-two.normalized"

wrong_mode=$tests/wrong-mode
/usr/bin/mkdir "$wrong_mode"
/usr/bin/mkdir "$wrong_mode/out"
/usr/bin/chmod 0777 "$wrong_mode/out"
expect_refusal wrong-mode env RAR_OUTPUT_GUARD_TESTING=1 "$guard" --test-root "$wrong_mode"
/usr/bin/grep -F 'output_guard error=metadata path=out' "$tests/wrong-mode.stderr" >/dev/null

foreign=$tests/foreign-owner
/usr/bin/mkdir "$foreign"
foreign_uid=0
[ "$(/usr/bin/id -u)" -ne 0 ] || foreign_uid=65534
expect_refusal foreign-owner env RAR_OUTPUT_GUARD_TESTING=1 \
    RAR_OUTPUT_GUARD_EXPECTED_UID=$foreign_uid "$guard" --test-root "$foreign"
/usr/bin/grep -F 'output_guard error=metadata path=out' "$tests/foreign-owner.stderr" >/dev/null

unexpected=$tests/unexpected
/usr/bin/mkdir -p "$unexpected/out/r0"
: > "$unexpected/out/r0/unallocated"
expect_refusal unexpected env RAR_OUTPUT_GUARD_TESTING=1 "$guard" --test-root "$unexpected"
/usr/bin/grep -F 'output_guard error=unexpected-node path=out/r0/unallocated' "$tests/unexpected.stderr" >/dev/null

symlink=$tests/symlink
/usr/bin/mkdir -p "$symlink/out"
/usr/bin/ln -s out "$symlink/out/r0"
expect_refusal symlink env RAR_OUTPUT_GUARD_TESTING=1 "$guard" --test-root "$symlink"
/usr/bin/grep -F 'output_guard error=unexpected-node path=out/r0' "$tests/symlink.stderr" >/dev/null

special=$tests/special
/usr/bin/mkdir -p "$special/out/r0"
/usr/bin/mkfifo "$special/out/r0/preauth"
expect_refusal special env RAR_OUTPUT_GUARD_TESTING=1 "$guard" --test-root "$special"
/usr/bin/grep -F 'output_guard error=unexpected-node path=out/r0/preauth' "$tests/special.stderr" >/dev/null

for directory in \
    out/r0/preauth/acquisition/derived-build/one \
    out/r0/preauth/acquisition/derived-build/two \
    out/r0/preauth/acquisition/host-tools \
    out/r0/preauth/build/one out/r0/preauth/build/two \
    out/r0/artifacts/x86_64 out/r0/vm/x86_64; do
    probe=$tests/success-one/$directory/.host-owner-write-probe
    [ ! -e "$probe" ]
    : > "$probe"
    /bin/rm "$probe"
done

printf '%s\n' \
    'ownership_guard_tests=passed' \
    'root_writable_workspace_mounts=0' \
    'root_stage_import=not-required' \
    'target_execution=not-attempted' \
    'qemu_execution=not-attempted' \
    'emulator_execution=not-attempted' \
    'vm_execution=not-attempted'
