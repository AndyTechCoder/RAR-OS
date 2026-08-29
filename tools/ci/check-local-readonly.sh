#!/bin/sh
set -eu

PATH=/usr/bin:/bin
export PATH

git_bin=/usr/bin/git
root=$(CDPATH= cd -- "$(/usr/bin/dirname -- "$0")/../.." && pwd -P)
cd "$root"

run_git() {
    /usr/bin/env -i \
        GIT_CONFIG_NOSYSTEM=1 \
        GIT_OPTIONAL_LOCKS=0 \
        GIT_TERMINAL_PROMPT=0 \
        HOME=/nonexistent-rar-local-check-home \
        LANG=C \
        LC_ALL=C \
        PATH=/usr/bin:/bin \
        XDG_CONFIG_HOME=/nonexistent-rar-local-check-config \
        "$git_bin" --no-pager \
        -c core.fsmonitor=false \
        -c core.untrackedCache=false \
        -c "safe.directory=$root" \
        -C "$root" \
        "$@"
}

run_git diff --no-ext-diff --no-textconv --check
run_git diff --no-ext-diff --no-textconv --check origin/main...HEAD

run_git ls-files '*.sh' | while IFS= read -r script; do
    /bin/sh -n "$script"
done

/bin/sh tools/ci/check-host-policy.sh

printf '%s\n' 'local read-only checks passed'
