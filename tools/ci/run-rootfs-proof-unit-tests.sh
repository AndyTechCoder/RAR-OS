#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
[ "${RAR_ROOTFS_PROOF_SOURCE_TESTS-}" = 1 ] || exit 1
[ "${GITHUB_ACTIONS-}" = true ] || exit 1
[ "${CI-}" = true ] || exit 1
[ "${RAR_CI_RUNNER_OS-}" = Linux ] || exit 1
[ "${RAR_CI_RUNNER_ARCH-}" = X64 ] || exit 1
[ -n "${RAR_EXPECTED_SOURCE_REVISION-}" ] || exit 1
[ "$(/usr/bin/git -C "$root" rev-parse HEAD)" = "$RAR_EXPECTED_SOURCE_REVISION" ] || exit 1
[ -d /build ] && [ ! -L /build ] || exit 1

rustc=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc
[ -x "$rustc" ] && [ ! -L "$rustc" ] || exit 1
/bin/sh "$root/tools/ci/check-rootfs-proof-source.sh" >/dev/null
"$rustc" --edition 2024 --test -D warnings -C debuginfo=0 \
    -o /build/rar-rootfs-proof-tests "$root/tools/rar-lab/rootfs-proof/lib.rs"
/build/rar-rootfs-proof-tests --test-threads=1
printf '%s\n' 'rootfs proof cloud-only unit tests passed'
