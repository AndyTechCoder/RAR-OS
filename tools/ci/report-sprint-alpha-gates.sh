#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

safe_root='/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha'
identity_file=$safe_root/.rar-os-workspace-identity
identity_sha256=f71483fc7335d5c0949541bad24143b437c379250c70e47dbe7a0b766decd496
minimum_free_kib=10485760
maximum_workspace_kib=8388608

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
case "$root" in "$safe_root/repository" | "$safe_root/worktrees/"*) ;; *) exit 1 ;; esac
[ -f "$identity_file" ] && [ ! -L "$identity_file" ] || exit 1
identity_output=$(/usr/bin/shasum -a 256 "$identity_file") || exit 1

workspace_boundary=blocked
[ "${identity_output%% *}" = "$identity_sha256" ] && workspace_boundary=ready

capacity_state() {
    available=$1
    case "$available" in '' | *[!0-9]*) printf unknown ;; *)
        if [ "$available" -ge "$minimum_free_kib" ]; then printf ready; else printf blocked; fi
    esac
}
internal_free_kib=$(/bin/df -Pk / | /usr/bin/awk 'END { print $4 }')
ssd_free_kib=$(/bin/df -Pk "$safe_root" | /usr/bin/awk 'END { print $4 }')
internal_disk=$(capacity_state "$internal_free_kib")
ssd_capacity=$(capacity_state "$ssd_free_kib")
workspace_kib=$(/usr/bin/du -sk "$safe_root" | /usr/bin/awk 'NR == 1 { print $1 }')
case "$workspace_kib" in '' | *[!0-9]*) workspace_budget=unknown ;; *)
    if [ "$workspace_kib" -le "$maximum_workspace_kib" ]; then workspace_budget=ready; else workspace_budget=blocked; fi
esac

adr_0020=$(/bin/sh "$root/tools/ci/classify-proposed-adr.sh" \
    "$root/docs/adr/0020-alpha-reference-oracle-isolation.md" 0020 \
    "$root/docs/approval-record.md")
adr_0021=$(/bin/sh "$root/tools/ci/classify-proposed-adr.sh" \
    "$root/docs/adr/0021-alpha-boot-payload-boundary.md" 0021 \
    "$root/docs/approval-record.md")
adr_0022=$(/bin/sh "$root/tools/ci/classify-proposed-adr.sh" \
    "$root/docs/proposals/0022-alpha-graphics-input-authority.md" 0022 \
    "$root/docs/approval-record.md")
adr_0023=$(/bin/sh "$root/tools/ci/classify-proposed-adr.sh" \
    "$root/docs/proposals/0023-alpha-boot-determinism-and-entry-state.md" 0023 \
    "$root/docs/approval-record.md")
adr_0024=$(/bin/sh "$root/tools/ci/classify-proposed-adr.sh" \
    "$root/docs/proposals/0024-alpha-controller-helper-build-trust.md" 0024 \
    "$root/docs/approval-record.md")
contract_structure=blocked
if /bin/sh "$root/tools/ci/check-alpha-preimplementation-contracts.sh" >/dev/null; then
    contract_structure=ready
fi
lab_contracts=$(/usr/bin/sed -n 's/^readiness=//p' "$root/spec/alpha/lab/development-lab-profile-v2.fields")
boot_contracts=$(/usr/bin/sed -n 's/^readiness=//p' "$root/spec/alpha/boot/alpha-boot-v0.fields")
preimplementation_contracts=blocked
reference_verdict_contract=blocked
fixtures=$root/spec/alpha/lab/fixtures
if /bin/sh "$root/tools/ci/check-reference-verdict-v0.sh" "$fixtures/reference-verdict-accepted.v0" milestone-f "$fixtures/controller-context.v0" "$fixtures/source-context.v0" "$fixtures/comparison-transcript.v0" "$fixtures/reference-inventory.v0" "$fixtures/comparison-evidence.v0" "$fixtures/reference-harness.v0" >/dev/null &&
    /bin/sh "$root/tools/ci/check-reference-verdict-v0.sh" "$fixtures/reference-verdict-not-required.v0" milestone-a "$fixtures/controller-context.v0" "$fixtures/source-context.v0" "$fixtures/comparison-transcript.v0" none none none >/dev/null; then
    reference_verdict_contract=source-ready
fi
image_inputs=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/rar-lab/images/image-inputs-v1.env")
crypto_references=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/sprint-alpha/alpha-crypto-references-v1.env")
qmp_client=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/sprint-alpha/qmp-client-v1.env")
lab_profile=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/sprint-alpha/development-lab-v1.env")
lab_profile_v2=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/sprint-alpha/development-lab-v2.env")
lab_controller_v2=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/sprint-alpha/development-controller-v2.plan")
controller_helper=$(/usr/bin/sed -n 's/^state=//p' "$root/tools/sprint-alpha/controller-helper-v0.env")

local_repository_gates=ready
for state in "$workspace_boundary" "$internal_disk" "$ssd_capacity" "$workspace_budget"; do
    [ "$state" = ready ] || local_repository_gates=blocked
done
[ "$adr_0020" = accepted ] || local_repository_gates=blocked
[ "$adr_0021" = accepted ] || local_repository_gates=blocked
[ "$contract_structure" = ready ] || local_repository_gates=blocked
[ "$lab_contracts" = source-ready-pending-review ] || local_repository_gates=blocked
[ "$boot_contracts" = ready ] || local_repository_gates=blocked
[ "$image_inputs" = ready ] || local_repository_gates=blocked
[ "$crypto_references" = ready ] || local_repository_gates=blocked
[ "$qmp_client" = ready ] || local_repository_gates=blocked
[ "$lab_profile" = ready ] || local_repository_gates=blocked
[ "$lab_profile_v2" = ready ] || local_repository_gates=blocked
[ "$lab_controller_v2" = ready ] || local_repository_gates=blocked
[ "$controller_helper" = ready ] || local_repository_gates=blocked

printf '%s\n' \
    'schema=rar-sprint-alpha-gate-report-v1' \
    'source_revision=strict-preflight-required' \
    'branch=strict-preflight-required' \
    "workspace_boundary=$workspace_boundary" \
    'checkpoint=strict-preflight-required' \
    "internal_disk=$internal_disk" \
    "ssd_capacity=$ssd_capacity" \
    "workspace_budget=$workspace_budget" \
    'permission_profile=manual-evidence-required' \
    "adr_0020=$adr_0020" \
    "adr_0021=$adr_0021" \
    "adr_0022=$adr_0022" \
    "adr_0023=$adr_0023" \
    "adr_0024=$adr_0024" \
    "contract_structure=$contract_structure" \
    "lab_contracts=$lab_contracts" \
    "boot_contracts=$boot_contracts" \
    "preimplementation_contracts=$preimplementation_contracts" \
    "reference_verdict_contract=$reference_verdict_contract" \
    'gui_input_authority=decision-required' \
    "image_inputs=$image_inputs" \
    "crypto_references=$crypto_references" \
    "qmp_client=$qmp_client" \
    "lab_profile=$lab_profile" \
    "lab_profile_v2=$lab_profile_v2" \
    "lab_controller_v2=$lab_controller_v2" \
    "controller_helper=$controller_helper" \
    'remote_workflow=external-evidence-required' \
    'pr_gate=external-evidence-required' \
    'target_implementation=not-started' \
    "local_repository_gates=$local_repository_gates" \
    'overall=blocked'
