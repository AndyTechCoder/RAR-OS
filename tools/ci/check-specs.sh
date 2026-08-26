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
docs/sprint-alpha.md
SPRINT_STATUS.md
docs/tasks/release-0.md
docs/tasks/sprint-alpha-vertical.md
docs/proposals/0022-alpha-graphics-input-authority.md
docs/proposals/0023-alpha-boot-determinism-and-entry-state.md
docs/proposals/0024-alpha-controller-helper-build-trust.md
spec/alpha/lab/README.md
spec/alpha/lab/development-lab-profile-v2.fields
spec/alpha/lab/image-inventory-v2.fields
spec/alpha/lab/crypto-reference-inventory-v2.fields
spec/alpha/lab/comparison-transcript-v0.fields
spec/alpha/lab/controller-state-machine-v0.fields
spec/alpha/lab/controller-handoff-v0.fields
spec/alpha/lab/controller-handoff-manifest-v0.fields
spec/alpha/lab/controller-handoff-cases.v0
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
tools/ci/check-sprint-static.sh
tools/ci/check-local-sprint-preflight.sh
tools/ci/report-sprint-alpha-gates.sh
tools/ci/check-sprint-alpha-gate-report-policy.sh
tools/ci/classify-proposed-adr.sh
tools/ci/test-proposed-adr-classifier-policy.sh
tools/ci/check-alpha-preimplementation-contracts.sh
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
tools/ci/check-reference-evidence-v0.sh
tools/ci/test-reference-evidence-v0-policy.sh
tools/ci/check-reference-verdict-v0.sh
tools/ci/test-reference-verdict-v0-policy.sh
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
tools/ci/test-workspace-budget-policy.sh
tools/ci/verify-pinned-file.sh
tools/ci/test-pinned-file-policy.sh
tools/ci/check-qmp-client-contract.sh
tools/ci/check-qmp-client-source.sh
tools/ci/test-qmp-client-source-policy.sh
tools/ci/check-development-image-inputs.sh
tools/ci/check-development-image-sources.sh
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
tools/sprint-alpha/x86_64-q35-v1.profile
tools/sprint-alpha/alpha-crypto-references-v1.env
tools/sprint-alpha/qmp-client-v1.env
tools/sprint-alpha/qmp-client-v1.md
tools/rar-lab/qmp-client/README.md
tools/rar-lab/qmp-client/build-plan.v1
tools/rar-lab/qmp-client/json.rs
tools/rar-lab/qmp-client/main.rs
tools/rar-lab/controller-handoff/README.md
tools/rar-lab/controller-handoff/build-plan.v0
tools/rar-lab/controller-handoff/fixtures/manifest-golden.v0.hex
tools/rar-lab/controller-handoff/lib.rs
tools/rar-lab/controller-handoff/manifest.rs
tools/rar-lab/controller-handoff/sha256.rs
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

for script in tools/ci/check-specs.sh tools/ci/check-sprint-static.sh tools/ci/check-local-sprint-preflight.sh tools/ci/check-remote-sprint-preflight.sh tools/ci/test-local-sprint-preflight-policy.sh tools/ci/check-alpha-preimplementation-contracts.sh tools/ci/test-alpha-preimplementation-contract-policy.sh tools/ci/check-development-lab-profile-v2.sh tools/ci/test-development-lab-profile-v2-policy.sh tools/ci/check-development-controller-v2.sh tools/ci/test-development-controller-v2-policy.sh tools/ci/check-controller-handoff-core.sh tools/ci/check-reference-evidence-v0.sh tools/ci/test-reference-evidence-v0-policy.sh tools/ci/check-reference-verdict-v0.sh tools/ci/test-reference-verdict-v0-policy.sh tools/ci/run-development-probe.sh tools/ci/run-cloud-target-probe.sh tools/ci/launch-cloud-target.sh tools/ci/prepare-launch-control.sh tools/ci/wait-for-launch-release.sh tools/ci/check-workspace-budget.sh tools/ci/verify-cloud-target-tools.sh tools/ci/development-probe-status.sh tools/ci/test-development-probe-policy.sh tools/ci/check-host-policy.sh tools/ci/test-host-policy.sh spec/alpha/lab/fixtures/generate.sh spec/fixtures/release-0/generate.sh spec/fixtures/release-0/run.sh sdk/generated/release-0/generate.sh sdk/generated/release-0/check.sh; do
    [ -x "$script" ] || fail "required script is not executable: $script"
done

grep -qx 'Status: Owner-approved execution contract — 2026-08-25' docs/tasks/sprint-alpha-vertical.md || fail "Sprint Alpha vertical packet is not approved"
[ "$(sed -n '1p' spec/alpha/evidence/acceptance-v1.plan)" = schema=rar-alpha-acceptance-plan-v1 ] || fail "Alpha evidence protocol schema is invalid"
[ "$(awk -F '|' '!/^#/ && !/^schema=/ && NF { count++; if (NF != 5 || $1 !~ /^[A-G]$/ || $2 !~ /^(none|continue|key:[a-z0-9-]+|pointer:[0-9]+,[0-9]+,[0-9]+)$/ || $3 !~ /^[a-z0-9:-]+$/ || $4 !~ /^[a-z0-9-]+$/ || $5 !~ /^[01]$/) bad=1 } END { if (bad) exit 1; print count + 0 }' spec/alpha/evidence/acceptance-v1.plan)" -eq 45 ] || fail "Alpha evidence protocol is incomplete or malformed"
/bin/sh tools/ci/check-development-lab-profile.sh >/dev/null
/bin/sh tools/ci/check-development-lab-profile-v2.sh >/dev/null
/bin/sh tools/ci/check-development-controller-v2.sh >/dev/null
/bin/sh tools/ci/check-controller-handoff-core.sh >/dev/null
/bin/sh tools/ci/check-reference-evidence-v0.sh spec/alpha/lab/fixtures/comparison-evidence.v0 spec/alpha/lab/fixtures/comparison-transcript.v0 spec/alpha/lab/fixtures/reference-inventory.v0 spec/alpha/lab/fixtures/reference-harness.v0 >/dev/null
/bin/sh tools/ci/check-reference-verdict-v0.sh spec/alpha/lab/fixtures/reference-verdict-accepted.v0 milestone-f spec/alpha/lab/fixtures/controller-context.v0 spec/alpha/lab/fixtures/source-context.v0 spec/alpha/lab/fixtures/comparison-transcript.v0 spec/alpha/lab/fixtures/reference-inventory.v0 spec/alpha/lab/fixtures/comparison-evidence.v0 spec/alpha/lab/fixtures/reference-harness.v0 >/dev/null
/bin/sh tools/ci/check-reference-verdict-v0.sh spec/alpha/lab/fixtures/reference-verdict-not-required.v0 milestone-a spec/alpha/lab/fixtures/controller-context.v0 spec/alpha/lab/fixtures/source-context.v0 spec/alpha/lab/fixtures/comparison-transcript.v0 none none none >/dev/null
/bin/sh tools/ci/check-sprint-alpha-gate-report-policy.sh >/dev/null
/bin/sh tools/ci/test-proposed-adr-classifier-policy.sh >/dev/null
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/proposals/0022-alpha-graphics-input-authority.md 0022 \
    docs/approval-record.md)" = owner-decision-required ] || fail "ADR 0022 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/proposals/0023-alpha-boot-determinism-and-entry-state.md 0023 \
    docs/approval-record.md)" = owner-decision-required ] || fail "ADR 0023 decision state is inconsistent"
[ "$(/bin/sh tools/ci/classify-proposed-adr.sh \
    docs/proposals/0024-alpha-controller-helper-build-trust.md 0024 \
    docs/approval-record.md)" = owner-decision-required ] || fail "ADR 0024 decision state is inconsistent"
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
[ "$adr_count" -eq 21 ] || fail "expected exactly 21 indexed ADRs"

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

grep -Eq 'uses: actions/checkout@[0-9a-f]{40}([[:space:]]|$)' .github/workflows/specifications.yml || fail "GitHub checkout action is not pinned by commit"
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
grep -q '11bd71901bbe5b1630ceea73d27597364c9af683' "$class_b_inventory" || fail "Class B inventory omits the checkout action commit"
grep -q 'ubuntu-24.04-20260714.240.1' "$class_b_inventory" || fail "Class B inventory omits the runner image version"

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

local_lock_sha256=$(sha256_of tools/toolchain/host-tools.lock) || fail "cannot hash local tool lock"
ci_lock_sha256=$(sha256_of tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock) || fail "cannot hash CI tool lock"
[ "$local_lock_sha256" = f7e9baf24aaff9eaa2a2032cf0a9919568cca817d6b5d0c7e6891bce05ec979a ] || fail "local tool lock digest changed without bootstrap authority update"
[ "$ci_lock_sha256" = 6752b1b21ac8fa93a671ff9444173e4c3bbc4cdcbe4cf5cd39820371dc79aa24 ] || fail "CI tool lock digest changed without bootstrap authority update"
grep -qx "macos_lock_sha256=$local_lock_sha256" tools/toolchain/host-tools.manifest || fail "host tool manifest local lock digest is stale"
grep -qx "ci_lock_sha256=$ci_lock_sha256" tools/toolchain/host-tools.manifest || fail "host tool manifest CI lock digest is stale"
if grep -q '^  runner_evidence:$' .github/workflows/specifications.yml ||
    grep -q '^    needs: runner_evidence$' .github/workflows/specifications.yml; then
    fail "CI runner evidence and validation must not be split across hosted runners"
fi
grep -q '^  validate:$' .github/workflows/specifications.yml || fail "same-runner CI validation job is missing"
grep -q 'name: attest-runner-and-validate' .github/workflows/specifications.yml || fail "same-runner attestation/validation identity is missing"
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
if grep -q 'workflow_dispatch' .github/workflows/specifications.yml; then
    fail "Specifications workflow must not execute branch-selected workflow code"
fi
grep -q '^  repository_dispatch:$' .github/workflows/development-probe.yml || fail "Development Probe is not default-branch dispatched"
if grep -q 'workflow_dispatch' .github/workflows/development-probe.yml; then
    fail "Development Probe must not execute branch-selected workflow code"
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
grep -Fq -- 'uid=$host_uid,gid=$host_gid,mode=1777' .github/workflows/specifications.yml || fail "CI tmpfs is not writable by the runner identity"
grep -Fq -- '--env GITHUB_ACTIONS' .github/workflows/specifications.yml || fail "CI container does not receive the GitHub Actions boundary marker"
grep -Fq -- '--env CI' .github/workflows/specifications.yml || fail "CI container does not receive the CI boundary marker"
grep -q 'rar-image-plan-v3' tools/rarbuild/contracts/rar-image-plan-v3.fields || fail "image-plan v3 contract is missing"

tools/ci/check-host-policy.sh
tools/ci/test-host-policy.sh

echo "specification checks passed"
