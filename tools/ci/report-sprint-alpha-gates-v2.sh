#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
source_root=$repository_root
if [ "$#" -eq 1 ] && [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ]; then
    scratch=$(/bin/sh "$repository_root/tools/ci/require-ephemeral-policy-test-root.sh")
    [ "$scratch" = /tmp ] || exit 1
    case "$1" in "$scratch"/*) source_root=$1 ;; *) exit 1 ;; esac
elif [ "$#" -ne 0 ]; then
    exit 1
fi

[ -d "$source_root" ] && [ ! -L "$source_root" ] || exit 1
source_root=$(CDPATH= cd -- "$source_root" && pwd -P)
case "$source_root" in
    "$repository_root") ;;
    /tmp/*) [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ] || exit 1 ;;
    *) exit 1 ;;
esac

fail() {
    printf 'Sprint Alpha gate report v2 rejected: %s\n' "$1" >&2
    exit 1
}

hash_file() {
    file=$1
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || return 1
    if [ -x /usr/bin/sha256sum ]; then
        /usr/bin/sha256sum "$file" | /usr/bin/awk '{ print $1 }'
    else
        /usr/bin/shasum -a 256 "$file" | /usr/bin/awk '{ print $1 }'
    fi
}

contract=$source_root/tools/ci/contracts/sprint-alpha-gate-report-v2.fields
[ -f "$contract" ] && [ ! -L "$contract" ] && [ -s "$contract" ] || fail 'missing, symbolic, or empty reporter contract'
[ "$(/usr/bin/wc -l < "$contract" | /usr/bin/tr -d ' ')" -eq 88 ] || fail 'reporter contract line count changed'
for row in \
    'schema=rar-sprint-alpha-gate-report-contract-v2' \
    'status=source-ready-blocking' \
    'report_schema=rar-sprint-alpha-gate-report-v2' \
    'adr_0026_path=docs/adr/0026-alpha-platform-payload-and-state-sources.md' \
    'approval_path=docs/approval-record.md' \
    'platform_envelope_path=unavailable-pending-reviewed-platform-contract' \
    'core_bootstrap_path=unavailable-pending-reviewed-platform-contract' \
    'component_bundle_contract_path=unavailable-pending-reviewed-platform-contract' \
    'component_bundle_fixture_path=unavailable-pending-reviewed-platform-contract' \
    'initial_system_contract_path=unavailable-pending-reviewed-platform-contract' \
    'initial_system_fixture_path=unavailable-pending-reviewed-platform-contract' \
    'initial_preserved_contract_path=unavailable-pending-reviewed-platform-contract' \
    'initial_preserved_fixture_path=unavailable-pending-reviewed-platform-contract' \
    'platform_fixture_manifest_path=unavailable-pending-reviewed-platform-contract' \
    'acceptance_v2_contract_path=spec/alpha/evidence/acceptance-v2.fields' \
    'acceptance_v2_plan_path=spec/alpha/evidence/acceptance-v2.plan' \
    'acceptance_v2_plan_sha256=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd' \
    'historical_report_path=tools/ci/report-sprint-alpha-gates.sh' \
    'historical_report_sha256=78782b52f2b063c0dd56f3778e17242dc21088729da7f075a2bc74ab038fa8f8' \
    'platform_state_values=missing,invalid,pending-review,ready' \
    'acceptance_state_values=invalid,reviewed-implementation-required,ready' \
    'platform_activation_rule=all-exact-paths+identities+readiness+validator-must-be-bound-by-separately-reviewed-update' \
    'external_evidence_rule=report-never-proves-permission-profile+checkpoint+workflow+pr+target-execution' \
    'target_rule=report-does-not-build+image+launch+execute+or-authorize-target-code'; do
    [ "$(/usr/bin/grep -Fxc -- "$row" "$contract")" -eq 1 ] || fail "contract row missing or duplicated: $row"
done

report_contract_sha256=$(hash_file "$contract") || fail 'reporter contract identity unavailable'
printf '%s\n' "$report_contract_sha256" | /usr/bin/grep -Eq '^[0-9a-f]{64}$' || fail 'reporter contract identity malformed'
[ "$report_contract_sha256" = 965e7fa198d9a62b6a391e980207f857e75cbd44383766e21e1be839c03ee12c ] || fail 'reporter contract bytes changed'

classify() {
    number=$1
    name=$2
    /bin/sh "$source_root/tools/ci/classify-proposed-adr.sh" \
        "$source_root/docs/adr/$number-$name.md" "$number" \
        "$source_root/docs/approval-record.md"
}

adr_0020=$(classify 0020 alpha-reference-oracle-isolation)
adr_0021=$(classify 0021 alpha-boot-payload-boundary)
adr_0022=$(classify 0022 alpha-graphics-input-authority)
adr_0023=$(classify 0023 alpha-boot-determinism-and-entry-state)
adr_0024=$(classify 0024 alpha-controller-helper-build-trust)
adr_0025=$(classify 0025 alpha-gui-continuity-evidence-sequencing)
adr_0026=$(classify 0026 alpha-platform-payload-and-state-sources)

platform_contract_state=missing
if [ -e "$source_root/spec/alpha/platform" ] || [ -L "$source_root/spec/alpha/platform" ]; then
    platform_contract_state=invalid
fi
platform_identity=unavailable

acceptance_v2_state=invalid
acceptance_v2_plan_sha256=unavailable
acceptance_contract=$source_root/spec/alpha/evidence/acceptance-v2.fields
acceptance_plan=$source_root/spec/alpha/evidence/acceptance-v2.plan
if [ -f "$acceptance_contract" ] && [ ! -L "$acceptance_contract" ] && [ -s "$acceptance_contract" ] &&
    [ -f "$acceptance_plan" ] && [ ! -L "$acceptance_plan" ] && [ -s "$acceptance_plan" ]; then
    acceptance_v2_plan_sha256=$(hash_file "$acceptance_plan") || acceptance_v2_plan_sha256=unavailable
    acceptance_status=$(/usr/bin/sed -n 's/^status=//p' "$acceptance_contract")
    acceptance_bound=$(/usr/bin/sed -n 's/^plan_sha256=//p' "$acceptance_contract")
    if [ "$acceptance_status" = experimental-inactive-pending-review ] &&
        [ "$acceptance_bound" = ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd ] &&
        [ "$acceptance_v2_plan_sha256" = "$acceptance_bound" ] &&
        /bin/sh "$source_root/tools/ci/check-acceptance-v2.sh" >/dev/null 2>&1; then
        acceptance_v2_state=reviewed-implementation-required
    elif [ "$acceptance_status" = ready ] &&
        [ "$acceptance_bound" = "$acceptance_v2_plan_sha256" ] &&
        /bin/sh "$source_root/tools/ci/check-acceptance-v2.sh" >/dev/null 2>&1; then
        acceptance_v2_state=ready
    fi
fi

controller_state=blocked
lab_profile_state=blocked
helper_state=blocked
controller_readiness=blocked
boot_contract_state=missing
boot_contract=$source_root/spec/alpha/boot/alpha-boot-v0.fields
if [ -L "$boot_contract" ]; then
    boot_contract_state=invalid
elif [ -f "$boot_contract" ] && [ -s "$boot_contract" ]; then
    if [ "$(/usr/bin/grep -c '^readiness=' "$boot_contract")" -eq 1 ]; then
        boot_readiness=$(/usr/bin/sed -n 's/^readiness=//p' "$boot_contract")
        case "$boot_readiness" in
            *[!a-z0-9-]* | '') boot_contract_state=invalid ;;
            *) boot_contract_state=pending-review ;;
        esac
    else
        boot_contract_state=invalid
    fi
fi

platform_source_set=blocked
milestone_a_readiness=blocked
milestone_b_readiness=blocked
milestone_e_readiness=blocked
local_repository_gates=blocked
overall=blocked

printf '%s\n' \
    'schema=rar-sprint-alpha-gate-report-v2' \
    "report_contract_sha256=$report_contract_sha256" \
    'source_revision=strict-preflight-required' \
    'branch=strict-preflight-required' \
    'checkpoint=strict-preflight-required' \
    'permission_profile=manual-evidence-required' \
    'workspace_boundary=strict-preflight-required' \
    'ssd_capacity=strict-preflight-required' \
    'workspace_budget=strict-preflight-required' \
    "adr_0020=$adr_0020" \
    "adr_0021=$adr_0021" \
    "adr_0022=$adr_0022" \
    "adr_0023=$adr_0023" \
    "adr_0024=$adr_0024" \
    "adr_0025=$adr_0025" \
    "adr_0026=$adr_0026" \
    "boot_contract_state=$boot_contract_state" \
    "controller_state=$controller_state" \
    "lab_profile_state=$lab_profile_state" \
    "helper_state=$helper_state" \
    "controller_readiness=$controller_readiness" \
    "platform_envelope_state=$platform_contract_state" \
    "platform_envelope_contract_sha256=$platform_identity" \
    "core_bootstrap_state=$platform_contract_state" \
    "core_bootstrap_contract_sha256=$platform_identity" \
    "component_bundle_state=$platform_contract_state" \
    'component_bundle_fixture_path=unavailable' \
    'component_bundle_fixture_sha256=unavailable' \
    "initial_system_state=$platform_contract_state" \
    'initial_system_fixture_path=unavailable' \
    'initial_system_fixture_sha256=unavailable' \
    "initial_preserved_state=$platform_contract_state" \
    'initial_preserved_fixture_path=unavailable' \
    'initial_preserved_fixture_sha256=unavailable' \
    "platform_fixture_manifest_state=$platform_contract_state" \
    'platform_fixture_manifest_sha256=unavailable' \
    "platform_source_set=$platform_source_set" \
    "acceptance_protocol_v2=$acceptance_v2_state" \
    "acceptance_v2_plan_sha256=$acceptance_v2_plan_sha256" \
    "milestone_a_readiness=$milestone_a_readiness" \
    "milestone_b_readiness=$milestone_b_readiness" \
    "milestone_e_readiness=$milestone_e_readiness" \
    'remote_workflow=external-evidence-required' \
    'pr_gate=external-evidence-required' \
    'target_implementation=not-evaluated' \
    "local_repository_gates=$local_repository_gates" \
    "overall=$overall"
