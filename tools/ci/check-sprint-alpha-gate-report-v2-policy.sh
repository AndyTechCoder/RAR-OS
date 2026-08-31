#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
reporter=$root/tools/ci/report-sprint-alpha-gates-v2.sh
contract=$root/tools/ci/contracts/sprint-alpha-gate-report-v2.fields
historical=$root/tools/ci/report-sprint-alpha-gates.sh
fail() { printf 'Sprint Alpha gate report v2 policy rejected: %s\n' "$1" >&2; exit 1; }

for file in "$reporter" "$contract" "$historical"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
done
[ "$(/usr/bin/head -1 "$reporter")" = '#!/bin/sh' ] || fail 'reporter interpreter changed'

hash_file() {
    if [ -x /usr/bin/sha256sum ]; then
        /usr/bin/sha256sum "$1" | /usr/bin/awk '{ print $1 }'
    else
        /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
    fi
}

[ "$(hash_file "$contract")" = 965e7fa198d9a62b6a391e980207f857e75cbd44383766e21e1be839c03ee12c ] || fail 'reporter contract bytes changed'
[ "$(hash_file "$historical")" = 78782b52f2b063c0dd56f3778e17242dc21088729da7f075a2bc74ab038fa8f8 ] || fail 'historical v1 reporter bytes changed'

for required in \
    'schema=rar-sprint-alpha-gate-report-v2' \
    'docs/adr/0026-alpha-platform-payload-and-state-sources.md' \
    'platform_envelope_state=' \
    'platform_envelope_contract_sha256=' \
    'core_bootstrap_state=' \
    'core_bootstrap_contract_sha256=' \
    'component_bundle_state=' \
    'component_bundle_fixture_path=unavailable' \
    'component_bundle_fixture_sha256=unavailable' \
    'initial_system_state=' \
    'initial_system_fixture_path=unavailable' \
    'initial_system_fixture_sha256=unavailable' \
    'initial_preserved_state=' \
    'initial_preserved_fixture_path=unavailable' \
    'initial_preserved_fixture_sha256=unavailable' \
    'platform_fixture_manifest_state=' \
    'platform_fixture_manifest_sha256=unavailable' \
    'platform_source_set=' \
    'acceptance_protocol_v2=' \
    'acceptance_v2_plan_sha256=' \
    'milestone_a_readiness=' \
    'milestone_b_readiness=' \
    'milestone_e_readiness=' \
    'target_implementation=not-evaluated' \
    'overall='; do
    /usr/bin/grep -Fq -- "$required" "$reporter" || fail "required reporter binding missing: $required"
done

! /usr/bin/grep -Fq 'docs/proposals/0026-' "$reporter" || fail 'historical ADR proposal used as authority'
! /usr/bin/grep -Fq 'acceptance-v1.plan' "$reporter" || fail 'historical acceptance v1 can influence v2'
! /usr/bin/grep -Fq 'internal_disk=' "$reporter" || fail 'internal Mac capacity became an Alpha gate'
! /usr/bin/grep -Ei '(^|[^A-Za-z])(git|gh|curl|wget|ssh|scp|docker|podman|qemu|rustc|cargo|clang|gcc|ld|objcopy|sudo|rm|mv|cp|install|chmod|chown)([^A-Za-z]|$)' "$reporter" >/dev/null || fail 'reporter contains forbidden command family'
! /usr/bin/grep -E '(^|[[:space:]])(>|>>|tee)([[:space:]]|$)' "$reporter" >/dev/null || fail 'reporter contains an external write path'

report=$(/bin/sh "$reporter") || fail 'reporter did not produce a fail-closed report'
[ "$(printf '%s\n' "$report" | /usr/bin/wc -l | /usr/bin/tr -d ' ')" -eq 47 ] || fail 'report field count changed'
expected_keys='schema
report_contract_sha256
source_revision
branch
checkpoint
permission_profile
workspace_boundary
ssd_capacity
workspace_budget
adr_0020
adr_0021
adr_0022
adr_0023
adr_0024
adr_0025
adr_0026
boot_contract_state
controller_state
lab_profile_state
helper_state
controller_readiness
platform_envelope_state
platform_envelope_contract_sha256
core_bootstrap_state
core_bootstrap_contract_sha256
component_bundle_state
component_bundle_fixture_path
component_bundle_fixture_sha256
initial_system_state
initial_system_fixture_path
initial_system_fixture_sha256
initial_preserved_state
initial_preserved_fixture_path
initial_preserved_fixture_sha256
platform_fixture_manifest_state
platform_fixture_manifest_sha256
platform_source_set
acceptance_protocol_v2
acceptance_v2_plan_sha256
milestone_a_readiness
milestone_b_readiness
milestone_e_readiness
remote_workflow
pr_gate
target_implementation
local_repository_gates
overall'
actual_keys=$(printf '%s\n' "$report" | /usr/bin/awk -F= 'NF >= 2 { print $1 }')
[ "$actual_keys" = "$expected_keys" ] || fail 'report field order or names changed'

if [ -f "$root/spec/alpha/platform/contract-set-v0.manifest" ]; then
    platform_state_expectations='platform_envelope_state=pending-review
core_bootstrap_state=pending-review
component_bundle_state=pending-review
initial_system_state=pending-review
initial_preserved_state=pending-review
platform_fixture_manifest_state=pending-review'
else
    [ ! -e "$root/spec/alpha/platform" ] && [ ! -L "$root/spec/alpha/platform" ] ||
        fail 'partial platform topology exists without its contract manifest'
    platform_state_expectations='platform_envelope_state=missing
core_bootstrap_state=missing
component_bundle_state=missing
initial_system_state=missing
initial_preserved_state=missing
platform_fixture_manifest_state=missing'
fi

for expected in \
    'adr_0026=accepted' \
    'platform_source_set=blocked' \
    'acceptance_protocol_v2=reviewed-implementation-required' \
    'acceptance_v2_plan_sha256=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd' \
    'milestone_a_readiness=blocked' \
    'milestone_b_readiness=blocked' \
    'overall=blocked'; do
    [ "$(printf '%s\n' "$report" | /usr/bin/grep -Fxc -- "$expected")" -eq 1 ] || fail "current fail-closed state changed: $expected"
done
printf '%s\n' "$platform_state_expectations" | while IFS= read -r expected; do
    [ "$(printf '%s\n' "$report" | /usr/bin/grep -Fxc -- "$expected")" -eq 1 ] || fail "current fail-closed state changed: $expected"
done

printf '%s\n' 'Sprint Alpha gate report v2 policy passed'
