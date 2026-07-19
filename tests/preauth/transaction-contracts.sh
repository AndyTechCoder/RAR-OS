#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

fail() {
    echo "transaction-contract: $1" >&2
    exit 1
}

lock=spec/lab/preauth/locks/r0-x86_64-preauth-input-v4.lock
fields=spec/lab/preauth/closure-input-lock-v4.fields

actual_keys=$(sed 's/=.*//' "$lock")
expected_keys=$(sed -n '1,$p' "$fields")
[ "$actual_keys" = "$expected_keys" ] || fail 'input-lock field order differs from schema'
[ "$(grep -c '^schema=' "$lock")" -eq 1 ] || fail 'input-lock schema is duplicated'
grep -qx 'schema=rar-preauth-closure-input-lock-v4' "$lock" || fail 'wrong input-lock schema'
grep -qx 'launch_authority=none' "$lock" || fail 'transaction lock grants authority'
policy=spec/lab/preauth/preauth-input-delivery-v1.policy
policy_sha=$(sha256sum "$policy" | cut -d ' ' -f 1)
grep -qx "acquisition_policy_sha256=$policy_sha" "$lock" || fail 'input delivery policy hash mismatch'
grep -qx 'network_phase=producer-only' "$policy" || fail 'producer network phase absent'
grep -qx 'transaction_network=none' "$policy" || fail 'transaction network prohibition absent'
for forbidden in authority graph certification attestation owner session artifact disk profile command; do
 ! grep -q "^$forbidden=" spec/lab/preauth/preauth-input-bundle-v1.fields || fail "input bundle authority leak: $forbidden"
done

for forbidden in \
    canonical_oci_archive_sha256 canonical_oci_index_sha256 selected_oci_manifest_sha256 \
    docker_config_sha256 buildx_config_sha256 loaded_image_config_sha256 \
    artifact_sha256 disk_seed_sha256 disk_initial_sha256 transaction_graph_sha256 \
    attestation review_certificate owner_authorization launch_session source_revision; do
    ! grep -q "^$forbidden=" "$lock" || fail "source-dependent field in input lock: $forbidden"
done

graph=spec/lab/preauth/transaction-graph-v1.fields
for required in \
    source_revision input_lock_sha256 raw_oci_index_sha256 canonical_oci_index_sha256 \
    raw_to_canonical_index_relation selected_oci_manifest_sha256 docker_config_sha256 \
    buildx_config_sha256 loaded_image_config_sha256 compressed_layer_descriptor_set_sha256 \
    rootfs_diff_id_set_sha256 artifact_first_sha256 artifact_second_sha256 artifact_sha256 \
    disk_seed_sha256 disk_initial_sha256 profile_sha256 command_sha256 execution_host_sha256 \
    supervisor_sha256 resolver_sha256 spawner_sha256 wrapper_sha256 \
    resource_controller_sha256 publication_receipt_sha256 record_sha256; do
    [ "$(grep -cx "$required" "$graph")" -eq 1 ] || fail "transaction graph field missing or duplicate: $required"
done

for forbidden in authority authorization session nonce signature transition_version; do
    ! grep -q "$forbidden" "$graph" || fail "authority field leaked into transaction graph: $forbidden"
done

accept_lock_schema() {
    [ "$1" = rar-preauth-closure-input-lock-v4 ]
}
for rejected in \
    rar-preauth-closure-v2 rar-preauth-closure-v3 rar-preauth-identity-graph-v2 \
    rar-preauth-disposable-disk-v1 rar-execution-host-v1; do
    if accept_lock_schema "$rejected"; then
        fail "legacy schema accepted: $rejected"
    fi
done

grep -qx 'mutable_child_policy' spec/lab/preauth/disk-v2.fields || fail 'disk-v2 child policy missing'
grep -qx 'descriptor_slot_schema' spec/lab/preauth/execution-host-v2.fields || fail 'host-v2 descriptor slots missing'
grep -qx 'runtime_disk_slot' spec/lab/vm-profile/profile-v2.fields || fail 'profile-v2 descriptor slot missing'
grep -qx 'executable_slot' spec/lab/vm-profile/command-v2.fields || fail 'command-v2 executable slot missing'

# The production entrypoint's refusal paths must leave the repository byte-identical.
repository_state(){
    find out -type f 2>/dev/null | LC_ALL=C sort | while IFS= read -r state_file; do
        printf '%s %s\n' "$state_file" "$(cksum < "$state_file")"
    done
}
malformed=out/r0/preauth/.contract-malformed-bundle
mkdir -p out/r0/preauth
printf 'not-a-bundle' > "$malformed"
before=$(repository_state)
set +e
usage_output=$(tools/toolchain/preauth-transaction 2>&1); usage_status=$?
authority_output=$(RAR_TRANSACTION_NETWORK=none AWS_ACCESS_KEY_ID=forbidden tools/toolchain/preauth-transaction --prepare "$malformed" 2>&1); authority_status=$?
malformed_output=$(RAR_TRANSACTION_NETWORK=none tools/toolchain/preauth-transaction --prepare "$malformed" 2>&1); malformed_status=$?
set -e
[ "$usage_status" -eq 73 ] && [ "$authority_status" -eq 73 ] && [ "$malformed_status" -eq 73 ] || fail 'refusal exit status'
printf '%s\n' "$usage_output" | grep -q 'preauth-transaction:usage-refused' || fail 'usage refusal diagnostic'
printf '%s\n' "$authority_output" | grep -q 'preauth-transaction:authority-environment' || fail 'authority refusal diagnostic'
printf '%s\n' "$malformed_output" | grep -q 'preauth-transaction:' || fail 'malformed refusal diagnostic'
after=$(repository_state)
[ "$before" = "$after" ] || fail 'refusal produced repository effects'
rm -f "$malformed"

printf '%s\n' 'transaction contract checks passed'
