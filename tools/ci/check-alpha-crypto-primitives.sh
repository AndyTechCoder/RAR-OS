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
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes core/crypto/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/crypto/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Alpha crypto: hashes/Ed25519/AEAD initial focused tests and no_std compile passed; signing/runtime gates not claimed'
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes core/modern/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/modern/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern core: focused manifest/journal model tests and no_std compile passed; disk/lifecycle/runtime gates not claimed'
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes nucleus/modern/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings nucleus/modern/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern lifecycle: focused mechanism model tests and no_std compile passed; kernel runtime integration not claimed'

/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test -C strip=symbols -C debuginfo=0 -C opt-level=1 -C debug-assertions=yes -C overflow-checks=yes services/modern/lib.rs -o "$work/focused-tests"
"$work/focused-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings services/modern/lib.rs -o "$work/focused.rlib"
printf '%s\n' 'Modern PIO: bounded transport tests and no_std compile passed; device/runtime integration not claimed'

# Keep at most one stripped test executable and one no_std library in the
# existing cloud-only tmpfs; no owner files or retained evidence are affected.
set -- $(/usr/bin/du -sk "$work")
[ "$1" -le 8192 ]
printf 'Modern focused scratch KiB: %s (limit 8192)\n' "$1"
