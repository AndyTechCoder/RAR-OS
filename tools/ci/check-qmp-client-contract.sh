#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=${1-$root/tools/sprint-alpha/qmp-client-v1.env}
controller_root=${2-$root}
[ -f "$contract" ] && [ ! -L "$contract" ] || exit 1
[ "$(/usr/bin/wc -l < "$contract" | /usr/bin/tr -d ' ')" -eq 11 ] || exit 1
if /usr/bin/grep -Ev '^(schema|state|owner|version|license|source_tree|source_sha256|build_plan_sha256|binary_sha256|verbs|replacement)=[A-Za-z0-9._,/-]+$' "$contract" | /usr/bin/grep -q .; then exit 1; fi
# shellcheck disable=SC1090
. "$contract"
[ "$schema|$owner|$version|$license" = 'rar-qmp-client-contract-v1|RAR|1|Proprietary-RAR' ] || exit 1
[ "$verbs" = 'wait-ready,continue,key-chord,pointer,serial-offset,wait-trace,capture,quit' ] || exit 1
[ "$replacement" = versioned-host-tool-contract ] || exit 1
case "$state" in
    blocked) [ "$source_tree|$source_sha256|$build_plan_sha256|$binary_sha256" = 'unavailable|unavailable|unavailable|unavailable' ] || exit 1 ;;
    ready)
        [ "$source_tree" = /controller/tools/rar-lab/qmp-client ] || exit 1
        for digest in "$source_sha256" "$build_plan_sha256" "$binary_sha256"; do
            [ "${#digest}" -eq 64 ] || exit 1
            case "$digest" in *[!0-9a-f]*) exit 1 ;; esac
        done
        actual_tree=$controller_root/${source_tree#/controller/}
        [ -d "$actual_tree" ] && [ ! -L "$actual_tree" ] || exit 1
        /bin/mkdir -p "$controller_root/out"
        [ -d "$controller_root/out" ] && [ ! -L "$controller_root/out" ] || exit 1
        actual_source=$(/bin/sh "$root/tools/ci/hash-source-tree.sh" "$actual_tree" "$controller_root/out") || exit 1
        [ "$actual_source" = "$source_sha256" ] || exit 1
        plan=$actual_tree/build-plan.v1
        [ -f "$plan" ] && [ ! -L "$plan" ] || exit 1
        if command -v sha256sum >/dev/null 2>&1; then plan_output=$(sha256sum "$plan"); else plan_output=$(/usr/bin/shasum -a 256 "$plan"); fi
        [ "${plan_output%% *}" = "$build_plan_sha256" ] || exit 1
        ;;
    *) exit 1 ;;
esac
printf 'QMP client contract validation passed: state=%s\n' "$state"
