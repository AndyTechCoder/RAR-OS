#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

budget_check=false
case "$root" in '/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha/'*) budget_check=true ;; esac
[ "$budget_check" = false ] || /bin/sh tools/ci/check-workspace-budget.sh >/dev/null
# Bound every individual disposable fixture produced by this phase to 64 MiB.
ulimit -f 131072

tools/ci/check-specs.sh
/bin/sh -n \
    tools/ci/check-sprint-static.sh \
    tools/ci/check-local-sprint-preflight.sh \
    tools/ci/report-sprint-alpha-gates.sh \
    tools/ci/check-sprint-alpha-gate-report-policy.sh \
    tools/ci/report-sprint-alpha-gates-v2.sh \
    tools/ci/check-sprint-alpha-gate-report-v2-policy.sh \
    tools/ci/test-sprint-alpha-gate-report-v2-policy.sh \
    tools/ci/classify-proposed-adr.sh \
    tools/ci/test-proposed-adr-classifier-policy.sh \
    tools/ci/check-alpha-preimplementation-contracts.sh \
    tools/ci/check-acceptance-v2.sh \
    tools/ci/verify-accepted-evidence-v0.sh \
    tools/ci/test-accepted-evidence-v0-policy.sh \
    tools/ci/test-alpha-preimplementation-contract-policy.sh \
    tools/ci/check-remote-sprint-preflight.sh \
    tools/ci/test-local-sprint-preflight-policy.sh \
    tools/ci/check-development-lab-profile.sh \
    tools/ci/test-development-lab-profile-policy.sh \
    tools/ci/check-development-lab-profile-v2.sh \
    tools/ci/test-development-lab-profile-v2-policy.sh \
    tools/ci/check-development-controller-v2.sh \
    tools/ci/test-development-controller-v2-policy.sh \
    tools/ci/check-controller-handoff-core.sh \
    tools/ci/check-controller-handoff-attempt-v0.sh \
    tools/ci/test-controller-handoff-attempt-v0-policy.sh \
    tools/ci/check-controller-helper-inventory-v0.sh \
    tools/ci/observe-controller-helper-closure.sh \
    tools/ci/check-controller-helper-closure-observer-source.sh \
    tools/ci/verify-controller-helper-closure-candidate.sh \
    tools/ci/check-controller-helper-closure-verifier-source.sh \
    tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh \
    tools/ci/check-controller-helper-closure-verifier-validation-source.sh \
    tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh \
    tools/ci/check-controller-helper-closure-verifier-case-dispositions-source.sh \
    tools/ci/check-controller-helper-closure-verifier-case-templates-source.sh \
    tools/ci/check-controller-helper-closure-verifier-operator-inventory-source.sh \
    tools/ci/check-controller-helper-closure-verifier-scalar-semantics-source.sh \
    tools/ci/check-controller-helper-closure-verifier-basic-filesystem-semantics-source.sh \
    tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh \
    tools/ci/test-controller-helper-inventory-v0-policy.sh \
    tools/ci/check-controller-helper-build-evidence-v0.sh \
    tools/ci/check-controller-helper-build-receipt-v0.sh \
    tools/ci/check-controller-helper-test-evidence-v0.sh \
    tools/ci/test-controller-helper-evidence-v0-policy.sh \
    tools/ci/check-reference-evidence-v0.sh \
    tools/ci/test-reference-evidence-v0-policy.sh \
    tools/ci/check-reference-verdict-v0.sh \
    tools/ci/test-reference-verdict-v0-policy.sh \
    tools/ci/test-release-0-reference-harness-policy.sh \
    tools/ci/test-specifications-authority-policy.sh \
    tools/ci/check-portable-stat-policy.sh \
    tools/ci/test-portable-stat-policy.sh \
    tools/ci/verify-remote-checkpoint.sh \
    tools/ci/test-remote-checkpoint-policy.sh \
    tools/ci/verify-frozen-artifact.sh \
    tools/ci/test-frozen-artifact-policy.sh \
    tools/ci/run-alpha-scenario.sh \
    tools/ci/check-alpha-dependencies.sh \
    tools/ci/test-alpha-dependency-policy.sh \
    tools/ci/check-alpha-crypto-references.sh \
    tools/ci/test-alpha-crypto-reference-policy.sh \
    tools/ci/check-trusted-launcher-policy.sh \
    tools/ci/test-trusted-launcher-policy.sh \
    tools/ci/verify-launch-evidence.sh \
    tools/ci/test-launch-evidence-policy.sh \
    tools/ci/wait-for-launch-release.sh \
    tools/ci/test-launch-handshake-policy.sh \
    tools/ci/prepare-launch-control.sh \
    tools/ci/check-workspace-budget.sh \
    tools/ci/check-workspace-budget-values.sh \
    tools/ci/require-ephemeral-policy-test-root.sh \
    tools/ci/check-ephemeral-policy-test-confinement.sh \
    tools/ci/run-ephemeral-policy-tests.sh \
    tools/ci/run-qmp-client-unit-tests.sh \
    tools/ci/test-workspace-budget-policy.sh \
    tools/ci/verify-pinned-file.sh \
    tools/ci/test-pinned-file-policy.sh \
    tools/ci/check-qmp-client-contract.sh \
    tools/ci/check-qmp-client-source.sh \
    tools/ci/test-qmp-client-source-policy.sh \
    tools/ci/check-development-image-inputs.sh \
    tools/ci/check-development-image-sources.sh \
    tools/ci/check-containerfile-static-policy.sh \
    tools/ci/test-development-image-policy.sh \
    tools/ci/hash-source-tree.sh \
    tools/ci/run-development-probe.sh \
    tools/ci/run-cloud-target-probe.sh \
    tools/ci/launch-cloud-target.sh \
    tools/ci/verify-cloud-target-tools.sh \
    tools/ci/development-probe-status.sh \
    tools/ci/test-development-probe-policy.sh \
    tools/rarbuild/rarbuild \
    tools/rarbuild/bootstrap-lib.sh \
    tests/host-safety/run.sh \
    tests/bootstrap/run.sh \
    spec/fixtures/release-0/generate.sh \
    spec/fixtures/release-0/run.sh \
    sdk/generated/release-0/generate.sh \
    sdk/generated/release-0/check.sh

tools/ci/test-development-probe-policy.sh
/bin/sh tools/ci/check-ephemeral-policy-test-confinement.sh
/bin/sh tools/ci/check-sprint-alpha-gate-report-policy.sh
/bin/sh tools/ci/check-sprint-alpha-gate-report-v2-policy.sh
/bin/sh tools/ci/test-proposed-adr-classifier-policy.sh
/bin/sh tools/ci/check-alpha-preimplementation-contracts.sh
/bin/sh tools/ci/check-acceptance-v2.sh
/bin/sh tools/ci/check-controller-helper-closure-observer-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-validation-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-case-dispositions-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-case-templates-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-operator-inventory-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-scalar-semantics-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-basic-filesystem-semantics-source.sh
/bin/sh tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh
/bin/sh tools/ci/test-controller-handoff-attempt-v0-policy.sh
/bin/sh tools/ci/test-remote-checkpoint-policy.sh
/bin/sh tools/ci/test-workspace-budget-policy.sh
printf '%s\n' 'mutation policy evidence: external read-only-source CI step required'

[ "$budget_check" = false ] || /bin/sh tools/ci/check-workspace-budget.sh >/dev/null

echo "sprint static checks passed"
