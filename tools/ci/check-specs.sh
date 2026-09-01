#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

fail() {
    echo "$1" >&2
    exit 1
}

required_files='README.md
BACKLOG.md
AGENTS.md
CONTRIBUTING.md
Cargo.toml
rust-toolchain.toml
rustfmt.toml
.editorconfig
.gitignore
.github/workflows/specifications.yml
.github/workflows/development-probe.yml
.codex/config.toml
.codex/rar-os-ssd-user-fragment.toml
.codex/rules/host-safety.rules
.codex/agents/architect.toml
.codex/agents/explorer.toml
.codex/agents/implementer.toml
.codex/agents/release_manager.toml
.codex/agents/reviewer.toml
.codex/agents/security_reviewer.toml
docs/README.md
docs/approval-record.md
docs/publication-record.md
docs/host-safety.md
docs/handoff-prompt.md
docs/v1-alpha-execution.md
docs/runbooks/github-actions-account-unblock.md
docs/security-remediation-status.md
docs/sprint-alpha.md
docs/sprint-alpha-dashboard.md
SPRINT_STATUS.md
docs/tasks/release-0.md
docs/tasks/sprint-alpha-vertical.md
docs/tasks/sprint-alpha-milestone-a-execution-map.md
docs/tasks/sprint-alpha-milestones-b-g-execution-map.md
docs/tasks/sprint-alpha-accepted-evidence-publication.md
docs/tasks/sprint-alpha-compact-bdf-integration.md
docs/tasks/sprint-alpha-controller-helper-integration.md
docs/tasks/sprint-alpha-controller-helper-c1-contracts.md
docs/tasks/sprint-alpha-controller-helper-c2-observer.md
docs/tasks/sprint-alpha-boot-platform-contract-integration.md
docs/adr/0022-alpha-graphics-input-authority.md
docs/adr/0023-alpha-boot-determinism-and-entry-state.md
docs/adr/0024-alpha-controller-helper-build-trust.md
docs/adr/0025-alpha-gui-continuity-evidence-sequencing.md
docs/adr/0026-alpha-platform-payload-and-state-sources.md
docs/adr/0027-alpha-bootstrap-retirement-and-dma-closure.md
docs/adr/0028-alpha-artifact-and-service-identities.md
docs/adr/0029-alpha-state-ticket-lifecycle.md
docs/adr/0031-alpha-compact-pci-bdf-encoding.md
docs/proposals/alpha-owner-choice-brief.md
docs/proposals/0022-alpha-graphics-input-authority.md
docs/proposals/0023-alpha-boot-determinism-and-entry-state.md
docs/proposals/0024-alpha-controller-helper-build-trust.md
docs/proposals/0025-alpha-gui-continuity-evidence-sequencing.md
docs/proposals/0026-alpha-platform-payload-and-state-sources.md
docs/proposals/0027-alpha-bootstrap-retirement-and-dma-closure.md
docs/proposals/0028-alpha-artifact-and-service-identities.md
docs/proposals/0029-alpha-state-ticket-lifecycle.md
docs/proposals/0030-alpha-accepted-evidence-publication-recovery.md
docs/proposals/0031-alpha-compact-pci-bdf-encoding.md
docs/proposals/alpha-boot-followup-choice-brief.md
docs/proposals/alpha-decision-integration-plan.md
docs/tasks/sprint-alpha-completion-evidence-map.md
spec/alpha/lab/README.md
spec/alpha/lab/development-lab-profile-v2.fields
spec/alpha/lab/image-inventory-v2.fields
spec/alpha/lab/crypto-reference-inventory-v2.fields
spec/alpha/lab/comparison-transcript-v0.fields
spec/alpha/lab/controller-state-machine-v0.fields
spec/alpha/lab/controller-handoff-v0.fields
spec/alpha/lab/controller-handoff-manifest-v0.fields
spec/alpha/lab/controller-handoff-cases.v0
spec/alpha/lab/controller-handoff-attempt-v0.fields
spec/alpha/lab/controller-handoff-attempt-cases.v0
spec/alpha/lab/controller-helper-inventory-v0.fields
spec/alpha/lab/controller-helper-closure-observation-v0.fields
spec/alpha/lab/controller-helper-closure-verification-v0.fields
spec/alpha/lab/controller-helper-closure-verifier-test-plan-v0.fields
spec/alpha/lab/controller-helper-closure-verifier-validation-v0.fields
spec/alpha/lab/controller-helper-closure-verifier-errors-v0
spec/alpha/lab/controller-helper-closure-verifier-precedence-v0
spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
spec/alpha/lab/controller-helper-closure-verifier-case-dispositions-v0
spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
spec/alpha/lab/controller-helper-closure-verifier-scalar-semantics-v0
spec/alpha/lab/controller-helper-closure-verifier-basic-filesystem-semantics-v0
spec/alpha/lab/controller-helper-closure-verifier-scalar-repair-semantics-v0
spec/alpha/lab/controller-helper-closure-verifier-synchronized-link-semantics-v0
spec/alpha/lab/controller-helper-closure-verifier-observation-repair-semantics-v0
spec/alpha/lab/controller-helper-closure-verifier-rebuild-observation-canonical-semantics-v0
spec/alpha/lab/controller-helper-build-evidence-v0.fields
spec/alpha/lab/controller-helper-build-receipt-v0.fields
spec/alpha/lab/controller-helper-test-evidence-v0.fields
spec/alpha/lab/controller-helper-cases.v0
spec/alpha/lab/controller-helper-runtime-v0.fields
spec/alpha/lab/controller-helper-runtime-cases.v0
spec/alpha/lab/controller-helper-closure-observer-test-v0.fields
spec/alpha/lab/controller-helper-closure-observer-run-evidence-v0.fields
spec/alpha/lab/controller-helper-closure-verifier-faults-v0.fields
spec/alpha/lab/controller-helper-closure-verifier-cases-v0
spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields
spec/alpha/lab/controller-helper-closure-acceptance-v0.fields
spec/alpha/lab/controller-helper-test-evidence-v1.fields
spec/alpha/lab/controller-helper-build-evidence-v1.fields
spec/alpha/lab/fixtures/controller-helper/build-evidence.v0
spec/alpha/lab/fixtures/controller-helper/build-1-receipt.v0
spec/alpha/lab/fixtures/controller-helper/build-2-receipt.v0
spec/alpha/lab/fixtures/controller-helper/build-1.log.v0
spec/alpha/lab/fixtures/controller-helper/build-2.log.v0
spec/alpha/lab/fixtures/controller-helper/build-plan.v0
spec/alpha/lab/fixtures/controller-helper/builder-inventory.v0
spec/alpha/lab/fixtures/controller-helper/compiler-closure.v0
spec/alpha/lab/fixtures/controller-helper/compiler.v0
spec/alpha/lab/fixtures/controller-helper/golden-vector.v0
spec/alpha/lab/fixtures/controller-helper/helper-build-1.v0
spec/alpha/lab/fixtures/controller-helper/helper-build-2.v0
spec/alpha/lab/fixtures/controller-helper/helper-final.v0
spec/alpha/lab/fixtures/controller-helper/runner-image.v0
spec/alpha/lab/fixtures/controller-helper/source-tree.v0
spec/alpha/lab/fixtures/controller-helper/test-cases.v0
spec/alpha/lab/fixtures/controller-helper/test-evidence.v0
spec/alpha/lab/fixtures/controller-helper/test.log.v0
spec/alpha/lab/reference-evidence-v0.fields
spec/alpha/lab/fixtures/controller-context.v0
spec/alpha/lab/fixtures/source-context.v0
spec/alpha/lab/fixtures/reference-inventory.v0
spec/alpha/lab/fixtures/reference-harness.v0
spec/alpha/lab/fixtures/comparison-transcript.v0
spec/alpha/lab/fixtures/comparison-evidence.v0
spec/alpha/lab/fixtures/reference-verdict-accepted.v0
spec/alpha/lab/fixtures/reference-verdict-not-required.v0
spec/alpha/lab/fixtures/generate.sh
spec/alpha/lab/cases.v0
spec/alpha/boot/README.md
spec/alpha/boot/alpha-boot-v0.fields
spec/alpha/boot/cases.v0
spec/alpha/evidence/README.md
spec/alpha/evidence/acceptance-v1.plan
spec/alpha/evidence/acceptance-v2.plan
spec/alpha/evidence/acceptance-v2.fields
spec/alpha/evidence/acceptance-v2-cases.v0
spec/alpha/evidence/acceptance-v2-selection-digests.v0
spec/alpha/evidence/accepted-evidence-v0.fields
spec/alpha/evidence/accepted-evidence-v0-cases.v0
docs/adr/0011-release-0-reproducibility-gate-phasing.md
docs/adr/0012-release-0-host-bootstrap-trust-and-snapshot.md
docs/release-0/build/prompt-4-remediation.md
docs/adr/0013-pre-copy-trust-and-mmio-authority.md
docs/adr/0014-hardware-binding-and-record-identity.md
docs/adr/0015-deterministic-validation-precedence.md
docs/adr/0016-release-0-entry-validation-and-authority-closure.md
docs/adr/0017-sprint-alpha-development-lab.md
docs/adr/0018-end-of-week-demonstrator.md
docs/adr/0019-alpha-layer-signing.md
docs/adr/0020-alpha-reference-oracle-isolation.md
docs/adr/0021-alpha-boot-payload-boundary.md
docs/release-0/contracts/README.md
spec/boot/handoff-v1.fields
spec/hardware/rhd-v1.fields
spec/fixtures/release-0/cases.v1
spec/fixtures/release-0/generate.sh
spec/fixtures/release-0/reference.rs
spec/fixtures/release-0/run.sh
spec/fixtures/release-0/validation-precedence.v1
spec/fixtures/release-0/conformance-scenarios.v1
sdk/generated/release-0/generate.sh
sdk/generated/release-0/check.sh
sdk/generated/release-0/lib.rs
tools/ci/check-specs.sh
tools/ci/check-specifications-authority.sh
tools/ci/check-sprint-static.sh
tools/ci/check-local-sprint-preflight.sh
tools/ci/check-local-readonly.sh
tools/ci/report-sprint-alpha-gates.sh
tools/ci/check-sprint-alpha-gate-report-policy.sh
tools/ci/contracts/sprint-alpha-gate-report-v2.fields
tools/ci/report-sprint-alpha-gates-v2.sh
tools/ci/check-sprint-alpha-gate-report-v2-policy.sh
tools/ci/test-sprint-alpha-gate-report-v2-policy.sh
tools/ci/classify-proposed-adr.sh
tools/ci/test-proposed-adr-classifier-policy.sh
tools/ci/check-alpha-preimplementation-contracts.sh
tools/ci/check-acceptance-v2.sh
tools/ci/verify-accepted-evidence-v0.sh
tools/ci/test-accepted-evidence-v0-policy.sh
tools/ci/test-alpha-preimplementation-contract-policy.sh
tools/ci/check-remote-sprint-preflight.sh
tools/ci/test-local-sprint-preflight-policy.sh
tools/ci/check-development-lab-profile.sh
tools/ci/test-development-lab-profile-policy.sh
tools/ci/check-development-lab-profile-v2.sh
tools/ci/test-development-lab-profile-v2-policy.sh
tools/ci/check-development-controller-v2.sh
tools/ci/test-development-controller-v2-policy.sh
tools/ci/check-controller-handoff-core.sh
tools/ci/check-controller-handoff-attempt-v0.sh
tools/ci/test-controller-handoff-attempt-v0-policy.sh
tools/ci/check-controller-helper-inventory-v0.sh
tools/ci/observe-controller-helper-closure.sh
tools/ci/check-controller-helper-closure-observer-source.sh
tools/ci/contracts/controller-helper-closure-observer-case-evidence-v0.fields
tools/ci/fixtures/controller-helper-closure-observer/run-evidence-valid.v0
tools/ci/fixtures/controller-helper-closure-observer/run-evidence-malformed.v0
tools/ci/fixtures/controller-helper-closure-observer/run-evidence-cases.v0
tools/ci/check-controller-helper-closure-observer-run-evidence-source.sh
tools/ci/verify-controller-helper-closure-observer-run-evidence.sh
tools/ci/test-controller-helper-closure-observer-run-evidence-policy.sh
tools/ci/verify-controller-helper-closure-candidate.sh
tools/ci/check-controller-helper-closure-verifier-source.sh
tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh
tools/ci/check-controller-helper-closure-verifier-validation-source.sh
tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh
tools/ci/check-controller-helper-closure-verifier-case-dispositions-source.sh
tools/ci/check-controller-helper-closure-verifier-case-templates-source.sh
tools/ci/check-controller-helper-closure-verifier-operator-inventory-source.sh
tools/ci/check-controller-helper-closure-verifier-scalar-semantics-source.sh
tools/ci/check-controller-helper-closure-verifier-basic-filesystem-semantics-source.sh
tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh
tools/ci/check-controller-helper-closure-verifier-synchronized-link-semantics-source.sh
tools/ci/check-controller-helper-closure-verifier-observation-repair-semantics-source.sh
tools/ci/check-controller-helper-closure-verifier-rebuild-observation-canonical-semantics-source.sh
tools/ci/test-controller-helper-inventory-v0-policy.sh
tools/ci/check-controller-helper-build-evidence-v0.sh
tools/ci/check-controller-helper-build-receipt-v0.sh
tools/ci/check-controller-helper-test-evidence-v0.sh
tools/ci/test-controller-helper-evidence-v0-policy.sh
tools/ci/check-controller-helper-runtime-source.sh
tools/ci/check-controller-helper-closure-observer-test-source.sh
tools/ci/check-controller-helper-closure-verifier-faults-source.sh
tools/ci/check-controller-helper-closure-verifier-cases-source.sh
tools/ci/check-controller-helper-closure-verifier-evidence-source.sh
tools/ci/check-controller-helper-closure-acceptance-source.sh
tools/ci/check-controller-helper-test-evidence-v1.sh
tools/ci/check-controller-helper-build-evidence-v1.sh
tools/ci/test-controller-helper-evidence-v1-policy.sh
tools/ci/check-reference-evidence-v0.sh
tools/ci/test-reference-evidence-v0-policy.sh
tools/ci/check-reference-verdict-v0.sh
tools/ci/test-reference-verdict-v0-policy.sh
tools/ci/test-release-0-reference-harness-policy.sh
tools/ci/test-specifications-authority-policy.sh
tools/ci/check-portable-stat-policy.sh
tools/ci/test-portable-stat-policy.sh
tools/ci/verify-remote-checkpoint.sh
tools/ci/test-remote-checkpoint-policy.sh
tools/ci/verify-frozen-artifact.sh
tools/ci/test-frozen-artifact-policy.sh
tools/ci/run-alpha-scenario.sh
tools/ci/check-alpha-dependencies.sh
tools/ci/test-alpha-dependency-policy.sh
tools/ci/check-alpha-crypto-references.sh
tools/ci/test-alpha-crypto-reference-policy.sh
tools/ci/check-trusted-launcher-policy.sh
tools/ci/test-trusted-launcher-policy.sh
tools/ci/verify-launch-evidence.sh
tools/ci/test-launch-evidence-policy.sh
tools/ci/wait-for-launch-release.sh
tools/ci/test-launch-handshake-policy.sh
tools/ci/prepare-launch-control.sh
tools/ci/check-workspace-budget.sh
tools/ci/check-workspace-budget-values.sh
tools/ci/require-ephemeral-policy-test-root.sh
tools/ci/check-ephemeral-policy-test-confinement.sh
tools/ci/run-ephemeral-policy-tests.sh
tools/ci/run-qmp-client-unit-tests.sh
tools/ci/run-rootfs-proof-unit-tests.sh
tools/ci/policy-test-modes.v0
tools/ci/test-workspace-budget-policy.sh
tools/ci/verify-pinned-file.sh
tools/ci/test-pinned-file-policy.sh
tools/ci/check-qmp-client-contract.sh
tools/ci/check-qmp-client-source.sh
tools/ci/test-qmp-client-source-policy.sh
tools/ci/check-rootfs-proof-source.sh
tools/ci/check-development-image-inputs.sh
tools/ci/check-development-image-sources.sh
tools/ci/check-containerfile-static-policy.sh
tools/ci/test-development-image-policy.sh
tools/ci/hash-source-tree.sh
tools/ci/run-development-probe.sh
tools/ci/run-cloud-target-probe.sh
tools/ci/launch-cloud-target.sh
tools/ci/verify-cloud-target-tools.sh
tools/ci/development-probe-status.sh
tools/ci/test-development-probe-policy.sh
tools/ci/check-host-policy.sh
tools/ci/test-host-policy.sh
tools/ci/fixtures/host-policy/README.md
tools/sprint-alpha/README.md
tools/sprint-alpha/development-lab-v1.env
tools/sprint-alpha/development-lab-v2.env
tools/sprint-alpha/development-controller-v2.plan
tools/sprint-alpha/controller-helper-v0.env
tools/sprint-alpha/x86_64-q35-v1.profile
tools/sprint-alpha/alpha-crypto-references-v1.env
tools/sprint-alpha/qmp-client-v1.env
tools/sprint-alpha/qmp-client-v1.md
tools/rar-lab/qmp-client/README.md
tools/rar-lab/qmp-client/build-plan.v1
tools/rar-lab/qmp-client/json.rs
tools/rar-lab/qmp-client/main.rs
tools/rar-lab/rootfs-proof/README.md
tools/rar-lab/rootfs-proof/build-plan.v0
tools/rar-lab/rootfs-proof/lib.rs
tools/rar-lab/controller-handoff/README.md
tools/rar-lab/controller-handoff/accepted_evidence.rs
tools/rar-lab/controller-handoff/attempt.rs
tools/rar-lab/controller-handoff/build-plan.v0
tools/rar-lab/controller-handoff/contract.rs
tools/rar-lab/controller-handoff/fixtures/accepted-evidence-golden-f.v0
tools/rar-lab/controller-handoff/fixtures/accepted-evidence-golden.v0
tools/rar-lab/controller-handoff/fixtures/active-header-prehash.v0.hex
tools/rar-lab/controller-handoff/fixtures/manifest-golden.v0.hex
tools/rar-lab/controller-handoff/fixtures/recovery-header-prehash.v0.hex
tools/rar-lab/controller-handoff/fixtures/transition-prehash.v0.hex
tools/rar-lab/controller-handoff/lib.rs
tools/rar-lab/controller-handoff/linux.rs
tools/rar-lab/controller-handoff/manifest.rs
tools/rar-lab/controller-handoff/sha256.rs
tools/rar-lab/controller-handoff/transaction.rs
tools/rar-lab/crypto-reference/README.md
tools/rar-lab/crypto-reference/libsodium-reference.c
tools/rar-lab/images/README.md
tools/rar-lab/images/build.Containerfile
tools/rar-lab/images/image-inputs-v1.env
tools/rar-lab/images/launch-base.Containerfile
tools/rar-lab/images/launch.Containerfile
tools/rarbuild/bootstrap-lib.sh
tools/rarbuild/contracts/rar-host-check-v2.fields
tools/rarbuild/contracts/rar-host-test-v2.fields
tools/rarbuild/contracts/rar-build-plan-v3.fields
tools/rarbuild/contracts/rar-image-plan-v3.fields
tools/rarbuild/contracts/rar-build-evidence-v3.fields
tools/toolchain/class-b-host-tools.v1
tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock
tools/toolchain/rust-host-closure.aarch64-apple-darwin.sha256
tools/toolchain/sdk-link-closure.aarch64-apple-darwin.sha256'

printf '%s\n' "$required_files" | while IFS= read -r file; do
    [ -f "$file" ] || fail "missing regular required file: $file"
    [ ! -L "$file" ] || fail "required file must not be a symbolic link: $file"
    [ -s "$file" ] || fail "empty required file: $file"
done

for script in tools/ci/check-specs.sh tools/ci/check-specifications-authority.sh tools/ci/test-specifications-authority-policy.sh tools/ci/check-sprint-static.sh tools/ci/check-local-sprint-preflight.sh tools/ci/check-remote-sprint-preflight.sh tools/ci/test-local-sprint-preflight-policy.sh tools/ci/check-alpha-preimplementation-contracts.sh tools/ci/check-acceptance-v2.sh tools/ci/verify-accepted-evidence-v0.sh tools/ci/test-accepted-evidence-v0-policy.sh tools/ci/test-alpha-preimplementation-contract-policy.sh tools/ci/check-development-lab-profile-v2.sh tools/ci/test-development-lab-profile-v2-policy.sh tools/ci/check-development-controller-v2.sh tools/ci/test-development-controller-v2-policy.sh tools/ci/check-controller-handoff-core.sh tools/ci/check-controller-handoff-attempt-v0.sh tools/ci/test-controller-handoff-attempt-v0-policy.sh tools/ci/check-controller-helper-inventory-v0.sh tools/ci/observe-controller-helper-closure.sh tools/ci/check-controller-helper-closure-observer-source.sh tools/ci/check-controller-helper-closure-observer-run-evidence-source.sh tools/ci/verify-controller-helper-closure-observer-run-evidence.sh tools/ci/test-controller-helper-closure-observer-run-evidence-policy.sh tools/ci/verify-controller-helper-closure-candidate.sh tools/ci/check-controller-helper-closure-verifier-source.sh tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh tools/ci/check-controller-helper-closure-verifier-validation-source.sh tools/ci/check-controller-helper-closure-verifier-case-dispositions-source.sh tools/ci/check-controller-helper-closure-verifier-case-templates-source.sh tools/ci/check-controller-helper-closure-verifier-operator-inventory-source.sh tools/ci/check-controller-helper-closure-verifier-scalar-semantics-source.sh tools/ci/check-controller-helper-closure-verifier-basic-filesystem-semantics-source.sh tools/ci/check-controller-helper-closure-verifier-synchronized-link-semantics-source.sh tools/ci/test-controller-helper-inventory-v0-policy.sh tools/ci/check-controller-helper-build-evidence-v0.sh tools/ci/check-controller-helper-build-receipt-v0.sh tools/ci/check-controller-helper-test-evidence-v0.sh tools/ci/test-controller-helper-evidence-v0-policy.sh tools/ci/check-controller-helper-runtime-source.sh tools/ci/check-controller-helper-closure-observer-test-source.sh tools/ci/check-controller-helper-closure-verifier-faults-source.sh tools/ci/check-controller-helper-closure-verifier-cases-source.sh tools/ci/check-controller-helper-closure-verifier-evidence-source.sh tools/ci/check-controller-helper-closure-acceptance-source.sh tools/ci/check-controller-helper-test-evidence-v1.sh tools/ci/check-controller-helper-build-evidence-v1.sh tools/ci/test-controller-helper-evidence-v1-policy.sh tools/ci/check-reference-evidence-v0.sh tools/ci/test-reference-evidence-v0-policy.sh tools/ci/check-reference-verdict-v0.sh tools/ci/test-reference-verdict-v0-policy.sh tools/ci/test-release-0-reference-harness-policy.sh tools/ci/check-portable-stat-policy.sh tools/ci/test-portable-stat-policy.sh tools/ci/check-containerfile-static-policy.sh tools/ci/run-development-probe.sh tools/ci/run-cloud-target-probe.sh tools/ci/launch-cloud-target.sh tools/ci/prepare-launch-control.sh tools/ci/wait-for-launch-release.sh tools/ci/check-workspace-budget.sh tools/ci/check-workspace-budget-values.sh tools/ci/require-ephemeral-policy-test-root.sh tools/ci/check-ephemeral-policy-test-confinement.sh tools/ci/run-qmp-client-unit-tests.sh tools/ci/run-rootfs-proof-unit-tests.sh tools/ci/check-rootfs-proof-source.sh tools/ci/verify-cloud-target-tools.sh tools/ci/development-probe-status.sh tools/ci/test-development-probe-policy.sh tools/ci/check-host-policy.sh tools/ci/test-host-policy.sh spec/alpha/lab/fixtures/generate.sh spec/fixtures/release-0/generate.sh spec/fixtures/release-0/run.sh sdk/generated/release-0/generate.sh sdk/generated/release-0/check.sh; do
    [ -x "$script" ] || fail "required script is not executable: $script"
done
[ -x tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh ] || \
    fail 'required script is not executable: tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh'
[ -x tools/ci/check-controller-helper-closure-verifier-observation-repair-semantics-source.sh ] || \
    fail 'required script is not executable: tools/ci/check-controller-helper-closure-verifier-observation-repair-semantics-source.sh'
[ -x tools/ci/check-controller-helper-closure-verifier-rebuild-observation-canonical-semantics-source.sh ] || \
    fail 'required script is not executable: tools/ci/check-controller-helper-closure-verifier-rebuild-observation-canonical-semantics-source.sh'
[ -x tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh ] || \
    fail 'required script is not executable: tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh'

tools/ci/check-portable-stat-policy.sh

grep -qx 'Status: Owner-approved execution contract — 2026-08-25' docs/tasks/sprint-alpha-vertical.md || fail "Sprint Alpha vertical packet is not approved"
for policy_file in AGENTS.md docs/host-safety.md; do
    for directive in \
        '- No-deletion scope: files, directories, scratch, artifacts, and worktrees.' \
        '- No-overwrite scope: moving or copying over an existing path is forbidden.' \
        '- Duration: this remains in force after merge until explicitly lifted by the owner.' \
        '- Future removal rule: after an explicit lift, only one exact registered worktree may be removed after clean pushed commits, exact remote merge verification, and separate review.'; do
        [ "$(grep -Fxc -- "$directive" "$policy_file")" -eq 1 ] || fail "no-deletion directive is missing, duplicated, or altered in $policy_file: $directive"
    done
done
[ "$(sed -n '1p' spec/alpha/evidence/acceptance-v1.plan)" = schema=rar-alpha-acceptance-plan-v1 ] || fail "Alpha evidence protocol schema is invalid"
[ "$(awk -F '|' '!/^#/ && !/^schema=/ && NF { count++; if (NF != 5 || $1 !~ /^[A-G]$/ || $2 !~ /^(none|continue|key:[a-z0-9-]+|pointer:[0-9]+,[0-9]+,[0-9]+)$/ || $3 !~ /^[a-z0-9:-]+$/ || $4 !~ /^[a-z0-9-]+$/ || $5 !~ /^[01]$/) bad=1 } END { if (bad) exit 1; print count + 0 }' spec/alpha/evidence/acceptance-v1.plan)" -eq 45 ] || fail "Alpha evidence protocol is incomplete or malformed"
/bin/sh tools/ci/check-acceptance-v2.sh >/dev/null
/bin/sh tools/ci/check-development-lab-profile.sh >/dev/null
/bin/sh tools/ci/check-development-lab-profile-v2.sh >/dev/null
/bin/sh tools/ci/check-development-controller-v2.sh >/dev/null
/bin/sh tools/ci/check-controller-handoff-core.sh >/dev/null
/bin/sh tools/ci/check-controller-handoff-attempt-v0.sh >/dev/null
/bin/sh tools/ci/test-controller-handoff-attempt-v0-policy.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-inventory-v0.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-observer-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-validation-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-case-dispositions-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-case-templates-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-operator-inventory-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-scalar-semantics-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-basic-filesystem-semantics-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-synchronized-link-semantics-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-observation-repair-semantics-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-rebuild-observation-canonical-semantics-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-runtime-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-observer-test-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-observer-run-evidence-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-faults-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-cases-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-verifier-evidence-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-closure-acceptance-source.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-test-evidence-v1.sh >/dev/null
/bin/sh tools/ci/check-controller-helper-build-evidence-v1.sh >/dev/null
printf '%s\n' 'C1 evidence mutation policy: external read-only-source CI run required'
/bin/sh tools/ci/check-controller-helper-build-evidence-v0.sh spec/alpha/lab/fixtures/controller-helper/build-evidence.v0 . adr-0024-alternative-a runner-closure 1111111111111111111111111111111111111111 spec/alpha/lab/fixtures/controller-helper/runner-image.v0 spec/alpha/lab/fixtures/controller-helper/source-tree.v0 spec/alpha/lab/fixtures/controller-helper/build-plan.v0 spec/alpha/lab/fixtures/controller-helper/golden-vector.v0 spec/alpha/lab/fixtures/controller-helper/builder-inventory.v0 spec/alpha/lab/fixtures/controller-helper/compiler-closure.v0 spec/alpha/lab/fixtures/controller-helper/compiler.v0 spec/alpha/lab/fixtures/controller-helper/helper-build-1.v0 spec/alpha/lab/fixtures/controller-helper/helper-build-2.v0 spec/alpha/lab/fixtures/controller-helper/helper-final.v0 spec/alpha/lab/fixtures/controller-helper/build-1-receipt.v0 spec/alpha/lab/fixtures/controller-helper/build-2-receipt.v0 spec/alpha/lab/fixtures/controller-helper/build-1.log.v0 spec/alpha/lab/fixtures/controller-helper/build-2.log.v0 spec/alpha/lab/fixtures/controller-helper/test-evidence.v0 spec/alpha/lab/fixtures/controller-helper/test-cases.v0 spec/alpha/lab/fixtures/controller-helper/test.log.v0 >/dev/null
/bin/sh tools/ci/check-reference-evidence-v0.sh spec/alpha/lab/fixtures/comparison-evidence.v0 spec/alpha/lab/fixtures/comparison-transcript.v0 spec/alpha/lab/fixtures/reference-inventory.v0 spec/alpha/lab/fixtures/reference-harness.v0 >/dev/null
/bin/sh tools/ci/check-reference-verdict-v0.sh spec/alpha/lab/fixtures/reference-verdict-accepted.v0 milestone-f spec/alpha/lab/fixtures/controller-context.v0 spec/alpha/lab/fixtures/source-context.v0 spec/alpha/lab/fixtures/comparison-transcript.v0 spec/alpha/lab/fixtures/reference-inventory.v0 spec/alpha/lab/fixtures/comparison-evidence.v0 spec/alpha/lab/fixtures/reference-harness.v0 >/dev/null
/bin/sh tools/ci/check-reference-verdict-v0.sh spec/alpha/lab/fixtures/reference-verdict-not-required.v0 milestone-a spec/alpha/lab/fixtures/controller-context.v0 spec/alpha/lab/fixtures/source-context.v0 spec/alpha/lab/fixtures/comparison-transcript.v0 none none none >/dev/null
/bin/sh tools/ci/check-sprint-alpha-gate-report-policy.sh >/dev/null
historical_gate_report_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 \
    tools/ci/report-sprint-alpha-gates.sh | /usr/bin/awk '{ print $1 }')
[ "$historical_gate_report_sha" = 78782b52f2b063c0dd56f3778e17242dc21088729da7f075a2bc74ab038fa8f8 ] || \
    fail "historical gate-report v1 bytes changed"
if /bin/sh tools/ci/report-sprint-alpha-gates.sh >/dev/null 2>&1; then
    fail "historical gate-report v1 remains active after ADR 0022-0026 acceptance"
fi
/bin/sh tools/ci/check-sprint-alpha-gate-report-v2-policy.sh >/dev/null
gate_report_v2=$(/bin/sh tools/ci/report-sprint-alpha-gates-v2.sh)
[ "$(printf '%s\n' "$gate_report_v2" | grep -Fxc 'schema=rar-sprint-alpha-gate-report-v2')" -eq 1 ] || fail "gate-report v2 schema is unavailable"
[ "$(printf '%s\n' "$gate_report_v2" | grep -Fxc 'adr_0026=accepted')" -eq 1 ] || fail "gate-report v2 does not classify canonical ADR 0026"
[ "$(printf '%s\n' "$gate_report_v2" | grep -Fxc 'platform_source_set=blocked')" -eq 1 ] || fail "gate-report v2 does not fail closed on unbound platform sources"
[ "$(printf '%s\n' "$gate_report_v2" | grep -Fxc 'acceptance_protocol_v2=reviewed-implementation-required')" -eq 1 ] || fail "gate-report v2 overstates acceptance v2 activation"
[ "$(printf '%s\n' "$gate_report_v2" | grep -Fxc 'overall=blocked')" -eq 1 ] || fail "gate-report v2 overstates Alpha readiness"
/bin/sh tools/ci/test-proposed-adr-classifier-policy.sh >/dev/null
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0022-alpha-graphics-input-authority.md 0022 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0022 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0023-alpha-boot-determinism-and-entry-state.md 0023 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0023 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0024-alpha-controller-helper-build-trust.md 0024 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0024 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0025-alpha-gui-continuity-evidence-sequencing.md 0025 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0025 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0026-alpha-platform-payload-and-state-sources.md 0026 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0026 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0027-alpha-bootstrap-retirement-and-dma-closure.md 0027 \
    docs/approval-record.md 'Alternative B')" = accepted ] || fail "ADR 0027 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0028-alpha-artifact-and-service-identities.md 0028 \
    docs/approval-record.md 'Alternative A')" = accepted ] || fail "ADR 0028 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0029-alpha-state-ticket-lifecycle.md 0029 \
    docs/approval-record.md 'Alternative B')" = accepted ] || fail "ADR 0029 decision state is inconsistent"
for proposal in \
    docs/proposals/0027-alpha-bootstrap-retirement-and-dma-closure.md \
    docs/proposals/0028-alpha-artifact-and-service-identities.md \
    docs/proposals/0029-alpha-state-ticket-lifecycle.md; do
    case "$proposal" in
        *0027-*) historical_sha=f6c9ac77a03cf4777698d21d2fdf4b8813c0bc6ac1a73e6c1af17e6ff35ef12e ;;
        *0028-*) historical_sha=c548f7b263075b5b3645fd12adeb7b3e72615e18e939219fbfad52df28932307 ;;
        *0029-*) historical_sha=ca5f797243b172eb1ea94e80a66e24a7ac430134eb1047e3c43e5b7ce5f1c864 ;;
        *) fail "unexpected historical proposal path: $proposal" ;;
    esac
    actual_historical_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$proposal" | /usr/bin/awk '{ print $1 }')
    [ "$actual_historical_sha" = "$historical_sha" ] || \
        fail "$proposal escaped its immutable historical byte boundary"
    grep -qx 'Status: Historical proposal — superseded on 2026-08-30' "$proposal" || \
        fail "$proposal regained non-historical status"
    grep -qx 'Decision: Undecided at proposal publication' "$proposal" || \
        fail "$proposal claims a retrospective proposal decision"
    grep -Fqx 'This file preserves the considered alternatives and is not an authority source.' "$proposal" || \
        fail "$proposal regained decision authority"
done
grep -Fqx 'Canonical decision: [ADR 0027](../adr/0027-alpha-bootstrap-retirement-and-dma-closure.md).' \
    docs/proposals/0027-alpha-bootstrap-retirement-and-dma-closure.md || fail "ADR 0027 historical proposal lost its canonical link"
grep -Fqx 'Canonical decision: [ADR 0028](../adr/0028-alpha-artifact-and-service-identities.md).' \
    docs/proposals/0028-alpha-artifact-and-service-identities.md || fail "ADR 0028 historical proposal lost its canonical link"
grep -Fqx 'Canonical decision: [ADR 0029](../adr/0029-alpha-state-ticket-lifecycle.md).' \
    docs/proposals/0029-alpha-state-ticket-lifecycle.md || fail "ADR 0029 historical proposal lost its canonical link"
grep -Fqx '`I approve ADR 0027 Alternative B, ADR 0028 Alternative A, and ADR 0029 Alternative B for experimental Alpha specification work under the documented safety limits.`' \
    docs/proposals/alpha-boot-followup-choice-brief.md || fail "Alpha boot follow-up exact owner-choice sentence drifted"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/proposals/0030-alpha-accepted-evidence-publication-recovery.md 0030 \
    docs/approval-record.md)" = owner-decision-required ] || fail "ADR 0030 proposal overstates authority"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0031-alpha-compact-pci-bdf-encoding.md 0031 \
    docs/approval-record.md 'Alternative A')" = accepted ] || fail "ADR 0031 canonical decision is not accepted"
grep -qx 'Status: Historical proposal — superseded on 2026-08-31' \
    docs/proposals/0031-alpha-compact-pci-bdf-encoding.md || fail "ADR 0031 proposal regained non-historical status"
grep -qx 'Decision: Undecided at proposal publication' \
    docs/proposals/0031-alpha-compact-pci-bdf-encoding.md || fail "ADR 0031 proposal claims a retrospective decision"
grep -Fqx 'Canonical decision: [ADR 0031](../adr/0031-alpha-compact-pci-bdf-encoding.md).' \
    docs/proposals/0031-alpha-compact-pci-bdf-encoding.md || fail "ADR 0031 historical proposal lost its canonical link"
grep -Fqx 'This file preserves the considered alternatives and is not an authority source.' \
    docs/proposals/0031-alpha-compact-pci-bdf-encoding.md || fail "ADR 0031 proposal regained decision authority"
historical_0031_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 \
    docs/proposals/0031-alpha-compact-pci-bdf-encoding.md | /usr/bin/awk '{ print $1 }')
[ "$historical_0031_sha" = e22b9fdc3b284fa4256922108572519bc547d4f8dd7a7f01b0aea3a82b2e13d3 ] || \
    fail "ADR 0031 historical proposal escaped its immutable byte boundary"
grep -Fqx '`I approve ADR 0031 Alternative A for experimental Alpha compact PCI BDF encoding under the documented safety limits.`' \
    docs/proposals/0031-alpha-compact-pci-bdf-encoding.md || fail "ADR 0031 exact owner-approval sentence drifted"
grep -qx 'Status: Owner-approved D0 integration — P0-A blocked until D0 exact-main validation' \
    docs/tasks/sprint-alpha-compact-bdf-integration.md || fail "ADR 0031 integration packet overstates authority"
compact_bdf_packet_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 \
    docs/tasks/sprint-alpha-compact-bdf-integration.md | /usr/bin/awk '{ print $1 }')
[ "$compact_bdf_packet_sha" = 6d85db4080166fb896874bf996cf0b8bbe5d147e267f26889fbaed71de531443 ] || \
    fail "ADR 0031 integration packet escaped its reviewed byte boundary"
grep -qx 'Status: Non-authoritative preparation — implementation remains blocked' \
    docs/tasks/sprint-alpha-milestone-a-execution-map.md || fail "Milestone A execution map overstates authority"
grep -qx 'Status: Non-authoritative preparation — implementation remains sequential and gated' \
    docs/tasks/sprint-alpha-milestones-b-g-execution-map.md || fail "Milestones B-G execution map overstates authority"
grep -qx 'Status: Non-authoritative preparation — ADR 0030 owner decision required' \
    docs/tasks/sprint-alpha-accepted-evidence-publication.md || fail "accepted-evidence publication packet overstates authority"
publication_packet_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 \
    docs/tasks/sprint-alpha-accepted-evidence-publication.md | /usr/bin/awk '{ print $1 }')
[ "$publication_packet_sha" = 1f3318b7d6df2fd31a918bada01ffcdad79b43515209f303df1e96af23a4a2d3 ] || \
    fail "accepted-evidence publication packet escaped its reviewed byte boundary"
grep -qx 'Status: Owner-approved D0 integration — P0 blocked until exact-main validation' \
    docs/tasks/sprint-alpha-boot-platform-contract-integration.md || fail "boot/platform integration packet overstates authority"
boot_platform_packet_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 \
    docs/tasks/sprint-alpha-boot-platform-contract-integration.md | /usr/bin/awk '{ print $1 }')
[ "$boot_platform_packet_sha" = 89494696da7100d8397b25bd450675732ee8a0f540b2cfbb23381a1f63147e4e ] || \
    fail "boot/platform integration packet escaped its reviewed byte boundary"
grep -qx 'Status: Non-authoritative integration plan — decisions accepted, activation gated' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha decision integration plan overstates authority"
grep -Fqx '## Gate 1 — before Milestone A implementation' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha decision Gate 1 is missing"
grep -Fqx '## Gate 2 — before Milestone B implementation' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha decision Gate 2 is missing"
grep -Fqx '## Gate 3 — before Milestone E graphics/input implementation' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha decision Gate 3 is missing"
grep -Fqx '## Implementation task transition' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation task transition is missing"
grep -Fq 'implementation task. After PR #7 is green, reviewed, merged, and verified—and' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition omits the PR #7 merge gate"
grep -Fq 'after every accepted-decision, contract, controller, Lab, SSD-profile, capacity,' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition omits cumulative gates"
grep -Fq 'and immutable-checkpoint precondition passes—the release driver creates the' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition omits the immutable-checkpoint gate"
grep -Fq 'packet-required fresh SSD worktree and `codex/sprint-alpha-vertical` branch from' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition omits the fresh vertical worktree"
grep -Fq 'the verified `main` merge. Exactly one Medium-effort writer task owns the active' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition omits verified main or one Medium writer"
grep -Fq 'milestone paths, and that task has no persistent goal.' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition permits a persistent goal"
grep -Fq 'No preparation task may carry unmerged state, inherited target authority, or a' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation transition permits inherited preparation state"
grep -Fq 'starts at the exact verified `main` merge with no inherited working diff.' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha implementation checklist permits an inherited diff"
grep -Fq "PR #7's exact merge and distinct green \`main\` workflow were" \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha transition omits completed PR #5 supersession ordering"
grep -Fq 'so PR #5 is now closed, unmerged, and superseded with' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha transition omits the verified PR #5 outcome"
grep -Fq 'PR #5 is never merged, rebased, or' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha transition permits PR #5 integration"
grep -Fq 'wholesale cherry-picked into the Sprint Alpha line' \
    docs/proposals/alpha-decision-integration-plan.md || fail "Alpha transition permits wholesale PR #5 cherry-picking"
grep -Fq 'Never merge PR #5 into the Alpha line.' BACKLOG.md || fail "Backlog permits PR #5 merge"
grep -Fq 'Record ADR 0025 Alternative B.' BACKLOG.md || fail "Backlog omits accepted ADR 0025"
grep -Fq 'Complete the reviewed protocol/controller v2 cutover before Milestone B.' BACKLOG.md || fail "Backlog does not bind ADR 0025 activation to pre-B"
grep -Fq "Record the owner's ADR 0023, ADR 0024, and ADR 0026 choices." BACKLOG.md || fail "Backlog omits the recorded pre-A decisions"
grep -Fq 'controller/helper evidence before Milestone A.' BACKLOG.md || fail "Backlog does not bind ADR 0023/0024/0026 activation to pre-A"
grep -Fq 'Record ADR 0022 Alternative C.' BACKLOG.md || fail "Backlog omits accepted ADR 0022"
grep -Fq 'Complete its reviewed peripheral-grant contract before Milestone E.' BACKLOG.md || fail "Backlog does not bind ADR 0022 activation to pre-E"
grep -qx 'Status: Non-authoritative preparation — no completion evidence exists yet' \
    docs/tasks/sprint-alpha-completion-evidence-map.md || fail "Alpha completion evidence map overstates readiness"
grep -qx 'Status: Explanatory only — not authority or completion evidence' \
    docs/sprint-alpha-dashboard.md || fail "Alpha dashboard overstates authority"
grep -Fqx -- '- Target implementation: 0 of 7 milestones (A–G).' \
    docs/sprint-alpha-dashboard.md || fail "Alpha dashboard overstates target implementation"
grep -Fqx -- '- Working bootable GUI: does not exist yet.' \
    docs/sprint-alpha-dashboard.md || fail "Alpha dashboard overstates GUI readiness"
grep -Fqx -- '- Sprint Alpha completion evidence: none.' \
    docs/sprint-alpha-dashboard.md || fail "Alpha dashboard overstates completion evidence"
grep -Fq 'A boot → B Nucleus → C components/IPC → D recovery → E GUI/apps → F signed updates → G retained proof' \
    docs/sprint-alpha-dashboard.md || fail "Alpha dashboard loses the sequential execution path"
[ "$(grep -Ec '^### [1-8]\. ' docs/tasks/sprint-alpha-completion-evidence-map.md)" -eq 8 ] || fail "Alpha completion evidence map must retain exactly eight items"
item=1
while [ "$item" -le 8 ]; do
    grep -Eq "^### $item\\. " docs/tasks/sprint-alpha-completion-evidence-map.md || fail "Alpha completion evidence item $item is missing"
    item=$((item + 1))
done
grep -Fq 'A guest marker is one correlated observation, never sufficient proof by' \
    docs/tasks/sprint-alpha-completion-evidence-map.md || fail "Alpha evidence map permits marker-only completion"
grep -Fq 'Anything less remains incomplete, regardless of the final guest marker.' \
    docs/tasks/sprint-alpha-completion-evidence-map.md || fail "Alpha evidence closure rule is missing"
grep -Fqx '`Approve ADR 0022 Alternative C, ADR 0023 Alternative C, ADR 0024 Alternative A, ADR 0025 Alternative B, and ADR 0026 Alternative C.`' \
    docs/proposals/alpha-owner-choice-brief.md || fail "Alpha owner approval sentence is missing or ambiguous"
grep -Fqx 'Status: Historical decision aid — canonical decisions recorded elsewhere' \
    docs/proposals/alpha-owner-choice-brief.md || fail "Alpha owner brief overstates current authority"
grep -Fq 'Root reads only the' docs/tasks/sprint-alpha-milestone-a-execution-map.md || fail "Milestone A map omits Root staging ownership"
grep -Fq 'fixed Recovery, Nucleus, Core-bootstrap, component-bundle, initial-system,' \
    docs/tasks/sprint-alpha-milestone-a-execution-map.md || fail "Milestone A map omits ADR 0026 source staging"
grep -Fq 'Recovery—not Nucleus—owns outer source-set' \
    docs/tasks/sprint-alpha-milestone-a-execution-map.md || fail "Milestone A map misassigns outer source validation"
grep -Fq 'Required exact observations: 11 rows' \
    docs/tasks/sprint-alpha-milestones-b-g-execution-map.md || fail "Milestone C evidence count disagrees with ADR 0025"
grep -Fq 'Required exact observations: 7 captured rows—GUI continuity' \
    docs/tasks/sprint-alpha-milestones-b-g-execution-map.md || fail "Milestone E evidence count disagrees with ADR 0025"
grep -Fq 'all 45 ordered acceptance rows (A:5, B:7, C:11,' \
    docs/tasks/sprint-alpha-milestones-b-g-execution-map.md || fail "Final evidence buckets disagree with ADR 0025"
/bin/sh tools/ci/check-alpha-preimplementation-contracts.sh >/dev/null
[ "$(sed -n '1p' tools/sprint-alpha/x86_64-q35-v1.profile)" = 'schema=rar-development-machine-profile-v1' ] || fail "Sprint Alpha machine profile schema is invalid"
grep -qx 'acceleration=tcg' tools/sprint-alpha/x86_64-q35-v1.profile || fail "Sprint Alpha machine profile must use software emulation"
for disabled_boundary in 'network=none' 'audio=none' 'host_sharing=none' 'passthrough=none'; do
    grep -qx "$disabled_boundary" tools/sprint-alpha/x86_64-q35-v1.profile || fail "Sprint Alpha machine profile boundary is missing: $disabled_boundary"
done

[ "$(sed -n '2,$p' spec/fixtures/release-0/cases.v1 | awk -F '|' 'NR > 1 { count++ } END { print count + 0 }')" -eq 23 ] || fail "R0-002 binary fixture manifest is incomplete"
[ "$(grep -c '^validation-predicate|' spec/boot/handoff-v1.fields)" -eq 37 ] || fail "R0-002 predicate table is incomplete"
[ "$(grep -c '^single|' spec/fixtures/release-0/validation-precedence.v1)" -eq 37 ] || fail "R0-002 focused precedence fixtures are incomplete"
[ "$(grep -c '^dual|' spec/fixtures/release-0/validation-precedence.v1)" -eq 36 ] || fail "R0-002 adjacent precedence fixtures are incomplete"
[ "$(grep -c '^security-dual|' spec/fixtures/release-0/validation-precedence.v1)" -eq 8 ] || fail "R0-002 security-sensitive precedence fixtures are incomplete"
[ "$(awk -F '|' 'NR > 2 && NF { count++ } END { print count + 0 }' spec/fixtures/release-0/conformance-scenarios.v1)" -eq 196 ] || fail "R0-002 architecture/provider conformance scenarios are incomplete"
printf '%s\n' \
    'spec/fixtures/release-0/run.sh --ci' \
    'sdk/generated/release-0/check.sh --compile' | while IFS= read -r command; do
    grep -Fq "$command" .github/workflows/specifications.yml || fail "R0-002 exact-head CI command is missing: $command"
done

check_markdown_links() {
    document=$1
    base=$(dirname -- "$document")

    sed -n 's/.*](\([^)]*\.md\)).*/\1/p' "$document" | while IFS= read -r target; do
        case "$target" in
            http://* | https://* | mailto:* | \#*)
                continue
                ;;
            /*)
                fail "absolute Markdown target in $document: $target"
                ;;
        esac

        target=${target%%#*}
        path="$base/$target"
        [ -f "$path" ] || fail "broken Markdown target in $document: $target"
        [ ! -L "$path" ] || fail "symbolic-link Markdown target in $document: $target"

        resolved_directory=$(CDPATH= cd -- "$(dirname -- "$path")" && pwd -P)
        resolved="$resolved_directory/$(basename -- "$path")"
        case "$resolved" in
            "$root" | "$root"/*) ;;
            *) fail "Markdown target resolves outside the repository in $document: $target" ;;
        esac

        [ -s "$resolved" ] || fail "empty Markdown target in $document: $target"
    done
}

check_markdown_links README.md
check_markdown_links docs/README.md

index_targets=$(sed -n 's/.*](\([^)]*\.md\)).*/\1/p' docs/README.md)
duplicates=$(printf '%s\n' "$index_targets" | sort | uniq -d)
[ -z "$duplicates" ] || fail "duplicate specification index target: $duplicates"

adr_files=$(sed -n 's/^- \[ADR [^]]*\](\(adr\/[^)]*\.md\))$/docs\/\1/p' docs/README.md)
adr_count=$(printf '%s\n' "$adr_files" | awk 'NF { count++ } END { print count + 0 }')
[ "$adr_count" -eq 30 ] || fail "expected exactly 30 indexed ADRs"

approval_date=$(sed -n 's/^Date: //p' docs/approval-record.md)
case "$approval_date" in
    [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;;
    *) fail "approval record has no unique ISO date" ;;
esac

grep -qx 'Status: Approved' docs/approval-record.md || fail "approval record status is not approved"
grep -qx 'Approval: approved' docs/approval-record.md || fail "approval statement is not approved"
grep -q '^Approver: .\+' docs/approval-record.md || fail "approval record has no approver"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0020-alpha-reference-oracle-isolation.md 0020 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0020 is not bound to owner approval"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/adr/0021-alpha-boot-payload-boundary.md 0021 \
    docs/approval-record.md)" = accepted ] || fail "ADR 0021 is not bound to owner approval"
for accepted_adr in 0022 0023 0024 0025 0026 0027 0028 0029; do
    accepted_path=$(printf 'docs/adr/%s-' "$accepted_adr")
    accepted_file=$(find docs/adr -maxdepth 1 -type f -name "${accepted_adr}-*.md")
    [ "$(printf '%s\n' "$accepted_file" | awk 'NF { count++ } END { print count + 0 }')" -eq 1 ] || fail "ADR $accepted_adr canonical file is not unique"
    case "$accepted_file" in "$accepted_path"*) ;; *) fail "ADR $accepted_adr canonical path is invalid" ;; esac
    [ "$(/bin/sh tools/ci/classify-proposed-adr.sh "$accepted_file" "$accepted_adr" docs/approval-record.md)" = accepted ] || fail "ADR $accepted_adr is not bound to owner approval"
done
grep -qx "Status: Gate 0 approved on $approval_date" docs/README.md || fail "index approval date disagrees with approval record"
grep -q "Gate 0 was approved on $approval_date" README.md || fail "root README approval date disagrees with approval record"
grep -qx 'Status: Draft PR open' docs/publication-record.md || fail "initial publication record status is inconsistent"
grep -q 'https://github.com/AndyTechCoder/RAR-OS/pull/1' docs/publication-record.md || fail "initial publication PR is not recorded"

approved_direction_files='docs/constitution.md
docs/glossary.md
docs/from-scratch-policy.md
docs/replaceability.md
docs/simplicity-principles.md
docs/release-roadmap.md
docs/tiers-and-profiles.md
docs/architecture.md
docs/security-and-recovery.md
docs/interfaces-and-formats.md
docs/rar-lab.md
docs/documentation-policy.md
docs/handoff.md'

printf '%s\n' "$approved_direction_files" | while IFS= read -r file; do
    grep -qx "Status: Gate 0 approved direction — $approval_date" "$file" || fail "Gate 0 status mismatch: $file"
done

grep -qx "Status: Ready — Gate 0 owner approval recorded $approval_date" docs/tasks/release-0.md || fail "Release 0 packet is not ready"
grep -qx 'Status: Approved for Prompt 2 after repository publication' docs/handoff-prompt.md || fail "handoff prompt status is inconsistent"
grep -qx 'Status: Approved for execution; begins after repository publication and GitHub authentication' docs/v1-alpha-execution.md || fail "execution runbook status is inconsistent"

for number in 0001 0002 0003 0004 0005 0006 0007 0008 0009 0010 0011 0012 0013 0014 0015 0016 0017 0018 0019 0020 0021; do
    matches=$(printf '%s\n' "$adr_files" | grep -c "/$number-")
    [ "$matches" -eq 1 ] || fail "expected one indexed ADR for $number"
done

grep -q 'ADRs 0001–0018' docs/tasks/release-0.md || fail "Release 0 approved ADR range is stale"
grep -q 'Build-plan and evidence schemas use version 3' docs/adr/0011-release-0-reproducibility-gate-phasing.md || fail "ADR 0011 build-plan/evidence schema version is stale"
for number in 0001 0002 0003 0004 0005 0006 0007 0008 0009 0010 0011 0012 0013 0014 0015 0016 0017 0018; do
    if grep -q "ADR $number" docs/tasks/release-0.md; then
        printf '%s\n' "$adr_files" | grep -q "/$number-" || fail "task-referenced ADR $number is not indexed and approved"
    fi
done

printf '%s\n' "$adr_files" | while IFS= read -r adr; do
    [ -s "$adr" ] || fail "missing or empty indexed ADR: $adr"
    case "$adr" in
        docs/adr/0013-* | docs/adr/0014-* | docs/adr/0015-* | docs/adr/0016-*) adr_approval_date=2026-07-17 ;;
        docs/adr/0017-*) adr_approval_date=2026-08-20 ;;
        docs/adr/0018-*) adr_approval_date=2026-08-25 ;;
        docs/adr/0019-* | docs/adr/0020-* | docs/adr/0021-*) adr_approval_date=2026-08-26 ;;
        docs/adr/0022-* | docs/adr/0023-* | docs/adr/0024-* | docs/adr/0025-* | docs/adr/0026-*) adr_approval_date=2026-08-29 ;;
        docs/adr/0027-* | docs/adr/0028-* | docs/adr/0029-*) adr_approval_date=2026-08-30 ;;
        docs/adr/0031-*) adr_approval_date=2026-08-31 ;;
        *) adr_approval_date=$approval_date ;;
    esac
    grep -qx "Status: Accepted — $adr_approval_date" "$adr" || fail "ADR status mismatch: $adr"

    for heading in \
        '## Context' \
        '## Decision drivers' \
        '## Considered options' \
        '## Decision' \
        '## Consequences' \
        '## Security and data impact' \
        '## Compatibility and migration' \
        '## Validation' \
        '## Replacement path'; do
        grep -qx "$heading" "$adr" || fail "missing $heading in $adr"
    done
done

for adr in docs/adr/*.md; do
    printf '%s\n' "$adr_files" | grep -qx "$adr" || fail "unindexed ADR file: $adr"
done

if grep -RInE '(^|[^A-Za-z])(TODO|TBD|FIXME)([^A-Za-z]|$)' README.md AGENTS.md CONTRIBUTING.md BACKLOG.md docs; then
    fail "unresolved handoff marker found"
fi

if grep -RInE 'pending (owner )?(review|approval)|explicit owner approval remains|Accepted for proposed handoff|Ready after Gate 0 owner approval' BACKLOG.md docs; then
    fail "stale pre-approval status found"
fi

if grep -nE '^- `\[[^x]\]` \*\*P0' BACKLOG.md; then
    fail "Gate 0 P0 backlog item does not use the complete status"
fi

if find . -path ./.git -prune -o -name '._*' -prune -o -path ./out -prune -o -path ./spec/fixtures/release-0/bin -prune -o -path ./spec/alpha/lab/fixtures/comparison-transcript.v0 -prune -o -path ./spec/alpha/lab/fixtures/comparison-evidence.v0 -prune -o -type f -exec grep -nHE '[[:blank:]]+$' {} +; then
    fail "trailing whitespace found"
fi

if find . -path ./.git -prune -o -name '._*' -prune -o -path ./out -prune -o -path ./spec/fixtures/release-0/bin -prune -o -path ./spec/alpha/lab/fixtures/comparison-transcript.v0 -prune -o -path ./spec/alpha/lab/fixtures/comparison-evidence.v0 -prune -o -type f -exec grep -nHE '^(<<<<<<<|=======|>>>>>>>)' {} +; then
    fail "merge-conflict marker found"
fi

checkout_use='        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1'
[ "$(grep -Fxc "$checkout_use" .github/workflows/specifications.yml)" -eq 3 ] || fail "Specifications workflow checkout identity/count is not exact"
[ "$(grep -Fxc "$checkout_use" .github/workflows/development-probe.yml)" -eq 2 ] || fail "Development Probe checkout identity/count is not exact"
grep -qx 'channel = "1.95.0"' rust-toolchain.toml || fail "Rust toolchain is not pinned to 1.95.0"
if ! grep -qx 'members = \[\]' Cargo.toml; then
    grep -qx 'alpha_workspace = true' Cargo.toml || fail 'nonempty workspace lacks the explicit Alpha marker'
    grep -q '^Status: Owner-approved execution contract' docs/tasks/sprint-alpha-vertical.md || fail 'Alpha workspace lacks its execution contract'
fi
/bin/sh tools/ci/check-alpha-dependencies.sh >/dev/null

crypto_refs=tools/sprint-alpha/alpha-crypto-references-v1.env
grep -qx 'schema=rar-alpha-crypto-reference-inventory-v1' "$crypto_refs" || fail 'Alpha crypto reference inventory schema invalid'
grep -qx 'reference_1=OpenSSL' "$crypto_refs" || fail 'first Alpha crypto reference is not fixed'
grep -qx 'version_1=3.0.13' "$crypto_refs" || fail 'OpenSSL reference version drifted'
grep -qx 'license_1=Apache-2.0' "$crypto_refs" || fail 'OpenSSL reference license missing'
grep -qx 'reference_2=libsodium' "$crypto_refs" || fail 'second Alpha crypto reference is not fixed'
grep -qx 'version_2=1.0.19' "$crypto_refs" || fail 'libsodium reference version drifted'
grep -qx 'license_2=ISC' "$crypto_refs" || fail 'libsodium reference license missing'
/bin/sh tools/ci/check-alpha-crypto-references.sh >/dev/null
/bin/sh tools/ci/check-qmp-client-contract.sh >/dev/null
/bin/sh tools/ci/check-qmp-client-source.sh >/dev/null
/bin/sh tools/ci/check-rootfs-proof-source.sh >/dev/null
/bin/sh tools/ci/check-development-image-sources.sh >/dev/null

class_b_inventory=tools/toolchain/class-b-host-tools.v1
[ "$(sed -n '1p' "$class_b_inventory")" = 'schema=rar-class-b-host-tool-inventory-v1' ] || fail "Class B inventory schema is invalid"
[ "$(sed -n '2p' "$class_b_inventory")" = 'id|platform|version|integrity|license|provenance|setup|status' ] || fail "Class B inventory header is invalid"
[ "$(sed -n '3,$p' "$class_b_inventory" | awk 'END { print NR + 0 }')" -eq 15 ] || fail "Class B inventory entry count is invalid"

class_b_ids='macos-sealed-bootstrap
macos-apple-git
macos-rust-toolchain
xcode-macos-sdk
rust-official-oci-image
ci-rust-toolchain
ci-dash
ci-coreutils
ci-grep
ci-gcc
ci-git
ci-linux-sysroot
actions-checkout
github-hosted-runner
github-runner-container-engine'
printf '%s\n' "$class_b_ids" | while IFS= read -r id; do
    [ "$(grep -c "^$id|" "$class_b_inventory")" -eq 1 ] || fail "Class B inventory ID is missing or duplicated: $id"
done

sed -n '3,$p' "$class_b_inventory" | while IFS='|' read -r id platform version integrity license provenance setup status extra; do
    [ -n "$id" ] && [ -n "$platform" ] && [ -n "$version" ] && [ -n "$integrity" ] && [ -n "$license" ] && [ -n "$provenance" ] && [ -n "$setup" ] && [ -n "$status" ] && [ -z "${extra-}" ] || fail "Class B inventory row is incomplete: $id"
    case "$id$platform$version$integrity$license$setup" in
        *[!A-Za-z0-9._:/+-]*) fail "Class B inventory row is not canonical: $id" ;;
    esac
    case "$provenance" in https://*) ;; *) fail "Class B provenance is not an HTTPS source: $id" ;; esac
    case "$status" in diagnostic-only | pinned-executable | pinned-orchestrator | external-attested-noncertifying) ;; *) fail "Class B inventory status is invalid: $id" ;; esac
done

grep -qx 'schema=rar-host-tool-manifest-v4' tools/toolchain/host-tools.manifest || fail "host tool manifest schema is stale"
grep -qx 'class_b_inventory=tools/toolchain/class-b-host-tools.v1' tools/toolchain/host-tools.manifest || fail "host tool manifest omits the Class B inventory"
grep -Eq '^class_b_inventory_sha256=[0-9a-f]{64}$' tools/toolchain/host-tools.manifest || fail "host tool manifest omits the Class B inventory digest"
grep -q 'f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' "$class_b_inventory" || fail "Class B inventory omits the OCI digest"
grep -q '3d3c42e5aac5ba805825da76410c181273ba90b1' "$class_b_inventory" || fail "Class B inventory omits the checkout action commit"
grep -q 'ubuntu-24.04-20260823.283.1' "$class_b_inventory" || fail "Class B inventory omits the runner image version"
grep -qx 'ci_checkout=actions-checkout-v7.0.1-git-sha1-3d3c42e5aac5ba805825da76410c181273ba90b1' \
    tools/toolchain/host-tools.manifest || fail "host tool manifest checkout identity is stale"

sha256_of() {
    if [ -x /usr/bin/sha256sum ]; then
        digest_output=$(LC_ALL=C /usr/bin/sha256sum "$1") || return 1
    elif [ -x /usr/bin/shasum ]; then
        digest_output=$(LC_ALL=C /usr/bin/shasum -a 256 "$1") || return 1
    else
        return 1
    fi
    digest=${digest_output%% *}
    [ "${#digest}" -eq 64 ] || return 1
    case "$digest" in *[!0-9a-f]*) return 1 ;; esac
    printf '%s\n' "$digest"
}

controller_helper_packet_sha256=$(sha256_of docs/tasks/sprint-alpha-controller-helper-integration.md) || fail "cannot hash ADR 0024 integration packet"
[ "$controller_helper_packet_sha256" = 5fcaf8652aa0ebf23f44b403176b9a1716f90456f798d39b708c612bac60724a ] || fail "ADR 0024 integration packet changed without review"
grep -Fqx 'Status: Authoritative preparation packet - execution remains phase-gated' docs/tasks/sprint-alpha-controller-helper-integration.md || fail "ADR 0024 integration packet status changed"
grep -Fqx 'This packet authorizes no RAR target build, image, VM, boot, launch, or Mac' docs/tasks/sprint-alpha-controller-helper-integration.md || fail "ADR 0024 packet lost target-execution denial"
grep -Fqx 'Any expansion beyond ADR 0024 Alternative A requires a new ADR and owner' docs/tasks/sprint-alpha-controller-helper-integration.md || fail "ADR 0024 packet lost owner-decision boundary"
grep -Fqx -- '- [Sprint Alpha ADR 0024 controller/helper integration packet](tasks/sprint-alpha-controller-helper-integration.md)' docs/README.md || fail "ADR 0024 integration packet is not indexed"
c1_packet_sha256=$(sha256_of docs/tasks/sprint-alpha-controller-helper-c1-contracts.md) || fail "cannot hash ADR 0024 C1 packet"
[ "$c1_packet_sha256" = d3c323ab5ddda959e7c7cda4d0ced3e3e7fe77ae0ba589b2b47ec0a109db78e5 ] || fail "ADR 0024 C1 packet changed without review"
grep -Fqx 'Status: Authoritative source-only child packet - implementation requires exact-main validation' docs/tasks/sprint-alpha-controller-helper-c1-contracts.md || fail "ADR 0024 C1 packet status changed"
grep -Fqx '`sprint-alpha-controller-helper-integration.md`. This child packet grants no' docs/tasks/sprint-alpha-controller-helper-c1-contracts.md || fail "ADR 0024 C1 packet lost authority denial"
grep -Fqx -- '- [Sprint Alpha ADR 0024 C1 contract-closure packet](tasks/sprint-alpha-controller-helper-c1-contracts.md)' docs/README.md || fail "ADR 0024 C1 packet is not indexed"
c2_packet_sha256=$(sha256_of docs/tasks/sprint-alpha-controller-helper-c2-observer.md) || fail "cannot hash ADR 0024 C2 packet"
[ "$c2_packet_sha256" = ad9aa51d523200ed20a606644e36dc8265ee35ecdc2203dc8afa59e88fa08486 ] || fail "ADR 0024 C2 packet changed without review"
grep -Fqx 'Status: Authoritative source-only C2 child packet - implementation requires exact-main validation' docs/tasks/sprint-alpha-controller-helper-c2-observer.md || fail "ADR 0024 C2 packet status changed"
grep -Fqx 'This packet itself grants no workflow execution,' docs/tasks/sprint-alpha-controller-helper-c2-observer.md || fail "ADR 0024 C2 packet lost authority denial"
grep -Fqx -- '- [Sprint Alpha ADR 0024 C2 observer-discovery packet](tasks/sprint-alpha-controller-helper-c2-observer.md)' docs/README.md || fail "ADR 0024 C2 packet is not indexed"
local_lock_sha256=$(sha256_of tools/toolchain/host-tools.lock) || fail "cannot hash local tool lock"
ci_lock_sha256=$(sha256_of tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock) || fail "cannot hash CI tool lock"
readonly_gate_sha256=$(sha256_of tools/ci/check-local-readonly.sh) || fail "cannot hash local read-only gate"
host_policy_checker_sha256=$(sha256_of tools/ci/check-host-policy.sh) || fail "cannot hash host-policy checker"
local_preflight_sha256=$(sha256_of tools/ci/check-local-sprint-preflight.sh) || fail "cannot hash local sprint preflight"
local_preflight_policy_test_sha256=$(sha256_of tools/ci/test-local-sprint-preflight-policy.sh) || fail "cannot hash local sprint preflight policy test"
[ "$local_lock_sha256" = f7e9baf24aaff9eaa2a2032cf0a9919568cca817d6b5d0c7e6891bce05ec979a ] || fail "local tool lock digest changed without bootstrap authority update"
[ "$ci_lock_sha256" = 6752b1b21ac8fa93a671ff9444173e4c3bbc4cdcbe4cf5cd39820371dc79aa24 ] || fail "CI tool lock digest changed without bootstrap authority update"
[ "$readonly_gate_sha256" = 3b6e3cb28802ea90e3b89760773c8ebe47acfc493669e076a7fcb1a72ad76666 ] || fail "local read-only gate changed without safety review"
[ "$host_policy_checker_sha256" = fd2cdbd6886c0beb85492b842a40791861bff0c6a5bab01d1fba40ae183d6a0e ] || fail "local read-only gate dependency changed without safety review"
[ "$local_preflight_sha256" = 6f18d4edfc1ccc35f62fa4702a5b877398f45e67c9acd67689949e2b41eb6334 ] || fail "local sprint preflight changed without safety review"
[ "$local_preflight_policy_test_sha256" = acc9cfb08c580cc6ba468e06b4e40edebfe7aec29043bd79ba66ea48199065f8 ] || fail "local sprint preflight policy test changed without safety review"
/bin/sh -n tools/ci/check-local-readonly.sh
grep -qx 'PATH=/usr/bin:/bin' tools/ci/check-local-readonly.sh || fail "local read-only gate does not pin PATH"
grep -qx 'git_bin=/usr/bin/git' tools/ci/check-local-readonly.sh || fail "local read-only gate does not pin Git"
grep -Fqx 'root=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")/../.." && pwd -P)' tools/ci/check-local-readonly.sh || fail "local read-only gate does not pin dirname"
grep -Fqx '    /usr/bin/env -i \' tools/ci/check-local-readonly.sh || fail "local read-only gate does not clear the Git environment"
grep -Fqx '        GIT_CONFIG_NOSYSTEM=1 \' tools/ci/check-local-readonly.sh || fail "local read-only gate permits system Git configuration"
grep -Fqx '        XDG_CONFIG_HOME=/nonexistent-rar-local-check-config \' tools/ci/check-local-readonly.sh || fail "local read-only gate permits user Git configuration"
grep -Fqx '        -C "$root" \' tools/ci/check-local-readonly.sh || fail "local read-only gate does not bind the repository root"
grep -qx 'PATH=/usr/bin:/bin' tools/ci/check-host-policy.sh || fail "host-policy checker does not pin PATH"
grep -Fq 'LC_ALL=C /usr/bin/sha256sum "$file" | /usr/bin/awk' tools/ci/check-host-policy.sh || fail "host-policy checker does not pin the Linux digest path"
grep -Fq 'LC_ALL=C /usr/bin/shasum -a 256 "$file" | /usr/bin/awk' tools/ci/check-host-policy.sh || fail "host-policy checker does not pin the Mac digest path"
grep -qx 'uname_bin=/usr/bin/uname' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight lost its absolute host-system probe"
grep -Fqx 'uname_system=$("$uname_bin" -s)' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight stopped deriving its host system from the absolute probe"
grep -Fqx '[ "$uname_system" = Darwin ] || fail '\''this check is only for the owner Mac'\''' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight lost its Mac-only boundary"
grep -Fq 'rev-parse --absolute-git-dir' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight does not confine the Git metadata directory"
grep -Fq 'rev-parse --path-format=absolute --git-common-dir' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight does not confine the common Git metadata directory"
grep -Fqx '    /usr/bin/env -i \' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight does not clear the Git environment"
grep -Fqx '        GIT_OPTIONAL_LOCKS=0 \' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight permits optional Git writes"
grep -qx 'minimum_ssd_free_kib=10485760' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight lost the 10-GiB SSD reserve"
for policy_script in \
    tools/ci/check-workspace-budget.sh \
    tools/ci/check-local-sprint-preflight.sh; do
    [ "$(grep -Fxc 'maximum_workspace_kib=9437184' "$policy_script")" -eq 1 ] ||
        fail "workspace ceiling is missing, duplicated, or inconsistent in $policy_script"
done
grep -qx 'minimum_free_kib=10485760' tools/ci/check-workspace-budget.sh || fail "workspace budget lost the 10-GiB SSD reserve"
grep -qx 'maximum_output_kib=524288' tools/ci/check-workspace-budget.sh || fail "workspace budget lost the 512-MiB output ceiling"
grep -Fqx '[ "$workspace_kib" -le "$maximum_workspace_kib" ] || exit 1' tools/ci/check-workspace-budget.sh || fail "workspace budget comparison changed"
grep -Fqx '[ "$workspace_kib" -le "$maximum_workspace_kib" ] ||' tools/ci/check-local-sprint-preflight.sh || fail "local preflight workspace comparison changed"
grep -Fq 'above 9 GiB total RAR OS workspace' AGENTS.md || fail "agent policy lost the 9-GiB workspace ceiling"
grep -Fq '10-GiB free, 9-GiB workspace, and 512-MiB output ceilings.' docs/host-safety.md || fail "host-safety policy lost the reviewed workspace limits"
grep -Fq '8 GiB (8388608 KiB) to 9 GiB (9437184 KiB)' docs/approval-record.md || fail "workspace-ceiling owner approval is not recorded"
if grep -Fq 'above 8 GiB total RAR OS workspace' AGENTS.md ||
    grep -Fq '10-GiB free, 8-GiB workspace' docs/host-safety.md; then
    fail "stale 8-GiB workspace authority remains"
fi
grep -Fqx 'ssd_free_kib=$(/bin/df -Pk "$safe_root" | /usr/bin/awk '\''END { print $4 }'\'')' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight SSD capacity probe changed"
grep -Fqx 'workspace_kib=$(/usr/bin/du -sk "$safe_root" | /usr/bin/awk '\''NR == 1 { print $1 }'\'')' tools/ci/check-local-sprint-preflight.sh || fail "local sprint preflight workspace capacity probe changed"
[ "$(grep -c 'df' tools/ci/check-local-sprint-preflight.sh)" -eq 1 ] || fail "local sprint preflight has an unreviewed capacity probe"
[ "$(grep -c '/usr/bin/du' tools/ci/check-local-sprint-preflight.sh)" -eq 1 ] || fail "local sprint preflight has an unreviewed workspace-size probe"
grep -Fq 'Internal-Mac capacity is' docs/tasks/sprint-alpha-vertical.md || fail "Sprint Alpha packet omits the SSD-only capacity clarification"
grep -Fq 'internal-Mac free space is not an Alpha' docs/approval-record.md || fail "owner SSD-only capacity clarification is not recorded"
grep -Fq 'usable SSD reserve/headroom' AGENTS.md || fail "agent preimplementation gate omits SSD headroom"
if grep -Fq 'internal-disk headroom' AGENTS.md; then
    fail "agent preimplementation gate retains stale internal-Mac headroom"
fi
grep -qx "macos_lock_sha256=$local_lock_sha256" tools/toolchain/host-tools.manifest || fail "host tool manifest local lock digest is stale"
grep -qx "ci_lock_sha256=$ci_lock_sha256" tools/toolchain/host-tools.manifest || fail "host tool manifest CI lock digest is stale"
if grep -q '^  runner_evidence:$' .github/workflows/specifications.yml ||
    grep -q '^    needs: runner_evidence$' .github/workflows/specifications.yml; then
    fail "CI runner evidence and validation must not be split across hosted runners"
fi
grep -q '^  validate:$' .github/workflows/specifications.yml || fail "same-runner CI validation job is missing"
grep -q 'name: attest-runner-and-validate' .github/workflows/specifications.yml || fail "same-runner attestation/validation identity is missing"
/usr/bin/awk '
    /^  validate:$/ {
        if (in_validate) bad=1
        in_validate=1
        validate_jobs++
        next
    }
    in_validate && /^  [A-Za-z0-9_-]+:$/ { in_validate=0 }
    in_validate && /^    timeout-minutes:/ {
        validate_timeouts++
        if ($0 == "    timeout-minutes: 30") approved_timeout++
        else bad=1
    }
    END {
        if (bad || validate_jobs != 1 || validate_timeouts != 1 || approved_timeout != 1) exit 1
    }
' .github/workflows/specifications.yml || fail "Specifications validate-job timeout is missing, misplaced, or ambiguous"
grep -Fq '[ "${ImageOS-}" = ubuntu24 ]' .github/workflows/specifications.yml || fail "CI runner image OS is not attested"
grep -Fq 'rar_ci_image_version=${ImageVersion-}' .github/workflows/specifications.yml || fail "CI runner image version is not attested"
for runner_handoff in \
    'RAR_CI_RUNNER_IMAGE_OS=$ImageOS' \
    'RAR_CI_RUNNER_IMAGE_VERSION=$ImageVersion' \
    'RAR_CI_RUNNER_OS=$RUNNER_OS' \
    'RAR_CI_RUNNER_ARCH=$RUNNER_ARCH'; do
    grep -Fq "$runner_handoff" .github/workflows/specifications.yml || fail "CI runner evidence handoff is incomplete: $runner_handoff"
done
grep -q 'docker run --rm --read-only' .github/workflows/specifications.yml || fail "same-runner pinned container launch is missing"
grep -q -- '--network none' .github/workflows/specifications.yml || fail "CI container network is not disabled"
grep -q '^concurrency:$' .github/workflows/specifications.yml || fail "required CI concurrency control is missing"
grep -q 'cancel-in-progress: true' .github/workflows/specifications.yml || fail "obsolete required CI runs are not cancelled"
grep -qx '  pull_request_target:' .github/workflows/specifications.yml || fail "Specifications PR controller is not loaded from the trusted base"
if grep -qx '  pull_request:' .github/workflows/specifications.yml; then
    fail "Specifications workflow permits proposal-controlled PR execution"
fi
if grep -q 'workflow_dispatch' .github/workflows/specifications.yml; then
    fail "Specifications workflow must not execute branch-selected workflow code"
fi
grep -Fqx "run-name: Specifications source \${{ github.event_name == 'pull_request_target' && github.event.pull_request.head.sha || github.sha }}" .github/workflows/specifications.yml || fail "Specifications run identity is not bound to the exact source SHA"
grep -Fqx "      RAR_TRUSTED_CONTROLLER_SHA: \${{ github.event_name == 'pull_request_target' && github.event.pull_request.base.sha || github.sha }}" .github/workflows/specifications.yml || fail "Specifications controller SHA is not exact"
grep -Fqx "      RAR_EXPECTED_SOURCE_REVISION: \${{ github.event_name == 'pull_request_target' && github.event.pull_request.head.sha || github.sha }}" .github/workflows/specifications.yml || fail "Specifications source SHA is not exact"
grep -Fqx "      RAR_EXPECTED_SOURCE_REPOSITORY: \${{ github.event_name == 'pull_request_target' && github.event.pull_request.head.repo.full_name || github.repository }}" .github/workflows/specifications.yml || fail "Specifications source repository is not exact"
grep -Fqx '          [ "$RAR_EXPECTED_SOURCE_REPOSITORY" = "$RAR_CANONICAL_REPOSITORY" ]' .github/workflows/specifications.yml || fail "Specifications workflow accepts non-canonical PR repositories"
[ "$(grep -Fxc '          path: controller' .github/workflows/specifications.yml)" -eq 1 ] || fail "Specifications trusted controller checkout is missing or ambiguous"
[ "$(grep -Fxc '          path: primary-source' .github/workflows/specifications.yml)" -eq 1 ] || fail "Specifications primary source checkout is missing or ambiguous"
[ "$(grep -Fxc '          path: mutation-source' .github/workflows/specifications.yml)" -eq 1 ] || fail "Specifications mutation source checkout is missing or ambiguous"
[ "$(grep -Fxc "        if: steps.authority.outputs.execution == 'full'" .github/workflows/specifications.yml)" -eq 2 ] || fail "Specifications executable phases are not authority-gated"
grep -Fq 'controller/tools/ci/check-specifications-authority.sh' .github/workflows/specifications.yml || fail "Specifications workflow does not invoke the trusted authority checker"
grep -Fqx '        GIT_CONFIG_NOSYSTEM=1 \' tools/ci/check-specifications-authority.sh || fail "Specifications authority checker permits system Git configuration"
grep -Fqx '        GIT_OPTIONAL_LOCKS=0 \' tools/ci/check-specifications-authority.sh || fail "Specifications authority checker permits optional Git writes"
grep -Fq 'ls-files -s -- "$@"' tools/ci/check-specifications-authority.sh || fail "Specifications authority checker does not compare tracked object identities"
grep -Fqx '    tools \' tools/ci/check-specifications-authority.sh || fail "Specifications authority closure omits repository tools"
grep -Fqx '    tests \' tools/ci/check-specifications-authority.sh || fail "Specifications authority closure omits executable test harnesses"
grep -Fq "\$1 !~ /^100(644|755)\$/" tools/ci/check-specifications-authority.sh || fail "Specifications authority checker permits symlink or submodule authority"
grep -Fqx "    /usr/bin/printf '%s\\n' 'execution=full' >> \"\$output\"" tools/ci/check-specifications-authority.sh || fail "Specifications authority checker cannot enable exact-closure execution"
grep -Fqx "    /usr/bin/printf '%s\\n' 'execution=deferred' >> \"\$output\"" tools/ci/check-specifications-authority.sh || fail "Specifications authority checker cannot defer controller changes"
grep -Fq 'run_event=push' tools/ci/check-remote-sprint-preflight.sh || fail "remote preflight does not prefer resulting-main validation"
grep -Fq 'Bind executable validation authority to trusted controller' tools/ci/check-remote-sprint-preflight.sh || fail "remote preflight omits authority-step evidence"
grep -Fq 'Validate in pinned read-only container on attested runner' tools/ci/check-remote-sprint-preflight.sh || fail "remote preflight omits full validation evidence"
grep -Fq 'Run mutation policy tests with read-only source' tools/ci/check-remote-sprint-preflight.sh || fail "remote preflight omits mutation validation evidence"
grep -Fq "required workflow was deferred, skipped, duplicated, or incomplete" tools/ci/check-remote-sprint-preflight.sh || fail "remote preflight does not reject deferred validation"
grep -q '^  repository_dispatch:$' .github/workflows/development-probe.yml || fail "Development Probe is not default-branch dispatched"
if grep -q 'workflow_dispatch' .github/workflows/development-probe.yml; then
    fail "Development Probe must not execute branch-selected workflow code"
fi
check_development_probe_workflow() {
awk '
function finish_step() {
    if (step == "Prepare bounded evidence" ||
        step == "Run probe with complete log and real status" ||
        step == "Preserve pre-probe failure truthfully") {
        if (evidence_bindings != 1) exit 20
    } else if (evidence_bindings != 0) {
        exit 21
    }
    if (step == "Retain complete probe evidence") {
        if (artifact_paths != 1) exit 22
    } else if (artifact_paths != 0) {
        exit 23
    }
}
index($0, "${{ runner.") {
    runner_references++
    if (!steps_started) exit 10
}
$0 == "    steps:" { steps_started = 1; next }
steps_started && /^      - name: / {
    if (step != "") finish_step()
    step = substr($0, 15)
    seen[step]++
    if (seen[step] != 1) exit 24
    evidence_bindings = 0
    artifact_paths = 0
    section = ""
    next
}
steps_started && /^        [A-Za-z0-9_-]+:/ {
    section = $0
    sub(/^        /, "", section)
    sub(/:.*/, "", section)
}
$0 == "          RAR_PROBE_EVIDENCE_DIR: ${{ runner.temp }}/rar-development-probe" {
    if (section != "env") exit 25
    evidence_bindings++
}
$0 == "          path: ${{ runner.temp }}/rar-development-probe" {
    if (section != "with") exit 26
    artifact_paths++
}
END {
    if (step != "") finish_step()
    if (!steps_started ||
        seen["Prepare bounded evidence"] != 1 ||
        seen["Run probe with complete log and real status"] != 1 ||
        seen["Preserve pre-probe failure truthfully"] != 1 ||
        seen["Retain complete probe evidence"] != 1 ||
        runner_references != 4) exit 30
}
' "$1"
}
check_development_probe_workflow .github/workflows/development-probe.yml || fail "Development Probe runner context or evidence-path placement is invalid"
if sed 's/^      - name: Run probe with complete log and real status$/      - name: Prepare bounded evidence/' \
    .github/workflows/development-probe.yml | check_development_probe_workflow -; then
    fail "Development Probe placement checker accepts duplicate/missing consumers"
fi
if awk '
$0 == "      - name: Prepare bounded evidence" { in_target = 1 }
in_target && $0 == "        env:" { print "        run: |"; in_target = 0; next }
{ print }
' .github/workflows/development-probe.yml | check_development_probe_workflow -; then
    fail "Development Probe placement checker accepts an env decoy in a run block"
fi
if awk '
$0 == "      - name: Retain complete probe evidence" { in_target = 1 }
in_target && $0 == "        with:" { print "        env:"; in_target = 0; next }
{ print }
' .github/workflows/development-probe.yml | check_development_probe_workflow -; then
    fail "Development Probe placement checker accepts an artifact-path decoy outside with"
fi
grep -q 'path: controller' .github/workflows/development-probe.yml || fail "Development Probe omits trusted controller checkout"
grep -q 'path: source' .github/workflows/development-probe.yml || fail "Development Probe omits isolated source checkout"
grep -q 'controller/tools/ci/run-development-probe.sh' .github/workflows/development-probe.yml || fail "Development Probe does not use trusted controller"
grep -q 'github.event.client_payload.source_sha' .github/workflows/development-probe.yml || fail "Development Probe omits explicit source identity"
grep -q 'uses: actions/upload-artifact@[0-9a-f]\{40\}' .github/workflows/development-probe.yml || fail "Development Probe evidence upload is not pinned"
grep -q 'pipe_status=("${PIPESTATUS\[@\]}")' .github/workflows/development-probe.yml || fail "Development Probe does not preserve pipeline statuses"
grep -q 'probe_status=${pipe_status\[0\]}' .github/workflows/development-probe.yml || fail "Development Probe does not preserve the real command status"
grep -q 'log_status=${pipe_status\[1\]}' .github/workflows/development-probe.yml || fail "Development Probe does not preserve log-capture status"
grep -q 'development-probe-status.sh "$probe_status" "$log_status"' .github/workflows/development-probe.yml || fail "Development Probe does not combine pipeline failures safely"
grep -q 'complete.log' .github/workflows/development-probe.yml || fail "Development Probe does not retain complete logs"
grep -q 'result.json' .github/workflows/development-probe.yml || fail "Development Probe does not retain a structured result"
grep -Fq 'name: development-probe-${{ github.run_id }}-${{ github.run_attempt }}' .github/workflows/development-probe.yml || fail "Development Probe artifact name is not payload-independent"
grep -Fq '"probe":"unverified","controller_sha":"unverified","source_sha":"unverified"' .github/workflows/development-probe.yml || fail "Development Probe fallback result is not payload-independent"
grep -q 'milestone-a | milestone-b | milestone-c | milestone-d | milestone-e | milestone-f | milestone-g' tools/ci/run-development-probe.sh || fail "A-G probes do not share the cloud target boundary"
grep -q 'source plus compiler/linker only' tools/ci/run-cloud-target-probe.sh || fail 'untrusted build phase is missing'
grep -q 'no source mount' tools/ci/run-cloud-target-probe.sh || fail 'trusted launch phase is missing'
grep -q 'exact emulator argument vector' tools/ci/run-cloud-target-probe.sh || fail 'trusted launcher ownership is missing'
for boundary in \
    '--network none' \
    '--user "$container_uid:$container_gid"' \
    '--cpus "$cpu_count"' \
    '--memory "${memory_mib}m"' \
    '--pids-limit 256' \
    '--security-opt no-new-privileges' \
    '--cap-drop ALL' \
    'target=/controller,readonly' \
    'target=/workspace,readonly' \
    '/usr/bin/timeout --signal=TERM'; do
    grep -Fq -- "$boundary" tools/ci/run-cloud-target-probe.sh || fail "cloud target boundary missing: $boundary"
done
grep -q 'build_id=$(docker create ' tools/ci/run-cloud-target-probe.sh || fail "cloud build container identity is not captured"
grep -q 'launch_id=$(docker create ' tools/ci/run-cloud-target-probe.sh || fail "cloud launch container identity is not captured"
grep -q 'docker start --attach "$build_id"' tools/ci/run-cloud-target-probe.sh || fail "cloud build lifecycle is not attached"
grep -q 'docker start "$launch_id"' tools/ci/run-cloud-target-probe.sh || fail "cloud launch lifecycle is not started"
grep -q 'docker cp "$launch_id:/evidence/."' tools/ci/run-cloud-target-probe.sh || fail "live bounded evidence is not copied"
grep -q 'docker wait "$launch_id"' tools/ci/run-cloud-target-probe.sh || fail "cloud launch lifecycle is not joined"
grep -q 'docker rm --force "$build_id"' tools/ci/run-cloud-target-probe.sh || fail "cloud build cleanup is not enforced"
grep -q 'docker rm --force "$launch_id"' tools/ci/run-cloud-target-probe.sh || fail "cloud launch cleanup is not enforced"
grep -q '^    set -e$' tools/ci/run-cloud-target-probe.sh || fail "cloud target preflight does not fail closed"
grep -q '^\[ "$state" = ready \]' tools/ci/run-cloud-target-probe.sh || fail "cloud target profile does not fail closed"
grep -q 'verify-cloud-target-tools.sh' tools/ci/run-cloud-target-probe.sh || fail "cloud target tool verification is bypassed"
grep -q 'controller identity mismatch' tools/ci/run-cloud-target-probe.sh || fail "cloud target controller identity is not verified"
grep -q 'source identity mismatch' tools/ci/run-cloud-target-probe.sh || fail "cloud target source identity is not verified"
for verified_input in compiler linker; do
    grep -q "^verify_file $verified_input " tools/ci/verify-cloud-target-tools.sh || fail "cloud target input is not byte verified: $verified_input"
done
for verified_input in qemu firmware machine-profile; do
    grep -q "^verify_file $verified_input " tools/ci/launch-cloud-target.sh || fail "trusted launch input is not byte verified: $verified_input"
done
/bin/sh tools/ci/check-trusted-launcher-policy.sh tools/ci/launch-cloud-target.sh >/dev/null
grep -q -- '--read-only' .github/workflows/specifications.yml || fail "CI container root is not read-only"
grep -Fq 'host_uid=$(/usr/bin/id -u)' .github/workflows/specifications.yml || fail "CI runner UID capture is missing"
grep -Fq 'host_gid=$(/usr/bin/id -g)' .github/workflows/specifications.yml || fail "CI runner GID capture is missing"
grep -Fq -- '--user "$host_uid:$host_gid"' .github/workflows/specifications.yml || fail "CI container does not use the runner identity"
/usr/bin/awk '
    BEGIN { approved = "            --cpus 2 --memory 2048m --memory-swap 2048m --pids-limit 256 \\" }
    function has_resource_option(line) {
        return line ~ /(^|[[:space:]])(--cpus([=[:space:]])|--memory([=[:space:]])|--memory-swap([=[:space:]])|--pids-limit([=[:space:]])|-m([=[:space:]]))/
    }
    /^[[:space:]]*docker run / {
        if (in_docker) bad=1
        in_docker=1
        vectors=0
        docker_runs++
    }
    has_resource_option($0) {
        if (in_docker && $0 == approved) vectors++
        else bad=1
    }
    in_docker && $0 !~ /\\[[:space:]]*$/ {
        if (vectors != 1) bad=1
        in_docker=0
    }
    END { if (bad || in_docker || docker_runs != 2) exit 1 }
' .github/workflows/specifications.yml || fail "each CI container must carry one exact, non-overridable resource vector"
grep -Fq -- 'uid=$host_uid,gid=$host_gid,mode=1777' .github/workflows/specifications.yml || fail "CI tmpfs is not writable by the runner identity"
grep -Fq -- '--env GITHUB_ACTIONS' .github/workflows/specifications.yml || fail "CI container does not receive the GitHub Actions boundary marker"
grep -Fq -- '--env CI' .github/workflows/specifications.yml || fail "CI container does not receive the CI boundary marker"
grep -q 'rar-image-plan-v3' tools/rarbuild/contracts/rar-image-plan-v3.fields || fail "image-plan v3 contract is missing"

tools/ci/check-host-policy.sh

echo "specification checks passed"
