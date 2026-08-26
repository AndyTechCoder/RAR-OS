#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
/bin/mkdir -p "$root/out"
work=$(mktemp -d "$root/out/workspace-budget.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
/bin/mkdir -p "$work/repository/out" "$work/worktrees/task"
/usr/bin/printf '%s\n' fixture > "$work/repository/out/result"
RAR_MINIMUM_SSD_FREE_KIB=0 RAR_MAXIMUM_WORKSPACE_KIB=4096 RAR_MAXIMUM_OUTPUT_KIB=2048 \
    /bin/sh "$root/tools/ci/check-workspace-budget.sh" "$work" >/dev/null
if RAR_MINIMUM_SSD_FREE_KIB=0 RAR_MAXIMUM_WORKSPACE_KIB=0 RAR_MAXIMUM_OUTPUT_KIB=2048 \
    /bin/sh "$root/tools/ci/check-workspace-budget.sh" "$work" >/dev/null 2>&1; then exit 1; fi
if RAR_MINIMUM_SSD_FREE_KIB=0 RAR_MAXIMUM_WORKSPACE_KIB=4096 RAR_MAXIMUM_OUTPUT_KIB=0 \
    /bin/sh "$root/tools/ci/check-workspace-budget.sh" "$work" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'workspace budget negative checks passed'
