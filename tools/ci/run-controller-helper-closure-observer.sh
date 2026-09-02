#!/usr/bin/bash
set -euo pipefail
PATH=/usr/bin:/bin
LC_ALL=C
LANG=C
umask 077
export PATH LC_ALL LANG

fail() { printf 'controller-helper observer controller failed: %s\n' "$1" >&2; exit 1; }
hash_file() { /usr/bin/sha256sum -- "$1" | /usr/bin/awk '{ print $1 }'; }
size_file() { /usr/bin/stat -c %s "$1"; }

source_root=${1-}
evidence=${2-}
[ -d "$source_root" ] && [ ! -L "$source_root" ] || fail 'source root invalid'
[ "$source_root" = "${GITHUB_WORKSPACE-}" ] || fail 'source root is not exact checkout'
[ -d "$evidence" ] && [ ! -L "$evidence" ] || fail 'evidence root invalid'
case "$evidence" in "${RUNNER_TEMP-}"/controller-helper-closure-observer) ;; *) fail 'evidence root is outside controller scratch' ;; esac
[ -z "$(/usr/bin/find "$evidence" -mindepth 1 -maxdepth 1 -print -quit)" ] || fail 'evidence root not empty'
[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] || fail 'CI boundary absent'
[ "${GITHUB_EVENT_NAME-}" = push ] && [ "${GITHUB_REF-}" = refs/heads/main ] && [ "${GITHUB_REPOSITORY-}" = AndyTechCoder/RAR-OS ] || fail 'canonical context absent'
[ "${GITHUB_SHA-}" = "${RAR_TRUSTED_CONTROLLER_SHA-}" ] && [ "${GITHUB_SHA-}" = "${RAR_EXPECTED_SOURCE_REVISION-}" ] || fail 'exact-main mismatch'
for name in DOCKER_HOST DOCKER_CONTEXT DOCKER_TLS_VERIFY DOCKER_CERT_PATH DOCKER_CONFIG; do
    [ -z "${!name-}" ] || fail 'Docker endpoint override present'
done

image=rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
docker_config="${RUNNER_TEMP-}/controller-helper-closure-observer-docker-config-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
case "$docker_config" in
    "${RUNNER_TEMP-}"/controller-helper-closure-observer-docker-config-"$GITHUB_RUN_ID"-"$GITHUB_RUN_ATTEMPT") ;;
    *) fail 'Docker config root identity mismatch' ;;
esac
[ ! -e "$docker_config" ] && [ ! -L "$docker_config" ] || fail 'Docker config root preexists'
/usr/bin/install -d -m 700 "$docker_config" || fail 'cannot create Docker config root'
[ -z "$(/usr/bin/find "$docker_config" -mindepth 1 -maxdepth 1 -print -quit)" ] || fail 'Docker config root not empty'
container=
cleanup_failed=0
container_absent() {
    local id=$1 remaining
    remaining=$(/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock container ls -a --no-trunc --filter "id=$id" --format '{{.ID}}') || return 1
    [ -z "$remaining" ]
}
cleanup() {
    local rc=$?
    trap - EXIT HUP INT TERM
    if [ -n "$container" ]; then
        /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock rm --force "$container" >/dev/null 2>&1 || cleanup_failed=1
        container_absent "$container" || cleanup_failed=1
    fi
    if [ -e "$docker_config" ] || [ -L "$docker_config" ]; then
        if [ -d "$docker_config" ] && [ ! -L "$docker_config" ]; then
            /usr/bin/rmdir "$docker_config" || cleanup_failed=1
        else
            cleanup_failed=1
        fi
    fi
    [ ! -e "$docker_config" ] && [ ! -L "$docker_config" ] || cleanup_failed=1
    [ "$cleanup_failed" -eq 0 ] || rc=1
    exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' HUP INT TERM
container=$(/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock create --pull=never --read-only --network none --user 65532:65532 \
    --cpus 1 --memory 512m --memory-swap 512m --pids-limit 64 \
    --security-opt no-new-privileges --cap-drop ALL \
    --tmpfs /tmp:rw,noexec,nosuid,nodev,size=64m,uid=65532,gid=65532,mode=700 \
    --tmpfs /evidence:rw,noexec,nosuid,nodev,size=4m,uid=65532,gid=65532,mode=700 \
    --mount "type=bind,source=$source_root,target=/workspace,readonly" \
    "$image" /usr/bin/env -i \
    PATH=/usr/bin:/bin LC_ALL=C LANG=C GITHUB_ACTIONS=true CI=true \
    GITHUB_EVENT_NAME=push GITHUB_REF=refs/heads/main GITHUB_REPOSITORY=AndyTechCoder/RAR-OS \
    "GITHUB_SHA=$GITHUB_SHA" "GITHUB_RUN_ID=$GITHUB_RUN_ID" "GITHUB_RUN_ATTEMPT=$GITHUB_RUN_ATTEMPT" \
    "RAR_TRUSTED_CONTROLLER_SHA=$RAR_TRUSTED_CONTROLLER_SHA" "RAR_EXPECTED_SOURCE_REVISION=$RAR_EXPECTED_SOURCE_REVISION" \
    "RAR_CI_RUNNER_IMAGE_OS=$RAR_CI_RUNNER_IMAGE_OS" "RAR_CI_RUNNER_IMAGE_VERSION=$RAR_CI_RUNNER_IMAGE_VERSION" \
    "RAR_CI_RUNNER_OS=$RAR_CI_RUNNER_OS" "RAR_CI_RUNNER_ARCH=$RAR_CI_RUNNER_ARCH" \
    "RAR_CI_BOOTSTRAP_IMAGE=$RAR_CI_BOOTSTRAP_IMAGE" \
    "RAR_EXPECTED_SUBJECT_SHA256=$RAR_EXPECTED_SUBJECT_SHA256" \
    "RAR_EXPECTED_FIXTURE_SHA256=$RAR_EXPECTED_FIXTURE_SHA256" \
    "RAR_EXPECTED_TOOL_PINS_SHA256=$RAR_EXPECTED_TOOL_PINS_SHA256" \
    /usr/bin/dash /workspace/tools/ci/controller-helper-closure-observer-harness.sh)
case "$container" in ''|*[!0-9a-f]*) fail 'container identity invalid' ;; esac
decode_stream() {
    local root=$1 state=case-begin line bytes
    local case_out=$root/controller-helper-closure-observer.cases.v0
    local manifest_out=$root/controller-helper-closure.sha256
    local receipt_out=$root/controller-helper-closure.receipt
    local case_bytes=0 manifest_bytes=0 receipt_bytes=0
    while IFS= read -r line; do
        [ "${#line}" -le 1024 ] || return 1
        case "$state" in
            case-begin)
                [ "$line" = 'RAR-C2B-BEGIN:cases' ] || return 1
                set -C; exec 3> "$case_out" || return 1; set +C; state=cases
                ;;
            cases)
                if [ "$line" = 'RAR-C2B-END:cases' ]; then
                    exec 3>&- || return 1
                    state=manifest-begin
                else
                    bytes=$((${#line} + 1)); case_bytes=$((case_bytes + bytes))
                    [ "$case_bytes" -le 32768 ] || return 1
                    printf '%s\n' "$line" >&3 || return 1
                fi
                ;;
            manifest-begin)
                [ "$line" = 'RAR-C2B-BEGIN:manifest' ] || return 1
                set -C; exec 4> "$manifest_out" || return 1; set +C; state=manifest
                ;;
            manifest)
                if [ "$line" = 'RAR-C2B-END:manifest' ]; then
                    exec 4>&- || return 1
                    state=receipt-begin
                else
                    bytes=$((${#line} + 1)); manifest_bytes=$((manifest_bytes + bytes))
                    [ "$manifest_bytes" -le 1048576 ] || return 1
                    printf '%s\n' "$line" >&4 || return 1
                fi
                ;;
            receipt-begin)
                [ "$line" = 'RAR-C2B-BEGIN:receipt' ] || return 1
                set -C; exec 5> "$receipt_out" || return 1; set +C; state=receipt
                ;;
            receipt)
                if [ "$line" = 'RAR-C2B-END:receipt' ]; then
                    exec 5>&- || return 1
                    state=done
                else
                    bytes=$((${#line} + 1)); receipt_bytes=$((receipt_bytes + bytes))
                    [ "$receipt_bytes" -le 4096 ] || return 1
                    printf '%s\n' "$line" >&5 || return 1
                fi
                ;;
            done) return 1 ;;
            *) return 1 ;;
        esac
    done
    [ "$state" = done ] && [ "$case_bytes" -gt 0 ] && [ "$manifest_bytes" -gt 0 ] && [ "$receipt_bytes" -gt 0 ]
}

set +e
/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start --attach "$container" | decode_stream "$evidence"
pipe_status=("${PIPESTATUS[@]}")
set -e
[ "${#pipe_status[@]}" -eq 2 ] || fail 'observer stream status missing'
[ "${pipe_status[0]}" -eq 0 ] || fail 'isolated observer failed'
[ "${pipe_status[1]}" -eq 0 ] || fail 'candidate evidence stream rejected'
status=$(/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock inspect --format '{{.State.ExitCode}}' "$container")
[ "$status" -eq 0 ] || fail 'isolated observer exit mismatch'
/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock rm --force "$container" >/dev/null || fail 'cannot remove isolated observer'
container_absent "$container" || fail 'isolated observer container remains'
container=

case_file=$evidence/controller-helper-closure-observer.cases.v0
manifest=$evidence/controller-helper-closure.sha256
receipt=$evidence/controller-helper-closure.receipt
record=$evidence/controller-helper-closure-observer-run-evidence.v0
case_sha=$(hash_file "$case_file")
manifest_sha=$(hash_file "$manifest")
receipt_sha=$(hash_file "$receipt")
case_bytes=$(size_file "$case_file")
manifest_bytes=$(size_file "$manifest")
receipt_bytes=$(size_file "$receipt")
set -C
exec 4> "$record" || fail 'cannot create outer record'
set +C
printf '%s\n' \
    'schema=rar-alpha-controller-helper-closure-observer-run-evidence-v0' \
    'status=candidate-not-reviewed-not-ready' \
    'repository=AndyTechCoder/RAR-OS' \
    'ref=refs/heads/main' \
    'event=push' \
    "controller_sha=$RAR_TRUSTED_CONTROLLER_SHA" \
    "source_sha=$RAR_EXPECTED_SOURCE_REVISION" \
    "run_id=$GITHUB_RUN_ID" \
    "run_attempt=$GITHUB_RUN_ATTEMPT" \
    "runner_image_os=$RAR_CI_RUNNER_IMAGE_OS" \
    "runner_image_version=$RAR_CI_RUNNER_IMAGE_VERSION" \
    "runner_os=$RAR_CI_RUNNER_OS" \
    "runner_arch=$RAR_CI_RUNNER_ARCH" \
    "oci_image=$RAR_CI_BOOTSTRAP_IMAGE" \
    "wrapper_sha256=$RAR_EXPECTED_WRAPPER_SHA256" \
    "subject_sha256=$RAR_EXPECTED_SUBJECT_SHA256" \
    "fixture_sha256=$RAR_EXPECTED_FIXTURE_SHA256" \
    "tool_pins_sha256=$RAR_EXPECTED_TOOL_PINS_SHA256" \
    "case_evidence_sha256=$case_sha" \
    "manifest_sha256=$manifest_sha" \
    "receipt_sha256=$receipt_sha" \
    "artifact_name=$RAR_EXPECTED_ARTIFACT_NAME" \
    'retention_days=14' \
    'observed_exit=0' \
    "case_evidence_bytes=$case_bytes" \
    "manifest_bytes=$manifest_bytes" \
    "receipt_bytes=$receipt_bytes" \
    'output_count=4' \
    'verdict=candidate-not-reviewed-not-ready' \
    "record_nonce=$RAR_EXPECTED_RECORD_NONCE" >&4
exec 4>&-
record_sha=$(/usr/bin/sed -n '1,30p' "$record" | /usr/bin/sha256sum | /usr/bin/awk '{ print $1 }')
printf 'record_sha256=%s\n' "$record_sha" >> "$record"
