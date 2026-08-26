#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
alpha=${1-$root/spec/alpha}
lab=$alpha/lab
boot=$alpha/boot

fail() {
    printf 'Alpha preimplementation contract blocked: %s\n' "$1" >&2
    exit 1
}

require_file() {
    [ -f "$1" ] && [ ! -L "$1" ] || fail "missing or symbolic file: $1"
    [ -s "$1" ] || fail "empty file: $1"
}

require_line() {
    [ "$(/usr/bin/grep -Fxc -- "$2" "$1")" -eq 1 ] ||
        fail "missing or duplicate contract row: $2"
}

validate_field_file() {
    /usr/bin/awk -F '|' '
        NF == 0 { next }
        NF == 1 {
            if ($0 !~ /^[a-z0-9_]+=[^[:cntrl:]]+$/) bad = 1
            split($0, pair, "=")
            if (++single[pair[1]] != 1) bad = 1
            next
        }
        {
            if ($1 !~ /^[a-z0-9_]+$/) bad = 1
            for (i = 1; i <= NF; i++) if ($i == "" || $i ~ /[[:cntrl:]]/) bad = 1
            if (++row[$0] != 1) bad = 1
        }
        END { exit bad ? 1 : 0 }
    ' "$1" || fail "malformed or duplicate field contract row: $1"
}

validate_case_file() {
    expected_schema=$2
    expected_count=$3
    /usr/bin/awk -F '|' -v schema="$expected_schema" -v count="$expected_count" '
        NR == 1 { if ($0 != schema) bad = 1; next }
        NR == 2 { if ($0 != "id|contract|expected" && $0 != "id|stage|expected") bad = 1; next }
        NR > 2 {
            if (NF != 3 || $1 !~ /^[a-z0-9][a-z0-9-]*$/ ||
                $2 !~ /^[a-z0-9][a-z0-9-]*$/ ||
                $3 !~ /^(accept|reject|reject-no-authority)$/) bad = 1
            if (++id[$1] != 1) bad = 1
            rows++
        }
        END { if (rows != count) bad = 1; exit bad ? 1 : 0 }
    ' "$1" || fail "malformed, duplicate, or incomplete case table: $1"
}

digest_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        output=$(sha256sum "$1")
    else
        output=$(/usr/bin/shasum -a 256 "$1")
    fi
    printf '%s' "${output%% *}"
}

require_digest() {
    [ "$(digest_file "$1")" = "$2" ] || fail "contract bytes changed without rebinding: $1"
}

for file in \
    "$lab/README.md" \
    "$lab/development-lab-profile-v2.fields" \
    "$lab/image-inventory-v2.fields" \
    "$lab/crypto-reference-inventory-v2.fields" \
    "$lab/comparison-transcript-v0.fields" \
    "$lab/controller-state-machine-v0.fields" \
    "$lab/cases.v0" \
    "$boot/README.md" \
    "$boot/alpha-boot-v0.fields" \
    "$boot/cases.v0"; do
    require_file "$file"
done

for fields in \
    "$lab/development-lab-profile-v2.fields" \
    "$lab/image-inventory-v2.fields" \
    "$lab/crypto-reference-inventory-v2.fields" \
    "$lab/comparison-transcript-v0.fields" \
    "$lab/controller-state-machine-v0.fields" \
    "$boot/alpha-boot-v0.fields"; do
    validate_field_file "$fields"
done

require_digest "$lab/development-lab-profile-v2.fields" 86ca738fdfdef78b68d750375039ab316dff2976f0c1dd7f440eea59a881e06c
require_digest "$lab/image-inventory-v2.fields" aa4c763f78c04b904e517221677fdad2e3a5a1e9b9d2d4d71dfbf26208fed9bd
require_digest "$lab/crypto-reference-inventory-v2.fields" 62914ec46eb5ce005ed94b22dcbd5937aadb8424890532eca0a87e204a1635e5
require_digest "$lab/comparison-transcript-v0.fields" 9d60bc1870fbf8cb8184e6bde512cb4eebd47d20ed63ef36e6f8182f5d596aa5
require_digest "$lab/controller-state-machine-v0.fields" 07bdd7852147eb438f06e192f187b0cd357cb45427c5a1052c753471b42cb585
require_digest "$lab/cases.v0" 966d84739240b871d2dd22e362ce07ec0e82706cbde32dfd4e493c0bd9758342
require_digest "$boot/alpha-boot-v0.fields" 8a97440b2366e3554cca8948c47d0df8e3146230a1d049ead48a105612623e0e
require_digest "$boot/cases.v0" 370f829f791681cb4c1fb96dbf850f9535751a7a64295534562ea47a9f84bee3

if find "$lab" "$boot" ! -name '._*' -type l -print | /usr/bin/grep -q .; then
    fail 'contract tree contains a symbolic link'
fi
if find "$lab" "$boot" ! -name '._*' -type f -exec /usr/bin/grep -nHE '[[:blank:]]+$' {} + |
    /usr/bin/grep -q .; then
    fail 'contract tree contains trailing whitespace'
fi

lab_profile=$lab/development-lab-profile-v2.fields
image_inventory=$lab/image-inventory-v2.fields
crypto_inventory=$lab/crypto-reference-inventory-v2.fields
transcript=$lab/comparison-transcript-v0.fields
require_line "$lab_profile" 'schema=rar-alpha-development-lab-profile-schema-v2'
require_line "$lab_profile" 'status=experimental-inactive'
require_line "$lab_profile" 'readiness=source-ready-pending-review'
require_line "$lab_profile" 'roles=build,reference,launch'
require_line "$lab_profile" 'role_rule|build|source:read-only,compiler:required,linker:required,reference:forbidden,qemu:forbidden,firmware:forbidden,network:none,credentials:none'
require_line "$lab_profile" 'role_rule|reference|source:forbidden,target-image:forbidden,transcript:read-only,references:required,compiler:forbidden,linker:forbidden,qemu:forbidden,firmware:forbidden,network:none,credentials:none'
require_line "$lab_profile" 'role_rule|launch|source:forbidden,target-image:read-only,references:forbidden,compiler:forbidden,linker:forbidden,qemu:required,firmware:required,network:none,credentials:none'
require_line "$lab_profile" 'runtime_mount_rule|build|/workspace:source:read-only,/output/artifact:artifact-only:read-write-noexec-nodev-nosuid:64MiB,/output/transcript:transcript-only:read-write-noexec-nodev-nosuid:1MiB'
require_line "$lab_profile" 'runtime_mount_rule|reference|/input/transcript:transcript-one-file:read-only-noexec-nodev-nosuid,/output/reference:comparison-evidence-only:read-write-noexec-nodev-nosuid:1MiB'
require_line "$lab_profile" 'runtime_forbidden_mount|build|controller,machine-profile,reference-binary,reference-evidence,launch-evidence,qemu,firmware,host-path'
require_line "$lab_profile" 'runtime_forbidden_mount|reference|source,controller,machine-profile,target-image,build-output,launch-evidence,qemu,firmware,host-path'
require_line "$lab_profile" 'runtime_forbidden_mount|launch|source,build-output,transcript,reference-binary,reference-evidence,compiler,linker,host-path'
require_line "$lab_profile" 'runtime_environment_rule|all|empty-baseline,exact-reviewed-name-allowlist,no-secret,no-credential,no-host-environment-inheritance'
require_line "$lab_profile" 'runtime_authority_rule|all|read-only-root,uid-65532,gid-65532,no-network,no-capabilities,no-new-privileges,no-device,no-host-pid,no-host-ipc,no-host-uts,no-privileged,no-extra-mount-or-output'
require_line "$lab_profile" 'runtime_handoff_rule=controller-opens-bounded-regular-output-no-follow,checks-owner-mode-link-count-size-and-hash,copies-to-fresh-controller-owned-file,rechecks-same-descriptor,unmounts-prior-role-before-next-role'
require_line "$lab_profile" 'identity_rule|image-digests|build,reference,launch-pairwise-distinct'
require_line "$lab_profile" 'blocked_rule|all-activating-identities|unavailable'
[ "$(/usr/bin/grep -c '^required_field|' "$lab_profile")" -eq 36 ] || fail 'Lab profile field set is incomplete'

require_line "$image_inventory" 'schema=rar-alpha-image-inventory-schema-v2'
require_line "$image_inventory" 'role_absence|build|reference,reference-harness,qemu,firmware,qmp-client'
require_line "$image_inventory" 'role_absence|reference|source,compiler,linker,qemu,firmware,qmp-client'
require_line "$image_inventory" 'role_absence|launch|source,compiler,linker,reference,reference-harness'
require_line "$image_inventory" 'reproducibility=two-independent-byte-identical-oci-exports'
[ "$(/usr/bin/grep -c '^entry_field|' "$image_inventory")" -eq 7 ] || fail 'image inventory entry schema is incomplete'

require_line "$crypto_inventory" 'schema=rar-alpha-crypto-reference-inventory-schema-v2'
require_line "$crypto_inventory" 'role=reference-only'
require_line "$crypto_inventory" 'boundary_rule=no-source,no-target-image,no-launch-authority,no-network,no-credentials'
require_line "$crypto_inventory" 'target_rule=never-linked,never-shipped,never-runtime-loaded'
[ "$(/usr/bin/grep -c '^required_field|' "$crypto_inventory")" -eq 18 ] || fail 'crypto inventory field set is incomplete'

require_line "$transcript" 'schema=rar-alpha-comparison-transcript-v0'
require_line "$transcript" 'maximum_total_bytes=1048576'
require_line "$transcript" 'maximum_record_count=512'
require_line "$transcript" 'maximum_message_bytes=1024'
require_line "$transcript" 'reference_rule=both-references-recompute-every-record-and-match-each-other-and-target'
require_line "$transcript" 'failure_rule=reject-before-signing-evidence'
[ "$(/usr/bin/grep -c '^wire_field|TranscriptHeaderV0|' "$transcript")" -eq 9 ] || fail 'transcript header layout is incomplete'
[ "$(/usr/bin/grep -c '^wire_field|TranscriptRecordV0|' "$transcript")" -eq 10 ] || fail 'transcript record layout is incomplete'
validate_case_file "$lab/cases.v0" 'schema=rar-alpha-lab-contract-cases-v0' 34

boot_contract=$boot/alpha-boot-v0.fields
require_line "$boot_contract" 'schema=rar-alpha-x86_64-boot-v0'
require_line "$boot_contract" 'status=draft-incomplete'
require_line "$boot_contract" 'readiness=blocked-on-byte-layout-memory-attributes-timer-and-x86-control-state'
require_line "$boot_contract" 'root_path=\EFI\BOOT\BOOTX64.EFI'
require_line "$boot_contract" 'recovery_path=\RAR\ALPHA\RECOVERY.ELF'
require_line "$boot_contract" 'nucleus_path=\RAR\ALPHA\NUCLEUS.ELF'
require_line "$boot_contract" 'elf_forbidden=PT_DYNAMIC,PT_INTERP,PT_TLS,relocations,shared-objects'
require_line "$boot_contract" 'elf_permissions=read-required,write-xor-execute,W^X'
require_line "$boot_contract" 'entry_registers=RDI:0x01800000,RSI:total_bytes,others:no-authority'
require_line "$boot_contract" 'entry_cpu_state=long-mode,ring0,interrupts-disabled,direction-clear,x87-reset,sse2-enabled'
require_line "$boot_contract" 'uefi_forbidden_after_exit=all-firmware-pointers,all-runtime-services,all-boot-services'
require_line "$boot_contract" 'uefi_exit_retry=maximum-4,only-invalid-parameter-retries,refresh-map-key,no-other-allocation-after-final-map'
require_line "$boot_contract" 'r0_source_producer=recovery-only'
require_line "$boot_contract" 'r0_source_precondition=complete-writes,producer-write-revoked,dma-revoked,immutable-where-required'
require_line "$boot_contract" 'r0_device_authority=apic-exact-mmio-descriptor,serial-exact-io-port-descriptor,no-unused-device-descriptor'
require_line "$boot_contract" 'limitations=no-signatures,no-rollback-counter,no-A-B,no-production-entropy,no-persistent-format,no-update-compatibility,no-physical-support'
[ "$(/usr/bin/grep -c '^wire_field|RootRecoveryHeaderV0|' "$boot_contract")" -eq 20 ] || fail 'Root-to-Recovery header layout is incomplete'
[ "$(/usr/bin/grep -c '^wire_field|RootRecoveryMapRecordV0|' "$boot_contract")" -eq 7 ] || fail 'Root-to-Recovery map layout is incomplete'
[ "$(/usr/bin/grep -c '^memory_type_rule|' "$boot_contract")" -eq 15 ] || fail 'UEFI memory mapping table is incomplete'
validate_case_file "$boot/cases.v0" 'schema=rar-alpha-boot-cases-v0' 41

handoff_expected=$(/usr/bin/sed -n 's/^r0_handoff_contract_sha256=//p' "$boot_contract")
hardware_expected=$(/usr/bin/sed -n 's/^r0_hardware_contract_sha256=//p' "$boot_contract")
profile_expected=$(/usr/bin/sed -n 's/^machine_profile_sha256=//p' "$boot_contract")
[ "$handoff_expected" = "$(digest_file "$root/spec/boot/handoff-v1.fields")" ] || fail 'R0 handoff contract binding changed'
[ "$hardware_expected" = "$(digest_file "$root/spec/hardware/rhd-v1.fields")" ] || fail 'R0 hardware contract binding changed'
[ "$profile_expected" = "$(digest_file "$root/tools/sprint-alpha/x86_64-q35-v1.profile")" ] || fail 'machine profile binding changed'

printf '%s\n' 'Alpha preimplementation contract structure passed'
