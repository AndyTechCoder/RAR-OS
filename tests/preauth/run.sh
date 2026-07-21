#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
. tools/toolchain/preauth-build-root.sh
tests/preauth/transaction-contracts.sh
tests/preauth/input-producer-contracts.sh
tools/ci/check-preauth-cutover.sh
mkdir -p out/r0
rustc_path=$(preauth_build_pinned_rustc_path "$root") || { printf '%s\n' 'preauth-tests:pinned-rustc' >&2; exit 1; }
"$rustc_path" --edition=2024 --test tests/preauth/src/main.rs -o out/r0/preauth-tests
out/r0/preauth-tests --test-threads=1
"$rustc_path" --edition=2024 --test tools/toolchain/preauth-transfer-telemetry.rs -o out/r0/preauth-transfer-telemetry-tests
out/r0/preauth-transfer-telemetry-tests --test-threads=1
