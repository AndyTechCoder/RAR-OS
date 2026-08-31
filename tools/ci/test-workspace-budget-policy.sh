#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
values=$root/tools/ci/check-workspace-budget-values.sh
minimum_free_kib=10485760
maximum_workspace_kib=9437184
maximum_output_kib=524288

for file in \
    "$root/tools/ci/check-workspace-budget.sh" \
    "$root/tools/ci/check-local-sprint-preflight.sh" \
    "$root/tools/ci/report-sprint-alpha-gates.sh"; do
    [ "$(grep -Fxc "maximum_workspace_kib=$maximum_workspace_kib" "$file")" -eq 1 ] || exit 1
done
grep -Fq 'above 9 GiB total RAR OS workspace' "$root/AGENTS.md" || exit 1
grep -Fq '10-GiB free, 9-GiB workspace, and 512-MiB output ceilings.' \
    "$root/docs/host-safety.md" || exit 1
grep -Fq '8 GiB (8388608 KiB) to 9 GiB (9437184 KiB)' \
    "$root/docs/approval-record.md" || exit 1

/bin/sh "$values" "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib" \
    "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib"
if /bin/sh "$values" 10485759 "$maximum_workspace_kib" "$maximum_output_kib" \
    "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib"; then exit 1; fi
if /bin/sh "$values" "$minimum_free_kib" 9437185 "$maximum_output_kib" \
    "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib"; then exit 1; fi
if /bin/sh "$values" "$minimum_free_kib" "$maximum_workspace_kib" 524289 \
    "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib"; then exit 1; fi
for bad in '' -1 +1 1x 999999999999999999999999999999; do
    if /bin/sh "$values" "$bad" "$maximum_workspace_kib" "$maximum_output_kib" \
        "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib" 2>/dev/null; then
        exit 1
    fi
done
if /bin/sh "$values" 100 50 5 0 100 10; then exit 1; fi
if /bin/sh "$root/tools/ci/check-workspace-budget.sh" /; then exit 1; fi
if RAR_MINIMUM_SSD_FREE_KIB=0 /bin/sh "$root/tools/ci/check-workspace-budget.sh"; then exit 1; fi
if RAR_MAXIMUM_WORKSPACE_KIB=0 /bin/sh "$root/tools/ci/check-workspace-budget.sh"; then exit 1; fi
if RAR_MAXIMUM_OUTPUT_KIB=0 /bin/sh "$root/tools/ci/check-workspace-budget.sh"; then exit 1; fi
printf '%s\n' 'workspace budget negative checks passed'
