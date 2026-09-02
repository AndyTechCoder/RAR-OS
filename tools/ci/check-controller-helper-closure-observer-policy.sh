#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=${1-}
[ -n "$root" ] || root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
fail() { printf 'controller-helper observer policy failed: %s\n' "$1" >&2; exit 1; }
workflow=$root/.github/workflows/controller-helper-closure-observer.yml
wrapper=$root/tools/ci/run-controller-helper-closure-observer.sh
harness=$root/tools/ci/controller-helper-closure-observer-harness.sh
observer=$root/tools/ci/observe-controller-helper-closure.sh
catalog=$root/tools/ci/fixtures/controller-helper-closure-observer/cases.v0
fixture=$root/tools/ci/fixtures/controller-helper-closure-observer/base-closure.v0
pins=$root/tools/ci/fixtures/controller-helper-closure-observer/tool-pins.v0
receipt=$root/tools/ci/fixtures/controller-helper-closure-observer/expected-observation.receipt.v0
for file in "$workflow" "$wrapper" "$harness" "$observer" "$catalog" "$fixture" "$pins" "$receipt"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "required file missing: $file"
done
/bin/sh -n "$harness" "$observer" || fail 'POSIX shell syntax invalid'
[ "$(/usr/bin/awk 'NR == 1 { print; exit }' "$wrapper")" = '#!/usr/bin/bash' ] || fail 'wrapper Bash identity changed'
for forbidden in workflow_dispatch repository_dispatch pull_request pull_request_target schedule workflow_call workflow_run; do
    ! /usr/bin/grep -Eq "^[[:space:]]*$forbidden:" "$workflow" || fail "forbidden trigger: $forbidden"
done
[ "$(/usr/bin/grep -Fxc '  push:' "$workflow")" -eq 1 ] || fail 'push trigger not exact'
[ "$(/usr/bin/grep -Fxc '      - main' "$workflow")" -eq 1 ] || fail 'main branch not exact'
[ "$(/usr/bin/grep -Fxc '  contents: read' "$workflow")" -eq 1 ] || fail 'permissions not read-only'
[ "$(/usr/bin/grep -Fxc '    runs-on: ubuntu-24.04' "$workflow")" -eq 1 ] || fail 'runner not pinned'
[ "$(/usr/bin/grep -Fxc '    timeout-minutes: 15' "$workflow")" -eq 1 ] || fail 'observe-job deadline changed'
[ "$(/usr/bin/grep -Fc 'actions/checkout@' "$workflow")" -eq 0 ] || fail 'checkout action receives an implicit token'
[ "$(/usr/bin/grep -Fxc '        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2' "$workflow")" -eq 1 ] || fail 'upload action pin changed'
[ "$(/usr/bin/grep -Fxc '      - name: Acquire exact main into bounded storage' "$workflow")" -eq 1 ] || fail 'bounded anonymous acquisition step missing'
for required in \
    'checkout_image=rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
    'identity="rar-c2b-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"' \
    'keeper="$identity-keeper"' 'keeper_uid=65532' 'keeper_gid=65532' \
    'docker_config="$RUNNER_TEMP/$identity-docker-config"' \
    'workspace_parent=$(/usr/bin/dirname "$GITHUB_WORKSPACE")' \
    'partial="$workspace_parent/.rar-c2b-checkout-partial-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"' \
    'partial_device=$(/usr/bin/stat -c %d "$partial")' \
    'partial_inode=$(/usr/bin/stat -c %i "$partial")' \
    'parent_device=$(/usr/bin/stat -c %d "$workspace_parent")' \
    'parent_inode=$(/usr/bin/stat -c %i "$workspace_parent")' \
    'container_absent() {' 'volume_absent() {' 'keeper_policy_exact() {' 'remove_container() {' 'force_remove_created_container() {' 'remove_volume() {' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock container ls -a' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock volume ls' \
    'trap cleanup EXIT' "trap 'exit 130' HUP INT TERM" \
    'if [[ "$cleanup_failed" -ne 0 ]]; then exit 1; fi' \
    '"$RUNNER_TEMP"/rar-c2b-"$GITHUB_RUN_ID"-"$GITHUB_RUN_ATTEMPT"-docker-config)' \
    '/usr/bin/install -d -m 700 "$docker_config"' '/usr/bin/rmdir "$docker_config"' \
    '[ -z "$(/usr/bin/find "$docker_config" -mindepth 1 -maxdepth 1 -print -quit)" ]' \
    'checkout_repo_digest=rust@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
    'needs_pull=0' \
    'image ls --no-trunc --digests --format '"'"'{{.Repository}}@{{.Digest}}'"'"'' \
    'if /usr/bin/grep -Fqx -- "$checkout_repo_digest" <<< "$image_inventory"; then' \
    'cached_platform=$(/usr/bin/timeout --signal=KILL 10s /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock image inspect --format '"'"'{{.Os}}/{{.Architecture}}'"'"' "$checkout_image")' \
    'if [[ "$cached_platform" != linux/amd64 ]]; then needs_pull=1; fi' \
    'if [[ "$needs_pull" -eq 1 ]]; then' \
    '/usr/bin/timeout --signal=KILL 180s /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock pull --quiet --platform linux/amd64 "$checkout_image"' \
    'platform=$(/usr/bin/timeout --signal=KILL 10s /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock image inspect --format '"'"'{{.Os}}/{{.Architecture}}'"'"' "$checkout_image")' \
    '[[ "$platform" == linux/amd64 ]]' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock volume create --driver local' '--opt type=tmpfs --opt device=tmpfs' \
    'o=size=67108864,uid=$runner_uid,gid=$runner_gid,mode=0700,noexec,nosuid,nodev' \
    'volume inspect --format '"'"'{{index .Options "o"}}'"'"'' \
    '[[ "$runner_uid" != "$keeper_uid" ]]' '[[ "$runner_gid" != "$keeper_gid" ]]' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock create --pull=never --name "$keeper" --interactive --read-only --network none' \
    '--user "$keeper_uid:$keeper_gid" --cpus 0.25 --memory 64m --memory-swap 64m --pids-limit 8' \
    '--mount "type=volume,source=$volume,target=/keep,readonly,volume-nocopy"' \
    '{{.State.Running}}|{{.State.Restarting}}|{{.RestartCount}}|{{.Config.User}}|{{.HostConfig.NetworkMode}}|{{.HostConfig.ReadonlyRootfs}}|{{.HostConfig.RestartPolicy.Name}}|{{len .Mounts}}' \
    'true|false|0|$keeper_uid:$keeper_gid|none|true|no|1|volume|$volume|/keep|false' \
    'a6f559e00b69a4aa4d8cb607be18d9386c5aee55c509e2c075549dcf00e00fc7' \
    '[ ! -r /keep ] || fail "volume readable"' '[ ! -x /keep ] || fail "volume searchable"' \
    'if IFS= read -r _; then fail "unexpected input"; fi' 'fail "stdin closed"' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start "$keeper"' 'keeper_policy_exact' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock create --pull=never --name "$acquisition" --read-only --network bridge' \
    '--mount "type=volume,source=$volume,target=/checkout"' \
    'GIT_CONFIG_NOSYSTEM=1' 'GIT_TERMINAL_PROMPT=0 GIT_ASKPASS=/bin/false' \
    '356db14e102d68a1a37d8a1ac577dfd678d45d46e92f468bef8b7154e7bfdc60' \
    '/usr/bin/git init /checkout' \
    '/usr/bin/git -C /checkout remote add origin https://github.com/AndyTechCoder/RAR-OS.git' \
    '-c filter.lfs.smudge= -c filter.lfs.required=false' \
    'fetch --no-tags --depth=1 origin "$1"' 'checkout --detach FETCH_HEAD' \
    '/usr/bin/timeout --signal=KILL 120s /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start --attach "$acquisition"' \
    'force_remove_created_container "$acquisition"' 'container_absent "$transfer"' 'remove_container "$keeper"' 'container_absent "$keeper"' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock create --pull=never --name "$transfer" --read-only --network none' \
    '--mount "type=volume,source=$volume,target=/source,readonly"' \
    '--mount "type=bind,source=$partial,target=/destination"' \
    '/usr/bin/cp -a /source/.git /destination/.git' '/usr/bin/cp -a /source/. /destination/' \
    '/usr/bin/timeout --signal=KILL 30s /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start --attach "$transfer"' \
    'remove_container "$transfer"' 'remove_volume "$volume"' \
    '[[ "$partial_device" == "$parent_device" ]]' \
    '[[ "$(/usr/bin/stat -c %d "$partial")" == "$partial_device" ]]' \
    '[[ "$(/usr/bin/stat -c %i "$partial")" == "$partial_inode" ]]' \
    '[[ "$(/usr/bin/stat -c %d "$workspace_parent")" == "$parent_device" ]]' \
    '[[ "$(/usr/bin/stat -c %i "$workspace_parent")" == "$parent_inode" ]]' \
    '/usr/bin/chmod -R a-w "$partial"' '/usr/bin/rmdir "$GITHUB_WORKSPACE"' \
    '/usr/bin/mv -T "$partial" "$GITHUB_WORKSPACE"' \
    '[[ "$(/usr/bin/stat -c %d "$GITHUB_WORKSPACE")" == "$partial_device" ]]' \
    '[[ "$(/usr/bin/stat -c %i "$GITHUB_WORKSPACE")" == "$partial_inode" ]]'; do
    /usr/bin/grep -Fq -- "$required" "$workflow" || fail "bounded acquisition boundary missing: $required"
done
[ "$(/usr/bin/grep -Fc -- '--network bridge' "$workflow")" -eq 1 ] || fail 'acquisition network authority changed'
[ "$(/usr/bin/grep -Fc -- '--pull=never' "$workflow")" -eq 3 ] || fail 'workflow image pull denial changed'
workflow_docker_calls=$(/usr/bin/grep -Fc '/usr/bin/docker' "$workflow")
[ "$workflow_docker_calls" -eq 23 ] || fail 'workflow Docker call inventory changed'
[ "$workflow_docker_calls" -eq "$(/usr/bin/grep -Fc '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock' "$workflow")" ] || fail 'workflow Docker endpoint or client config is ambient'
docker_inventory=$(/usr/bin/awk 'index($0, "/usr/bin/docker") { print }' "$workflow") || fail 'cannot extract workflow Docker inventory'
docker_inventory_digest_output=$(/usr/bin/printf '%s\n' "$docker_inventory" | /usr/bin/shasum -a 256) || fail 'cannot hash workflow Docker inventory'
docker_inventory_sha256=${docker_inventory_digest_output%% *}
[ "$docker_inventory_sha256" = 96c014ad4025d9bc83d01dbb4e1997ca8b36c8b1a344dc945941d71fc425f53d ] || fail 'workflow Docker call inventory bytes changed'
docker_create_blocks=$(/usr/bin/awk '
    index($0, "--name \"$keeper\" --interactive") {
        if (capture != "") bad=1
        capture="keeper"
        blocks++
        print
        next
    }
    index($0, "--name \"$acquisition\" --read-only") {
        if (capture != "") bad=1
        capture="acquisition"
        blocks++
        print
        next
    }
    index($0, "--name \"$transfer\" --read-only") {
        if (capture != "") bad=1
        capture="transfer"
        blocks++
        print
        next
    }
    capture != "" {
        if ((capture == "keeper" && index($0, "start \"$keeper\"")) ||
            (capture == "acquisition" && index($0, "start --attach \"$acquisition\"")) ||
            (capture == "transfer" && index($0, "start --attach \"$transfer\""))) {
            capture=""
            next
        }
        print
    }
    END { if (bad || capture != "" || blocks != 3) exit 1 }
' "$workflow") || fail 'cannot extract complete Docker create spans'
docker_create_blocks_digest_output=$(/usr/bin/printf '%s\n' "$docker_create_blocks" | /usr/bin/shasum -a 256) || fail 'cannot hash complete Docker create spans'
docker_create_blocks_sha256=${docker_create_blocks_digest_output%% *}
[ "$docker_create_blocks_sha256" = 1a8566ddd28ca5fc1502849e5967add2c5cc566563b4238b1bd2bb5bc7aae59c ] || fail 'complete Docker create spans changed'
/usr/bin/awk '
    BEGIN {
        needle="/usr/bin/docker"
        prefix="/usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock "
        keeper_start="          /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock start \"$keeper\" >/dev/null"
        acquisition_start="          /usr/bin/timeout --signal=KILL 120s /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock start --attach \"$acquisition\""
        transfer_start="          /usr/bin/timeout --signal=KILL 30s /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock start --attach \"$transfer\""
    }
    {
        scan=$0
        per_line=0
        while ((at=index(scan, needle)) != 0) {
            per_line++
            docker_occurrences++
            scan=substr(scan, at + length(needle))
        }
        if (per_line > 1) bad=1
    }
    index($0, prefix) {
        docker_calls++
        rest=substr($0, index($0, prefix) + length(prefix))
        if (rest !~ /^(container ls|volume ls|volume inspect|inspect|rm|volume rm|image ls|image inspect|pull|volume create|create|start)([[:space:]]|$)/) bad=1
        if (rest ~ /^start([[:space:]]|$)/ &&
            $0 != keeper_start && $0 != acquisition_start && $0 != transfer_start) bad=1
    }
    END { exit !(docker_calls == 23 && docker_occurrences == 23 && bad == 0) }
' "$workflow" || fail 'workflow Docker inventory, operation, or physical-line shape changed'
[ "$(/usr/bin/grep -Fc ' pull --quiet --platform linux/amd64 "$checkout_image"' "$workflow")" -eq 1 ] || fail 'exact image acquisition path changed'
[ "$(/usr/bin/grep -Fc ' image inspect --format '"'"'{{.Os}}/{{.Architecture}}'"'"' "$checkout_image"' "$workflow")" -eq 2 ] || fail 'image platform proof changed'
[ "$(/usr/bin/grep -Fc 'needs_pull=1' "$workflow")" -eq 2 ] || fail 'image absence/platform decision changed'
/usr/bin/awk '
    $0 == "          needs_pull=0" {
        if (started || done) bad=1
        started=1; state=1; next
    }
    !started { next }
    state == 1 && index($0, "          image_inventory=$(/usr/bin/timeout ") == 1 { state=2; next }
    state == 2 && $0 == "          if /usr/bin/grep -Fqx -- \"$checkout_repo_digest\" <<< \"$image_inventory\"; then" { state=3; next }
    state == 3 && index($0, "            cached_platform=$(/usr/bin/timeout ") == 1 { state=4; next }
    state == 4 && $0 == "            if [[ \"$cached_platform\" != linux/amd64 ]]; then needs_pull=1; fi" { state=5; next }
    state == 5 && $0 == "          else" { state=6; next }
    state == 6 && $0 == "            needs_pull=1" { state=7; next }
    state == 7 && $0 == "          fi" { state=8; next }
    state == 8 && $0 == "          if [[ \"$needs_pull\" -eq 1 ]]; then" { state=9; next }
    state == 9 && index($0, "            /usr/bin/timeout --signal=KILL 180s ") == 1 &&
        index($0, " pull --quiet --platform linux/amd64 \"$checkout_image\"") { state=10; next }
    state == 10 && $0 == "          fi" { state=11; next }
    state == 11 && index($0, "          platform=$(/usr/bin/timeout ") == 1 { state=12; next }
    state == 12 && $0 == "          [[ \"$platform\" == linux/amd64 ]]" {
        state=0; started=0; done=1; next
    }
    started { bad=1 }
    END { exit !(done == 1 && bad == 0 && state == 0 && started == 0) }
' "$workflow" || fail 'image acquisition control flow changed'
/usr/bin/awk '
    $0 == "            --mount \"type=volume,source=$volume,target=/checkout\" \\" {
        if (role || expected_workdir || acquisition_done) bad=1
        expected_workdir="acquisition"; next
    }
    $0 == "            --mount \"type=bind,source=$partial,target=/destination\" \\" {
        if (role || expected_workdir || transfer_done) bad=1
        expected_workdir="transfer"; next
    }
    expected_workdir {
        expected = expected_workdir == "acquisition" ? "            --workdir /checkout \\" : "            --workdir /destination \\"
        if ($0 != expected) bad=1
        role=expected_workdir; expected_workdir=""; next
    }
    role && $0 == "              set -eu" {
        if ((getline pwd_line) <= 0) bad=1
        expected = role == "acquisition" ? "              [ \"$(pwd -P)\" = /checkout ] || fail \"working directory\"" : "              [ \"$(pwd -P)\" = /destination ] || fail \"working directory\""
        if (pwd_line != expected) bad=1
        if ((getline hash_line) <= 0 || index(hash_line, "              [ \"$(/usr/bin/sha256sum /usr/bin/git ") != 1) bad=1
        if ((getline first_operation) <= 0) bad=1
        if (role == "acquisition" && first_operation != "              /usr/bin/git init /checkout || fail \"git init\"") bad=1
        if (role == "transfer") {
            if (first_operation != "              /usr/bin/cp -a /source/.git /destination/.git || fail \"git metadata copy\"") bad=1
            if ((getline second_operation) <= 0 || second_operation != "              /usr/bin/cp -a /source/. /destination/ || fail \"copy\"") bad=1
        }
        if (role == "acquisition") acquisition_done=1
        if (role == "transfer") transfer_done=1
        role=""; next
    }
    END { exit !(acquisition_done == 1 && transfer_done == 1 && bad == 0 && role == "" && expected_workdir == "") }
' "$workflow" || fail 'container working-directory control flow changed'
/usr/bin/awk '
    $0 == "              /usr/bin/git init /checkout || fail \"git init\"" {
        if (role || acquisition_done) bad=1
        role="acquisition"; state=1; next
    }
    $0 == "              /usr/bin/cp -a /source/. /destination/ || fail \"copy\"" {
        if (role || transfer_done) bad=1
        role="transfer"; state=1; next
    }
    role == "acquisition" && state == 1 && $0 == "              [ -d /checkout/.git ] && [ ! -L /checkout/.git ] || fail \"git directory\"" { state=2; next }
    role == "transfer" && state == 1 && $0 == "              [ -d /destination/.git ] && [ ! -L /destination/.git ] || fail \"git directory\"" { state=2; next }
    state == 2 {
        expected = role == "acquisition" ? "              GIT_DIR=/checkout/.git" : "              GIT_DIR=/destination/.git"
        if ($0 != expected) bad=1
        state=3; next
    }
    state == 3 {
        expected = role == "acquisition" ? "              GIT_WORK_TREE=/checkout" : "              GIT_WORK_TREE=/destination"
        if ($0 != expected) bad=1
        state=4; next
    }
    state == 4 && $0 == "              export GIT_DIR GIT_WORK_TREE" { state=5; next }
    role == "acquisition" && state == 5 && index($0, "              /usr/bin/git -C /checkout remote add origin ") == 1 { acquisition_done=1; role=""; state=0; next }
    role == "transfer" && state == 5 && $0 == "              actual_sha=$(/usr/bin/git -C /destination rev-parse HEAD) || fail \"rev-parse\"" { transfer_done=1; role=""; state=0; next }
    role && state { bad=1 }
    END { exit !(acquisition_done == 1 && transfer_done == 1 && bad == 0 && role == "" && state == 0) }
' "$workflow" || fail 'Git environment control flow changed'
[ "$(/usr/bin/grep -Fc '/usr/bin/rmdir "$docker_config"' "$workflow")" -eq 1 ] || fail 'workflow Docker config cleanup changed'
[ "$(/usr/bin/grep -Fc 'find "$docker_config" -mindepth 1 -maxdepth 1 -print -quit' "$workflow")" -eq 2 ] || fail 'workflow Docker config emptiness proof changed'
/usr/bin/grep -Fq 'for name in DOCKER_HOST DOCKER_CONTEXT DOCKER_TLS_VERIFY DOCKER_CERT_PATH DOCKER_CONFIG; do' "$workflow" || fail 'workflow Docker selector denial missing'
[ "$(/usr/bin/grep -Fc -- '--network none' "$workflow")" -eq 2 ] || fail 'keeper/transfer network denial changed'
[ "$(/usr/bin/grep -Fxc '          keeper_policy_exact' "$workflow")" -eq 4 ] || fail 'keeper lifecycle checks changed'
[ "$(/usr/bin/grep -Fc 'remove_container "$keeper"' "$workflow")" -eq 2 ] || fail 'keeper cleanup ordering changed'
[ "$(/usr/bin/grep -Fc 'container_absent "$keeper"' "$workflow")" -eq 5 ] || fail 'keeper absence proofs changed'
[ "$(/usr/bin/grep -Fxc '          /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start "$keeper" >/dev/null' "$workflow")" -eq 1 ] || fail 'keeper start path changed'
keeper_start_paths=$(/usr/bin/grep -E '/usr/bin/docker --config "\$docker_config" --host unix:///var/run/docker\.sock[[:space:]]+(container[[:space:]]+)?start[^[:cntrl:]]*"\$keeper"' "$workflow")
[ "$keeper_start_paths" = '          /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start "$keeper" >/dev/null' ] || fail 'unreviewed keeper start or input authority enabled'
! /usr/bin/grep -Eq -- '/usr/bin/docker --config "\$docker_config" --host unix:///var/run/docker\.sock[[:space:]]+(container[[:space:]]+)?(attach|exec|restart|stop|kill|pause|unpause)([[:space:]]|$)|--restart([=[:space:]]|$)' "$workflow" || fail 'keeper control or restart authority enabled'
/usr/bin/awk '
    index($0, "          /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock create --pull=never --name \"$keeper\" --interactive --read-only --network none") == 1 {
        if (state != 0) bad=1
        state=1
        next
    }
    state == 1 && $0 == "          /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock start \"$keeper\" >/dev/null" { state=2; next }
    state == 2 && $0 == "          keeper_policy_exact" { state=3; next }
    state == 3 && index($0, "          /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock create --pull=never --name \"$acquisition\" ") == 1 { state=4; next }
    state == 4 && index($0, "          /usr/bin/timeout --signal=KILL 120s /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock start --attach \"$acquisition\"") == 1 { state=5; next }
    state == 5 && $0 == "          force_remove_created_container \"$acquisition\"" { state=6; next }
    state == 6 && $0 == "          keeper_policy_exact" { state=7; next }
    state == 7 && index($0, "          /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock create --pull=never --name \"$transfer\" ") == 1 { state=8; next }
    state == 8 && $0 == "          keeper_policy_exact" { state=9; next }
    state == 9 && index($0, "          /usr/bin/timeout --signal=KILL 30s /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock start --attach \"$transfer\"") == 1 { state=10; next }
    state == 10 && $0 == "          remove_container \"$transfer\"" { state=11; next }
    state == 11 && $0 == "          [[ \"$transfer_start\" -eq 0 && \"$transfer_exit\" -eq 0 ]]" { state=12; next }
    state == 12 && $0 == "          keeper_policy_exact" { state=13; next }
    state == 13 && $0 == "          remove_container \"$keeper\"" { state=14; next }
    state == 14 && $0 == "          container_absent \"$keeper\"" { state=15; next }
    state == 15 && $0 == "          remove_volume \"$volume\"" { state=16; next }
    state == 16 && $0 == "          container_absent \"$acquisition\"" { state=17; next }
    state == 17 && $0 == "          container_absent \"$transfer\"" { state=18; next }
    state == 18 && $0 == "          container_absent \"$keeper\"" { state=19; next }
    state == 19 && $0 == "          volume_absent \"$volume\"" { state=20; next }
    state > 0 && state < 20 && (
        $0 == "          keeper_policy_exact" ||
        $0 == "          remove_container \"$keeper\"" ||
        $0 == "          container_absent \"$keeper\"" ||
        $0 == "          remove_volume \"$volume\"" ||
        index($0, "--name \"$acquisition\"") ||
        index($0, "--name \"$transfer\"") ||
        index($0, "start --attach \"$acquisition\"") ||
        index($0, "start --attach \"$transfer\"")
    ) { bad=1 }
    END { exit !(state == 20 && bad == 0) }
' "$workflow" || fail 'keeper acquisition/transfer lifecycle order changed'
[ "$(/usr/bin/grep -Fc '[ "$bytes" -le 67108864 ] && [ "$files" -le 8192 ] && [ "$objects" -le 32768 ]' "$workflow")" -eq 2 ] || fail 'checkout ceilings changed'
[ "$(/usr/bin/grep -Fc 'actual_sha=$(/usr/bin/git -C /' "$workflow")" -eq 2 ] || fail 'checkout identity capture changed'
[ "$(/usr/bin/grep -Fc 'status_output=$(/usr/bin/git -C /' "$workflow")" -eq 2 ] || fail 'checkout status capture changed'
[ "$(/usr/bin/grep -Fc 'refs_output=$(/usr/bin/git -C /' "$workflow")" -eq 2 ] || fail 'checkout ref capture changed'
[ "$(/usr/bin/grep -Fc 'object_counts=$(/usr/bin/git -C /' "$workflow")" -eq 2 ] || fail 'checkout object capture changed'
[ "$(/usr/bin/grep -Fc ') || fail "count-objects"' "$workflow")" -eq 2 ] || fail 'Git object failure is masked'
[ "$(/usr/bin/grep -Fc 'remove_container "$acquisition"' "$workflow")" -eq 1 ] || fail 'acquisition fallback cleanup changed'
[ "$(/usr/bin/grep -Fc 'force_remove_created_container "$acquisition"' "$workflow")" -eq 1 ] || fail 'immediate acquisition removal changed'
[ "$(/usr/bin/grep -Fc 'remove_container "$transfer"' "$workflow")" -eq 2 ] || fail 'transfer cleanup ordering changed'
[ "$(/usr/bin/grep -Fc 'remove_volume "$volume"' "$workflow")" -eq 2 ] || fail 'volume cleanup ordering changed'
/usr/bin/grep -Fq '"$workspace_parent"/.rar-c2b-checkout-partial-"$GITHUB_RUN_ID"-"$GITHUB_RUN_ATTEMPT")' "$workflow" || fail 'partial cleanup is not identity-bound'
! /usr/bin/grep -Fq '|| true' "$workflow" || fail 'checkout cleanup failure is suppressed'
! /usr/bin/grep -Eq -- '(source|target)=/var/run/docker\.sock|/var/run/docker\.sock:|DOCKER_HOST=' "$workflow" || fail 'checkout propagates Docker endpoint authority'
! /usr/bin/grep -Eq '\$\{\{[[:space:]]*github\.token|GITHUB_TOKEN|ACTIONS_RUNTIME_TOKEN|PASSWORD|SECRET|CREDENTIAL' "$workflow" "$wrapper" || fail 'credential value access present'
[ "$(/usr/bin/grep -Fc 'GITHUB_TOKEN=forbidden' "$harness")" -eq 1 ] || fail 'credential negative fixture changed'
[ "$(/usr/bin/grep -Fc 'TOKEN|PASSWORD|SECRET|CREDENTIAL' "$harness")" -eq 1 ] || fail 'credential detector fixture changed'
! /usr/bin/grep -Eq '\$\{?[A-Za-z0-9_]*(TOKEN|PASSWORD|SECRET|CREDENTIAL)[A-Za-z0-9_]*\}?' "$harness" || fail 'harness credential value access present'
[ "$(/usr/bin/grep -Fxc '          actual_sha=$(git -C "$GITHUB_WORKSPACE" rev-parse HEAD)' "$workflow")" -eq 1 ] || fail 'exact checkout verification missing'
[ "$(/usr/bin/grep -Fxc '          [[ "$actual_sha" == "$GITHUB_SHA" ]]' "$workflow")" -eq 1 ] || fail 'checkout identity is not exact'
[ "$(/usr/bin/grep -Fxc '          [[ -z "$(git -C "$GITHUB_WORKSPACE" status --porcelain=v1 --untracked-files=all)" ]]' "$workflow")" -eq 1 ] || fail 'checkout cleanliness is not workspace-bound'
[ "$(/usr/bin/grep -Fxc '          [[ ! -w "$GITHUB_WORKSPACE/tools/ci/run-controller-helper-closure-observer.sh" ]]' "$workflow")" -eq 1 ] || fail 'checkout writability check is not workspace-bound'
[ "$(/usr/bin/grep -Fxc '          retention-days: 14' "$workflow")" -eq 1 ] || fail 'retention changed'
[ "$(/usr/bin/grep -Fxc '          overwrite: false' "$workflow")" -eq 1 ] || fail 'artifact overwrite enabled'
! /usr/bin/grep -B2 -F 'uses: actions/upload-artifact@' "$workflow" | /usr/bin/grep -Fq 'if: always()' || fail 'artifact upload bypasses validation'
for required in \
    '--read-only' '--network none' '--user 65532:65532' '--cpus 1' '--memory 512m' \
    '--memory-swap 512m' '--pids-limit 64' '--security-opt no-new-privileges' \
    '--cap-drop ALL' '--tmpfs /tmp:rw,noexec,nosuid,nodev,size=64m' \
    '--tmpfs /evidence:rw,noexec,nosuid,nodev,size=4m' \
    'target=/workspace,readonly' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock create' '--pull=never' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start --attach' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock inspect' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock rm --force' \
    '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock container ls -a --no-trunc --filter "id=$id" --format '"'"'{{.ID}}'"'"'' \
    'container_absent() {' 'cleanup() {' 'trap cleanup EXIT' "trap 'exit 130' HUP INT TERM" \
    'docker_config="${RUNNER_TEMP-}/controller-helper-closure-observer-docker-config-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"' \
    '/usr/bin/install -d -m 700 "$docker_config"' '/usr/bin/rmdir "$docker_config"' \
    '[ -z "$(/usr/bin/find "$docker_config" -mindepth 1 -maxdepth 1 -print -quit)" ]' \
    'for name in DOCKER_HOST DOCKER_CONTEXT DOCKER_TLS_VERIFY DOCKER_CERT_PATH DOCKER_CONFIG; do' \
    '[ -z "${!name-}" ] || fail '"'"'Docker endpoint override present'"'"'' \
    '[ "$cleanup_failed" -eq 0 ] || rc=1' \
    'container_absent "$container" || fail '"'"'isolated observer container remains'"'"'' \
    '/usr/bin/env -i'; do
    /usr/bin/grep -Fq -- "$required" "$wrapper" || fail "wrapper boundary missing: $required"
done
! /usr/bin/grep -Eq -- '--privileged|--cap-add|--network[ =](host|bridge)|docker exec|docker cp|docker pull|docker login|--env([ =]|$)' "$wrapper" || fail 'wrapper gains forbidden container or inherited-environment authority'
! /usr/bin/grep -Eq -- '(source|target)=/var/run/docker\.sock|/var/run/docker\.sock:|DOCKER_HOST=' "$wrapper" "$harness" || fail 'observer path propagates Docker endpoint authority'
! /usr/bin/grep -Eq -- '--pull=(always|missing)' "$workflow" "$wrapper" || fail 'implicit image acquisition enabled'
[ "$(/usr/bin/grep -Fc ' pull ' "$workflow")" -eq 1 ] || fail 'workflow image acquisition authority changed'
[ "$(/usr/bin/grep -Fc ' pull ' "$wrapper")" -eq 0 ] || fail 'wrapper image acquisition authority added'
! /usr/bin/grep -Eq -- '/usr/bin/docker[^[:cntrl:]]*[[:space:]]login([[:space:]]|$)' "$workflow" "$wrapper" || fail 'Docker credential acquisition enabled'
! /usr/bin/grep -Fq '|| true' "$wrapper" || fail 'wrapper cleanup failure is suppressed'
[ "$(/usr/bin/grep -Fc -- '--pull=never' "$wrapper")" -eq 1 ] || fail 'wrapper image pull denial changed'
wrapper_docker_calls=$(/usr/bin/grep -Fc '/usr/bin/docker' "$wrapper")
[ "$wrapper_docker_calls" -gt 0 ] || fail 'wrapper Docker boundary missing'
[ "$wrapper_docker_calls" -eq "$(/usr/bin/grep -Fc '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock' "$wrapper")" ] || fail 'wrapper Docker endpoint or client config is ambient'
[ "$(/usr/bin/grep -Fc '/usr/bin/rmdir "$docker_config"' "$wrapper")" -eq 1 ] || fail 'wrapper Docker config cleanup changed'
[ "$(/usr/bin/grep -Fc 'find "$docker_config" -mindepth 1 -maxdepth 1 -print -quit' "$wrapper")" -eq 1 ] || fail 'wrapper Docker config emptiness proof changed'
[ "$(/usr/bin/grep -Fc '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock rm --force "$container"' "$wrapper")" -eq 2 ] || fail 'wrapper removal paths changed'
[ "$(/usr/bin/grep -Fc 'container_absent "$container"' "$wrapper")" -eq 2 ] || fail 'wrapper absence verification paths changed'
! /usr/bin/grep -Fq 'docker cp' "$wrapper" "$harness" || fail 'stopped tmpfs evidence copy is forbidden'
[ "$(/usr/bin/grep -Fxc 'decode_stream() {' "$wrapper")" -eq 1 ] || fail 'stream decoder missing or duplicated'
[ "$(/usr/bin/grep -Fxc '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start --attach "$container" | decode_stream "$evidence"' "$wrapper")" -eq 1 ] || fail 'exact stream handoff changed'
[ "$(/usr/bin/grep -Fxc 'pipe_status=("${PIPESTATUS[@]}")' "$wrapper")" -eq 1 ] || fail 'stream status binding missing'
for required in \
    '[ "${#pipe_status[@]}" -eq 2 ]' \
    '[ "${pipe_status[0]}" -eq 0 ]' \
    '[ "${pipe_status[1]}" -eq 0 ]' \
    '[ "$case_bytes" -le 32768 ]' \
    '[ "$manifest_bytes" -le 1048576 ]' \
    '[ "$receipt_bytes" -le 4096 ]'; do
    /usr/bin/grep -Fq -- "$required" "$wrapper" || fail "stream bound missing: $required"
done
[ "$(/usr/bin/grep -Fxc '/usr/bin/dash "$subject" || fail '"'"'production observer failed'"'"'' "$harness")" -eq 1 ] || fail 'production observer count changed'
/usr/bin/grep -Fq 'generate_subject() {' "$harness" || fail 'bound generated-subject harness missing'
/usr/bin/grep -Fq 'done < "$subject" > "$case_subject"' "$harness" || fail 'generated subject is not derived from production source'
/usr/bin/grep -Fq 'case_base=/tmp/controller-helper-closure-observer-cases' "$harness" || fail 'bounded case root missing'
/usr/bin/grep -Fq 'unexpected inherited descriptor: 9' "$harness" || fail 'generated descriptor fault missing'
/usr/bin/grep -Fq "hardlinked=x" "$harness" || fail 'generated hardlink fault missing'
/usr/bin/grep -Fq 'injected phase mutation failure' "$harness" || fail 'generated phase fault missing'
/usr/bin/grep -Fq 'hardlinked=$(/usr/bin/find -P "$root" -type f -links +1 -printf x -quit)' "$observer" || fail 'production no-hardlink predicate missing'
/usr/bin/grep -Fq "[ -z \"\$hardlinked\" ] || fail 'closure contains a hardlinked regular file'" "$observer" || fail 'production hardlink rejection missing'
for marker in \
    'RAR-C2B-BEGIN:cases' 'RAR-C2B-END:cases' \
    'RAR-C2B-BEGIN:manifest' 'RAR-C2B-END:manifest' \
    'RAR-C2B-BEGIN:receipt' 'RAR-C2B-END:receipt'; do
    [ "$(/usr/bin/grep -Fc "$marker" "$wrapper")" -eq 1 ] || fail "stream marker changed: $marker"
done
[ "$(/usr/bin/grep -Fxc "    printf 'RAR-C2B-BEGIN:%s\\n' \"\$label\"" "$harness")" -eq 1 ] || fail 'stream begin emitter changed'
[ "$(/usr/bin/grep -Fxc "    printf 'RAR-C2B-END:%s\\n' \"\$label\"" "$harness")" -eq 1 ] || fail 'stream end emitter changed'
for emission in \
    'emit_file cases "$case_file"' \
    'emit_file manifest "$evidence/controller-helper-closure.sha256"' \
    'emit_file receipt "$evidence/controller-helper-closure.receipt"'; do
    [ "$(/usr/bin/grep -Fxc "$emission" "$harness")" -eq 1 ] || fail "stream emission changed: $emission"
done
/usr/bin/awk '
    /emit_file cases "\$case_file"/ { cases=NR }
    /emit_file manifest "\$evidence\/controller-helper-closure.sha256"/ { manifest=NR }
    /emit_file receipt "\$evidence\/controller-helper-closure.receipt"/ { receipt=NR }
    END { exit !(cases && manifest && receipt && cases < manifest && manifest < receipt) }
' "$harness" || fail 'stream emission order changed'
/usr/bin/awk '
    /docker start --attach/ { start=NR }
    /docker rm --force "\$container"/ { removed=NR }
    /record=\$evidence\/controller-helper-closure-observer-run-evidence.v0/ { record=NR }
    END { exit !(start && removed && record && start < removed && removed < record) }
' "$wrapper" || fail 'producer revocation order changed'
[ "$(/usr/bin/grep -Fxc 'case_count=21' "$catalog")" -eq 1 ] || fail 'runtime catalog count changed'
[ "$(/usr/bin/grep -Ec '^O[0-9][0-9][0-9]\|' "$catalog")" -eq 21 ] || fail 'runtime catalog incomplete'
/usr/bin/grep -Fq "'case_count=21'" "$harness" || fail 'case evidence count changed'
[ "$(/usr/bin/grep -Fxc 'effect_rule=never-touch-real-toolchain,checkout,retained-evidence,lock,inventory,profile,gate,readiness' "$fixture")" -eq 1 ] || fail 'fixture effect boundary changed'
[ "$(/usr/bin/wc -l < "$receipt" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'receipt shape line count changed'
for denial in helper_compiled=false helper_executed=false target_compiled=false readiness=false; do
    /usr/bin/grep -Fqx "$denial" "$receipt" || fail "receipt denial missing: $denial"
done
! /usr/bin/grep -Eq '^[[:space:]]*(/usr/bin/|/bin/)?(cargo|rustc|ld|qemu|curl|wget|nc|git|gh)([[:space:]]|$)' "$wrapper" "$harness" || fail 'forbidden runtime command present'
for artifact in \
    controller-helper-closure-observer.cases.v0 \
    controller-helper-closure.sha256 \
    controller-helper-closure.receipt \
    controller-helper-closure-observer-run-evidence.v0; do
    [ "$(/usr/bin/grep -Fxc "            \${{ runner.temp }}/controller-helper-closure-observer/$artifact" "$workflow")" -eq 1 ] || fail "artifact path changed: $artifact"
done
/usr/bin/grep -Fq 'Independently validate exact candidate evidence' "$workflow" || fail 'independent validation missing'
/usr/bin/grep -Fq 'Retain four validated candidate files' "$workflow" || fail 'validated retention missing'
printf '%s\n' 'controller-helper observer policy passed: anonymous exact-main checkout and isolated candidate'
