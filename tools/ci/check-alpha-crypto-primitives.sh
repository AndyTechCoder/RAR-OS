#!/bin/sh
# Focused primitive tests only in the existing isolated cloud Specifications job.
set -eu
[ "${GITHUB_ACTIONS-}" = true ]
[ "${CI-}" = true ]
[ "${RAR_CI_RUNNER_OS-}" = Linux ]
[ "$(uname -s)" = Linux ]
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ]
ulimit -f 32768
ulimit -t 60
work=$(mktemp -d /tmp/rar-crypto-tests.XXXXXXXX)
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --test core/crypto/lib.rs -o "$work/crypto-tests"
"$work/crypto-tests"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc --edition 2024 --crate-type lib -D warnings core/crypto/lib.rs -o "$work/libcrypto.rlib"
printf '%s\n' 'Alpha crypto: SHA-512/Ed25519 initial focused tests and no_std compile passed; signing/runtime gates not claimed'
