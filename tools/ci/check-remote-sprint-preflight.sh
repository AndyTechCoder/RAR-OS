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

run_title="Specifications source $head"
run_event=push
run_id=$($gh_bin api "/repos/$repository/actions/workflows/$workflow/runs?event=push&head_sha=$head&per_page=20" \
    --jq '.workflow_runs | map(select(.head_sha == "'"$head"'" and .display_title == "'"$run_title"'" and .event == "push")) | .[0].id // empty') ||
    fail 'cannot inspect resulting-main workflow runs'
if [ -z "$run_id" ]; then
    run_event=pull_request_target
    run_id=$($gh_bin api "/repos/$repository/actions/workflows/$workflow/runs?event=pull_request_target&per_page=100" \
        --jq '.workflow_runs | map(select(.display_title == "'"$run_title"'" and .event == "pull_request_target")) | .[0].id // empty') ||
        fail 'cannot inspect trusted-controller pull-request workflow runs'
fi
[ -n "$run_id" ] || fail 'no required workflow run exists for the exact head'
case "$run_id" in *[!0-9]*) fail 'required workflow identity is malformed' ;; esac

run_identity=$($gh_bin api "/repos/$repository/actions/runs/$run_id" \
    --jq '.name + "|" + .display_title + "|" + .event + "|" + .head_sha + "|" + (.run_attempt | tostring) + "|" + .status + "|" + (.conclusion // "")') ||
    fail 'cannot inspect required workflow result'
IFS='|' read -r observed_name observed_title observed_event controller_sha run_attempt run_status run_conclusion <<EOF
$run_identity
EOF
[ "$observed_name" = Specifications ] || fail 'required workflow name is wrong'
[ "$observed_title" = "$run_title" ] || fail 'required workflow source identity is wrong'
[ "$observed_event" = "$run_event" ] || fail 'required workflow event is wrong'
case "$controller_sha" in *[!0-9a-f]*|'') fail 'workflow controller SHA is malformed' ;; esac
[ "${#controller_sha}" -eq 40 ] || fail 'workflow controller SHA length is invalid'
case "$run_attempt" in '' | *[!0-9]*) fail 'workflow attempt is malformed' ;; esac
[ "$run_attempt" -ge 1 ] || fail 'workflow attempt is invalid'
[ "$run_status|$run_conclusion" = 'completed|success' ] ||
    fail "required workflow is not successful: $run_status|$run_conclusion"
[ "$run_event" != push ] || [ "$controller_sha" = "$head" ] ||
    fail 'resulting-main controller is not the exact source commit'

full_jobs=$($gh_bin api "/repos/$repository/actions/runs/$run_id/jobs?filter=latest&per_page=100" \
    --jq '[.jobs[] | select(.name == "attest-runner-and-validate" and .status == "completed" and .conclusion == "success" and ([.steps[] | select(.name == "Bind executable validation authority to trusted controller" and .status == "completed" and .conclusion == "success")] | length) == 1 and ([.steps[] | select(.name == "Validate in pinned read-only container on attested runner" and .status == "completed" and .conclusion == "success")] | length) == 1 and ([.steps[] | select(.name == "Run mutation policy tests with read-only source" and .status == "completed" and .conclusion == "success")] | length) == 1)] | length') ||
    fail 'cannot inspect full-validation workflow steps'
[ "$full_jobs" = 1 ] || fail 'required workflow was deferred, skipped, duplicated, or incomplete'

printf 'sprint remote preflight passed: branch=%s source=%s controller=%s event=%s run=%s attempt=%s full_jobs=%s\n' \
    "$branch" "$head" "$controller_sha" "$run_event" "$run_id" "$run_attempt" "$full_jobs"
