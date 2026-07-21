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

# Refusal snapshots live outside the repository and cover the complete tree. No Git
# internals are excluded: these wrappers never invoke Git and therefore have no
# production reason to write below .git.
contract_scratch=$(mktemp -d "${TMPDIR:-/tmp}/rar-transaction-contract.XXXXXX") || fail 'contract scratch'
chmod 0700 "$contract_scratch"
. tools/toolchain/preauth-build-root.sh
rustc_path=$(preauth_build_pinned_rustc_path "$root") || fail 'pinned rustc unavailable'
if [ -n "${RAR_PREAUTH_BUILD_ROOT-}" ]; then
    contract_build_parent=$RAR_PREAUTH_BUILD_ROOT/transaction-contract-roots
    [ ! -e "$contract_build_parent" ] || fail 'contract build parent collision'
else
    contract_build_parent=$contract_scratch/executable-roots
fi
mkdir -m 700 "$contract_build_parent"
build_root=$contract_build_parent/build-root
mkdir -m 700 "$build_root"
repository_state(){
    tests/preauth/snapshot-repository-tree.sh "$1"
}
assert_unchanged(){ repository_state "$contract_scratch/after"; cmp -s "$contract_scratch/before" "$contract_scratch/after" || fail "$1 changed the repository tree"; }
invoke_refusal(){
    refusal_name=$1; shift
    repository_state "$contract_scratch/before"
    set +e
    "$@" >"$contract_scratch/$refusal_name.stdout" 2>"$contract_scratch/$refusal_name.stderr"
    refusal_status=$?
    set -e
    [ "$refusal_status" -eq 73 ] || fail "$refusal_name exit status"
    [ ! -s "$contract_scratch/$refusal_name.stdout" ] || fail "$refusal_name wrote stdout"
    assert_unchanged "$refusal_name"
}

invoke_refusal usage tools/toolchain/preauth-transaction
grep -qx 'preauth-transaction:usage-refused' "$contract_scratch/usage.stderr" || fail 'usage diagnostic'
invoke_refusal authority env RAR_PREAUTH_BUILD_ROOT="$build_root" RAR_TRANSACTION_NETWORK=none AWS_ACCESS_KEY_ID=forbidden \
    tools/toolchain/preauth-transaction --prepare AGENTS.md
grep -qx 'preauth-transaction:authority-environment' "$contract_scratch/authority.stderr" || fail 'authority diagnostic'
invoke_refusal malformed env RAR_PREAUTH_BUILD_ROOT="$build_root" RAR_TRANSACTION_NETWORK=none \
    tools/toolchain/preauth-transaction --prepare AGENTS.md
grep -q '^preauth-transaction:' "$contract_scratch/malformed.stderr" || fail 'malformed diagnostic'
invoke_refusal base-invalid env RAR_PREAUTH_BUILD_ROOT="$build_root" \
    tools/toolchain/preauth-base-oci --canonicalize AGENTS.md out/r0/base-invalid.tar out/r0/base-invalid.tools
grep -q '^preauth-base-oci:' "$contract_scratch/base-invalid.stderr" || fail 'base invalid diagnostic'

# A canonical 50-object/36-package fixture exercises the real M2-incomplete path.
fixture_generator=$contract_build_parent/generate-valid-input-bundle
mkdir -m 700 "$contract_scratch/poisoned-bin" "$contract_scratch/readonly-rustup"
printf '%s\n' '#!/bin/sh' 'exit 99' > "$contract_scratch/poisoned-bin/rustc"
chmod 0700 "$contract_scratch/poisoned-bin/rustc"
chmod 0500 "$contract_scratch/readonly-rustup"
env PATH="$contract_scratch/poisoned-bin:$PATH" RUSTUP_HOME="$contract_scratch/readonly-rustup" \
    "$rustc_path" --edition=2024 tests/preauth/generate-valid-input-bundle.rs -o "$fixture_generator" \
    || fail 'fixture generator compile'
valid_bundle=out/r0/preauth-contract-valid.tar
"$fixture_generator" "$valid_bundle"
repository_state "$contract_scratch/before"
set +e
env RAR_PREAUTH_BUILD_ROOT="$build_root" RAR_TRANSACTION_NETWORK=none \
    tools/toolchain/preauth-transaction --prepare "$valid_bundle" \
    >"$contract_scratch/m2.stdout" 2>"$contract_scratch/m2.stderr"
m2_status=$?
set -e
[ "$m2_status" -eq 73 ] || fail 'M2-incomplete exit status'
[ ! -s "$contract_scratch/m2.stdout" ] || fail 'M2-incomplete wrote stdout'
grep -qx 'preauth-transaction:evidence:input_bundle_schema=rar-preauth-input-bundle-v1' "$contract_scratch/m2.stderr" || fail 'M2 schema evidence'
grep -qx 'preauth-transaction:evidence:input_object_count=50' "$contract_scratch/m2.stderr" || fail 'M2 object evidence'
grep -qx 'preauth-transaction:evidence:input_package_count=36' "$contract_scratch/m2.stderr" || fail 'M2 package evidence'
grep -qx 'preauth-transaction:m2-incomplete' "$contract_scratch/m2.stderr" || fail 'M2 refusal marker'
assert_unchanged M2-incomplete
rm -f "$valid_bundle"

# The shared root validator fails closed before mktemp for every unsafe root class.
check_root_refusal(){
    root_name=$1; candidate=$2
    set +e
    env RAR_PREAUTH_BUILD_ROOT="$candidate" RAR_TRANSACTION_NETWORK=none \
        tools/toolchain/preauth-transaction --prepare AGENTS.md \
        >"$contract_scratch/root-$root_name.stdout" 2>"$contract_scratch/root-$root_name.stderr"
    root_status=$?
    set -e
    [ "$root_status" -eq 73 ] && [ ! -s "$contract_scratch/root-$root_name.stdout" ] \
        || fail "build root accepted: $root_name"
    grep -q '^preauth-transaction:build-root-' "$contract_scratch/root-$root_name.stderr" \
        || fail "build root diagnostic: $root_name"
}
check_root_refusal repository-local "$root"
check_root_refusal root-alias "$root/."
mkdir -m 700 "$contract_scratch/real-parent"
ln -s "$contract_scratch/real-parent" "$contract_scratch/link-parent"
check_root_refusal symlinked-ancestor "$contract_scratch/link-parent"
mkdir -m 777 "$contract_scratch/unsafe-shared"
check_root_refusal unsafe-mode "$contract_scratch/unsafe-shared"
check_root_refusal wrong-owner /usr
check_root_refusal missing "$contract_scratch/missing"
printf 'not a directory\n' > "$contract_scratch/not-directory"
check_root_refusal non-directory "$contract_scratch/not-directory"

# A lying mktemp that returns a pre-existing leaf is treated as a collision.
collision_root=$contract_build_parent/collision-root; mkdir -m 700 "$collision_root"
collision_leaf=$collision_root/preauth-transaction.COLLIDE1; mkdir -m 700 "$collision_leaf"
collision_fake_bin=$contract_build_parent/collision-fake-bin
mkdir -m 700 "$collision_fake_bin"
printf '%s\n' '#!/bin/sh' "printf '%s\\n' './preauth-transaction.COLLIDE1'" > "$collision_fake_bin/mktemp"
chmod 0700 "$collision_fake_bin/mktemp"
set +e
env PATH="$collision_fake_bin:$PATH" RAR_PREAUTH_BUILD_ROOT="$collision_root" RAR_TRANSACTION_NETWORK=none \
    tools/toolchain/preauth-transaction --prepare AGENTS.md \
    >"$contract_scratch/collision.stdout" 2>"$contract_scratch/collision.stderr"
collision_status=$?
set -e
[ "$collision_status" -eq 73 ] && [ ! -s "$contract_scratch/collision.stdout" ] || fail 'build collision accepted'
grep -qx 'preauth-transaction:build-leaf-collision' "$contract_scratch/collision.stderr" || fail 'build collision diagnostic'
[ -d "$collision_leaf" ] || fail 'collision leaf overwritten'

# The shared trap removes its exact private leaf on generic failure and preserves
# conventional signal statuses while terminating and reaping its exact child.
cleanup_root=$contract_build_parent/cleanup-root; mkdir -m 700 "$cleanup_root"
(
    . tools/toolchain/preauth-build-root.sh
    RAR_PREAUTH_BUILD_ROOT=$cleanup_root
    preauth_build_root_create "$root" cleanup-failure cleanup-failure
    printf '%s\n' "$PREAUTH_BUILD_DIR" > "$contract_scratch/failure-leaf"
    preauth_build_install_traps
    exit 91
) >/dev/null 2>"$contract_scratch/failure.stderr" || :
failure_leaf=$(cat "$contract_scratch/failure-leaf")
[ ! -e "$failure_leaf" ] || fail 'compiler/error cleanup leaf remains'

assert_signal_result(){
    signal_label=$1
    signal_expected=$2
    signal_status=$3
    signal_leaf_file=$4
    signal_child_file=${5-}
    [ "$signal_status" -eq "$signal_expected" ] || fail "$signal_label status"
    [ ! -s "$contract_scratch/$signal_label.stdout" ] || fail "$signal_label wrote stdout"
    signal_leaf=$(cat "$signal_leaf_file")
    [ ! -e "$signal_leaf" ] || fail "$signal_label cleanup leaf remains"
    if [ -n "$signal_child_file" ]; then
        signal_child=$(cat "$signal_child_file")
        if kill -0 "$signal_child" 2>/dev/null; then
            fail "$signal_label child remains"
        fi
    fi
    repository_state "$contract_scratch/$signal_label.after"
    cmp -s "$contract_scratch/$signal_label.before" "$contract_scratch/$signal_label.after" \
        || fail "$signal_label changed the repository tree"
}

run_real_signal_case(){
    signal_name=$1
    signal_number=$2
    child_behavior=$3
    repeat_signal=${4-}
    label=signal-$signal_name-$child_behavior${repeat_signal:+-$repeat_signal}
    ready_file=$contract_scratch/$label.ready
    pid_file=$contract_scratch/$label.pid
    leaf_file=$contract_scratch/$label.leaf
    child_file=$contract_scratch/$label.child
    repository_state "$contract_scratch/$label.before"
    (
        while [ ! -s "$ready_file" ]; do :; done
        signal_target=$(cat "$pid_file")
        kill -"$signal_name" "$signal_target"
        if [ "$repeat_signal" = repeated ]; then
            sleep 1
            kill -TERM "$signal_target" 2>/dev/null || :
        fi
    ) &
    signaler_pid=$!
    set +e
    sh -c '
        set -eu
        repository=$1; selected_root=$2; label=$3; behavior=$4; pid_file=$5; child_file=$6; leaf_file=$7; ready_file=$8
        . "$repository/tools/toolchain/preauth-build-root.sh"
        RAR_PREAUTH_BUILD_ROOT=$selected_root
        preauth_build_root_create "$repository" "cleanup-$label" "cleanup-$label"
        preauth_build_install_traps
        printf "%s\n" "$$" > "$pid_file"
        printf "%s\n" "$PREAUTH_BUILD_DIR" > "$leaf_file"
        preauth_build_run_child sh -c '\''
            behavior=$1; child_file=$2; ready_file=$3
            case "$behavior" in
                cooperative) trap - HUP TERM ;;
                ignore-original) trap "" HUP; trap - TERM ;;
                ignore-original-term) trap "" HUP TERM ;;
                *) exit 97 ;;
            esac
            printf "%s\n" "$$" > "$child_file"
            printf "%s\n" ready > "$ready_file"
            while :; do :; done
        '\'' preauth-signal-child "$behavior" "$child_file" "$ready_file"
        exit "$?"
    ' preauth-signal-wrapper "$root" "$cleanup_root" "$label" "$child_behavior" "$pid_file" "$child_file" "$leaf_file" "$ready_file" \
        >"$contract_scratch/$label.stdout" 2>"$contract_scratch/$label.stderr"
    signal_status=$?
    set -e
    wait "$signaler_pid"
    assert_signal_result "$label" "$((128 + signal_number))" "$signal_status" "$leaf_file" "$child_file"
}

run_before_child_signal_case(){
    label=signal-HUP-before-child
    ready_file=$contract_scratch/$label.ready
    pid_file=$contract_scratch/$label.pid
    leaf_file=$contract_scratch/$label.leaf
    repository_state "$contract_scratch/$label.before"
    (
        while [ ! -s "$ready_file" ]; do :; done
        kill -HUP "$(cat "$pid_file")"
    ) &
    signaler_pid=$!
    set +e
    sh -c '
        set -eu
        repository=$1; selected_root=$2; label=$3; pid_file=$4; leaf_file=$5; ready_file=$6
        . "$repository/tools/toolchain/preauth-build-root.sh"
        RAR_PREAUTH_BUILD_ROOT=$selected_root
        preauth_build_root_create "$repository" "cleanup-$label" "cleanup-$label"
        preauth_build_install_traps
        printf "%s\n" "$$" > "$pid_file"
        printf "%s\n" "$PREAUTH_BUILD_DIR" > "$leaf_file"
        printf "%s\n" ready > "$ready_file"
        while :; do :; done
    ' preauth-before-child "$root" "$cleanup_root" "$label" "$pid_file" "$leaf_file" "$ready_file" \
        >"$contract_scratch/$label.stdout" 2>"$contract_scratch/$label.stderr"
    signal_status=$?
    set -e
    wait "$signaler_pid"
    assert_signal_result "$label" 129 "$signal_status" "$leaf_file"
}

run_direct_handler_case(){
    signal_name=$1
    signal_number=$2
    child_behavior=$3
    label=handler-$signal_name-$child_behavior
    leaf_file=$contract_scratch/$label.leaf
    child_file=$contract_scratch/$label.child
    ready_file=$contract_scratch/$label.ready
    repository_state "$contract_scratch/$label.before"
    set +e
    sh -c '
        set -eu
        repository=$1; selected_root=$2; label=$3; signal_number=$4; behavior=$5; child_file=$6; leaf_file=$7; ready_file=$8
        . "$repository/tools/toolchain/preauth-build-root.sh"
        RAR_PREAUTH_BUILD_ROOT=$selected_root
        preauth_build_root_create "$repository" "cleanup-$label" "cleanup-$label"
        preauth_build_install_traps
        printf "%s\n" "$PREAUTH_BUILD_DIR" > "$leaf_file"
        sh -c '\''
            behavior=$1; child_file=$2; ready_file=$3
            case "$behavior" in
                ignore-int) trap "" INT; trap - TERM ;;
                already-exited) : ;;
                *) exit 97 ;;
            esac
            printf "%s\n" "$$" > "$child_file"
            printf "%s\n" ready > "$ready_file"
            [ "$behavior" = already-exited ] && exit 0
            while :; do :; done
        '\'' preauth-handler-child "$behavior" "$child_file" "$ready_file" &
        preauth_build_child_pid=$!
        while [ ! -s "$ready_file" ]; do :; done
        [ "$behavior" != already-exited ] || sleep 1
        preauth_build_signal "$signal_number"
        exit 99
    ' preauth-signal-handler "$root" "$cleanup_root" "$label" "$signal_number" "$child_behavior" "$child_file" "$leaf_file" "$ready_file" \
        >"$contract_scratch/$label.stdout" 2>"$contract_scratch/$label.stderr"
    handler_status=$?
    set -e
    assert_signal_result "$label" "$((128 + signal_number))" "$handler_status" "$leaf_file" "$child_file"
}

run_real_signal_case TERM 15 cooperative
run_real_signal_case HUP 1 cooperative
run_real_signal_case HUP 1 ignore-original
run_real_signal_case HUP 1 ignore-original-term
run_real_signal_case HUP 1 ignore-original-term repeated
run_before_child_signal_case
# Non-interactive parents may start with INT ignored, which POSIX shells cannot
# make catchable. Exercise the shared handler boundary explicitly and prove that
# a child ignoring INT is still terminated by the bounded TERM escalation.
run_direct_handler_case INT 2 ignore-int
run_direct_handler_case HUP 1 already-exited

rm -rf "$contract_build_parent" "$contract_scratch"

printf '%s\n' 'transaction contract checks passed'
