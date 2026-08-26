#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
profile=${1-$root/tools/sprint-alpha/development-lab-v1.env}
machine_profile=${2-$root/tools/sprint-alpha/x86_64-q35-v1.profile}
qmp_contract=${3-$root/tools/sprint-alpha/qmp-client-v1.env}
qmp_controller_root=${4-$root}

fail() {
    printf 'Development Lab profile blocked: %s\n' "$1" >&2
    exit 1
}

[ -f "$profile" ] && [ ! -L "$profile" ] || fail 'profile is missing or symbolic'
[ -f "$machine_profile" ] && [ ! -L "$machine_profile" ] || fail 'machine profile is missing or symbolic'

if /usr/bin/grep -Ev '^(schema|state|build_oci_image|launch_oci_image|compiler_path|compiler_sha256|linker_path|linker_sha256|qemu_path|qemu_sha256|firmware_path|firmware_sha256|machine_profile_path|machine_profile_sha256|qmp_client_path|qmp_client_sha256|container_uid|container_gid|cpu_count|memory_mib|build_storage_mib|output_mib|timeout_seconds)=[A-Za-z0-9._:/@+-]+$' "$profile" | /usr/bin/grep -q .; then
    fail 'grammar is invalid'
fi
[ "$(/usr/bin/wc -l < "$profile" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'field count is invalid'
for key in schema state build_oci_image launch_oci_image compiler_path compiler_sha256 linker_path linker_sha256 qemu_path qemu_sha256 firmware_path firmware_sha256 machine_profile_path machine_profile_sha256 qmp_client_path qmp_client_sha256 container_uid container_gid cpu_count memory_mib build_storage_mib output_mib timeout_seconds; do
    [ "$(/usr/bin/grep -c "^$key=" "$profile")" -eq 1 ] || fail "key is missing or duplicated: $key"
done

# The restricted grammar above makes this reviewed data safe to load as values.
# shellcheck disable=SC1090
. "$profile"

[ "$schema" = rar-development-lab-profile-v1 ] || fail 'schema is invalid'
[ "$container_uid" = 65532 ] || fail 'container UID is invalid'
[ "$container_gid" = 65532 ] || fail 'container GID is invalid'
[ "$cpu_count" = 2 ] || fail 'CPU bound is invalid'
[ "$memory_mib" = 2048 ] || fail 'memory bound is invalid'
[ "$build_storage_mib" = 4096 ] || fail 'build-storage bound is invalid'
[ "$output_mib" = 64 ] || fail 'output bound is invalid'
[ "$timeout_seconds" = 1200 ] || fail 'timeout bound is invalid'

case "$state" in
    blocked)
        for value in "$build_oci_image" "$launch_oci_image" "$compiler_path" "$compiler_sha256" "$linker_path" "$linker_sha256" "$qemu_path" "$qemu_sha256" "$firmware_path" "$firmware_sha256" "$machine_profile_path" "$machine_profile_sha256" "$qmp_client_path" "$qmp_client_sha256"; do
            [ "$value" = unavailable ] || fail 'blocked profile contains an activating value'
        done
        ;;
    ready)
        fail 'v1 profile is permanently blocked by ADR 0020; use a reviewed v2 three-role controller'
        for image in "$build_oci_image" "$launch_oci_image"; do
            case "$image" in *@sha256:????????????????????????????????????????????????????????????????) ;; *) fail 'an OCI image is not digest-pinned' ;; esac
            image_digest=${image##*@sha256:}
            case "$image_digest" in *[!0-9a-f]*) fail 'an OCI digest is not lowercase hexadecimal' ;; esac
        done
        build_image_digest=${build_oci_image##*@sha256:}
        launch_image_digest=${launch_oci_image##*@sha256:}
        [ "$build_image_digest" != "$launch_image_digest" ] || fail 'build and launch image digests must be distinct'
        for digest in "$compiler_sha256" "$linker_sha256" "$qemu_sha256" "$firmware_sha256" "$machine_profile_sha256" "$qmp_client_sha256"; do
            [ "${#digest}" -eq 64 ] || fail 'a SHA-256 identity has the wrong length'
            case "$digest" in *[!0-9a-f]*) fail 'a SHA-256 identity is not lowercase hexadecimal' ;; esac
        done
        case "$compiler_path" in /opt/rar-toolchain/*) ;; *) fail 'compiler path escapes its root' ;; esac
        case "$linker_path" in /opt/rar-toolchain/*) ;; *) fail 'linker path escapes its root' ;; esac
        case "$qemu_path" in /opt/rar-lab/bin/*) ;; *) fail 'QEMU path escapes its root' ;; esac
        case "$firmware_path" in /opt/rar-lab/firmware/*) ;; *) fail 'firmware path escapes its root' ;; esac
        case "$qmp_client_path" in /opt/rar-lab/bin/*) ;; *) fail 'QMP client path escapes its root' ;; esac
        [ "$machine_profile_path" = /controller/tools/sprint-alpha/x86_64-q35-v1.profile ] || fail 'machine-profile path is not the reviewed controller path'
        for path in "$compiler_path" "$linker_path" "$qemu_path" "$firmware_path" "$machine_profile_path" "$qmp_client_path"; do
            case "$path" in *'/../'* | *'/./'* | */.. | */.) fail 'a configured path is not canonical' ;; esac
        done
        if command -v sha256sum >/dev/null 2>&1; then
            actual=$(sha256sum "$machine_profile")
        else
            actual=$(/usr/bin/shasum -a 256 "$machine_profile")
        fi
        [ "${actual%% *}" = "$machine_profile_sha256" ] || fail 'machine-profile hash does not match'
        /bin/sh "$root/tools/ci/check-qmp-client-contract.sh" "$qmp_contract" "$qmp_controller_root" >/dev/null || fail 'QMP client contract invalid'
        # shellcheck disable=SC1090
        . "$qmp_contract"
        [ "$state" = ready ] || fail 'QMP client contract is not ready'
        [ "$binary_sha256" = "$qmp_client_sha256" ] || fail 'QMP client binary identity disagrees with its contract'
        ;;
    *) fail 'state must be blocked or ready' ;;
esac

printf 'Development Lab profile validation passed: state=%s\n' "$state"
