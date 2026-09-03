#!/bin/sh
set -eu
LC_ALL=C
LANG=C
PATH=/usr/bin:/bin
export LC_ALL LANG PATH

[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] &&
    [ "${RAR_CI_RUNNER_OS-}" = Linux ] && [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ] || {
    printf '%s\n' 'C3VA evidence policy test is GitHub-hosted Linux CI only' >&2
    exit 1
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
validator=$root/tools/ci/verify-controller-helper-closure-verifier-evidence.sh
cases=$root/spec/alpha/lab/controller-helper-closure-verifier-cases-v0
policy_cases=$root/tools/ci/fixtures/controller-helper-closure-verifier/evidence-cases.v0
valid_seed=$root/tools/ci/fixtures/controller-helper-closure-verifier/evidence-valid.v0
malformed_seed=$root/tools/ci/fixtures/controller-helper-closure-verifier/evidence-malformed.v0
fail() { printf 'C3VA evidence policy test failed: %s\n' "$1" >&2; exit 1; }
sha_file() { /usr/bin/sha256sum -- "$1" | /usr/bin/awk '{ print $1 }'; }
size_file() { /usr/bin/stat -c %s -- "$1"; }

[ -f "$validator" ] && [ ! -L "$validator" ] || fail 'validator unavailable'
[ -f "$cases" ] && [ ! -L "$cases" ] || fail 'case catalog unavailable'
[ -f "$policy_cases" ] && [ ! -L "$policy_cases" ] || fail 'policy case catalog unavailable'
[ "$(grep -Ec '^case\|EP[0-9][0-9][0-9]\|' "$policy_cases")" -eq 20 ] ||
    fail 'policy case count is not 20'
grep -Fqx 'chunk=C|B000001|C000001|YQ==' "$valid_seed" ||
    fail 'canonical Base64 seed changed'
grep -Fqx 'chunk=C|B000001|C000001|YQ' "$malformed_seed" ||
    fail 'malformed Base64 seed changed'

scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'C3VA evidence policy test skipped: ephemeral scratch disabled'; exit 0; }
work=$(mktemp -d "$scratch/rar-c3va-evidence.XXXXXX")
case "$work" in "$scratch"/rar-c3va-evidence.*) ;; *) fail 'work root escaped ephemeral scratch' ;; esac
chmod 700 "$work"
cleanup() {
    rc=$?
    trap - EXIT HUP INT TERM
    /bin/rm -rf -- "$work"
    exit "$rc"
}
trap cleanup EXIT HUP INT TERM

empty_sha=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
revision=1111111111111111111111111111111111111111
digest=2222222222222222222222222222222222222222222222222222222222222222
nonce=3333333333333333333333333333333333333333333333333333333333333333
cases_sha=$(sha_file "$cases")
export RAR_EXPECTED_REPOSITORY=AndyTechCoder/RAR-OS
export RAR_EXPECTED_CONTROLLER_SHA=$revision
export RAR_EXPECTED_SOURCE_SHA=$revision
export RAR_EXPECTED_RUN_ID=9001
export RAR_EXPECTED_RUN_ATTEMPT=1
export RAR_EXPECTED_VERIFICATION_RECEIPT_SHA256=$empty_sha
export RAR_EXPECTED_SUBJECT_SHA256=$digest
export RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256=$digest
export RAR_EXPECTED_VALIDATION_SHA256=$digest
export RAR_EXPECTED_DISPOSITIONS_SHA256=$digest
export RAR_EXPECTED_TEMPLATES_SHA256=$digest
export RAR_EXPECTED_PRECEDENCE_SHA256=$digest
export RAR_EXPECTED_FAULTS_SHA256=$digest
export RAR_EXPECTED_CASES_SHA256=$cases_sha
export RAR_EXPECTED_BASE_FIXTURE_SHA256=$digest
export RAR_EXPECTED_FIXTURE_IMAGE_SHA256=$digest
export RAR_EXPECTED_TOOL_PINS_SHA256=$digest
export RAR_EXPECTED_ARTIFACT_NONCE=$nonce

plan=$work/plan
ledger=$work/ledger
body=$work/body
envelope=$work/envelope
piece=$work/piece
preimage=$work/preimage
valid=$work/valid.v0
field=$work/field.raw
receipt=$work/verification.receipt
case_row_file=$work/case.row
semantic=$work/semantic
: > "$ledger"
: > "$body"

printf '%s\n' \
    'schema=rar-alpha-controller-helper-closure-verifier-tools-v0' \
    "find_sha256=$digest" "sort_sha256=$digest" "wc_sha256=$digest" \
    "stat_sha256=$digest" "cmp_sha256=$digest" "id_sha256=$digest" \
    'status=reviewed-for-candidate-verification-only' > "$semantic"
tool_inventory_sha=$(sha_file "$semantic")
printf '%s\n' 'reviewed-observer-source-bytes-v0' > "$semantic"
observer_sha=$(sha_file "$semantic")
printf '%s  %s\n%s  %s\n' "$digest" a "$digest" b > "$semantic"
manifest_sha=$(sha_file "$semantic")
manifest_bytes=$(size_file "$semantic")
manifest_entries=2
write_candidate_receipt() {
    candidate_target=$1
    printf '%s\n' \
        'schema=rar-alpha-controller-helper-closure-observation-v0' \
        'status=observed-not-reviewed-not-ready' \
        "controller_sha=$revision" "source_sha=$revision" \
        'repository=AndyTechCoder/RAR-OS' 'ref=refs/heads/main' 'event=push' \
        'run_id=9001' 'run_attempt=1' 'runner_os=ubuntu24' \
        'runner_image_version=24.04.1' \
        'oci_image=sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
        'closure_root=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu' \
        "generator_sha256=$observer_sha" "find_sha256=$digest" "sort_sha256=$digest" \
        "manifest_sha256=$manifest_sha" "manifest_entries=$manifest_entries" "manifest_bytes=$manifest_bytes" \
        'helper_compiled=false' 'helper_executed=false' 'target_compiled=false' 'readiness=false' > "$candidate_target"
}
write_candidate_receipt "$semantic"
fixture_inventory_sha=$(sha_file "$semantic")
printf '%s\n' 'a|f|1|2|1|1|644|0|0' > "$semantic"
topology_sha=$(sha_file "$semantic")
second_pass_sha=$manifest_sha
export RAR_EXPECTED_TOOL_PINS_SHA256=$tool_inventory_sha
export RAR_EXPECTED_OBSERVER_SHA256=$observer_sha

{
    printf '%s\n' \
        'schema=rar-alpha-controller-helper-closure-verification-v0' \
        'status=candidate-exact-set-verified-not-reviewed-not-ready' \
        "controller_sha=$revision" \
        "source_sha=$revision" \
        'repository=AndyTechCoder/RAR-OS' \
        'run_id=9001' \
        'run_attempt=1' \
        'runner_os=ubuntu24' \
        'runner_image_version=24.04.1' \
        'oci_image=sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
        'closure_root=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu' \
        "verifier_sha256=$digest" \
        "observer_sha256=$observer_sha" \
        "tool_pins_sha256=$tool_inventory_sha" \
        "find_sha256=$digest" \
        "sort_sha256=$digest" \
        "wc_sha256=$digest" \
        "stat_sha256=$digest" \
        "cmp_sha256=$digest" \
        "id_sha256=$digest" \
        "candidate_receipt_sha256=$fixture_inventory_sha" \
        "candidate_manifest_sha256=$manifest_sha" \
        "recomputed_manifest_sha256=$manifest_sha" \
        'manifest_entries=2' \
        "manifest_bytes=$manifest_bytes" \
        "topology_sha256=$topology_sha" \
        "second_pass_sha256=$second_pass_sha" \
        'helper_compiled=false' \
        'helper_executed=false' \
        'target_compiled=false' \
        'readiness=false'
} > "$receipt"
receipt_sha=$(sha_file "$receipt")
export RAR_EXPECTED_VERIFICATION_RECEIPT_SHA256=$receipt_sha

{
    printf '%s\n' \
        'B000001|clean-success-pass-1|RUN' \
        'B000002|clean-success-pass-2|RUN'
    /usr/bin/awk -F '|' '
    BEGIN { blob=2 }
    /^case\|[VQX][0-9][0-9][0-9]\|/ {
        if ($3 ~ /-residual$/) {
            blob++
            printf "B%06d|residual-source-proof|%s\n",blob,$2
        } else {
            names[1]="runtime-pre-input-topology"
            names[2]="runtime-stdout"
            names[3]="runtime-stderr"
            names[4]="runtime-post-output-event-resource"
            for (i=1;i<=4;i++) {
                blob++
                printf "B%06d|%s|%s\n",blob,names[i],$2
            }
        }
    }
    END { if (blob!=709) exit 1 }
    ' "$cases"
} > "$plan" || fail 'cannot derive blob plan'

fields_for() {
    case "$1" in
        clean-success-pass-1|clean-success-pass-2)
            printf '%s\n' 'domain-header tool-inventory fixture-inventory topology canonical-manifest mount-identities output-inventory output-bytes event-bytes resource-bytes verification-receipt-inputs' ;;
        runtime-pre-input-topology)
            printf '%s\n' 'domain-header input-bytes pre-mount-inventory pre-topology mount-identities mutation-schedule' ;;
        runtime-stdout) printf '%s\n' 'stdout-bytes' ;;
        runtime-stderr) printf '%s\n' 'stderr-bytes' ;;
        runtime-post-output-event-resource)
            printf '%s\n' 'post-mount-inventory post-topology output-inventory output-bytes mutation-trigger mutation-acknowledgement observed-event resource-usage timeout-termination' ;;
        residual-source-proof) printf '%s\n' 'residual-source residual-proof' ;;
        *) return 1 ;;
    esac
}

observation_for() {
    case "$1" in
        domain-header) printf '%s\n' trusted-run-domain ;;
        tool-inventory|fixture-inventory|output-inventory|pre-mount-inventory|post-mount-inventory) printf '%s\n' complete-inventory ;;
        topology|pre-topology|post-topology) printf '%s\n' complete-topology ;;
        canonical-manifest) printf '%s\n' canonical-manifest ;;
        mount-identities) printf '%s\n' stable-nonaliased-identities ;;
        event-bytes) printf '%s\n' clean-event-capture ;;
        resource-bytes|resource-usage) printf '%s\n' within-controller-bounds ;;
        mutation-schedule) printf '%s\n' exact-catalog-binding-scheduled ;;
        mutation-trigger) printf '%s\n' exact-trigger-observed ;;
        mutation-acknowledgement) printf '%s\n' exact-trigger-acknowledged ;;
        observed-event|timeout-termination) printf '%s\n' catalog-oracle-byte-equal ;;
        residual-source) printf '%s\n' catalog-source-byte-equal ;;
        residual-proof) printf '%s\n' catalog-oracle-byte-equal ;;
        *) return 1 ;;
    esac
}
write_observed_result() {
    receipt_state=
    oldifs=$IFS
    IFS='|' read -r row_marker row_case row_kind row_source row_left row_right row_binding row_oracle <<EOF
$(/bin/cat "$case_row_file")
EOF
    IFS=$oldifs
    case "$row_kind" in
        disposition) primary=${row_oracle%%@*}; termination=exit-1; controller_exit=1 ;;
        precedence) primary=${row_oracle#first-error-}; termination=exit-1; controller_exit=1; receipt_state=not-specified-by-precedence-oracle ;;
        fault)
            primary=$row_source
            case "$row_oracle" in signal-25+controller-exit-map-153+*) termination=signal-25; controller_exit=153 ;; *) termination=exit-1; controller_exit=1 ;; esac
            ;;
        *) fail "cannot generate observed result for $row_kind" ;;
    esac
    if [ -z "${receipt_state-}" ]; then
        case "$row_oracle" in *no-valid-final-receipt*) receipt_state=no-valid-final-receipt ;; *no-receipt*) receipt_state=no-receipt ;; *) fail "receipt state unavailable for $row_case" ;; esac
    fi
    printf '%s\n' \
        'schema=rar-c3v-observed-result-v0' \
        "case_id=$row_case" \
        "catalog_kind=$row_kind" \
        "primary=$primary" \
        "termination=$termination" \
        "controller_exit=$controller_exit" \
        "receipt_state=$receipt_state" > "$payload"
}

materialize_field() {
    name=$1
    kind=$2
    case_id=$3
    case "$name" in
        verification-receipt-inputs) /bin/cp "$receipt" "$field"; return ;;
        input-bytes)
            /usr/bin/awk -F '|' -v c="$case_id" '$1=="case" && $2==c { print; found++ } END { if(found!=1) exit 1 }' "$cases" > "$field" || fail "cannot bind input $case_id"
            return ;;
        stdout-bytes)
            if [ "$case_id" = V001 ]; then
                /usr/bin/awk 'BEGIN { for(i=0;i<1800;i++) printf "A" }' > "$field"
                printf '\000\n\377' >> "$field"
            else
                printf 'stdout|%s\n' "$case_id" > "$field"
            fi
            return ;;
        stderr-bytes) printf 'stderr|%s\n' "$case_id" > "$field"; return ;;
        output-bytes) case "$kind" in clean-success-pass-1|clean-success-pass-2) printf 'output|clean-success-pass|RUN\n' ;; *) printf 'output|%s|%s\n' "$kind" "$case_id" ;; esac > "$field"; return ;;
    esac
    observation=$(observation_for "$name") || fail "no semantic observation for $name"
    semantic_kind=$kind
    case "$semantic_kind" in clean-success-pass-1|clean-success-pass-2) semantic_kind=clean-success-pass ;; esac
    if [ "$case_id" = RUN ]; then
        row_sha=$cases_sha
        oracle_sha=$digest
    else
        /usr/bin/awk -F '|' -v c="$case_id" '$1=="case" && $2==c { print; found++ } END { if(found!=1) exit 1 }' "$cases" > "$case_row_file" || fail "case row missing $case_id"
        row_sha=$(sha_file "$case_row_file")
        oracle=$(/usr/bin/awk -F '|' '{ print $8 }' "$case_row_file")
        printf '%s' "$oracle" > "$semantic"
        oracle_sha=$(sha_file "$semantic")
    fi
    payload=$work/payload.raw
    case "$name" in
        domain-header) printf '%s\n' 'reviewed-observer-source-bytes-v0' > "$payload" ;;
        tool-inventory)
            printf '%s\n' \
                'schema=rar-alpha-controller-helper-closure-verifier-tools-v0' \
                "find_sha256=$digest" "sort_sha256=$digest" "wc_sha256=$digest" \
                "stat_sha256=$digest" "cmp_sha256=$digest" "id_sha256=$digest" \
                'status=reviewed-for-candidate-verification-only' > "$payload"
            ;;
        fixture-inventory) write_candidate_receipt "$payload" ;;
        canonical-manifest) printf '%s  %s\n%s  %s\n' "$digest" a "$digest" b > "$payload" ;;
        topology) printf '%s\n' 'a|f|1|2|1|1|644|0|0' > "$payload" ;;
        observed-event) write_observed_result ;;
        timeout-termination|residual-proof) printf '%s' "$oracle" > "$payload" ;;
        mutation-schedule|mutation-trigger|mutation-acknowledgement|residual-source)
            /bin/cp "$case_row_file" "$payload"
            ;;
        *)
            printf 'captured|%s|%s|%s\n' "$semantic_kind" "$case_id" "$name" > "$payload"
            ;;
    esac
    payload_bytes=$(size_file "$payload")
    payload_sha=$(sha_file "$payload")
    {
        printf '%s\n' \
            'schema=rar-c3v-semantic-field-v0' \
            "kind=$semantic_kind" \
            "case_id=$case_id" \
            "field=$name" \
            "catalog_row_sha256=$row_sha" \
            "oracle_sha256=$oracle_sha" \
            "observation=$observation" \
            "payload_bytes=$payload_bytes" \
            "payload_sha256=$payload_sha" \
            'payload'
        /bin/cat "$payload"
    } > "$field"
}

chunk_total=0
decoded_total=0
while IFS='|' read -r blob kind case_id; do
    fields=$(fields_for "$kind") || fail "unknown planned kind $kind"
    set -- $fields
    : > "$envelope"
    printf '%s\n' 'rar-c3v-envelope-v0' "kind=$kind" "case_id=$case_id" "field_count=$#" >> "$envelope"
    ordinal=0
    for name do
        ordinal=$((ordinal + 1))
        nn=$(/usr/bin/printf '%02d' "$ordinal")
        materialize_field "$name" "$kind" "$case_id"
        field_bytes=$(size_file "$field")
        field_sha=$(sha_file "$field")
        printf '%s\n' "field.$nn.name=$name" "field.$nn.bytes=$field_bytes" "field.$nn.sha256=$field_sha" "field.$nn.data" >> "$envelope"
        /bin/cat "$field" >> "$envelope"
        printf '\n' >> "$envelope"
    done
    decoded=$(size_file "$envelope")
    sha=$(sha_file "$envelope")
    chunks=$(((decoded + 1435) / 1436))
    printf 'B|%s|%s|%s|%s|%s|%s\n' \
        "$blob" "$kind" "$case_id" "$decoded" "$chunks" "$sha" >> "$body"
    chunk_index=1
    offset=0
    while [ "$chunk_index" -le "$chunks" ]; do
        /bin/dd if="$envelope" of="$piece" bs=1 skip="$offset" count=1436 status=none
        payload=$(/usr/bin/base64 -w 0 "$piece")
        cid=$(/usr/bin/printf 'C%06d' "$chunk_index")
        printf 'C|%s|%s|%s\n' "$blob" "$cid" "$payload" >> "$body"
        offset=$((offset + $(size_file "$piece")))
        chunk_index=$((chunk_index + 1))
    done
    printf '%s|%s|%s|%s|%s\n' "$blob" "$kind" "$case_id" "$decoded" "$sha" >> "$ledger"
    chunk_total=$((chunk_total + chunks))
    decoded_total=$((decoded_total + decoded))
done < "$plan"

logical=0
while IFS='|' read -r marker case_id catalog_kind rest; do
    [ "$marker" = case ] || continue
    case "$case_id" in V[0-9][0-9][0-9]|Q[0-9][0-9][0-9]|X[0-9][0-9][0-9]) ;; *) continue ;; esac
    logical=$((logical + 1))
    raw_ids=$(
        /usr/bin/awk -F '|' -v c="$case_id" '
            $3==c {
                if (out!="") out=out","
                out=out $1
            }
            END { print out }
        ' "$ledger"
    )
    [ -n "$raw_ids" ] || fail "no blobs for $case_id"
    {
        printf '%s\n' \
            'rar-c3v-normalized-v0' \
            'repository=AndyTechCoder/RAR-OS' \
            "controller_sha=$revision" \
            "source_sha=$revision" \
            'run_id=9001' \
            'run_attempt=1' \
            "verification_receipt_sha256=$receipt_sha" \
            "case_id=$case_id" \
            'result=pass' \
            "raw_blob_ids=$raw_ids"
        /usr/bin/awk -F '|' -v c="$case_id" '
            $3==c {
                printf "blob=%s:%s:%s:%s\n",$1,$2,$4,$5
            }
        ' "$ledger"
    } > "$preimage"
    normalized_sha=$(sha_file "$preimage")
    printf 'N|%03d|%s|%s|pass|%s|%s\n' \
        "$logical" "$case_id" "$catalog_kind" "$raw_ids" "$normalized_sha" >> "$body"
done < "$cases"
[ "$logical" -eq 209 ] || fail 'normalized generation count mismatch'
{
    printf 'H|rar-alpha-controller-helper-closure-verifier-evidence-v0|AndyTechCoder/RAR-OS|%s|%s|9001|1|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|166|43|209|709|%s|%s|209|0|mechanically-verified-not-reviewed-not-ready\n' \
        "$revision" "$revision" "$receipt_sha" \
        "$digest" "$digest" "$digest" "$digest" "$digest" "$digest" "$digest" "$cases_sha" \
        "$digest" "$digest" "$RAR_EXPECTED_TOOL_PINS_SHA256" "$nonce" "$chunk_total" "$decoded_total"
    /bin/cat "$body"
} > "$valid"

validator_scratch=$work/validator
/bin/mkdir "$validator_scratch"
/bin/sh "$validator" "$valid" "$cases" "$validator_scratch" >/dev/null ||
    fail 'complete canonical evidence was rejected'

rewrite_first_blob() {
    mode=$1
    input=$2
    output=$3
    original=$work/rewrite.original
    changed=$work/rewrite.changed
    block=$work/rewrite.block
    : > "$original"
    /usr/bin/awk -F '|' '
        $1=="B" && $2=="B000001" { inside=1; next }
        inside && $1=="B" { exit }
        inside && $1=="C" { print $4 }
    ' "$input" |
    while IFS= read -r payload; do
        printf '%s' "$payload" | /usr/bin/base64 --decode >> "$original"
    done
    [ "$(size_file "$original")" -gt 1436 ] || fail 'multi-chunk positive vector is not multi-chunk'
    case "$mode" in
        inner-length)
            /usr/bin/sed '0,/field.01.bytes=[0-9][0-9][0-9]/s//field.01.bytes=999/' "$original" > "$changed"
            ;;
        inner-hash)
            /usr/bin/sed '0,/field.01.sha256=[0-9a-f]*/s//field.01.sha256=2222222222222222222222222222222222222222222222222222222222222222/' "$original" > "$changed"
            ;;
        *) fail "unknown rewrite mode $mode" ;;
    esac
    [ "$(size_file "$changed")" -eq "$(size_file "$original")" ] || fail "$mode rewrite changed outer length"
    changed_bytes=$(size_file "$changed")
    changed_sha=$(sha_file "$changed")
    changed_chunks=$(((changed_bytes + 1435) / 1436))
    printf 'B|B000001|clean-success-pass-1|RUN|%s|%s|%s\n' "$changed_bytes" "$changed_chunks" "$changed_sha" > "$block"
    rewrite_index=1
    rewrite_offset=0
    while [ "$rewrite_index" -le "$changed_chunks" ]; do
        /bin/dd if="$changed" of="$piece" bs=1 skip="$rewrite_offset" count=1436 status=none
        payload=$(/usr/bin/base64 -w 0 "$piece")
        cid=$(/usr/bin/printf 'C%06d' "$rewrite_index")
        printf 'C|B000001|%s|%s\n' "$cid" "$payload" >> "$block"
        rewrite_offset=$((rewrite_offset + $(size_file "$piece")))
        rewrite_index=$((rewrite_index + 1))
    done
    /usr/bin/sed -n '1p' "$input" > "$output"
    /bin/cat "$block" >> "$output"
    /usr/bin/awk -F '|' '$1=="B" && $2=="B000002" { keep=1 } keep { print }' "$input" >> "$output"
}

mutate() {
    id=$1
    input=$2
    output=$3
    case "$id" in
        EP001) : > "$output" ;;
        EP002)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="B" && $2=="B000003"{$4="V002";done=1}{print}' "$input" > "$output" ;;
        EP003)
            /usr/bin/sed "1s/$nonce/4444444444444444444444444444444444444444444444444444444444444444/" "$input" > "$output" ;;
        EP004) /usr/bin/sed '$d' "$input" > "$output" ;;
        EP005) { /bin/cat "$input"; printf '%s\n' 'X|extension'; } > "$output" ;;
        EP006)
            /usr/bin/sed '0,/B000003/s//B000004/' "$input" > "$output" ;;
        EP007)
            { /bin/cat "$input"; /usr/bin/tail -n 1 "$input"; } > "$output" ;;
        EP008)
            /usr/bin/sed '0,/runtime-pre-input-topology/s//runtime-stdout/' "$input" > "$output" ;;
        EP009) rewrite_first_blob inner-length "$input" "$output" ;;
        EP010) rewrite_first_blob inner-hash "$input" "$output" ;;
        EP011)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="C"{sub(/=$/,"",$4);done=1}{print}' "$input" > "$output" ;;
        EP012)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} NR==1{$24=708}{print}' "$input" > "$output" ;;
        EP013)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} NR==1{$25=$25-1}{print}' "$input" > "$output" ;;
        EP014)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} NR==1{$27=208}{print}' "$input" > "$output" ;;
        EP015)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} NR==1{$26=$26+1}{print}' "$input" > "$output" ;;
        EP016)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="B"{$5=16777217;done=1}{print}' "$input" > "$output" ;;
        EP017)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|";pad=""} !done && $1=="C"{for(i=0;i<2050;i++)pad=pad"A";$4=pad;done=1}{print}' "$input" > "$output" ;;
        EP018)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="N"{$6="B000001";done=1}{print}' "$input" > "$output" ;;
        EP019)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="N"{$7="2222222222222222222222222222222222222222222222222222222222222222";done=1}{print}' "$input" > "$output" ;;
        EP020)
            second=$(/usr/bin/awk -F '|' '$1=="N" && $3=="V002"{print $6;exit}' "$input")
            /usr/bin/awk -F '|' -v ids="$second" 'BEGIN{OFS="|"} !done && $1=="N"{$6=ids;done=1}{print}' "$input" > "$output" ;;
        *) fail "unknown policy case $id" ;;
    esac
}

rewrite_clean_semantic_blob() {
    target=$1
    target_kind=$2
    mode=$3
    input=$4
    output=$5
    original=$work/semantic.original
    changed=$work/semantic.changed
    replacement=$work/semantic.replacement
    block=$work/semantic.block
    semantic_payload_file=$work/semantic.payload
    : > "$original"
    /usr/bin/awk -F '|' -v target="$target" '
        $1=="B" && $2==target { inside=1; next }
        inside && $1=="B" { exit }
        inside && $1=="C" { print $4 }
    ' "$input" | while IFS= read -r encoded_part; do
        printf '%s' "$encoded_part" | /usr/bin/base64 --decode >> "$original"
    done
    case "$mode" in
        observer) nn=01; field_name=domain-header; observation=trusted-run-domain; printf '%s\n' reviewed-observer-source-bytes-v1 > "$semantic_payload_file" ;;
        tool-pins) nn=02; field_name=tool-inventory; observation=complete-inventory; printf '%s\n' 'schema=rar-alpha-controller-helper-closure-verifier-tools-v0' "find_sha256=4444444444444444444444444444444444444444444444444444444444444444" "sort_sha256=$digest" "wc_sha256=$digest" "stat_sha256=$digest" "cmp_sha256=$digest" "id_sha256=$digest" 'status=reviewed-for-candidate-verification-only' > "$semantic_payload_file" ;;
        candidate-receipt)
            nn=03; field_name=fixture-inventory; observation=complete-inventory
            write_candidate_receipt "$semantic"
            /usr/bin/sed "s/^generator_sha256=.*/generator_sha256=4444444444444444444444444444444444444444444444444444444444444444/" "$semantic" > "$semantic_payload_file"
            ;;
        topology) nn=04; field_name=topology; observation=complete-topology; printf '%s\n' 'b|f|1|2|1|1|644|0|0' > "$semantic_payload_file" ;;
        manifest) nn=05; field_name=canonical-manifest; observation=canonical-manifest; printf '%s  %s\n%s  %s\n' "4444444444444444444444444444444444444444444444444444444444444444" a "$digest" b > "$semantic_payload_file" ;;
        manifest-order) nn=05; field_name=canonical-manifest; observation=canonical-manifest; printf '%s  %s\n%s  %s\n' "$digest" b "$digest" a > "$semantic_payload_file" ;;
        *) fail "unknown semantic rewrite $mode" ;;
    esac
    payload_bytes=$(size_file "$semantic_payload_file")
    payload_sha=$(sha_file "$semantic_payload_file")
    {
        printf '%s\n' 'schema=rar-c3v-semantic-field-v0' 'kind=clean-success-pass' 'case_id=RUN' "field=$field_name" "catalog_row_sha256=$cases_sha" "oracle_sha256=$digest" "observation=$observation" "payload_bytes=$payload_bytes" "payload_sha256=$payload_sha" 'payload'
        /bin/cat "$semantic_payload_file"
    } > "$replacement"
    replacement_sha=$(sha_file "$replacement")
    /usr/bin/awk -v nn="$nn" -v replacement="$replacement" -v replacement_sha="$replacement_sha" '
        $0 ~ ("^field\\." nn "\\.sha256=") { print "field." nn ".sha256=" replacement_sha; next }
        $0=="field." nn ".data" {
            print
            while ((getline line < replacement) > 0) print line
            close(replacement)
            skip=1
            next
        }
        skip && $0 ~ "^field\\.[0-9][0-9]\\.name=" { skip=0; print ""; print; next }
        skip { next }
        { print }
    ' "$original" > "$changed"
    changed_bytes=$(size_file "$changed")
    changed_sha=$(sha_file "$changed")
    changed_chunks=$(((changed_bytes + 1435) / 1436))
    printf 'B|%s|%s|RUN|%s|%s|%s\n' "$target" "$target_kind" "$changed_bytes" "$changed_chunks" "$changed_sha" > "$block"
    rewrite_index=1; rewrite_offset=0
    while [ "$rewrite_index" -le "$changed_chunks" ]; do
        /bin/dd if="$changed" of="$piece" bs=1 skip="$rewrite_offset" count=1436 status=none
        encoded_part=$(/usr/bin/base64 -w 0 "$piece")
        cid=$(/usr/bin/printf 'C%06d' "$rewrite_index")
        printf 'C|%s|%s|%s\n' "$target" "$cid" "$encoded_part" >> "$block"
        rewrite_offset=$((rewrite_offset + $(size_file "$piece")))
        rewrite_index=$((rewrite_index + 1))
    done
    /usr/bin/awk -F '|' -v target="$target" '$1=="B" && $2==target { exit } { print }' "$input" > "$output"
    /bin/cat "$block" >> "$output"
    /usr/bin/awk -F '|' -v target="$target" 'seen && $1=="B" { keep=1 } $1=="B" && $2==target { seen=1; next } keep { print }' "$input" >> "$output"
}

executed=0
while IFS='|' read -r marker id class expected; do
    [ "$marker" = case ] || continue
    [ "$expected" = reject ] || fail "policy case $id is not reject"
    mutated=$work/$id.v0
    mutate "$id" "$valid" "$mutated"
    /bin/rm -rf -- "$validator_scratch"
    /bin/mkdir "$validator_scratch"
    if /bin/sh "$validator" "$mutated" "$cases" "$validator_scratch" >/dev/null 2>&1; then
        fail "$id $class mutation was accepted"
    fi
    executed=$((executed + 1))
done < "$policy_cases"
[ "$executed" -eq 20 ] || fail 'not all evidence policy mutations executed'
near_max_line=$work/near-max-single-line.v0
near_max_error=$work/near-max-single-line.err
/bin/dd if=/dev/zero bs=1048576 count=420 status=none | /usr/bin/tr '\000' A > "$near_max_line"
[ "$(size_file "$near_max_line")" -eq 440401920 ] || fail 'near-maximum line fixture size mismatch'
/bin/rm -rf -- "$validator_scratch"
/bin/mkdir "$validator_scratch"
if /bin/sh "$validator" "$near_max_line" "$cases" "$validator_scratch" >/dev/null 2>"$near_max_error"; then
    fail 'maximum-size no-newline evidence was accepted'
fi
/usr/bin/grep -Fq 'physical line bound exceeded' "$near_max_error" ||
    fail 'maximum-size no-newline evidence did not reach physical-line rejection'
/bin/rm -f -- "$near_max_line" "$near_max_error"
{
    /bin/dd if=/dev/zero bs=1048576 count=419 status=none
    /bin/dd if=/dev/zero bs=1048575 count=1 status=none
} | /usr/bin/tr '\000' '\010' > "$near_max_line"
printf '\n' >> "$near_max_line"
[ "$(size_file "$near_max_line")" -eq 440401920 ] || fail 'maximum-size control-line fixture size mismatch'
/bin/rm -rf -- "$validator_scratch"
/bin/mkdir "$validator_scratch"
if /bin/sh "$validator" "$near_max_line" "$cases" "$validator_scratch" >/dev/null 2>"$near_max_error"; then
    fail 'maximum-size control-byte evidence was accepted'
fi
/usr/bin/grep -Fq 'physical line bound exceeded' "$near_max_error" ||
    fail 'maximum-size control-byte evidence did not reach pre-read line rejection'
/bin/rm -f -- "$near_max_line" "$near_max_error"
semantic_executed=0
for semantic_mode in observer tool-pins candidate-receipt topology manifest manifest-order; do
    first=$work/semantic-first.v0
    mutated=$work/semantic-$semantic_mode.v0
    rewrite_clean_semantic_blob B000001 clean-success-pass-1 "$semantic_mode" "$valid" "$first"
    rewrite_clean_semantic_blob B000002 clean-success-pass-2 "$semantic_mode" "$first" "$mutated"
    /bin/rm -rf -- "$validator_scratch"
    /bin/mkdir "$validator_scratch"
    case "$semantic_mode" in
        observer) semantic_expected_error="observer source is not independently trusted" ;;
        tool-pins) semantic_expected_error="tool inventory is not trusted-header bound" ;;
        candidate-receipt) semantic_expected_error="candidate receipt value invalid: generator_sha256" ;;
        topology) semantic_expected_error="verification receipt value invalid: topology_sha256" ;;
        manifest) semantic_expected_error="candidate receipt value invalid: manifest_sha256" ;;
        manifest-order) semantic_expected_error="canonical manifest payload order invalid" ;;
    esac
    semantic_error=$work/semantic-$semantic_mode.err
    if /bin/sh "$validator" "$mutated" "$cases" "$validator_scratch" >/dev/null 2>"$semantic_error"; then
        fail "semantic receipt projection mutation accepted: $semantic_mode"
    fi
    /usr/bin/grep -Fq "$semantic_expected_error" "$semantic_error" ||
        fail "semantic mutation did not reach intended rejection: $semantic_mode"
    semantic_executed=$((semantic_executed + 1))
done
[ "$semantic_executed" -eq 6 ] || fail 'semantic receipt mutation count mismatch'
printf '%s\n' 'C3VA evidence policy tests passed: complete=1 rejected=20 semantic-rejected=6 runtime=none'
