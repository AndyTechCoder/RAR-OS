#!/bin/sh
set -eu

LC_ALL=C
LANG=C
export LC_ALL LANG

safe_root='/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha'
identity_file=$safe_root/.rar-os-workspace-identity
identity_sha256=f71483fc7335d5c0949541bad24143b437c379250c70e47dbe7a0b766decd496
canonical_https=https://github.com/AndyTechCoder/RAR-OS.git
canonical_ssh=git@github.com:AndyTechCoder/RAR-OS.git
minimum_ssd_free_kib=10485760
maximum_workspace_kib=8388608
git_bin=/usr/bin/git
uname_bin=/usr/bin/uname

fail() {
    printf 'sprint preflight blocked: %s\n' "$1" >&2
    exit 1
}

[ "$("$uname_bin" -s)" = Darwin ] || fail 'this check is only for the owner Mac'
[ -d "$safe_root" ] || fail 'the exact SSD workspace is not mounted'
[ ! -L "$safe_root" ] || fail 'the SSD workspace root must not be a symbolic link'
[ -f "$identity_file" ] && [ ! -L "$identity_file" ] ||
    fail 'the SSD workspace guard marker is missing or unsafe'
identity_output=$(/usr/bin/shasum -a 256 "$identity_file") ||
    fail 'the SSD workspace guard marker cannot be hashed'
[ "${identity_output%% *}" = "$identity_sha256" ] ||
    fail 'the SSD workspace guard marker does not match'

repo_root=$($git_bin rev-parse --show-toplevel 2>/dev/null) || fail 'not inside a Git worktree'
repo_root=$(CDPATH= cd -- "$repo_root" && pwd -P)
case "$repo_root" in
    "$safe_root/repository" | "$safe_root/worktrees/"*) ;;
    *) fail 'Git worktree resolves outside the dedicated SSD workspace' ;;
esac

remote=$($git_bin remote get-url origin 2>/dev/null) || fail 'origin is missing'
case "$remote" in
    "$canonical_https" | "$canonical_ssh") ;;
    *) fail 'origin is not the canonical RAR OS repository' ;;
esac

$git_bin status --porcelain | /usr/bin/grep -q . && fail 'worktree is not clean'

branch=$($git_bin symbolic-ref --quiet --short HEAD 2>/dev/null) ||
    fail 'detached HEAD is not an implementation checkpoint'
upstream=$($git_bin rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null) ||
    fail 'current branch has no pushed upstream'
case "$upstream" in origin/*) ;; *) fail 'upstream is not on canonical origin' ;; esac
divergence=$($git_bin rev-list --left-right --count '@{upstream}...HEAD' 2>/dev/null) ||
    fail 'cannot compare the local and upstream checkpoints'
set -- $divergence
[ "$#" -eq 2 ] && [ "$1" -eq 0 ] && [ "$2" -eq 0 ] ||
    fail "local branch is not identical to its fetched upstream: $divergence"

$git_bin worktree list --porcelain | /usr/bin/sed -n 's/^worktree //p' | while IFS= read -r path; do
    resolved=$(CDPATH= cd -- "$path" && pwd -P) || fail 'registered worktree is unavailable'
    case "$resolved" in
        "$safe_root/repository" | "$safe_root/worktrees/"*) ;;
        *) fail 'a registered worktree escapes the dedicated SSD workspace' ;;
    esac
done

ssd_free_kib=$(/bin/df -Pk "$safe_root" | /usr/bin/awk 'END { print $4 }')
case "$ssd_free_kib" in '' | *[!0-9]*) fail 'cannot determine SSD free space' ;; esac
[ "$ssd_free_kib" -ge "$minimum_ssd_free_kib" ] ||
    fail 'SSD has less than 10 GiB free; do not start unattended work'

workspace_kib=$(/usr/bin/du -sk "$safe_root" | /usr/bin/awk 'NR == 1 { print $1 }')
case "$workspace_kib" in '' | *[!0-9]*) fail 'cannot determine SSD workspace size' ;; esac
[ "$workspace_kib" -le "$maximum_workspace_kib" ] ||
    fail 'RAR OS SSD workspace exceeds the 8 GiB unattended-work ceiling'

printf 'sprint local preflight passed: branch=%s head=%s\n' \
    "$branch" "$($git_bin rev-parse HEAD)"
