#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'Sprint Alpha gate report v2 mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/gate-report-v2.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
fixture=$work/source
reporter=$root/tools/ci/report-sprint-alpha-gates-v2.sh

reset_fixture() {
    /bin/rm -rf "$fixture"
    /bin/mkdir -p "$fixture/tools/ci/contracts" "$fixture/tools/sprint-alpha" "$fixture/docs/adr" "$fixture/spec/alpha/evidence" "$fixture/spec/alpha/boot"
    /bin/cp "$root/tools/ci/contracts/sprint-alpha-gate-report-v2.fields" "$fixture/tools/ci/contracts/"
    /bin/cp "$root/tools/ci/classify-proposed-adr.sh" "$fixture/tools/ci/"
    /bin/cp "$root/tools/ci/check-acceptance-v2.sh" "$fixture/tools/ci/"
    /bin/cp "$root/tools/ci/run-alpha-scenario.sh" "$fixture/tools/ci/"
    /bin/cp "$root/tools/ci/verify-launch-evidence.sh" "$fixture/tools/ci/"
    /bin/cp "$root/docs/approval-record.md" "$fixture/docs/"
    for number in 0020 0021 0022 0023 0024 0025 0026; do
        file=$(/usr/bin/find "$root/docs/adr" -maxdepth 1 -type f -name "$number-*.md")
        [ "$(printf '%s\n' "$file" | /usr/bin/wc -l | /usr/bin/tr -d ' ')" -eq 1 ] || exit 1
        /bin/cp "$file" "$fixture/docs/adr/"
    done
    /bin/cp "$root/spec/alpha/evidence/acceptance-v1.plan" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/spec/alpha/evidence/acceptance-v2.plan" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/spec/alpha/evidence/acceptance-v2.fields" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/spec/alpha/evidence/acceptance-v2-cases.v0" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/spec/alpha/evidence/acceptance-v2-selection-digests.v0" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/spec/alpha/evidence/accepted-evidence-v0.fields" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/spec/alpha/evidence/accepted-evidence-v0-cases.v0" "$fixture/spec/alpha/evidence/"
    /bin/cp "$root/tools/sprint-alpha/development-controller-v2.plan" "$fixture/tools/sprint-alpha/"
    /bin/cp "$root/tools/sprint-alpha/development-lab-v2.env" "$fixture/tools/sprint-alpha/"
    /bin/cp "$root/tools/sprint-alpha/controller-helper-v0.env" "$fixture/tools/sprint-alpha/"
    /bin/cp "$root/spec/alpha/boot/alpha-boot-v0.fields" "$fixture/spec/alpha/boot/"
}

report() {
    RAR_POLICY_MUTATION_TESTS=1 /bin/sh "$reporter" "$fixture"
}

require_row() {
    expected=$1
    [ "$(report | /usr/bin/grep -Fxc -- "$expected")" -eq 1 ] || {
        printf 'gate report v2 mutation produced unexpected state: %s\n' "$expected" >&2
        exit 1
    }
}

expect_rejected() {
    label=$1
    if report >"$work/result" 2>&1; then
        printf 'unsafe gate report v2 mutation unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

reset_fixture
require_row 'platform_source_set=blocked'
require_row 'acceptance_protocol_v2=reviewed-implementation-required'
require_row 'milestone_a_readiness=blocked'

reset_fixture
/bin/rm -f "$fixture/tools/ci/contracts/sprint-alpha-gate-report-v2.fields"
expect_rejected missing-reporter-contract

reset_fixture
/usr/bin/printf '%s\n' 'unexpected=authority-expansion' >> "$fixture/tools/ci/contracts/sprint-alpha-gate-report-v2.fields"
expect_rejected extra-reporter-contract-row

reset_fixture
/bin/mv "$fixture/tools/ci/contracts/sprint-alpha-gate-report-v2.fields" "$work/contract"
/bin/ln -s "$work/contract" "$fixture/tools/ci/contracts/sprint-alpha-gate-report-v2.fields"
expect_rejected symbolic-reporter-contract

reset_fixture
/bin/mkdir -p "$fixture/spec/alpha/platform"
require_row 'platform_envelope_state=invalid'
require_row 'platform_source_set=blocked'

reset_fixture
/usr/bin/printf '%s\n' 'CANARY' > "$work/canary"
/bin/ln -s "$work/canary" "$fixture/spec/alpha/platform"
require_row 'platform_envelope_state=invalid'
require_row 'platform_source_set=blocked'

reset_fixture
for file in development-controller-v2.plan development-lab-v2.env controller-helper-v0.env; do
    /usr/bin/sed 's/^state=.*/state=ready/' "$fixture/tools/sprint-alpha/$file" > "$work/bad"
    /bin/mv "$work/bad" "$fixture/tools/sprint-alpha/$file"
done
require_row 'controller_state=blocked'
require_row 'lab_profile_state=blocked'
require_row 'helper_state=blocked'
require_row 'controller_readiness=blocked'

reset_fixture
/usr/bin/printf '%s\n' 'readiness=ready' >> "$fixture/spec/alpha/boot/alpha-boot-v0.fields"
require_row 'boot_contract_state=invalid'

reset_fixture
/bin/mv "$fixture/spec/alpha/boot/alpha-boot-v0.fields" "$work/boot-contract"
/bin/ln -s "$work/boot-contract" "$fixture/spec/alpha/boot/alpha-boot-v0.fields"
require_row 'boot_contract_state=invalid'

reset_fixture
/usr/bin/sed 's/status=experimental-inactive-pending-review/status=ready/' "$fixture/spec/alpha/evidence/acceptance-v2.fields" > "$work/bad"
/bin/mv "$work/bad" "$fixture/spec/alpha/evidence/acceptance-v2.fields"
require_row 'acceptance_protocol_v2=invalid'
require_row 'milestone_b_readiness=blocked'

reset_fixture
/bin/cp "$fixture/spec/alpha/evidence/acceptance-v1.plan" "$fixture/spec/alpha/evidence/acceptance-v2.plan"
require_row 'acceptance_protocol_v2=invalid'

reset_fixture
adr=$fixture/docs/adr/0026-alpha-platform-payload-and-state-sources.md
/usr/bin/sed 's/Status: Accepted — 2026-08-29/Status: Proposed/' "$adr" > "$work/bad"
/bin/mv "$work/bad" "$adr"
if report | /usr/bin/grep -Fqx 'adr_0026=accepted'; then
    printf '%s\n' 'unaccepted ADR 0026 unexpectedly satisfied gate report v2' >&2
    exit 1
fi
require_row 'milestone_a_readiness=blocked'

printf '%s\n' 'Sprint Alpha gate report v2 negative checks passed'
