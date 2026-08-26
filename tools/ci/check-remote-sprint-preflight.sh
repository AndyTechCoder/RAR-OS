#!/bin/sh
set -eu

LC_ALL=C
LANG=C
export LC_ALL LANG

repository=AndyTechCoder/RAR-OS
workflow=specifications.yml
git_bin=/usr/bin/git
gh_bin=$(command -v gh) || {
    printf '%s\n' 'sprint remote preflight blocked: GitHub CLI is unavailable' >&2
    exit 1
}

fail() {
    printf 'sprint remote preflight blocked: %s\n' "$1" >&2
    exit 1
}

[ "$#" -eq 1 ] || fail 'exactly one immutable checkpoint tag is required'
checkpoint_tag=$1

tools/ci/check-local-sprint-preflight.sh

head=$($git_bin rev-parse HEAD)
case "$head" in *[!0-9a-f]*) fail 'local HEAD is malformed' ;; esac
[ "${#head}" -eq 40 ] || fail 'local HEAD is not a full SHA'
branch=$($git_bin symbolic-ref --quiet --short HEAD) || fail 'detached HEAD'
case "$branch" in *[!A-Za-z0-9._/-]*) fail 'branch name is not canonical' ;; esac

remote_line=$($git_bin ls-remote --heads origin "refs/heads/$branch") ||
    fail 'cannot read the canonical GitHub branch'
remote_head=${remote_line%%[[:space:]]*}
[ "$remote_head" = "$head" ] || fail 'GitHub branch is not the exact local HEAD'

case "$checkpoint_tag" in *[!A-Za-z0-9._/-]*) fail 'checkpoint tag is not canonical' ;; esac
tag_records=$($git_bin ls-remote origin "refs/tags/$checkpoint_tag" "refs/tags/$checkpoint_tag^{}") ||
    fail 'cannot read the checkpoint tag'
/bin/sh tools/ci/verify-remote-checkpoint.sh "$checkpoint_tag" "$head" "$tag_records" ||
    fail 'annotated checkpoint tag is absent, lightweight, or points elsewhere'

run_id=$($gh_bin api "/repos/$repository/actions/workflows/$workflow/runs?head_sha=$head&per_page=20" \
    --jq '.workflow_runs | map(select(.head_sha == "'"$head"'")) | .[0].id // empty') ||
    fail 'cannot inspect required workflow runs'
[ -n "$run_id" ] || fail 'no required workflow run exists for the exact head'
case "$run_id" in *[!0-9]*) fail 'required workflow identity is malformed' ;; esac

run_status=$($gh_bin api "/repos/$repository/actions/runs/$run_id" \
    --jq '.status + "|" + (.conclusion // "")') ||
    fail 'cannot inspect required workflow result'
[ "$run_status" = 'completed|success' ] ||
    fail "required workflow is not successful: $run_status"
started_jobs=$($gh_bin api "/repos/$repository/actions/runs/$run_id/jobs?filter=latest&per_page=100" \
    --jq '[.jobs[] | select((.steps | length) > 0)] | length') ||
    fail 'cannot inspect required workflow job steps'
case "$started_jobs" in '' | *[!0-9]*) fail 'workflow job-step count is malformed' ;; esac
[ "$started_jobs" -ge 1 ] || fail 'required workflow failed before executing a step'

printf 'sprint remote preflight passed: branch=%s head=%s run=%s jobs=%s\n' \
    "$branch" "$head" "$run_id" "$started_jobs"
