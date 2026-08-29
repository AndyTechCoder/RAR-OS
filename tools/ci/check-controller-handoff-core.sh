#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
tree=$root/tools/rar-lab/controller-handoff
plan=$tree/build-plan.v0
fail() { printf 'controller handoff core rejected: %s\n' "$1" >&2; exit 1; }

[ -d "$tree" ] && [ ! -L "$tree" ] || fail 'source tree is missing or symbolic'
expected='README.md
attempt.rs
build-plan.v0
contract.rs
fixtures
fixtures/active-header-prehash.v0.hex
fixtures/manifest-golden.v0.hex
fixtures/recovery-header-prehash.v0.hex
fixtures/transition-prehash.v0.hex
lib.rs
linux.rs
manifest.rs
sha256.rs
transaction.rs'
actual=$(find "$tree" -mindepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || fail 'source tree allowlist mismatch'
find "$tree" -type l -print | /usr/bin/grep -q . && fail 'source tree contains a symbolic link'
for file in README.md attempt.rs build-plan.v0 contract.rs lib.rs linux.rs manifest.rs sha256.rs transaction.rs fixtures/active-header-prehash.v0.hex fixtures/manifest-golden.v0.hex fixtures/recovery-header-prehash.v0.hex fixtures/transition-prehash.v0.hex; do
    [ -f "$tree/$file" ] && [ ! -L "$tree/$file" ] && [ -s "$tree/$file" ] || fail "missing, symbolic, or empty source file: $file"
done

/usr/bin/grep -qx 'schema=rar-controller-handoff-build-plan-v0' "$plan" || fail 'build-plan schema mismatch'
/usr/bin/grep -qx 'rustc_channel=1.95.0' "$plan" || fail 'compiler channel mismatch'
/usr/bin/grep -qx 'rustc_identity=unavailable' "$plan" || fail 'unreviewed compiler identity became active'
/usr/bin/grep -qx 'source_modules=attempt.rs,contract.rs,linux.rs,manifest.rs,sha256.rs,transaction.rs' "$plan" || fail 'source module set mismatch'
/usr/bin/grep -qx 'dependency_count=0' "$plan" || fail 'dependency count mismatch'
/usr/bin/grep -qx 'target_linked=false' "$plan" || fail 'target boundary mismatch'
/usr/bin/grep -qx 'test_execution=blocked-pending-reviewed-isolated-cloud-host-tool-identity' "$plan" || fail 'test execution state mismatch'
/usr/bin/grep -qx 'status=linux-adapter+attempt-codec-source-only-no-executable' "$plan" || fail 'authority state mismatch'
/usr/bin/grep -qx '#!\[deny(unsafe_code)\]' "$tree/lib.rs" || fail 'crate unsafe-code denial is absent'
unsafe_files=$(/usr/bin/grep -El 'unsafe \{|unsafe extern' "$tree"/*.rs | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort)
[ "$unsafe_files" = linux.rs ] || fail 'unsafe operations escaped the sole Linux adapter boundary'
/usr/bin/grep -Fq '#[allow(unsafe_code)]' "$tree/lib.rs" || fail 'Linux adapter unsafe exception is not explicit'
/usr/bin/grep -Fq 'syscall 217 is getdents64' "$tree/linux.rs" || fail 'pinned getdents64 ABI invariant is missing'
/usr/bin/grep -Fq 'No other actor may mutate them during a transaction' "$tree/linux.rs" || fail 'cleanup exclusivity invariant is missing'
/usr/bin/grep -Fq 'pub(crate) fn from_verified_owned_fd' "$tree/linux.rs" || fail 'root descriptor adoption is not crate-confined'
! /usr/bin/grep -Eq 'pub fn .*([Pp]ath|[Ff]lag|[Rr]aw)' "$tree/linux.rs" || fail 'adapter exposes path, flag, or raw descriptor authority'
! /usr/bin/grep -Eq 'use std::(fs|process|net|env)|std::(fs|process|net|env)::|Command::|TcpStream|UdpSocket|openat\(|unlinkat\(' "$tree/attempt.rs" || fail 'attempt codec acquired host authority'
/usr/bin/grep -Fq 'pub const ACTIVE_HEADER_BYTES: usize = 512;' "$tree/attempt.rs" || fail 'active header bound mismatch'
/usr/bin/grep -Fq 'pub const TRANSITION_BYTES: usize = 512;' "$tree/attempt.rs" || fail 'transition bound mismatch'
/usr/bin/grep -Fq 'pub const RECOVERY_HEADER_BYTES: usize = 256;' "$tree/attempt.rs" || fail 'recovery header bound mismatch'
! /usr/bin/grep -Fq 'pub fn validate_after' "$tree/attempt.rs" || fail 'unreviewed transition authorization API is present'
! /usr/bin/grep -Fq 'pub fn validate_origin' "$tree/attempt.rs" || fail 'unreviewed inventory authorization API is present'
golden=$tree/fixtures/manifest-golden.v0.hex
[ "$(/usr/bin/wc -c < "$golden" | /usr/bin/tr -d ' ')" -eq 513 ] || fail 'golden vector length mismatch'
/usr/bin/awk 'NR != 1 || length($0) != 512 || $0 !~ /^[0-9a-f]+$/ { bad=1 } END { exit bad ? 1 : 0 }' "$golden" || fail 'golden vector grammar mismatch'
for vector in active-header-prehash.v0.hex transition-prehash.v0.hex; do
    [ "$(/usr/bin/wc -c < "$tree/fixtures/$vector" | /usr/bin/tr -d ' ')" -eq 1025 ] || fail "$vector length mismatch"
    /usr/bin/awk 'NR != 1 || length($0) != 1024 || $0 !~ /^[0-9a-f]+$/ { bad=1 } END { exit bad ? 1 : 0 }' "$tree/fixtures/$vector" || fail "$vector grammar mismatch"
done
[ "$(/usr/bin/wc -c < "$tree/fixtures/recovery-header-prehash.v0.hex" | /usr/bin/tr -d ' ')" -eq 513 ] || fail 'recovery header vector length mismatch'
/usr/bin/awk 'NR != 1 || length($0) != 512 || $0 !~ /^[0-9a-f]+$/ { bad=1 } END { exit bad ? 1 : 0 }' "$tree/fixtures/recovery-header-prehash.v0.hex" || fail 'recovery header vector grammar mismatch'
[ "$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$tree/fixtures/active-header-prehash.v0.hex" | /usr/bin/awk '{ print $1 }')" = 889f4bc32f4e4189d28f74a10638b52510dea42a6866dca7f48b5cb6bafbd0c9 ] || fail 'active header vector digest mismatch'
[ "$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$tree/fixtures/transition-prehash.v0.hex" | /usr/bin/awk '{ print $1 }')" = eff2a83c242d0ea19e82fc45b1a7f724be7d3d696ce3ae1ab8d5705360b56fd0 ] || fail 'transition header vector digest mismatch'
[ "$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$tree/fixtures/recovery-header-prehash.v0.hex" | /usr/bin/awk '{ print $1 }')" = 898d62e2f1514bec845e6d2aa8651a3cf637bdaf4e0a7200fc483cde431e16a8 ] || fail 'recovery header vector digest mismatch'

printf '%s\n' 'controller handoff core source checks passed: local-execution=forbidden structural-layouts=4 contextual-policy=absent'
