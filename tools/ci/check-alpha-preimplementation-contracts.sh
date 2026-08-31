#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
alpha=${1-$root/spec/alpha}
lab=$alpha/lab
boot=$alpha/boot
evidence=$alpha/evidence
p0_manifest=$alpha/platform/contract-set-v0.manifest

if [ -e "$p0_manifest" ] || [ -L "$p0_manifest" ]; then
    [ -f "$p0_manifest" ] && [ ! -L "$p0_manifest" ] && [ -s "$p0_manifest" ] ||
        { printf '%s\n' 'Alpha preimplementation contract blocked: P0 manifest is not a regular non-symbolic file' >&2; exit 1; }
    p0_active=1
else
    if [ -e "$alpha/platform" ] || [ -L "$alpha/platform" ] ||
        [ -e "$boot/alpha-machine-closure-v0.fields" ] || [ -L "$boot/alpha-machine-closure-v0.fields" ]; then
        printf '%s\n' 'Alpha preimplementation contract blocked: partial P0 topology exists without its manifest' >&2
        exit 1
    fi
    p0_active=0
fi

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
    "$lab/controller-handoff-v0.fields" \
    "$lab/controller-handoff-manifest-v0.fields" \
    "$lab/controller-handoff-cases.v0" \
    "$lab/controller-handoff-attempt-v0.fields" \
    "$lab/controller-handoff-attempt-cases.v0" \
    "$lab/controller-helper-inventory-v0.fields" \
    "$lab/controller-helper-build-evidence-v0.fields" \
    "$lab/controller-helper-build-receipt-v0.fields" \
    "$lab/controller-helper-test-evidence-v0.fields" \
    "$lab/controller-helper-cases.v0" \
    "$lab/fixtures/controller-helper/build-evidence.v0" \
    "$lab/fixtures/controller-helper/build-1-receipt.v0" \
    "$lab/fixtures/controller-helper/build-2-receipt.v0" \
    "$lab/fixtures/controller-helper/build-1.log.v0" \
    "$lab/fixtures/controller-helper/build-2.log.v0" \
    "$lab/fixtures/controller-helper/build-plan.v0" \
    "$lab/fixtures/controller-helper/builder-inventory.v0" \
    "$lab/fixtures/controller-helper/compiler-closure.v0" \
    "$lab/fixtures/controller-helper/compiler.v0" \
    "$lab/fixtures/controller-helper/golden-vector.v0" \
    "$lab/fixtures/controller-helper/helper-build-1.v0" \
    "$lab/fixtures/controller-helper/helper-build-2.v0" \
    "$lab/fixtures/controller-helper/helper-final.v0" \
    "$lab/fixtures/controller-helper/runner-image.v0" \
    "$lab/fixtures/controller-helper/source-tree.v0" \
    "$lab/fixtures/controller-helper/test-cases.v0" \
    "$lab/fixtures/controller-helper/test-evidence.v0" \
    "$lab/fixtures/controller-helper/test.log.v0" \
    "$lab/reference-evidence-v0.fields" \
    "$lab/fixtures/controller-context.v0" \
    "$lab/fixtures/source-context.v0" \
    "$lab/fixtures/reference-inventory.v0" \
    "$lab/fixtures/reference-harness.v0" \
    "$lab/fixtures/comparison-transcript.v0" \
    "$lab/fixtures/comparison-evidence.v0" \
    "$lab/fixtures/reference-verdict-accepted.v0" \
    "$lab/fixtures/reference-verdict-not-required.v0" \
    "$lab/cases.v0" \
    "$boot/README.md" \
    "$boot/alpha-boot-v0.fields" \
    "$boot/cases.v0" \
    "$evidence/acceptance-v2.plan" \
    "$evidence/acceptance-v2.fields" \
    "$evidence/acceptance-v2-cases.v0" \
    "$evidence/acceptance-v2-selection-digests.v0" \
    "$evidence/accepted-evidence-v0.fields" \
    "$evidence/accepted-evidence-v0-cases.v0"; do
    require_file "$file"
done

for fields in \
    "$lab/development-lab-profile-v2.fields" \
    "$lab/image-inventory-v2.fields" \
    "$lab/crypto-reference-inventory-v2.fields" \
    "$lab/comparison-transcript-v0.fields" \
    "$lab/controller-state-machine-v0.fields" \
    "$lab/controller-handoff-v0.fields" \
    "$lab/controller-handoff-manifest-v0.fields" \
    "$lab/controller-handoff-attempt-v0.fields" \
    "$lab/controller-helper-inventory-v0.fields" \
    "$lab/controller-helper-build-evidence-v0.fields" \
    "$lab/controller-helper-build-receipt-v0.fields" \
    "$lab/controller-helper-test-evidence-v0.fields" \
    "$lab/reference-evidence-v0.fields" \
    "$boot/alpha-boot-v0.fields"; do
    validate_field_file "$fields"
done

require_digest "$lab/development-lab-profile-v2.fields" fa2e0335d7192ae8dd843419a92b06f868df1a1a6e5b5eef202493a7ae849fda
require_digest "$lab/image-inventory-v2.fields" 76bb90f1e61721ffe1914ef004cf7b720f8a9b525251085f1e7bb45f4c8857c2
require_digest "$lab/crypto-reference-inventory-v2.fields" 4d70ad91c91f6b38ecf54595e623d09a397ddbf049160e4ace3a00ef29083d18
require_digest "$lab/comparison-transcript-v0.fields" 5f03fafed5eda2d373174aa0565ab08d009e07fe34e3bd7d2dc9c27c927dd9d7
require_digest "$lab/controller-state-machine-v0.fields" 57ad3d77c80318ffbbb64516866718da52fe62a6514c804d2ef6f49be33eaf1f
require_digest "$lab/controller-handoff-v0.fields" dc589f4c57891e1292f608c5b5514a97fe25df928b69342ac8e9e1f72560852e
require_digest "$lab/controller-handoff-manifest-v0.fields" ce13ec2588c21a8879d1eecf56ad9178d0f94806d3ffdd2d95af30ec206f9b02
require_digest "$lab/controller-handoff-cases.v0" e23032bf96424850f6840ce6136b486c2fea433b378fd902365c47aac776d7eb
require_digest "$lab/controller-handoff-attempt-v0.fields" e1de5fff796e6da039d7739da9771cd15c2a139bc15247a82ebe927fecfcf93b
require_digest "$lab/controller-handoff-attempt-cases.v0" 69a574038d6574bae00e0be1c368bac59c3a0850d0eb5e359721950de14a72a9
require_digest "$lab/controller-helper-inventory-v0.fields" f8a6a19aa3d776e237f28a7513745682eb9a7bcae2be6a3ced1c510192af961c
require_digest "$lab/controller-helper-build-evidence-v0.fields" 2d48e4575c09619286455b437b15f4adfcec9a27768382e405639be20204cbcf
require_digest "$lab/controller-helper-build-receipt-v0.fields" 23800a09f0480211357c7c01e233fed605792d4000b93954a46b9f091de16a2f
require_digest "$lab/controller-helper-test-evidence-v0.fields" bbef4dd6a0e343c8c9c1dae140cf65f4a24c7e80d1ceea9661c8d14706acb7fc
require_digest "$lab/controller-helper-cases.v0" 08e36412d104cc5c41169faf5d5e4dd423eb7fbfb0dba0623911423c51238e59
require_digest "$lab/controller-helper-runtime-v0.fields" 57620b332c0706ad2dd02a245593757ace511f2d9f58b49259c55346c6bd1b65
require_digest "$lab/controller-helper-runtime-cases.v0" addc112fdd9c88f5dc99a0f31eef452fbe1a54f808d348d3d13b8731da3103c0
require_digest "$lab/controller-helper-closure-observer-test-v0.fields" a4633dffa6727ace25cdd69705d3c006709a085726d93e6a2c42f287bccb1238
require_digest "$lab/controller-helper-closure-verifier-faults-v0.fields" 43f38cc2e75567a9e473c8f2c3d5f8ed6fffc03b8a33a70622bcdd1521901a00
require_digest "$lab/controller-helper-closure-verifier-cases-v0" d85c82abbba507fa7cbdf9af9c7ce6846139f1d93f0aa35fc6e340ff99e8e707
require_digest "$lab/controller-helper-closure-verifier-evidence-v0.fields" 0af4292b7eac63a7de68e34cd45e7cb4aa78bb81ecb0625f2d1aa47506cda009
require_digest "$lab/controller-helper-closure-acceptance-v0.fields" 8793c0b837c3e4e611038f2dd81dbba9befafc67a934edda5a0c0522da7df06a
require_digest "$lab/controller-helper-test-evidence-v1.fields" 62e0b135014a854d1187ef6ff9b24da9de07be50cd6e2e9b14cd9d59a0508ad7
require_digest "$lab/controller-helper-build-evidence-v1.fields" c2d174bf2d793079f98a7467f4b7bd12ceddc00e0c67d247a4b0531f66999f97
require_digest "$lab/reference-evidence-v0.fields" 2edbb270323d5fd074d3adc2929c695e0bb7ca957464ea814627ea82fc0c259e
require_digest "$lab/cases.v0" 966d84739240b871d2dd22e362ce07ec0e82706cbde32dfd4e493c0bd9758342
if [ "$p0_active" -eq 1 ]; then
    require_digest "$boot/alpha-boot-v0.fields" e2cd6322456224ac93cce70719f99f8d226bc73359786d1294115a27cb0d06fe
    boot_cases_digest=1a59e0d9135b018d46fbb70318f53ba79a876c580b8ef1ce174f0b5eeb7c7222
    boot_case_count=50
else
    require_digest "$boot/alpha-boot-v0.fields" 8a97440b2366e3554cca8948c47d0df8e3146230a1d049ead48a105612623e0e
    boot_cases_digest=370f829f791681cb4c1fb96dbf850f9535751a7a64295534562ea47a9f84bee3
    boot_case_count=41
fi
require_digest "$boot/cases.v0" "$boot_cases_digest"
require_digest "$evidence/acceptance-v2.plan" ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd
require_digest "$evidence/acceptance-v2.fields" 7a4416fb7429de5244694f985c3c62deaa91d9d441f746ece4cd675367ef6e15
require_digest "$evidence/acceptance-v2-cases.v0" e62b40fa7707a1b6540710328c0d049572c2a1f8de1d0dbc6d03c6d1dd2b62bf
require_digest "$evidence/acceptance-v2-selection-digests.v0" 851e3c7dd14509a20f958545b8efbc63b5302251a1a16e0aca967efc815a6c3f
require_digest "$evidence/accepted-evidence-v0.fields" 73874f4e3ea10bf356641365819fcc8075cd98f53c3f3c5fa28b2868a11c1703
require_digest "$evidence/accepted-evidence-v0-cases.v0" a19f413b79e365c8ccbf975e5b1454f8fd3be7b02455a60e243bcf8aa2c2351f
/bin/sh "$root/tools/ci/check-acceptance-v2.sh" >/dev/null || fail 'acceptance protocol v2 is invalid'

if find "$lab" "$boot" ! -name '._*' -type l -print | /usr/bin/grep -q .; then
    fail 'contract tree contains a symbolic link'
fi
if find "$lab" "$boot" ! -name '._*' ! -name 'comparison-transcript.v0' ! -name 'comparison-evidence.v0' -type f -exec /usr/bin/grep -nHE '[[:blank:]]+$' {} + |
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
[ "$(/usr/bin/grep -c '^required_field|' "$lab_profile")" -eq 38 ] || fail 'Lab profile field set is incomplete'

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
handoff=$lab/controller-handoff-v0.fields
require_line "$handoff" 'source_open_rule=openat-root-fd,O_RDONLY+O_CLOEXEC+O_NOFOLLOW+O_NONBLOCK,no-path-reopen,deadline-and-cancellation-bounded'
require_line "$handoff" 'copy_rule=descriptor-to-descriptor,bounded-buffer,checked-byte-count,no-sparse-assumption,no-external-command,no-path-copy'
require_line "$handoff" 'recheck_rule=fstat-same-source-fd,all-recorded-identity-fields-unchanged,EOF-exactly-after-recorded-size'
require_line "$handoff" 'enumeration_rule=one-retained-root-fd-per-output-mount,descriptor-relative-enumeration-before-open-and-after-all-copy-rechecks,ignore-only-dot+dot-dot,root-identity-unchanged,no-path-reopen'
require_line "$handoff" 'destination_open_rule=openat-destination-root-fd,O_RDWR+O_CREAT+O_EXCL+O_CLOEXEC+O_NOFOLLOW,mode-0600'
require_line "$handoff" 'destination_check_rule=seek-zero+read-same-destination-fd,exact-recorded-size+EOF,sha256-equals-source-copy,fstat-same-destination-fd,regular,current-controller-owner,mode-0600,nlink-1,size-unchanged'
require_line "$handoff" 'publication_rule=close-source+destination-fds-after-copy-recheck,create-manifest-relative-to-controller-manifest-root-fd,O_RDWR+O_CREAT+O_EXCL+O_CLOEXEC+O_NOFOLLOW,mode-0600,write-exactly-256,seek-zero,parse-exactly-256+require-EOF+fstat-same-manifest-fd,fdatasync-manifest,close-manifest-fd,fsync-manifest-root-before-next-role'
require_line "$handoff" 'failure_rule=close-open-fds,remove-only-destination+manifest-created-by-this-attempt-after-device+inode-match,fsync-each-affected-root,retain-bounded-controller-error,no-retained-manifest,no-next-role,no-publication,cleanup-uncertainty-permanently-blocks-progression'
require_line "$handoff" 'ordinal_rule=build-artifact-1,build-transcript-2,reference-comparison-evidence-1,launch-one-based-position-in-controller-fixed-acceptance-plan-allowlist'
[ "$(/usr/bin/grep -c '^role_output|' "$handoff")" -eq 3 ] || fail 'controller handoff role output set is incomplete'
manifest=$lab/controller-handoff-manifest-v0.fields
require_line "$manifest" 'manifest_bytes=256'
require_line "$manifest" 'canonical_rule=total-bytes-256,ordinal-1..999,basename-bytes-1..64,basename-ascii-lowercase-digit-dot-hyphen,unused-basename-tail-zero,flags-zero,all-reserved-zero,no-trailing-byte'
require_line "$manifest" 'ordinal_rule=phase-2+3-artifact-1,phase-2+3-transcript-2,phase-5-comparison-evidence-1,phase-7-one-based-position-in-controller-fixed-acceptance-plan-allowlist'
require_line "$manifest" 'durability_rule=fdatasync-manifest-then-close-then-fsync-controller-manifest-root-before-next-role'
[ "$(/usr/bin/grep -c '^wire_field|HandoffManifestV0|' "$manifest")" -eq 19 ] || fail 'controller handoff manifest layout is incomplete'
validate_case_file "$lab/controller-handoff-cases.v0" 'schema=rar-alpha-controller-handoff-cases-v0' 49
attempt=$lab/controller-handoff-attempt-v0.fields
require_line "$attempt" 'active_open_rule=O_RDWR+O_CREAT+O_EXCL+O_CLOEXEC+O_NOFOLLOW,mode-0600,one-active-attempt'
require_line "$attempt" 'running_rule=live-nonreusable-controller-owned-process-handle-required,pid-never-authority'
require_line "$attempt" 'commit_rule=committed-durable-before-active-removal,active-remove-after-device+inode-match+journal-root-fsync,next-phase-requires-both'
require_line "$attempt" 'recovery_rule=source-never-deleted,inventory-durable-before-delete,remove-only-inventory-entry-after-device+inode+type+owner+mode+links+size+sha256+mtime+ctime-match,missing-means-idempotently-removed,changed-means-blocked'
require_line "$attempt" 'activation_rule=source-contract-only,no-helper-spawn,no-process-FD-protocol,no-cloud-command,no-ready-identity,no-Mac-execution'
validate_case_file "$lab/controller-handoff-attempt-cases.v0" 'schema=rar-alpha-controller-handoff-attempt-cases-v0' 97
/bin/sh "$root/tools/ci/check-controller-handoff-attempt-v0.sh" "$attempt" "$lab/controller-handoff-attempt-cases.v0" >/dev/null || fail 'controller attempt recovery contract is invalid'
helper_inventory=$lab/controller-helper-inventory-v0.fields
helper_evidence=$lab/controller-helper-build-evidence-v0.fields
helper_receipt=$lab/controller-helper-build-receipt-v0.fields
helper_test_evidence=$lab/controller-helper-test-evidence-v0.fields
require_line "$helper_inventory" 'blocked_rule=decision+topology+all-builder+compiler+source+binary+evidence-identities-unavailable'
require_line "$helper_inventory" 'authority_rule=helper-filesystem-descriptors-only,no-process-spawn,no-network,no-container-api,no-cloud-api,no-credential,no-GitHub-write,no-target-launch'
require_line "$helper_inventory" 'activation_rule=inventory+compiler-closure+source+binary+build-evidence+test-evidence-reviewed-and-bound-before-v2-controller-ready'
require_line "$helper_evidence" 'binary_rule=build-1-sha256-equals-build-2-sha256-equals-final-binary-sha256,binary-bytes-1..16777216'
require_line "$helper_evidence" 'execution_rule=build-count-2,reproducible-yes,network-none,status-accepted,controller-observed-receipts-required'
require_line "$helper_evidence" 'failure_rule=missing,extra,duplicate,reordered,malformed,unapproved-decision,topology-mismatch,zero-digest,build-mismatch,oversize,networked,nonfresh,test-failure,or-nonzero-status-rejects'
require_line "$helper_test_evidence" 'schema=rar-alpha-controller-helper-test-evidence-v0'
require_line "$helper_receipt" 'producer_rule=trusted-outer-controller-after-builder-termination'
require_line "$helper_receipt" 'freshness_rule=build-ordinal-1-or-2,distinct-job+root-across-pair,fresh-root-yes,preexisting-output-no'
require_line "$helper_test_evidence" 'identity_rule=controller-sha-40-lowercase-hex,job-nonce+runner+source+binary+golden+case-results+log-digests-64-lowercase-nonzero-sha256-and-match-controller-selected-inputs'
require_line "$helper_test_evidence" 'result_rule=producer-trusted-outer-controller,test-count-13,failed-count-0,network-none,observed-exit-status-0,status-accepted'
validate_case_file "$lab/controller-helper-cases.v0" 'schema=rar-alpha-controller-helper-cases-v0' 40
helper_fixtures=$lab/fixtures/controller-helper
/bin/sh "$root/tools/ci/check-controller-helper-build-evidence-v0.sh" "$helper_fixtures/build-evidence.v0" "$root" adr-0024-alternative-a runner-closure 1111111111111111111111111111111111111111 "$helper_fixtures/runner-image.v0" "$helper_fixtures/source-tree.v0" "$helper_fixtures/build-plan.v0" "$helper_fixtures/golden-vector.v0" "$helper_fixtures/builder-inventory.v0" "$helper_fixtures/compiler-closure.v0" "$helper_fixtures/compiler.v0" "$helper_fixtures/helper-build-1.v0" "$helper_fixtures/helper-build-2.v0" "$helper_fixtures/helper-final.v0" "$helper_fixtures/build-1-receipt.v0" "$helper_fixtures/build-2-receipt.v0" "$helper_fixtures/build-1.log.v0" "$helper_fixtures/build-2.log.v0" "$helper_fixtures/test-evidence.v0" "$helper_fixtures/test-cases.v0" "$helper_fixtures/test.log.v0" >/dev/null || fail 'synthetic controller helper build evidence is invalid'
/bin/sh "$root/tools/ci/check-reference-evidence-v0.sh" "$lab/fixtures/comparison-evidence.v0" "$lab/fixtures/comparison-transcript.v0" "$lab/fixtures/reference-inventory.v0" "$lab/fixtures/reference-harness.v0" >/dev/null || fail 'reference evidence fixture is invalid'
/bin/sh "$root/tools/ci/check-reference-verdict-v0.sh" "$lab/fixtures/reference-verdict-accepted.v0" milestone-f "$lab/fixtures/controller-context.v0" "$lab/fixtures/source-context.v0" "$lab/fixtures/comparison-transcript.v0" "$lab/fixtures/reference-inventory.v0" "$lab/fixtures/comparison-evidence.v0" "$lab/fixtures/reference-harness.v0" >/dev/null || fail 'accepted reference verdict fixture is invalid'
/bin/sh "$root/tools/ci/check-reference-verdict-v0.sh" "$lab/fixtures/reference-verdict-not-required.v0" milestone-a "$lab/fixtures/controller-context.v0" "$lab/fixtures/source-context.v0" "$lab/fixtures/comparison-transcript.v0" none none none >/dev/null || fail 'not-required reference verdict fixture is invalid'

boot_contract=$boot/alpha-boot-v0.fields
require_line "$boot_contract" 'schema=rar-alpha-x86_64-boot-v0'
if [ "$p0_active" -eq 1 ]; then
    require_line "$boot_contract" 'status=experimental-pending-review'
    require_line "$boot_contract" 'readiness=blocked-until-contract-set-review-merge-exact-main-and-machine-evidence'
else
    require_line "$boot_contract" 'status=draft-incomplete'
    require_line "$boot_contract" 'readiness=blocked-on-byte-layout-memory-attributes-timer-and-x86-control-state'
fi
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
if [ "$p0_active" -eq 1 ]; then
    [ "$(/usr/bin/grep -c '^wire_field|RootRecoveryHeaderV0|' "$boot_contract")" -eq 30 ] || fail 'Root-to-Recovery header layout is incomplete'
else
    [ "$(/usr/bin/grep -c '^wire_field|RootRecoveryHeaderV0|' "$boot_contract")" -eq 20 ] || fail 'Root-to-Recovery header layout is incomplete'
fi
[ "$(/usr/bin/grep -c '^wire_field|RootRecoveryMapRecordV0|' "$boot_contract")" -eq 7 ] || fail 'Root-to-Recovery map layout is incomplete'
[ "$(/usr/bin/grep -c '^memory_type_rule|' "$boot_contract")" -eq 15 ] || fail 'UEFI memory mapping table is incomplete'
validate_case_file "$boot/cases.v0" 'schema=rar-alpha-boot-cases-v0' "$boot_case_count"

handoff_expected=$(/usr/bin/sed -n 's/^r0_handoff_contract_sha256=//p' "$boot_contract")
hardware_expected=$(/usr/bin/sed -n 's/^r0_hardware_contract_sha256=//p' "$boot_contract")
profile_expected=$(/usr/bin/sed -n 's/^machine_profile_sha256=//p' "$boot_contract")
[ "$handoff_expected" = "$(digest_file "$root/spec/boot/handoff-v1.fields")" ] || fail 'R0 handoff contract binding changed'
[ "$hardware_expected" = "$(digest_file "$root/spec/hardware/rhd-v1.fields")" ] || fail 'R0 hardware contract binding changed'
[ "$profile_expected" = "$(digest_file "$root/tools/sprint-alpha/x86_64-q35-v1.profile")" ] || fail 'machine profile binding changed'

if [ "$p0_active" -eq 1 ]; then
    /bin/sh "$root/tools/ci/check-alpha-boot-platform-contracts.sh" "$alpha" >/dev/null ||
        fail 'Alpha boot/platform P0 contract set is invalid'
fi

printf '%s\n' 'Alpha preimplementation contract structure passed'
