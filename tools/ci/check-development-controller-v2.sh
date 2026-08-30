#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
plan=${1-$root/tools/sprint-alpha/development-controller-v2.plan}
profile=${2-$root/tools/sprint-alpha/development-lab-v2.env}
contract=$root/spec/alpha/lab/controller-state-machine-v0.fields

fail() {
    printf 'Development controller v2 blocked: %s\n' "$1" >&2
    exit 1
}

for file in "$plan" "$profile" "$contract"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
done
/bin/sh "$root/tools/ci/check-development-lab-profile-v2.sh" "$profile" >/dev/null || fail 'v2 profile is invalid'

/usr/bin/awk -F '|' '
    BEGIN {
        expected_row[1] = "01|preflight|controller|A-G|validated-identities"
        expected_row[2] = "02|build-one|build|A-G|artifact-one+transcript-one"
        expected_row[3] = "03|build-two|build|A-G|artifact-two+transcript-two"
        expected_row[4] = "04|freeze|controller|A-G|frozen-artifact+frozen-transcript+digests"
        expected_row[5] = "05|reference|reference|F-G|comparison-evidence"
        expected_row[6] = "06|reference-verify|controller|F-G|reference-verdict"
        expected_row[7] = "07|launch|launch|A-G|launch-evidence"
        expected_row[8] = "08|evidence-verify|controller|A-G|accepted-evidence-record+reference-verdict-digest"
        expected_row[9] = "09|retain|controller|A-G|bounded-retained-evidence+completion-report+reference-verdict"
    }
    NR == 1 { if ($0 != "schema=rar-alpha-development-controller-plan-v0") bad = 1; next }
    NR == 2 { if ($0 != "state=blocked") bad = 1; next }
    NR == 3 { if ($0 != "contract=spec/alpha/lab/controller-state-machine-v0.fields") bad = 1; next }
    NR == 4 { if ($0 != "profile=tools/sprint-alpha/development-lab-v2.env") bad = 1; next }
    NR == 5 { if ($0 != "acceptance_protocol=spec/alpha/evidence/acceptance-v2.plan") bad = 1; next }
    NR == 6 { if ($0 != "acceptance_protocol_sha256=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd") bad = 1; next }
    NR == 7 { if ($0 != "accepted_evidence_contract=spec/alpha/evidence/accepted-evidence-v0.fields") bad = 1; next }
    NR == 8 { if ($0 != "accepted_evidence_contract_sha256=73874f4e3ea10bf356641365819fcc8075cd98f53c3f3c5fa28b2868a11c1703") bad = 1; next }
    NR >= 9 && NR <= 17 {
        position = NR - 8
        if (NF != 5 || $0 != expected_row[position]) bad = 1
        if ($2 !~ /^[a-z][a-z0-9-]*$/ || $3 !~ /^(controller|build|reference|launch)$/ || $4 !~ /^(A-G|F-G)$/ || $5 !~ /^[a-z0-9+-]+$/) bad = 1
        if (++name[$2] != 1) bad = 1
        actor[position] = $3
        phase[position] = $2
        next
    }
    NR == 18 { if ($0 != "activation=forbidden") bad = 1; next }
    { bad = 1 }
    END {
        if (NR != 18) bad = 1
        if (phase[1] != "preflight" || actor[1] != "controller") bad = 1
        if (phase[2] != "build-one" || actor[2] != "build") bad = 1
        if (phase[3] != "build-two" || actor[3] != "build") bad = 1
        if (phase[4] != "freeze" || actor[4] != "controller") bad = 1
        if (phase[5] != "reference" || actor[5] != "reference") bad = 1
        if (phase[6] != "reference-verify" || actor[6] != "controller") bad = 1
        if (phase[7] != "launch" || actor[7] != "launch") bad = 1
        if (phase[8] != "evidence-verify" || actor[8] != "controller") bad = 1
        if (phase[9] != "retain" || actor[9] != "controller") bad = 1
        exit bad ? 1 : 0
    }
' "$plan" || fail 'plan grammar, order, actor, or activation state is invalid'

for required in \
    'phase_count=9' \
    'phase|1|preflight|controller|A-G|trusted-main+source-identity+profile-instance+contracts+inventories|validated-identities|2' \
    'profile_rule=blocked-instance-validates-and-stops-before-any-effect,ready-instance-requires-all-activation-identities-and-reviews' \
    'acceptance_rule=controller+profile+launch+verifier+transcript-bind-exact-v2-digest,v1-or-missing+mismatched+duplicate-identity-rejects' \
    'accepted_evidence_rule=phase-8-recomputes-and-binds-fresh-attempt+probe+controller+source+artifact+protocol+profile+tools+outputs+handoff+reference+inventories,any-replay-or-mismatch-rejects' \
    'role_overlap_rule=at-most-one-role-container-running,controller-never-enters-build-or-reference-filesystem,build-and-reference-never-receive-controller-tree' \
    'inactive_rule=blocked-profile-means-no-container-command,no-cloud-command,no-network-command,no-credential-read' \
    'verdict_rule|reference|F-G:accepted-only-after-both-references-match-each-other-and-target-with-real-inventory-and-evidence-digests,A-E:not-required-only-from-trusted-controller-with-zero-inventory-and-evidence-digests,canonical-record-bound-to-probe+controller+source+transcript,phase-8-requires-exactly-one-verdict,retained-set-includes-verdict-and-digest' \
    'publication_rule=no-push,no-PR-change,no-merge,no-release,no-deployment' \
    'target_rule=cloud-only-never-Mac,artifact-unsigned-until-Milestone-F-evidence,reference-code-never-target-linked'; do
    [ "$(/usr/bin/grep -Fxc -- "$required" "$contract")" -eq 1 ] || fail "contract row missing or duplicated: $required"
done

/bin/sh "$root/tools/ci/check-acceptance-v2.sh" >/dev/null || fail 'acceptance protocol v2 is invalid'

printf '%s\n' 'Development controller v2 validation passed: state=blocked activation=forbidden phases=9'
