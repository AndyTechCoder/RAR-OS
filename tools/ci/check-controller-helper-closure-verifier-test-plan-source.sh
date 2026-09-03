#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
plan=$root/spec/alpha/lab/controller-helper-closure-verifier-test-plan-v0.fields

fail() {
    printf 'controller-helper closure verifier test-plan source check failed: %s\n' "$1" >&2
    exit 1
}

[ -f "$plan" ] && [ ! -L "$plan" ] || fail 'test plan is unavailable'
plan_sha=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$plan" | /usr/bin/awk '{ print $1 }')
[ "$plan_sha" = 5a8fadb329a9367b4f27711a9bed8c55db32f18e8f32d15289dcc505a8d99c26 ] || fail 'test-plan bytes escaped review'

for required in \
    'status=experimental-C3VA-candidate-source-only-unwired' \
    'execution_authority=none' \
    'coverage_status=complete-via-117-constructible-dispositions+37-executable-precedence+12-faults+43-reviewed-residual-proofs-in-controller-helper-closure-verifier-cases-v0+faults-v0.fields+evidence-v0.fields,still-not-an-acceptance-instance+not-ready-for-wiring' \
    'runtime_state=contracts+evidence-validator-policy-candidate-under-review;C3V-harness+run+fixture-image+tool-pin+candidate+evidence-instances-not-created' \
    'disposition_runtime_count=117' \
    'disposition_residual_count=30' \
    'precedence_runtime_count=37' \
    'precedence_residual_count=13' \
    'fault_runtime_count=12' \
    'runtime_case_count=166' \
    'residual_proof_count=43' \
    'logical_relationship_count=209' \
    'risk_note_case_count=42' \
    'risk_note_rule=case_01-through-case_42-are-nonexecutable-risk-grouping-not-runtime-case-identities' \
    'target_rule=no-rustc,no-cargo,no-linker,no-helper,no-target-build,no-firmware,no-QEMU,no-guest' \
    'publication_rule=no-GitHub-write,no-lock+inventory+profile+controller+gate+readiness-update' \
    'success_oracle=exit-0+empty-stderr+exact-canonical-31-line-receipt+all-false-fields+not-ready-status+no-other-output' \
    'acceptance_rule=blocked;complete-source-contracts-cannot-produce-an-acceptance-instance' \
    'validation_catalog=controller-helper-closure-verifier-validation-v0.fields+controller-helper-closure-verifier-errors-v0+controller-helper-closure-verifier-precedence-v0,inactive-source-only' \
    'input_domain=controller-helper-closure-verifier-input-domain-v0.fields,inactive-source-only,no-fixtures+cases+controller+execution-authority' \
    'coverage_gap=closed-by-controller-helper-closure-verifier-cases-v0+faults-v0.fields' \
    'precedence_status=37-executable-dual-invalid-oracles+13-catalog-only-residual-relations-bound-in-controller-helper-closure-verifier-cases-v0,not-runtime-tested+not-acceptance-evidence' \
    'evidence_status=lossless-bounded+validator-enforced-by-controller-helper-closure-verifier-evidence-v0.fields;C3V-runtime-instance+fixtures-still-required' \
    'activation_rule=exact-reviewed-validation-catalog+input-domain+117-constructible-disposition-cases+37-executable-runtime-precedence+12-fault-cases+43-reviewed-residual-proofs,evidence+normalized-verdict-contract+fixtures,controller-source,fixture-image,tool-pins,subject-pins,workflow-wiring,and-exact-main-validation-required-before-first-run' \
    'local_rule=plan-check-is-text+hash+limited-direct-nonwiring-only;never-run-verifier,test-controller,container,compiler,helper,target,VM,or-emulator-on-Mac'; do
    grep -Fqx "$required" "$plan" || fail "required invariant is missing: $required"
done

case_lines=$(grep -Ec '^case_[0-9][0-9]=' "$plan")
[ "$case_lines" -eq 42 ] || fail 'test-design case count is not exactly 42'
case_number=1
while [ "$case_number" -le 42 ]; do
    case_id=$(printf '%02d' "$case_number")
    grep -Eq "^case_${case_id}=[^:]+:[^:]+$" "$plan" || fail "case $case_id is missing or malformed"
    case_number=$((case_number + 1))
done
if grep -E '^case_[0-9][0-9]=.*(^|[-+])or([-+:]|$)' "$plan" >/dev/null; then
    fail 'case variants use ambiguous or semantics'
fi

[ ! -e "$root/tools/ci/run-controller-helper-closure-verifier-tests.sh" ] || fail 'an unreviewed executable test controller exists'
[ ! -e "$root/tools/ci/test-controller-helper-closure-verifier-runtime.sh" ] || fail 'an unreviewed runtime harness exists'
if grep -R -Fq 'controller-helper-closure-verifier-test-plan-v0' "$root/.github/workflows"; then
    fail 'inactive test plan is wired to GitHub Actions'
fi
if grep -R -Eq 'run-controller-helper-closure-verifier-tests|test-controller-helper-closure-verifier-runtime' "$root/.github/workflows"; then
    fail 'an unreviewed verifier test harness is wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'rust_toolchain_closure_manifest_sha256=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock contains an unreviewed closure digest'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier test design is complete, source-only, inactive, and directly unwired'
