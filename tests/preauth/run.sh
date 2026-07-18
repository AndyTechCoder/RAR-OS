#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
tests/preauth/transaction-contracts.sh
tools/ci/check-preauth-cutover.sh
mkdir -p out/r0
rustc --edition=2024 --test tests/preauth/src/main.rs -o out/r0/preauth-tests
out/r0/preauth-tests --test-threads=1
