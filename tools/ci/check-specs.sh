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
tools/rarbuild/contracts/rar-build-evidence-v3.fields
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

tools/ci/check-host-policy.sh
tools/ci/test-host-policy.sh

echo "specification checks passed"
