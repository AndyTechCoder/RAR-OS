#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

git diff --check
git diff --check origin/main...HEAD

git ls-files '*.sh' | while IFS= read -r script; do
    /bin/sh -n "$script"
done

/bin/sh tools/ci/check-host-policy.sh

printf '%s\n' 'local read-only checks passed'
