#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
os=$(/usr/bin/uname -s)

if [ "$os" = Darwin ]; then
    ssd_root='/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha'
    case "$root" in
        "$ssd_root/repository") ;;
        "$ssd_root/worktrees/"*)
            worktree_name=${root#"$ssd_root/worktrees/"}
            [ -n "$worktree_name" ] || exit 1
            case "$worktree_name" in */*) exit 1 ;; esac
            ;;
        *) exit 1 ;;
    esac
    printf '%s\n' disabled
    exit 0
fi

[ "$os" = Linux ] || exit 1
[ "$root" = /workspace ] || exit 1
[ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ] || exit 1
[ "${GITHUB_ACTIONS-}" = true ] || exit 1
[ "${CI-}" = true ] || exit 1
[ "${RAR_CI_RUNNER_IMAGE_OS-}" = ubuntu24 ] || exit 1
[ "${RAR_CI_RUNNER_OS-}" = Linux ] || exit 1
[ "${RAR_CI_RUNNER_ARCH-}" = X64 ] || exit 1
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || exit 1
/usr/bin/printf '%s\n' "${RAR_CI_RUNNER_IMAGE_VERSION-}" |
    /usr/bin/grep -Eq '^[0-9]{8}\.[0-9]+\.[0-9]+$' || exit 1
/usr/bin/printf '%s\n' "${RAR_EXPECTED_SOURCE_REVISION-}" |
    /usr/bin/grep -Eq '^[0-9a-f]{40}$' || exit 1

/usr/bin/awk '
    $5 == "/" { root_count++; if ($6 ~ /(^|,)ro(,|$)/) root_ro=1 }
    $5 == "/workspace" { workspace_count++; if ($6 ~ /(^|,)ro(,|$)/) workspace_ro=1 }
    END { exit !(root_count == 1 && root_ro && workspace_count == 1 && workspace_ro) }
' /proc/self/mountinfo || exit 1

scratch=/tmp
[ -d "$scratch" ] && [ ! -L "$scratch" ] || exit 1
[ "$(CDPATH= cd -- "$scratch" && pwd -P)" = "$scratch" ] || exit 1
[ "$(/usr/bin/stat -c %u "$scratch")" = "$(/usr/bin/id -u)" ] || exit 1
[ "$(/usr/bin/stat -c %a "$scratch")" = 1777 ] || exit 1
[ "$(/usr/bin/stat -f -c %T "$scratch")" = tmpfs ] || exit 1
/usr/bin/awk '
    $5 == "/tmp" {
        count++
        if ($6 ~ /(^|,)rw(,|$)/ && $6 ~ /(^|,)nosuid(,|$)/ && $6 ~ /(^|,)nodev(,|$)/) safe=1
    }
    END { exit !(count == 1 && safe) }
' /proc/self/mountinfo || exit 1
tmp_kib=$(/bin/df -kP "$scratch" | /usr/bin/awk 'NR == 2 { print $2 }')
case "$tmp_kib" in '' | *[!0-9]*) exit 1 ;; esac
[ "$tmp_kib" -gt 0 ] && [ "$tmp_kib" -le 131072 ] || exit 1

actual_revision=$(GIT_OPTIONAL_LOCKS=0 /usr/bin/git -C "$root" rev-parse HEAD)
[ "$actual_revision" = "$RAR_EXPECTED_SOURCE_REVISION" ] || exit 1
[ -z "$(GIT_OPTIONAL_LOCKS=0 /usr/bin/git -C "$root" status --porcelain --untracked-files=all)" ] || exit 1

printf '%s\n' "$scratch"
