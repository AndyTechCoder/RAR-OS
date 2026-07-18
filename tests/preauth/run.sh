#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

test_dir=out/r0/preauth/host-tests
mkdir -p "$test_dir"
rustc --edition=2024 --test tests/preauth/src/main.rs -o "$test_dir/preauth-tests"
"$test_dir/preauth-tests" --test-threads=1
printf '%s\n' \
    'target_execution=not-attempted' \
    'qemu_execution=not-attempted' \
    'emulator_execution=not-attempted' \
    'vm_execution=not-attempted' \
    'aws_calls=not-attempted'
