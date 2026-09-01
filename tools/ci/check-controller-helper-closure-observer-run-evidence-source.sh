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
[ "$(sha "$contract")" = a0519b80bd85522b9119dcf6dc971f45d307b7d1b37cf0e04c84b93ae74e8978 ] || fail 'run-evidence contract bytes escaped review'
[ "$(sha "$case_contract")" = 6f7dd9821bf4894960238a6daf46cbb1992dee487ce4e0981f8999cb5dcc9107 ] || fail 'case-evidence contract bytes escaped review'
[ "$(sha "$valid")" = 0796af3de903218ea4b8def6d23291719f1d80bc023d94fa4e01374a82e15315 ] || fail 'valid fixture bytes escaped review'
[ "$(sha "$malformed")" = f94d7af08097852d06bc2c44b09a0862d4d569cc2b770e754ca8fcad12b2d21f ] || fail 'malformed fixture bytes escaped review'
[ "$(sha "$cases")" = f4536ed872bcc3ca60ea335c2675fe38241dd66e45062013f2c315dfa0e52386 ] || fail 'case table bytes escaped review'
[ "$(sha "$validator")" = f0e769c1fd17919f6e2634db071032a58e4c17f33ab33e5b4737fcd57336f510 ] || fail 'validator bytes escaped review'
[ "$(sha "$policy")" = f1d40ac202cba47486558da54e86ee9e1a2fc99d3e1659ccb57d1240829ee2e3 ] || fail 'mutation policy bytes escaped review'
grep -Fqx 'status=experimental-complete-C2A-source-only-no-workflow' "$contract" || fail 'run-evidence status changed'
grep -Fqx 'execution_authority=none;format-and-validator-only' "$contract" || fail 'run-evidence authority changed'
grep -Fqx 'line_count=31' "$contract" || fail 'run-evidence line count changed'
grep -Fqx 'record_digest_rule=record_sha256-is-SHA256-of-the-exact-first-30-lines-in-field-order-with-each-line-LF-terminated;line-31-is-excluded;no-alternate-serialization' "$contract" || fail 'record digest preimage changed'
grep -Fqx 'cryptographic_residual_rule=actual-SHA256-zero-for-derived-output-or-record-digest-is-unconstructible-for-fixture-testing;validator-source-must-explicitly-reject;two-source-proofs-required' "$contract" || fail 'zero-digest residual rule changed'
grep -Fqx 'output_set_rule=exactly-controller-helper-closure-observer.cases.v0+controller-helper-closure.sha256+controller-helper-closure.receipt+controller-helper-closure-observer-run-evidence.v0;regular+non-symlink+single-link+no-extra-entry' "$contract" || fail 'output-set rule changed'
grep -Fqx 'case_set=O001-through-O021-exactly-once-in-numeric-order' "$case_contract" || fail 'case set changed'
grep -Fqx 'exit_rule=O001:0;O002-through-O021:1-controller-normalized-rejection;no-other-value' "$case_contract" || fail 'case exit mapping changed'
[ "$(/usr/bin/wc -l < "$valid" | /usr/bin/tr -d ' ')" -eq 31 ] || fail 'valid fixture line count changed'
[ "$(/usr/bin/tail -c 1 "$valid" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'valid fixture lacks terminal LF'
record_sha=$(/usr/bin/sed -n '1,30p' "$valid" | /usr/bin/shasum -a 256 | /usr/bin/awk '{ print $1 }')
[ "$(/usr/bin/sed -n '31p' "$valid")" = "record_sha256=$record_sha" ] || fail 'valid fixture record digest invalid'
grep -Fqx 'run-id=12345' "$malformed" || fail 'malformed bad-key fixture changed'
grep -Fqx 'case_count=40' "$cases" || fail 'case table count changed'
[ "$(/usr/bin/grep -Ec '^V[0-9][0-9][0-9]\|' "$cases")" -eq 40 ] || fail 'case table incomplete'
/bin/sh -n "$validator" "$policy"
workflows=$root/.github/workflows
workflow=$workflows/controller-helper-closure-observer.yml
validator_reference=tools/ci/verify-controller-helper-closure-observer-run-evidence.sh
validator_stem=verify-controller-helper-closure-observer-run-evidence
policy_reference=test-controller-helper-closure-observer-run-evidence-policy.sh
count_workflow_references() {
    needle=$1
    /usr/bin/find "$workflows" -type f \( -name '*.yml' -o -name '*.yaml' \) \
        -exec /usr/bin/awk -v needle="$needle" 'index($0, needle) { count++ } END { print count + 0 }' {} \; |
        /usr/bin/awk '{ count += $1 } END { print count + 0 }'
}
validator_exact_count=$(count_workflow_references "$validator_reference")
validator_stem_count=$(count_workflow_references "$validator_stem")
policy_count=$(count_workflow_references "$policy_reference")
[ "$policy_count" -eq 0 ] || fail 'C2 mutation policy is wired to a workflow'
if [ -e "$workflow" ] || [ -L "$workflow" ]; then
    [ -f "$workflow" ] && [ ! -L "$workflow" ] && [ -s "$workflow" ] ||
        fail 'C2B workflow is missing, symbolic, or empty'
    [ "$validator_exact_count" -eq 1 ] || fail 'C2B validator binding is missing or duplicated'
    [ "$validator_stem_count" -eq 1 ] || fail 'C2B validator has a partial or duplicate workflow binding'
    /usr/bin/awk '
        $0 == "      - name: Run isolated harness and one candidate observation" {
            producer_step = NR
            producer_count++
        }
        $0 == "      - name: Independently validate exact candidate evidence" {
            validator_step = NR
            validator_step_count++
        }
        $0 == "          tools/ci/verify-controller-helper-closure-observer-run-evidence.sh \\" {
            validator_reference = NR
            validator_reference_count++
        }
        $0 == "      - name: Retain four validated candidate files" {
            retention_step = NR
            retention_count++
        }
        END {
            if (producer_count != 1 ||
                validator_step_count != 1 ||
                validator_reference_count != 1 ||
                retention_count != 1 ||
                !(producer_step < validator_step &&
                  validator_step < validator_reference &&
                  validator_reference < retention_step)) exit 1
        }
    ' "$workflow" || fail 'C2B validator is not exactly between producer termination and retention'
    phase=bound-to-one-reviewed-C2B-workflow
else
    [ "$validator_exact_count" -eq 0 ] || fail 'dormant C2 validator is wired to a workflow'
    [ "$validator_stem_count" -eq 0 ] || fail 'dormant C2 validator has a partial workflow binding'
    phase=dormant-with-zero-workflow-bindings
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf 'controller-helper observer contracts are phase-aware, candidate-only, and %s\n' "$phase"
