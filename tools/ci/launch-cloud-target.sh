#!/bin/sh
set -eu

fail() { printf 'trusted-cloud-launch: %s\n' "$1" >&2; exit 73; }
verify_file() {
    label=$1 path=$2 expected=$3
    [ -f "$path" ] && [ ! -L "$path" ] || fail "$label is not a regular file"
    resolved=$(/usr/bin/readlink -f -- "$path") || fail "$label path cannot resolve"
    [ "$resolved" = "$path" ] || fail "$label path is not canonical"
    actual=$(/usr/bin/sha256sum "$path") || fail "$label cannot be hashed"
    [ "${actual%% *}" = "$expected" ] || fail "$label digest mismatch"
}

[ "${RAR_DEVELOPMENT_LAB-}" = cloud-launch-v1 ] || fail 'launch boundary missing'
[ "$(id -u)" = "${RAR_CONTAINER_UID-}" ] || fail 'container UID mismatch'
[ "$(id -g)" = "${RAR_CONTAINER_GID-}" ] || fail 'container GID mismatch'
case "${RAR_PROBE-}" in milestone-a | milestone-b | milestone-c | milestone-d | milestone-e | milestone-f | milestone-g) ;; *) fail 'scenario is invalid' ;; esac
[ "${RAR_LAUNCH_EVIDENCE_DIR-}" = /evidence ] || fail 'evidence boundary invalid'
case "${RAR_LAUNCH_TIMEOUT_SECONDS-}" in '' | *[!0-9]*) fail 'launch timeout invalid' ;; esac
[ "$RAR_LAUNCH_TIMEOUT_SECONDS" -ge 1 ] && [ "$RAR_LAUNCH_TIMEOUT_SECONDS" -le 3600 ] || fail 'launch timeout outside reviewed bound'
[ -d /evidence ] && [ ! -L /evidence ] || fail 'evidence directory unavailable'
[ -d /control ] && [ ! -L /control ] || fail 'controller handshake unavailable'
[ -d /control/to-host ] && [ ! -L /control/to-host ] || fail 'container-to-host channel unavailable'
[ -d /control/to-launch ] && [ ! -L /control/to-launch ] || fail 'host-to-container channel unavailable'
[ "$(/usr/bin/stat -c %a /control)" = 711 ] || fail 'control root permissions invalid'
[ "$(/usr/bin/stat -c %a /control/to-host)" = 733 ] || fail 'container-to-host permissions invalid'
[ "$(/usr/bin/stat -c %a /control/to-launch)" = 755 ] || fail 'host-to-container permissions invalid'

qemu=${RAR_QEMU_PATH-}
firmware=${RAR_FIRMWARE_PATH-}
profile=${RAR_MACHINE_PROFILE_PATH-}
qmp_client=${RAR_QMP_CLIENT_PATH-}
artifact=/artifact/rar-os-alpha.img
verify_file qemu "$qemu" "${RAR_QEMU_SHA256-}"
verify_file firmware "$firmware" "${RAR_FIRMWARE_SHA256-}"
verify_file machine-profile "$profile" "${RAR_MACHINE_PROFILE_SHA256-}"
verify_file qmp-client "$qmp_client" "${RAR_QMP_CLIENT_SHA256-}"
/bin/sh /controller/tools/ci/verify-frozen-artifact.sh "$artifact" "${RAR_ARTIFACT_SHA256-}" || fail 'artifact changed after freeze'

expected='schema=rar-development-machine-profile-v1
architecture=x86_64
machine=q35
acceleration=tcg
cpu=qemu64
guest_cpu_count=1
guest_memory_mib=256
firmware_mode=uefi-pflash-readonly
graphics=std-vga
keyboard=usb-kbd
pointer=usb-tablet
serial=stdio
monitor=qmp-unix
network=none
audio=none
reboot=disabled
snapshot=true
host_sharing=none
passthrough=none'
[ "$(/bin/cat "$profile")" = "$expected" ] || fail 'machine profile is not the exact allowlisted profile'

qmp_socket=/tmp/rar-qmp.sock
qemu_pid=
cleanup() {
    prior=$?
    trap - EXIT HUP INT TERM
    set +e
    [ -z "$qemu_pid" ] || /bin/kill "$qemu_pid" >/dev/null 2>&1
    [ -z "$qemu_pid" ] || wait "$qemu_pid" >/dev/null 2>&1
    exit "$prior"
}
trap cleanup EXIT
trap 'exit 75' HUP INT TERM

# Trusted main owns every argument. The sandbox forbids subprocess spawning,
# privilege elevation, obsolete interfaces, and resource-control escape.
"$qemu" \
    -nodefaults \
    -S \
    -sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny \
    -machine q35,accel=tcg \
    -cpu qemu64 -smp 1 -m 256 \
    -drive "if=pflash,format=raw,readonly=on,file=$firmware" \
    -drive "if=virtio,format=raw,readonly=on,file=$artifact" \
    -vga std -display none \
    -device qemu-xhci -device usb-kbd -device usb-tablet \
    -serial file:/evidence/serial.log \
    -monitor none -qmp "unix:$qmp_socket,server=on,wait=off" \
    -nic none -no-reboot -snapshot &
qemu_pid=$!

/bin/sh /controller/tools/ci/run-alpha-scenario.sh "$RAR_PROBE" "$qmp_client" "$qmp_socket" /evidence
wait "$qemu_pid"
qemu_pid=
/bin/sh /controller/tools/ci/verify-launch-evidence.sh /evidence "$RAR_PROBE" "${RAR_OUTPUT_MIB-}" || fail 'launch evidence invalid'
/usr/bin/printf '%s\n' ready > /control/to-host/evidence-ready
/bin/sh /controller/tools/ci/wait-for-launch-release.sh /control/to-launch "$RAR_LAUNCH_TIMEOUT_SECONDS" || fail 'controller did not release retained evidence'
printf 'trusted launch completed: probe=%s artifact_sha256=%s\n' "$RAR_PROBE" "$RAR_ARTIFACT_SHA256"
