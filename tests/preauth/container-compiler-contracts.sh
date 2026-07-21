#!/bin/sh
set -eu

fail() {
    printf 'container compiler contract: %s\n' "$1" >&2
    exit 1
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
. "$root/tools/toolchain/preauth-build-root.sh"

[ "$(uname -s)" = Linux ] || fail linux-ci-only
command -v docker >/dev/null 2>&1 || fail docker-unavailable
[ -z "${ACTIONS_ID_TOKEN_REQUEST_TOKEN-}" ] && [ -z "${ACTIONS_ID_TOKEN_REQUEST_URL-}" ] || fail authority-environment
[ -z "${AWS_ACCESS_KEY_ID-}" ] && [ -z "${AWS_SECRET_ACCESS_KEY-}" ] && [ -z "${AWS_SESSION_TOKEN-}" ] || fail authority-environment
[ -z "${GH_TOKEN-}" ] && [ -z "${GITHUB_TOKEN-}" ] || fail authority-environment

preauth_build_root_create "$root" preauth-container-compiler preauth-container-compiler
preauth_build_install_traps
/usr/bin/mkdir -m 700 "$PREAUTH_BUILD_DIR/docker-config"
DOCKER_CONFIG=$PREAUTH_BUILD_DIR/docker-config
export DOCKER_CONFIG

base=rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
registry_mirrors=$(docker info --format '{{json .RegistryConfig.Mirrors}}' 2>/dev/null || printf 'unavailable')
case "$registry_mirrors" in null|\[\]) :;; *) fail registry-mirror-configured;; esac
docker image inspect "$base" >/dev/null 2>&1 || docker pull "$base" >/dev/null
docker image inspect --format '{{join .RepoDigests "\n"}}' "$base" \
    | /usr/bin/grep -qx 'rust@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
    || fail image-binding

host_uid=$(id -u)
host_gid=$(id -g)
docker run --rm --read-only --network none \
    --user "$host_uid:$host_gid" --security-opt no-new-privileges --cap-drop ALL \
    --tmpfs "/tmp:rw,exec,nosuid,nodev,size=128m,uid=$host_uid,gid=$host_gid,mode=1777" \
    --env RAR_PREAUTH_CONTAINER_IMAGE=sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 \
    --env RAR_TEST_UID="$host_uid" --env RAR_TEST_GID="$host_gid" \
    --mount "type=bind,source=$root,target=/workspace,readonly" --workdir /workspace \
    "$base" /usr/bin/dash -e tests/preauth/container-compiler-contracts-inner.sh
