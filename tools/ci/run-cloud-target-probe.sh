#!/bin/sh
set -eu

fail() { printf 'cloud-target-probe: %s\n' "$1" >&2; exit 73; }
controller_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$controller_root"
probe=${1-}
case "$probe" in milestone-a | milestone-b | milestone-c | milestone-d | milestone-e | milestone-f | milestone-g) ;; *) fail 'unsupported target probe' ;; esac
fail 'v1 two-role controller is permanently retired by ADR 0020; use only a reviewed active v2 controller'
[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] && [ "${RUNNER_OS-}" = Linux ] || fail 'cloud CI boundary missing'
for name in GITHUB_WORKSPACE RUNNER_TEMP RAR_PROBE_EVIDENCE_DIR RAR_PROBE_CONTROLLER_ROOT RAR_PROBE_SOURCE_ROOT RAR_TRUSTED_CONTROLLER_SHA RAR_PROBE_SOURCE_SHA; do
    eval "value=\${$name-}"
    [ -n "$value" ] || fail "$name missing"
done
[ "$RAR_PROBE_CONTROLLER_ROOT" = "$controller_root" ] || fail 'controller root mismatch'
source_root=$(CDPATH= cd -- "$RAR_PROBE_SOURCE_ROOT" && pwd -P) || fail 'source root unavailable'
case "$source_root" in "$GITHUB_WORKSPACE"/source) ;; *) fail 'source root outside fixed checkout' ;; esac
[ "$(git rev-parse HEAD)" = "$RAR_TRUSTED_CONTROLLER_SHA" ] || fail 'controller identity mismatch'
[ "$(git -C "$source_root" rev-parse HEAD)" = "$RAR_PROBE_SOURCE_SHA" ] || fail 'source identity mismatch'
case "$RAR_PROBE_EVIDENCE_DIR" in "$RUNNER_TEMP"/*) ;; *) fail 'evidence root outside runner temporary storage' ;; esac

profile=tools/sprint-alpha/development-lab-v1.env
/bin/sh tools/ci/check-development-lab-profile.sh "$profile" tools/sprint-alpha/x86_64-q35-v1.profile >/dev/null || fail 'Development Lab profile invalid'
# shellcheck disable=SC1090
. "./$profile"
[ "$state" = ready ] || fail 'profile is not ready'
crypto_inventory=tools/sprint-alpha/alpha-crypto-references-v1.env
/bin/sh tools/ci/check-alpha-crypto-references.sh "$crypto_inventory" --require-ready >/dev/null || fail 'crypto references are not ready'
# shellcheck disable=SC1090
. "./$crypto_inventory"
source_driver=$source_root/tools/sprint-alpha/build-alpha-image.sh
container_driver=/workspace/tools/sprint-alpha/build-alpha-image.sh
[ -f "$source_driver" ] && [ ! -L "$source_driver" ] || fail 'untrusted build driver unavailable'

/bin/mkdir -p "$RAR_PROBE_EVIDENCE_DIR/frozen-artifact"
artifact_dir=$RAR_PROBE_EVIDENCE_DIR/frozen-artifact
/bin/mkdir -p "$RAR_PROBE_EVIDENCE_DIR/launch-evidence"
launch_evidence_dir=$RAR_PROBE_EVIDENCE_DIR/launch-evidence
/bin/mkdir -p "$RAR_PROBE_EVIDENCE_DIR/launch-control"
launch_control_dir=$RAR_PROBE_EVIDENCE_DIR/launch-control
/bin/sh tools/ci/prepare-launch-control.sh "$launch_control_dir" || fail 'launch control channels unavailable'
target_log=$RAR_PROBE_EVIDENCE_DIR/target-complete.log
output_blocks=$((output_mib * 2048))

set +e
(
    set -e
    ulimit -f "$output_blocks"
    /usr/bin/sha256sum "$profile" "$source_driver" tools/ci/launch-cloud-target.sh
    printf 'probe=%s\nbuild_oci_image=%s\nlaunch_oci_image=%s\n' "$probe" "$build_oci_image" "$launch_oci_image"
    build_id=
    launch_id=
    cleanup() {
        prior=$?
        trap - EXIT HUP INT TERM
        set +e
        [ -z "$launch_id" ] || docker rm --force "$launch_id" >/dev/null 2>&1 || prior=74
        [ -z "$build_id" ] || docker rm --force "$build_id" >/dev/null 2>&1 || prior=74
        exit "$prior"
    }
    trap cleanup EXIT
    trap 'exit 75' HUP INT TERM

    # Untrusted phase: source plus compiler/linker only. Two fresh containers
    # build the same locked inputs; their unsigned artifacts must be identical.
    first_sha256=
    build_number=1
    while [ "$build_number" -le 2 ]; do
        build_id=$(docker create --read-only --network none \
            --user "$container_uid:$container_gid" --cpus "$cpu_count" \
            --memory "${memory_mib}m" --memory-swap "${memory_mib}m" --pids-limit 256 \
            --security-opt no-new-privileges --cap-drop ALL \
            --tmpfs "/tmp:rw,nosuid,nodev,size=128m,uid=$container_uid,gid=$container_gid,mode=1777" \
            --tmpfs "/build:rw,exec,nosuid,nodev,size=${build_storage_mib}m,uid=$container_uid,gid=$container_gid,mode=700" \
            --mount "type=bind,source=$controller_root,target=/controller,readonly" \
            --mount "type=bind,source=$source_root,target=/workspace,readonly" --workdir /workspace \
            --env RAR_DEVELOPMENT_LAB=cloud-v1 --env RAR_BUILD_ROOT=/build \
            --env "RAR_CONTAINER_UID=$container_uid" --env "RAR_CONTAINER_GID=$container_gid" \
            --env "RAR_COMPILER_PATH=$compiler_path" --env "RAR_COMPILER_SHA256=$compiler_sha256" \
            --env "RAR_LINKER_PATH=$linker_path" --env "RAR_LINKER_SHA256=$linker_sha256" \
            "$build_oci_image" /bin/sh -eu /controller/tools/ci/verify-cloud-target-tools.sh "$container_driver")
        case "$build_id" in '' | *[!0-9a-f]*) fail 'invalid build-container identity' ;; esac
        /usr/bin/timeout --signal=TERM --kill-after=10 "$timeout_seconds" docker start --attach "$build_id"
        candidate=$artifact_dir/build-$build_number.img
        docker cp "$build_id:/build/rar-os-alpha.img" "$candidate"
        [ -f "$candidate" ] && [ ! -L "$candidate" ] || fail 'build artifact invalid'
        artifact_size=$(/usr/bin/stat -c %s "$candidate")
        case "$artifact_size" in '' | *[!0-9]*) fail 'artifact size malformed' ;; esac
        [ "$artifact_size" -gt 0 ] && [ "$artifact_size" -le $((output_mib * 1024 * 1024)) ] || fail 'artifact size outside bound'
        candidate_output=$(/usr/bin/sha256sum "$candidate")
        candidate_sha256=${candidate_output%% *}
        if [ "$build_number" -eq 1 ]; then first_sha256=$candidate_sha256; else [ "$candidate_sha256" = "$first_sha256" ] || fail 'independent builds are not reproducible'; fi
        docker rm "$build_id" >/dev/null
        build_id=
        build_number=$((build_number + 1))
    done
    /bin/mv "$artifact_dir/build-1.img" "$artifact_dir/rar-os-alpha.img"
    /bin/rm -f "$artifact_dir/build-2.img"
    artifact=$artifact_dir/rar-os-alpha.img
    artifact_sha256=$first_sha256
    printf 'reproducible_build_count=2\nartifact_sha256=%s\n' "$artifact_sha256"

    # Trusted phase: no source mount. Only default-branch launcher code owns the
    # reviewed profile, QEMU, firmware, and exact emulator argument vector.
    launch_id=$(docker create --read-only --network none \
        --user "$container_uid:$container_gid" --cpus "$cpu_count" \
        --memory "${memory_mib}m" --memory-swap "${memory_mib}m" --pids-limit 256 \
        --security-opt no-new-privileges --cap-drop ALL \
        --tmpfs "/tmp:rw,nosuid,nodev,size=128m,uid=$container_uid,gid=$container_gid,mode=700" \
        --tmpfs "/evidence:rw,nosuid,nodev,noexec,size=${output_mib}m,uid=$container_uid,gid=$container_gid,mode=700" \
        --mount "type=bind,source=$controller_root,target=/controller,readonly" \
        --mount "type=bind,source=$artifact_dir,target=/artifact,readonly" --workdir /controller \
        --mount "type=bind,source=$launch_control_dir,target=/control" \
        --env RAR_DEVELOPMENT_LAB=cloud-launch-v1 \
        --env "RAR_PROBE=$probe" --env RAR_LAUNCH_EVIDENCE_DIR=/evidence \
        --env "RAR_OUTPUT_MIB=$output_mib" \
        --env "RAR_LAUNCH_TIMEOUT_SECONDS=$timeout_seconds" \
        --env "RAR_ARTIFACT_SHA256=$artifact_sha256" \
        --env "RAR_CONTAINER_UID=$container_uid" --env "RAR_CONTAINER_GID=$container_gid" \
        --env "RAR_QEMU_PATH=$qemu_path" --env "RAR_QEMU_SHA256=$qemu_sha256" \
        --env "RAR_FIRMWARE_PATH=$firmware_path" --env "RAR_FIRMWARE_SHA256=$firmware_sha256" \
        --env "RAR_MACHINE_PROFILE_PATH=$machine_profile_path" --env "RAR_MACHINE_PROFILE_SHA256=$machine_profile_sha256" \
        --env "RAR_QMP_CLIENT_PATH=$qmp_client_path" --env "RAR_QMP_CLIENT_SHA256=$qmp_client_sha256" \
        "$launch_oci_image" /bin/sh -eu /controller/tools/ci/launch-cloud-target.sh)
    case "$launch_id" in '' | *[!0-9a-f]*) fail 'invalid launch-container identity' ;; esac
    docker start "$launch_id" >/dev/null
    counter=0
    while [ ! -f "$launch_control_dir/to-host/evidence-ready" ]; do
        counter=$((counter + 1))
        [ "$counter" -le "$timeout_seconds" ] || fail 'launch container did not publish evidence readiness'
        [ "$(docker inspect --format '{{.State.Running}}' "$launch_id")" = true ] || fail 'launch container stopped before evidence copy'
        /bin/sleep 1
    done
    docker cp "$launch_id:/evidence/." "$launch_evidence_dir"
    /bin/sh tools/ci/verify-launch-evidence.sh "$launch_evidence_dir" "$probe" "$output_mib"
    /usr/bin/printf '%s\n' release > "$launch_control_dir/to-launch/release"
    launch_status=$(/usr/bin/timeout --signal=TERM --kill-after=10 "$timeout_seconds" docker wait "$launch_id")
    [ "$launch_status" = 0 ] || fail "launch container failed: $launch_status"
    docker logs "$launch_id"
) > "$target_log" 2>&1
status=$?
set -e
/usr/bin/sed -n '1,$p' "$target_log"
exit "$status"
