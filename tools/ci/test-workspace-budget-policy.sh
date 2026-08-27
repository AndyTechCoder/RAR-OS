#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
values=$root/tools/ci/check-workspace-budget-values.sh
/bin/sh "$values" 100 50 5 10 100 10
if /bin/sh "$values" 9 50 5 10 100 10; then exit 1; fi
if /bin/sh "$values" 100 101 5 10 100 10; then exit 1; fi
if /bin/sh "$values" 100 50 11 10 100 10; then exit 1; fi
if /bin/sh "$values" 100 50 5 0 100 10; then exit 1; fi
if /bin/sh "$root/tools/ci/check-workspace-budget.sh" /; then exit 1; fi
if RAR_MINIMUM_SSD_FREE_KIB=0 /bin/sh "$root/tools/ci/check-workspace-budget.sh"; then exit 1; fi
printf '%s\n' 'workspace budget negative checks passed'
