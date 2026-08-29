#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'local preflight mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/local-preflight.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
safe=$work/safe
repo=$safe/repository
remote=$work/remote.git
/bin/mkdir -p "$safe"

/usr/bin/git init --bare "$remote" >/dev/null
/usr/bin/git clone "$remote" "$repo" >/dev/null 2>&1
/usr/bin/git -C "$repo" config user.name 'RAR preflight fixture'
/usr/bin/git -C "$repo" config user.email 'fixture@invalid.example'
/usr/bin/printf '%s\n' fixture > "$repo/README"
/usr/bin/printf '%s\n' '._*' > "$repo/.gitignore"
/usr/bin/git -C "$repo" add README .gitignore
/usr/bin/git -C "$repo" commit -m fixture >/dev/null
/usr/bin/git -C "$repo" push -u origin HEAD:main >/dev/null 2>&1
/usr/bin/git -C "$repo" branch -M main
/usr/bin/git -C "$repo" branch --set-upstream-to=origin/main main >/dev/null

/usr/bin/printf '%s\n' \
    'schema=rar-os-workspace-identity-v1' \
    'repository=AndyTechCoder/RAR-OS' \
    'safety_root=/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha' \
    > "$safe/.rar-os-workspace-identity"

fixture=$work/check.sh
/usr/bin/sed \
    -e "s|^safe_root=.*|safe_root='$safe'|" \
    -e "s|^canonical_https=.*|canonical_https='$remote'|" \
    -e "s|^minimum_ssd_free_kib=.*|minimum_ssd_free_kib=0|" \
    -e "s|^maximum_workspace_kib=.*|maximum_workspace_kib=1048576|" \
    -e 's|^uname_system=.*|uname_system=Darwin|' \
    "$root/tools/ci/check-local-sprint-preflight.sh" > "$fixture"

(cd "$repo" && /bin/sh "$fixture" >/dev/null)

wrong_system=$work/check-wrong-system.sh
/usr/bin/sed 's|^uname_system=.*|uname_system=Linux|' "$fixture" > "$wrong_system"
if (cd "$repo" && /bin/sh "$wrong_system" >/dev/null 2>&1); then exit 1; fi

ssd_low=$work/check-ssd-low.sh
/usr/bin/sed 's/^minimum_ssd_free_kib=.*/minimum_ssd_free_kib=999999999999/' "$fixture" > "$ssd_low"
if (cd "$repo" && /bin/sh "$ssd_low" >/dev/null 2>&1); then exit 1; fi

workspace_large=$work/check-workspace-large.sh
/usr/bin/sed 's/^maximum_workspace_kib=.*/maximum_workspace_kib=0/' "$fixture" > "$workspace_large"
if (cd "$repo" && /bin/sh "$workspace_large" >/dev/null 2>&1); then exit 1; fi

expect_rejected() {
    label=$1
    if (cd "$repo" && /bin/sh "$fixture" >/dev/null 2>&1); then
        printf 'unsafe preflight fixture unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

/usr/bin/printf '%s\n' dirty > "$repo/dirty"
expect_rejected dirty-worktree
/bin/rm -f "$repo/dirty"

/usr/bin/git -C "$repo" remote set-url origin "$work/wrong.git"
expect_rejected wrong-remote
/usr/bin/git -C "$repo" remote set-url origin "$remote"

/usr/bin/printf '%s\n' wrong > "$safe/.rar-os-workspace-identity"
expect_rejected wrong-workspace-identity
/usr/bin/printf '%s\n' \
    'schema=rar-os-workspace-identity-v1' \
    'repository=AndyTechCoder/RAR-OS' \
    'safety_root=/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha' \
    > "$safe/.rar-os-workspace-identity"

/usr/bin/git -C "$repo" branch --unset-upstream
expect_rejected missing-upstream
/usr/bin/git -C "$repo" branch --set-upstream-to=origin/main main >/dev/null

/usr/bin/printf '%s\n' ahead > "$repo/ahead"
/usr/bin/git -C "$repo" add ahead
/usr/bin/git -C "$repo" commit -m ahead >/dev/null
expect_rejected unpushed-commit
/usr/bin/git -C "$repo" push origin main >/dev/null 2>&1
(cd "$repo" && /bin/sh "$fixture" >/dev/null)

printf '%s\n' 'local sprint preflight negative checks passed'
