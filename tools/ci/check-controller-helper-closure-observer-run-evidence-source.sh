#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=$root/spec/alpha/lab/controller-helper-closure-observer-run-evidence-v0.fields
case_contract=$root/tools/ci/contracts/controller-helper-closure-observer-case-evidence-v0.fields
fixtures=$root/tools/ci/fixtures/controller-helper-closure-observer
valid=$fixtures/run-evidence-valid.v0
malformed=$fixtures/run-evidence-malformed.v0
cases=$fixtures/run-evidence-cases.v0
validator=$root/tools/ci/verify-controller-helper-closure-observer-run-evidence.sh
policy=$root/tools/ci/test-controller-helper-closure-observer-run-evidence-policy.sh
fail() { printf 'controller-helper observer run-evidence source check failed: %s\n' "$1" >&2; exit 1; }
sha() { /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
for file in "$contract" "$case_contract" "$valid" "$malformed" "$cases" "$validator" "$policy"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "required file missing, symbolic, or empty: $file"
done
[ "$(sha "$contract")" = a480c927784adff7153ce8e0342db854186ac9da98aa5ce399de6ea9ba4c52f3 ] || fail 'run-evidence contract bytes escaped review'
[ "$(sha "$case_contract")" = 7a8a8a2e0c81b4f65994014e2e0d54db5dc0ef0426f8fdbc30924318b666e042 ] || fail 'case-evidence contract bytes escaped review'
[ "$(sha "$valid")" = 6affe5c08498084a5fe65b2f25f61a5528106252c8d76b621eb5cf5c8e0c89f3 ] || fail 'valid fixture bytes escaped review'
[ "$(sha "$malformed")" = b72db26431fbac9ef5ce4967579d0bf994ff210802013ef5171f5f2394dd04c1 ] || fail 'malformed fixture bytes escaped review'
[ "$(sha "$cases")" = b1af2c9f1a927c58d99b86a7ce4a118b36389a1408e533944d859b7bfa53156a ] || fail 'case table bytes escaped review'
[ "$(sha "$validator")" = 8539bf9428e1e7f5871be9909de0fbfdc8e8780862bdb3349059cd79bc356e73 ] || fail 'validator bytes escaped review'
[ "$(sha "$policy")" = b1ff5ea55b0bcdfcabd755d99bd54fe18fcb22e5ad5c24909553ee91e26e3752 ] || fail 'mutation policy bytes escaped review'
grep -Fqx 'status=experimental-complete-C2A-source-only-no-workflow' "$contract" || fail 'run-evidence status changed'
grep -Fqx 'execution_authority=none;format-and-validator-only' "$contract" || fail 'run-evidence authority changed'
grep -Fqx 'line_count=31' "$contract" || fail 'run-evidence line count changed'
grep -Fqx 'output_set_rule=exactly-controller-helper-closure-observer.cases.v0+controller-helper-closure.sha256+controller-helper-closure.receipt+controller-helper-closure-observer-run-evidence.v0;regular+non-symlink+single-link+no-extra-entry' "$contract" || fail 'output-set rule changed'
grep -Fqx 'case_set=O001-through-O021-exactly-once-in-numeric-order' "$case_contract" || fail 'case set changed'
[ "$(/usr/bin/wc -l < "$valid" | /usr/bin/tr -d ' ')" -eq 31 ] || fail 'valid fixture line count changed'
[ "$(/usr/bin/tail -c 1 "$valid" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'valid fixture lacks terminal LF'
record_sha=$(/usr/bin/sed -n '1,30p' "$valid" | /usr/bin/shasum -a 256 | /usr/bin/awk '{ print $1 }')
[ "$(/usr/bin/sed -n '31p' "$valid")" = "record_sha256=$record_sha" ] || fail 'valid fixture record digest invalid'
grep -Fqx 'status=ready' "$malformed" || fail 'malformed ready-substitution fixture changed'
grep -Fqx 'case_count=20' "$cases" || fail 'case table count changed'
[ "$(/usr/bin/grep -Ec '^V[0-9][0-9][0-9]\|' "$cases")" -eq 20 ] || fail 'case table incomplete'
/bin/sh -n "$validator" "$policy"
if /usr/bin/grep -R -Fq 'verify-controller-helper-closure-observer-run-evidence.sh' "$root/.github/workflows"; then fail 'C2A validator is wired to a workflow'; fi
if /usr/bin/grep -R -Fq 'test-controller-helper-closure-observer-run-evidence-policy.sh' "$root/.github/workflows"; then fail 'C2A mutation policy is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'controller-helper observer C2A contracts are complete, source-only, unwired, and candidate-only'
