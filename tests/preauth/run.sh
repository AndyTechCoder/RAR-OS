#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
tests/preauth/transaction-contracts.sh
tests/preauth/input-producer-contracts.sh
tools/ci/check-preauth-cutover.sh
mkdir -p out/r0
if [ -x /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc ]; then
 rustc_path=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc
else
 rustc_path=$(command -v rustc) || { printf '%s\n' 'preauth-tests:rustc-unavailable' >&2; exit 1; }
fi
"$rustc_path" --edition=2024 --test tests/preauth/src/main.rs -o out/r0/preauth-tests
out/r0/preauth-tests --test-threads=1
"$rustc_path" --edition=2024 --test tools/toolchain/preauth-transfer-telemetry.rs -o out/r0/preauth-transfer-telemetry-tests
out/r0/preauth-transfer-telemetry-tests --test-threads=1
