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
.codex/config.toml
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
docs/tasks/release-0.md
docs/adr/0011-release-0-reproducibility-gate-phasing.md
docs/adr/0012-release-0-host-bootstrap-trust-and-snapshot.md
docs/release-0/build/prompt-4-remediation.md
tools/ci/check-specs.sh
tools/ci/check-host-policy.sh
tools/ci/test-host-policy.sh
tools/ci/fixtures/host-policy/README.md
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

for script in tools/ci/check-specs.sh tools/ci/check-host-policy.sh tools/ci/test-host-policy.sh; do
    [ -x "$script" ] || fail "required script is not executable: $script"
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
[ "$adr_count" -eq 12 ] || fail "expected exactly 12 indexed ADRs"

approval_date=$(sed -n 's/^Date: //p' docs/approval-record.md)
case "$approval_date" in
    [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;;
    *) fail "approval record has no unique ISO date" ;;
esac

grep -qx 'Status: Approved' docs/approval-record.md || fail "approval record status is not approved"
grep -qx 'Approval: approved' docs/approval-record.md || fail "approval statement is not approved"
grep -q '^Approver: .\+' docs/approval-record.md || fail "approval record has no approver"
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

for number in 0001 0002 0003 0004 0005 0006 0007 0008 0009 0010 0011 0012; do
    matches=$(printf '%s\n' "$adr_files" | grep -c "/$number-")
    [ "$matches" -eq 1 ] || fail "expected one indexed ADR for $number"
done

grep -q 'ADRs 0001–0012' docs/tasks/release-0.md || fail "Release 0 approved ADR range is stale"
grep -q 'Build-plan and evidence schemas use version 3' docs/adr/0011-release-0-reproducibility-gate-phasing.md || fail "ADR 0011 build-plan/evidence schema version is stale"
for number in 0001 0002 0003 0004 0005 0006 0007 0008 0009 0010 0011 0012; do
    if grep -q "ADR $number" docs/tasks/release-0.md; then
        printf '%s\n' "$adr_files" | grep -q "/$number-" || fail "task-referenced ADR $number is not indexed and approved"
    fi
done

printf '%s\n' "$adr_files" | while IFS= read -r adr; do
    [ -s "$adr" ] || fail "missing or empty indexed ADR: $adr"
    grep -qx "Status: Accepted — $approval_date" "$adr" || fail "ADR status mismatch: $adr"

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

if find . -path ./.git -prune -o -path ./out -prune -o -type f -exec grep -nHE '[[:blank:]]+$' {} +; then
    fail "trailing whitespace found"
fi

if find . -path ./.git -prune -o -path ./out -prune -o -type f -exec grep -nHE '^(<<<<<<<|=======|>>>>>>>)' {} +; then
    fail "merge-conflict marker found"
fi

grep -Eq 'uses: actions/checkout@[0-9a-f]{40}([[:space:]]|$)' .github/workflows/specifications.yml || fail "GitHub checkout action is not pinned by commit"
grep -qx 'channel = "1.95.0"' rust-toolchain.toml || fail "Rust toolchain is not pinned to 1.95.0"
grep -qx 'members = \[\]' Cargo.toml || fail "Gate 0 workspace must not contain implementation crates"

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
grep -Fq '[ "${ImageVersion-}" = 20260714.240.1 ]' .github/workflows/specifications.yml || fail "CI runner image version is not attested"
for runner_handoff in \
    'RAR_CI_RUNNER_IMAGE_OS=$ImageOS' \
    'RAR_CI_RUNNER_IMAGE_VERSION=$ImageVersion' \
    'RAR_CI_RUNNER_OS=$RUNNER_OS' \
    'RAR_CI_RUNNER_ARCH=$RUNNER_ARCH'; do
    grep -Fq "$runner_handoff" .github/workflows/specifications.yml || fail "CI runner evidence handoff is incomplete: $runner_handoff"
done
grep -q 'docker run --rm --read-only' .github/workflows/specifications.yml || fail "same-runner pinned container launch is missing"
grep -q -- '--read-only' .github/workflows/specifications.yml || fail "CI container root is not read-only"
grep -q 'rar-image-plan-v3' tools/rarbuild/contracts/rar-image-plan-v3.fields || fail "image-plan v3 contract is missing"

tools/ci/check-host-policy.sh
tools/ci/test-host-policy.sh

echo "specification checks passed"
