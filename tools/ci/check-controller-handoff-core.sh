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
build-plan.v0
contract.rs
fixtures
fixtures/manifest-golden.v0.hex
lib.rs
manifest.rs
sha256.rs
transaction.rs'
actual=$(find "$tree" -mindepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || fail 'source tree allowlist mismatch'
find "$tree" -type l -print | /usr/bin/grep -q . && fail 'source tree contains a symbolic link'
for file in README.md build-plan.v0 contract.rs lib.rs manifest.rs sha256.rs transaction.rs fixtures/manifest-golden.v0.hex; do
    [ -f "$tree/$file" ] && [ ! -L "$tree/$file" ] && [ -s "$tree/$file" ] || fail "missing, symbolic, or empty source file: $file"
done

/usr/bin/grep -qx 'schema=rar-controller-handoff-build-plan-v0' "$plan" || fail 'build-plan schema mismatch'
/usr/bin/grep -qx 'rustc_channel=1.95.0' "$plan" || fail 'compiler channel mismatch'
/usr/bin/grep -qx 'rustc_identity=unavailable' "$plan" || fail 'unreviewed compiler identity became active'
/usr/bin/grep -qx 'source_modules=contract.rs,manifest.rs,sha256.rs,transaction.rs' "$plan" || fail 'source module set mismatch'
/usr/bin/grep -qx 'dependency_count=0' "$plan" || fail 'dependency count mismatch'
/usr/bin/grep -qx 'target_linked=false' "$plan" || fail 'target boundary mismatch'
/usr/bin/grep -qx 'test_execution=blocked-pending-reviewed-isolated-cloud-host-tool-identity' "$plan" || fail 'test execution state mismatch'
/usr/bin/grep -qx 'status=safe-transaction-source-only-no-linux-adapter' "$plan" || fail 'authority state mismatch'
[ "$(/usr/bin/grep -Rxc '#!\[forbid(unsafe_code)\]' "$tree"/*.rs | /usr/bin/awk -F : '{ total += $2 } END { print total + 0 }')" -eq 1 ] || fail 'unsafe-code prohibition is absent or duplicated'
golden=$tree/fixtures/manifest-golden.v0.hex
[ "$(/usr/bin/wc -c < "$golden" | /usr/bin/tr -d ' ')" -eq 513 ] || fail 'golden vector length mismatch'
/usr/bin/awk 'NR != 1 || length($0) != 512 || $0 !~ /^[0-9a-f]+$/ { bad=1 } END { exit bad ? 1 : 0 }' "$golden" || fail 'golden vector grammar mismatch'

printf '%s\n' 'controller handoff core source checks passed: local-execution=forbidden golden-bytes=256'
