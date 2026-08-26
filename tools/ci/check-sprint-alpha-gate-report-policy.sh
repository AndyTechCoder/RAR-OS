#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
reporter=$root/tools/ci/report-sprint-alpha-gates.sh
[ -f "$reporter" ] && [ ! -L "$reporter" ] || exit 1
[ "$(/usr/bin/head -1 "$reporter")" = '#!/bin/sh' ] || exit 1
for required in \
    'schema=rar-sprint-alpha-gate-report-v1' \
    "safe_root='/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha'" \
    'permission_profile=manual-evidence-required' \
    'remote_workflow=external-evidence-required' \
    'pr_gate=external-evidence-required' \
    'gui_input_authority=decision-required' \
    'contract_structure=' \
    'lab_contracts=' \
    'boot_contracts=' \
    'preimplementation_contracts=' \
    'adr_0022=' \
    'adr_0023=' \
    'target_implementation=not-started'; do
    /usr/bin/grep -Fq "$required" "$reporter" || exit 1
done
for required in \
    "'source_revision=strict-preflight-required'" \
    "'checkpoint=strict-preflight-required'" \
    "'overall=blocked'" \
    'classify-proposed-adr.sh'; do
    /usr/bin/grep -Fq "$required" "$reporter" || exit 1
done
! /usr/bin/grep -Ei '(^|[^A-Za-z])(git|gh|curl|wget|ssh|scp|docker|podman|qemu|rustc|cargo|clang|gcc|ld|objcopy|sudo|rm|mv|cp|install)([^A-Za-z]|$)' "$reporter" >/dev/null || exit 1
! /usr/bin/grep -E '(^|[[:space:]])(>|>>|tee)([[:space:]]|$)' "$reporter" >/dev/null || exit 1
printf '%s\n' 'Sprint Alpha gate reporter policy passed'
