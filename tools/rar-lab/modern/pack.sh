#!/bin/sh
# Fixed cloud-only host packager; no target execution.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
[ "$CI" = true ]
[ "$GITHUB_ACTIONS" = true ]
[ "$RAR_CI_RUNNER_OS" = Linux ]
[ -f /packager.rs ] && [ ! -L /packager.rs ]
[ -f /artifact/modern.efi ] && [ ! -L /artifact/modern.efi ]
cd /tmp
ulimit -f 65536
: > /tmp/model-tests.log
/opt/rar-toolchain/bin/rustc --edition 2024 -C opt-level=1 -C strip=symbols \
    -C debuginfo=0 /packager.rs -o /tmp/rar-image
/tmp/rar-image /artifact/modern.efi /tmp/boot.img
printf 'RAR-MODERN-BOOT:'
base64 -w 0 /tmp/boot.img
printf '\n'
