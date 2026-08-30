#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

manifest=${1-}
fail() { printf 'accepted evidence v0 rejected: %s\n' "$1" >&2; exit 1; }
[ -f "$manifest" ] && [ -s "$manifest" ] && [ ! -L "$manifest" ] || fail 'record is missing, empty, non-regular, or symbolic'
size=$(/usr/bin/stat -c %s "$manifest" 2>/dev/null || /usr/bin/stat -f %z "$manifest")
links=$(/usr/bin/stat -c %h "$manifest" 2>/dev/null || /usr/bin/stat -f %l "$manifest")
[ "$size" -le 4096 ] && [ "$links" -eq 1 ] || fail 'record size or link count invalid'
[ "$(/usr/bin/tail -c 1 "$manifest" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'record is not LF terminated'
[ "$(/usr/bin/wc -l < "$manifest" | /usr/bin/tr -d ' ')" -eq 20 ] || fail 'record line count invalid'
if /usr/bin/od -An -tx1 "$manifest" | /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)'; then fail 'record contains NUL or CR'; fi

for name in RAR_ACCEPT_ATTEMPT_NONCE RAR_EXPECTED_PROBE RAR_TRUSTED_CONTROLLER_SHA \
    RAR_EXPECTED_SOURCE_REVISION RAR_ARTIFACT_SHA256 RAR_ACCEPTANCE_PROTOCOL_SHA256 \
    RAR_MACHINE_PROFILE_SHA256 RAR_QEMU_SHA256 RAR_FIRMWARE_SHA256 RAR_QMP_CLIENT_SHA256 \
    RAR_ACTIONS_SHA256 RAR_SERIAL_SHA256 RAR_FINAL_CAPTURE_SHA256 RAR_CAPTURE_SET_SHA256 \
    RAR_HANDOFF_MANIFEST_SET_SHA256 RAR_REFERENCE_VERDICT_SHA256 \
    RAR_ROLE_INVENTORIES_SHA256 RAR_ACCEPTED_OUTPUTS_SHA256; do
    eval "value=\${$name-}"
    [ -n "$value" ] || fail "missing trusted expectation: $name"
done
case "$RAR_EXPECTED_PROBE" in milestone-a|milestone-b|milestone-c|milestone-d|milestone-e|milestone-f|milestone-g) ;; *) fail 'probe invalid' ;; esac
case "$RAR_TRUSTED_CONTROLLER_SHA$RAR_EXPECTED_SOURCE_REVISION" in *[!0-9a-f]*) fail 'revision malformed' ;; esac
[ "${#RAR_TRUSTED_CONTROLLER_SHA}" -eq 40 ] && [ "${#RAR_EXPECTED_SOURCE_REVISION}" -eq 40 ] || fail 'revision length invalid'
zero_revision=0000000000000000000000000000000000000000
[ "$RAR_TRUSTED_CONTROLLER_SHA" != "$zero_revision" ] && [ "$RAR_EXPECTED_SOURCE_REVISION" != "$zero_revision" ] || fail 'revision is zero'
for value in "$RAR_ACCEPT_ATTEMPT_NONCE" "$RAR_ARTIFACT_SHA256" "$RAR_MACHINE_PROFILE_SHA256" \
    "$RAR_QEMU_SHA256" "$RAR_FIRMWARE_SHA256" "$RAR_QMP_CLIENT_SHA256" "$RAR_ACTIONS_SHA256" \
    "$RAR_SERIAL_SHA256" "$RAR_FINAL_CAPTURE_SHA256" "$RAR_CAPTURE_SET_SHA256" \
    "$RAR_HANDOFF_MANIFEST_SET_SHA256" "$RAR_REFERENCE_VERDICT_SHA256" \
    "$RAR_ROLE_INVENTORIES_SHA256" "$RAR_ACCEPTED_OUTPUTS_SHA256"; do
    case "$value" in *[!0-9a-f]*) fail 'digest malformed' ;; esac
    [ "${#value}" -eq 64 ] || fail 'digest length invalid'
done
zero=0000000000000000000000000000000000000000000000000000000000000000
for value in "$RAR_ACCEPT_ATTEMPT_NONCE" "$RAR_ARTIFACT_SHA256" "$RAR_MACHINE_PROFILE_SHA256" \
    "$RAR_QEMU_SHA256" "$RAR_FIRMWARE_SHA256" "$RAR_QMP_CLIENT_SHA256" "$RAR_ACTIONS_SHA256" \
    "$RAR_SERIAL_SHA256" "$RAR_FINAL_CAPTURE_SHA256" "$RAR_CAPTURE_SET_SHA256" \
    "$RAR_HANDOFF_MANIFEST_SET_SHA256" "$RAR_ROLE_INVENTORIES_SHA256" "$RAR_ACCEPTED_OUTPUTS_SHA256"; do
    [ "$value" != "$zero" ] || fail 'required digest is zero'
done
[ "$RAR_ACCEPTANCE_PROTOCOL_SHA256" = ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd ] || fail 'protocol downgrade or mismatch'
case "$RAR_EXPECTED_PROBE" in
    milestone-a|milestone-b|milestone-c|milestone-d|milestone-e) [ "$RAR_REFERENCE_VERDICT_SHA256" = "$zero" ] || fail 'reference verdict must be absent' ;;
    milestone-f|milestone-g) [ "$RAR_REFERENCE_VERDICT_SHA256" != "$zero" ] || fail 'reference verdict must be present' ;;
esac

expected="schema=rar-alpha-accepted-evidence-v0
attempt_nonce=$RAR_ACCEPT_ATTEMPT_NONCE
probe=$RAR_EXPECTED_PROBE
controller_revision=$RAR_TRUSTED_CONTROLLER_SHA
source_revision=$RAR_EXPECTED_SOURCE_REVISION
artifact_sha256=$RAR_ARTIFACT_SHA256
acceptance_protocol_sha256=$RAR_ACCEPTANCE_PROTOCOL_SHA256
machine_profile_sha256=$RAR_MACHINE_PROFILE_SHA256
qemu_sha256=$RAR_QEMU_SHA256
firmware_sha256=$RAR_FIRMWARE_SHA256
qmp_client_sha256=$RAR_QMP_CLIENT_SHA256
actions_sha256=$RAR_ACTIONS_SHA256
serial_sha256=$RAR_SERIAL_SHA256
final_capture_sha256=$RAR_FINAL_CAPTURE_SHA256
capture_set_sha256=$RAR_CAPTURE_SET_SHA256
handoff_manifest_set_sha256=$RAR_HANDOFF_MANIFEST_SET_SHA256
reference_verdict_sha256=$RAR_REFERENCE_VERDICT_SHA256
role_inventories_sha256=$RAR_ROLE_INVENTORIES_SHA256
accepted_outputs_sha256=$RAR_ACCEPTED_OUTPUTS_SHA256"
[ "$(/usr/bin/sed -n '1,19p' "$manifest")" = "$expected" ] || fail 'record binding, order, or field set mismatch'
if [ -x /usr/bin/sha256sum ]; then
    record_sha=$(/usr/bin/sed -n '1,19p' "$manifest" | /usr/bin/sha256sum | /usr/bin/awk '{ print $1 }')
else
    record_sha=$(/usr/bin/sed -n '1,19p' "$manifest" | /usr/bin/shasum -a 256 | /usr/bin/awk '{ print $1 }')
fi
[ "$(/usr/bin/sed -n '20p' "$manifest")" = "record_sha256=$record_sha" ] || fail 'record digest mismatch'
printf '%s\n' 'accepted evidence v0 validated: exact attempt/source/artifact/tool/output bindings'
