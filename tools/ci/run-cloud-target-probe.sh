#!/bin/sh
set -eu

fail() {
    echo "cloud-target-probe: $1" >&2
    exit 73
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

probe=${1-}
[ "$probe" = milestone-a ] || fail "unsupported target probe"
[ "${GITHUB_ACTIONS-}" = true ] || fail "GitHub Actions boundary missing"
[ "${CI-}" = true ] || fail "CI boundary missing"
[ "${RUNNER_OS-}" = Linux ] || fail "Linux runner required"
[ -n "${RUNNER_TEMP-}" ] || fail "runner temporary root missing"
[ -n "${RAR_PROBE_EVIDENCE_DIR-}" ] || fail "evidence directory missing"

case "$RAR_PROBE_EVIDENCE_DIR" in
    "$RUNNER_TEMP"/*) ;;
    *) fail "evidence directory is outside the disposable runner root" ;;
esac

profile=tools/sprint-alpha/development-lab-v1.env
driver=tools/sprint-alpha/probe-milestone-a.sh
[ -f "$profile" ] && [ ! -L "$profile" ] || fail "reviewed Development Lab profile unavailable"
[ -x "$driver" ] && [ ! -L "$driver" ] || fail "reviewed Milestone A driver unavailable"

# The profile is shell data, not executable code. Its grammar is deliberately
# restricted before the exact reviewed values are loaded.
if grep -Ev '^(schema|state|oci_image|compiler_sha256|linker_sha256|qemu_sha256|firmware_sha256|machine_profile_sha256|cpu_count|memory_mib|build_storage_mib|output_mib|timeout_seconds)=[A-Za-z0-9._:/@+-]+$' "$profile" | grep -q .; then
    fail "Development Lab profile grammar invalid"
fi
for key in schema state oci_image compiler_sha256 linker_sha256 qemu_sha256 firmware_sha256 machine_profile_sha256 cpu_count memory_mib build_storage_mib output_mib timeout_seconds; do
    [ "$(grep -c "^$key=" "$profile")" -eq 1 ] || fail "Development Lab profile key invalid: $key"
done
[ "$(wc -l < "$profile" | tr -d ' ')" -eq 13 ] || fail "Development Lab profile has unexpected fields"
# shellcheck disable=SC1090
. "./$profile"

[ "${schema-}" = rar-development-lab-profile-v1 ] || fail "profile schema invalid"
[ "${state-}" = ready ] || fail "profile is not reviewed and ready"
case "${oci_image-}" in *@sha256:????????????????????????????????????????????????????????????????) ;; *) fail "OCI image is not digest pinned" ;; esac
oci_digest=${oci_image##*@sha256:}
case "$oci_digest" in *[!0-9a-f]*) fail "OCI digest is not lowercase hexadecimal" ;; esac
for digest_name in compiler_sha256 linker_sha256 qemu_sha256 firmware_sha256 machine_profile_sha256; do
    eval "digest_value=\${$digest_name-}"
    [ "${#digest_value}" -eq 64 ] || fail "$digest_name length invalid"
    case "$digest_value" in *[!0-9a-f]*) fail "$digest_name is not lowercase hexadecimal" ;; esac
done
[ "${cpu_count-}" = 2 ] || fail "CPU bound invalid"
[ "${memory_mib-}" = 2048 ] || fail "memory bound invalid"
[ "${build_storage_mib-}" = 4096 ] || fail "storage bound invalid"
[ "${output_mib-}" = 64 ] || fail "output bound invalid"
[ "${timeout_seconds-}" = 1200 ] || fail "timeout bound invalid"

mkdir -p "$RAR_PROBE_EVIDENCE_DIR"
target_log=$RAR_PROBE_EVIDENCE_DIR/target-complete.log
output_blocks=$((output_mib * 2048))

set +e
(
    ulimit -f "$output_blocks"
    printf '%s\n' \
        "probe=$probe" \
        "oci_image=$oci_image" \
        "compiler_sha256=$compiler_sha256" \
        "linker_sha256=$linker_sha256" \
        "qemu_sha256=$qemu_sha256" \
        "firmware_sha256=$firmware_sha256" \
        "machine_profile_sha256=$machine_profile_sha256" \
        "cpu_count=$cpu_count" \
        "memory_mib=$memory_mib" \
        "build_storage_mib=$build_storage_mib" \
        "output_mib=$output_mib" \
        "timeout_seconds=$timeout_seconds"
    /usr/bin/sha256sum "$profile" "$driver"
    /usr/bin/timeout --signal=TERM --kill-after=10 "$timeout_seconds" \
        docker run --rm --read-only --network none \
        --cpus "$cpu_count" --memory "${memory_mib}m" --memory-swap "${memory_mib}m" \
        --pids-limit 256 --security-opt no-new-privileges --cap-drop ALL \
        --tmpfs "/tmp:rw,nosuid,nodev,size=128m,mode=1777" \
        --tmpfs "/build:rw,exec,nosuid,nodev,size=${build_storage_mib}m,mode=700" \
        --mount "type=bind,source=$root,target=/workspace,readonly" \
        --workdir /workspace \
        --env RAR_DEVELOPMENT_LAB=cloud-v1 \
        --env RAR_BUILD_ROOT=/build \
        --env "RAR_COMPILER_SHA256=$compiler_sha256" \
        --env "RAR_LINKER_SHA256=$linker_sha256" \
        --env "RAR_QEMU_SHA256=$qemu_sha256" \
        --env "RAR_FIRMWARE_SHA256=$firmware_sha256" \
        --env "RAR_MACHINE_PROFILE_SHA256=$machine_profile_sha256" \
        "$oci_image" \
        /bin/sh -eu "$driver"
) > "$target_log" 2>&1
status=$?
set -e

sed -n '1,$p' "$target_log"
exit "$status"
