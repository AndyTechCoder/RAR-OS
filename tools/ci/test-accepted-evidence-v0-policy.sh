#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'accepted evidence v0 mutation tests skipped: external read-only-source CI step required'; exit 0; }
work=$(mktemp -d "$scratch/accepted-evidence.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/verify-accepted-evidence-v0.sh
a=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
b=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
c=cccccccccccccccccccccccccccccccccccccccc
zero=0000000000000000000000000000000000000000000000000000000000000000
export RAR_ACCEPT_ATTEMPT_NONCE=$a RAR_EXPECTED_PROBE=milestone-a
export RAR_TRUSTED_CONTROLLER_SHA=$c RAR_EXPECTED_SOURCE_REVISION=$c
export RAR_ARTIFACT_SHA256=$b RAR_ACCEPTANCE_PROTOCOL_SHA256=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd
export RAR_MACHINE_PROFILE_SHA256=$b RAR_QEMU_SHA256=$b RAR_FIRMWARE_SHA256=$b RAR_QMP_CLIENT_SHA256=$b
export RAR_ACTIONS_SHA256=$b RAR_SERIAL_SHA256=$b RAR_FINAL_CAPTURE_SHA256=$b RAR_CAPTURE_SET_SHA256=$b
export RAR_HANDOFF_MANIFEST_SET_SHA256=$b RAR_REFERENCE_VERDICT_SHA256=$zero
export RAR_ROLE_INVENTORIES_SHA256=$b RAR_ACCEPTED_OUTPUTS_SHA256=$b

make_record() {
    path=$1
    /usr/bin/printf '%s\n' \
        schema=rar-alpha-accepted-evidence-v0 \
        "attempt_nonce=$RAR_ACCEPT_ATTEMPT_NONCE" "probe=$RAR_EXPECTED_PROBE" \
        "controller_revision=$RAR_TRUSTED_CONTROLLER_SHA" "source_revision=$RAR_EXPECTED_SOURCE_REVISION" \
        "artifact_sha256=$RAR_ARTIFACT_SHA256" "acceptance_protocol_sha256=$RAR_ACCEPTANCE_PROTOCOL_SHA256" \
        "machine_profile_sha256=$RAR_MACHINE_PROFILE_SHA256" "qemu_sha256=$RAR_QEMU_SHA256" \
        "firmware_sha256=$RAR_FIRMWARE_SHA256" "qmp_client_sha256=$RAR_QMP_CLIENT_SHA256" \
        "actions_sha256=$RAR_ACTIONS_SHA256" "serial_sha256=$RAR_SERIAL_SHA256" \
        "final_capture_sha256=$RAR_FINAL_CAPTURE_SHA256" "capture_set_sha256=$RAR_CAPTURE_SET_SHA256" \
        "handoff_manifest_set_sha256=$RAR_HANDOFF_MANIFEST_SET_SHA256" \
        "reference_verdict_sha256=$RAR_REFERENCE_VERDICT_SHA256" \
        "role_inventories_sha256=$RAR_ROLE_INVENTORIES_SHA256" \
        "accepted_outputs_sha256=$RAR_ACCEPTED_OUTPUTS_SHA256" > "$path"
    digest=$(/usr/bin/shasum -a 256 "$path" | /usr/bin/awk '{ print $1 }')
    /usr/bin/printf 'record_sha256=%s\n' "$digest" >> "$path"
}
record=$work/record.v0
make_record "$record"
/bin/sh "$checker" "$record" >/dev/null

for key in attempt_nonce probe controller_revision source_revision artifact_sha256 acceptance_protocol_sha256 \
    machine_profile_sha256 qemu_sha256 firmware_sha256 qmp_client_sha256 actions_sha256 serial_sha256 \
    final_capture_sha256 capture_set_sha256 handoff_manifest_set_sha256 reference_verdict_sha256 \
    role_inventories_sha256 accepted_outputs_sha256 record_sha256; do
    make_record "$record"
    /usr/bin/sed -i.bak "s/^$key=.*/$key=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd/" "$record"
    /bin/rm -f "$record.bak"
    if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
done
make_record "$record"
/usr/bin/sed -i.bak '3p' "$record"; /bin/rm -f "$record.bak"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/usr/bin/sed -i.bak '3d' "$record"; /bin/rm -f "$record.bak"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/usr/bin/awk 'NR==2 { second=$0; next } NR==3 { print; print second; next } { print }' "$record" > "$record.mut"
/bin/mv "$record.mut" "$record"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/usr/bin/printf 'unknown=1\n' >> "$record"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/usr/bin/sed -i.bak '2s/a/A/' "$record"; /bin/rm -f "$record.bak"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/usr/bin/sed -i.bak "6s/$b/$zero/" "$record"; /bin/rm -f "$record.bak"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
RAR_EXPECTED_PROBE=milestone-f RAR_REFERENCE_VERDICT_SHA256=$zero /bin/sh "$checker" "$record" >/dev/null 2>&1 && exit 1
make_record "$record"
/usr/bin/printf '%s' "$(/usr/bin/sed -n '1,20p' "$record")" > "$record.mut"
/bin/mv "$record.mut" "$record"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/usr/bin/awk 'BEGIN { for (i=0; i<5000; i++) printf "x" }' >> "$record"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
make_record "$record"
/bin/mv "$record" "$work/record-real.v0"
/bin/ln -s record-real.v0 "$record"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
/bin/rm -f "$record"
make_record "$record"
/bin/ln "$record" "$work/record-hardlink.v0"
if /bin/sh "$checker" "$record" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'accepted evidence v0 replay and framing negatives passed'
