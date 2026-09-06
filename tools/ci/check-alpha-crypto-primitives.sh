#!/bin/sh
# Focused primitive tests only in the existing isolated cloud Specifications job.
set -eu
[ "${GITHUB_ACTIONS-}" = true ]
[ "${CI-}" = true ]
[ "${RAR_CI_RUNNER_OS-}" = Linux ]
[ "$(uname -s)" = Linux ]
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ]
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
[ "$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")" = /tmp ]
[ -d /build ] && [ ! -L /build ]
[ -n "${RAR_EXPECTED_SOURCE_REVISION-}" ]
[ "$(/usr/bin/git -C "$root" rev-parse HEAD)" = "$RAR_EXPECTED_SOURCE_REVISION" ]
cd "$root"
ulimit -f 32768
ulimit -t 60
work=$(mktemp -d /build/rar-crypto-tests.XXXXXXXX)
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes core/crypto/lib.rs -o "$work/crypto-tests"
"$work/crypto-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/crypto/lib.rs -o "$work/libcrypto.rlib"
printf '%s\n' 'Alpha crypto: SHA-512/Ed25519 initial focused tests and no_std compile passed; signing/runtime gates not claimed'

# Data-only candidate provisioning checks. No network, Docker or target launch.
[ -x /usr/bin/python3 ]
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/reference_inventory.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/provision_reference.py" --self-test
printf '%s\n' 'Modern reference provisioning: pure inventory/acquisition/guard tests passed; candidate construction and activation not claimed'

# Compiler construction parsers only; no acquisition/build/activation here.
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_closure.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_archive.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_elf.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/compiler_inventory.py" --self-test
/usr/bin/python3 -I -B "$root/tools/rar-lab/modern/provision_compiler.py" --self-test
printf '%s\n' 'Modern compiler provisioning: pure fixture and guard tests passed; actual construction and activation not claimed'

# Supervisor fixtures spawn only tiny host Python test children in this container.
/usr/bin/python3 -I -B "$root/tools/ci/parallel-policy-tests.py" --self-test
printf '%s\n' 'Parallel policy supervisor: success/failure/timeout/output/signal fixtures passed'
