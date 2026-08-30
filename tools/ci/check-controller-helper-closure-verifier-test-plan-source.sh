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
[ "$plan_sha" = 6cd6c1824bc5a5bc8be7c398d4d36fec23019ac4c40b48376090de698c495ebe ] || fail 'test-plan bytes escaped review'

for required in \
    'status=experimental-incomplete-source-only' \
    'execution_authority=none' \
    'coverage_status=incomplete,not-an-acceptance-matrix,not-ready-for-controller-implementation-or-workflow-wiring' \
    'runtime_state=no-harness,no-run,no-fixture-image,no-tool-pin-instance,no-candidate,no-evidence' \
    'case_count=42' \
    'target_rule=no-rustc,no-cargo,no-linker,no-helper,no-target-build,no-firmware,no-QEMU,no-guest' \
    'publication_rule=no-GitHub-write,no-lock+inventory+profile+controller+gate+readiness-update' \
    'success_oracle=exit-0+empty-stderr+exact-canonical-31-line-receipt+all-false-fields+not-ready-status+no-other-output' \
    'acceptance_rule=blocked;this-incomplete-plan-cannot-produce-an-acceptance-verdict' \
    'validation_catalog=controller-helper-closure-verifier-validation-v0.fields+controller-helper-closure-verifier-errors-v0+controller-helper-closure-verifier-precedence-v0,inactive-source-only' \
    'precedence_status=source-order-only;constructible-dual-invalid-case+oracle-catalog-absent,not-runtime-tested,not-acceptance-evidence' \
    'evidence_status=absent,separate-versioned-canonical-case-evidence+normalized-verdict-contract+fixtures-required-before-controller-implementation' \
    'activation_rule=exact-reviewed-validation-catalog+separate-complete-explicit-case+constructible-runtime-precedence+input-domain+fault-contract,evidence+normalized-verdict-contract+fixtures,controller-source,fixture-image,tool-pins,subject-pins,workflow-wiring,and-exact-main-validation-required-before-first-run' \
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

printf '%s\n' 'controller-helper closure verifier test design is incomplete, inactive, and directly unwired'
