#!/bin/sh
set -eu
LC_ALL=C
LANG=C
PATH=/usr/bin:/bin
export LC_ALL LANG PATH

evidence=${1-}
cases=${2-}
scratch=${3-}
fail() { printf 'controller-helper closure verifier evidence rejected: %s\n' "$1" >&2; exit 1; }
sha_file() { /usr/bin/sha256sum -- "$1" | /usr/bin/awk '{ print $1 }'; }
size_file() { /usr/bin/stat -c %s -- "$1"; }
links_file() { /usr/bin/stat -c %h -- "$1"; }
canonical_unsigned() {
    case "${1-}" in ''|*[!0-9]*|0[0-9]*) return 1 ;; esac
    [ "${#1}" -le 20 ]
}
canonical_positive() {
    canonical_unsigned "${1-}" && [ "$1" != 0 ]
}
bounded_unsigned() {
    value=$1
    maximum=$2
    canonical_unsigned "$value" || return 1
    [ "${#value}" -lt "${#maximum}" ] && return 0
    [ "${#value}" -eq "${#maximum}" ] || return 1
    [ "$value" -le "$maximum" ]
}
nonzero_sha() {
    case "${1-}" in ''|*[!0-9a-f]*) return 1 ;; esac
    [ "${#1}" -eq 64 ] &&
        [ "$1" != 0000000000000000000000000000000000000000000000000000000000000000 ]
}
revision() {
    case "${1-}" in ''|*[!0-9a-f]*) return 1 ;; esac
    [ "${#1}" -eq 40 ] &&
        [ "$1" != 0000000000000000000000000000000000000000 ]
}
add_bounded() {
    current=$1
    increment=$2
    maximum=$3
    bounded_unsigned "$increment" "$maximum" || return 1
    next=$((current + increment))
    [ "$next" -ge "$current" ] && [ "$next" -le "$maximum" ] || return 1
    printf '%s\n' "$next"
}
expect_env() {
    eval "value=\${$1-}"
    [ -n "$value" ] || fail "trusted expectation missing: $1"
}

[ -f "$evidence" ] && [ ! -L "$evidence" ] && [ -s "$evidence" ] ||
    fail 'evidence missing, symbolic, or empty'
[ -f "$cases" ] && [ ! -L "$cases" ] && [ -s "$cases" ] ||
    fail 'case catalog missing, symbolic, or empty'
[ "$(links_file "$evidence")" -eq 1 ] || fail 'evidence is hardlinked'
[ -d "$scratch" ] && [ ! -L "$scratch" ] || fail 'scratch root missing or symbolic'
[ -z "$(/usr/bin/find "$scratch" -mindepth 1 -maxdepth 1 -print -quit)" ] ||
    fail 'scratch root not empty'
[ "$(size_file "$evidence")" -le 440401920 ] || fail 'evidence file oversized'
if ! /usr/bin/od -An -v -tu1 "$evidence" | /usr/bin/awk '{ for (i=1; i<=NF; i++) { if ($i==10) line_bytes=0; else { line_bytes++; if (line_bytes>8192) exit 1 } } }'; then
    fail 'physical line bound exceeded'
fi
[ "$(/usr/bin/tail -c 1 "$evidence" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] ||
    fail 'terminal LF missing'
if /usr/bin/od -An -tx1 "$evidence" |
    /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)'; then
    fail 'NUL or CR present'
fi
if ! /usr/bin/od -An -v -tu1 "$evidence" | /usr/bin/awk '{ for (i=1; i<=NF; i++) if ($i!=10 && ($i<32 || $i>126)) exit 1 }'; then
    fail 'non-ASCII or forbidden control byte present'
fi
if /usr/bin/grep -n '^$' "$evidence" >/dev/null; then
    fail 'blank outer record present'
fi
[ "$(/usr/bin/wc -l < "$evidence" | /usr/bin/tr -d ' ')" -le 207660 ] ||
    fail 'logical record bound exceeded'

for name in \
    RAR_EXPECTED_REPOSITORY \
    RAR_EXPECTED_CONTROLLER_SHA \
    RAR_EXPECTED_SOURCE_SHA \
    RAR_EXPECTED_RUN_ID \
    RAR_EXPECTED_RUN_ATTEMPT \
    RAR_EXPECTED_VERIFICATION_RECEIPT_SHA256 \
    RAR_EXPECTED_SUBJECT_SHA256 \
    RAR_EXPECTED_OBSERVER_SHA256 \
    RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256 \
    RAR_EXPECTED_VALIDATION_SHA256 \
    RAR_EXPECTED_DISPOSITIONS_SHA256 \
    RAR_EXPECTED_TEMPLATES_SHA256 \
    RAR_EXPECTED_PRECEDENCE_SHA256 \
    RAR_EXPECTED_FAULTS_SHA256 \
    RAR_EXPECTED_CASES_SHA256 \
    RAR_EXPECTED_BASE_FIXTURE_SHA256 \
    RAR_EXPECTED_FIXTURE_IMAGE_SHA256 \
    RAR_EXPECTED_TOOL_PINS_SHA256 \
    RAR_EXPECTED_ARTIFACT_NONCE; do
    expect_env "$name"
done
[ "$(add_bounded 396032119 1 396032120)" = 396032120 ] || fail 'checked bound self-test failed'
if add_bounded 396032120 1 396032120 >/dev/null 2>&1; then fail 'checked overflow self-test failed'; fi
[ "$RAR_EXPECTED_REPOSITORY" = AndyTechCoder/RAR-OS ] || fail 'repository expectation invalid'
revision "$RAR_EXPECTED_CONTROLLER_SHA" || fail 'controller revision malformed'
revision "$RAR_EXPECTED_SOURCE_SHA" || fail 'source revision malformed'
[ "$RAR_EXPECTED_CONTROLLER_SHA" = "$RAR_EXPECTED_SOURCE_SHA" ] ||
    fail 'controller/source revision mismatch'
canonical_positive "$RAR_EXPECTED_RUN_ID" || fail 'run ID malformed'
canonical_positive "$RAR_EXPECTED_RUN_ATTEMPT" || fail 'run attempt malformed'
for value in \
    "$RAR_EXPECTED_VERIFICATION_RECEIPT_SHA256" \
    "$RAR_EXPECTED_SUBJECT_SHA256" \
    "$RAR_EXPECTED_OBSERVER_SHA256" \
    "$RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256" \
    "$RAR_EXPECTED_VALIDATION_SHA256" \
    "$RAR_EXPECTED_DISPOSITIONS_SHA256" \
    "$RAR_EXPECTED_TEMPLATES_SHA256" \
    "$RAR_EXPECTED_PRECEDENCE_SHA256" \
    "$RAR_EXPECTED_FAULTS_SHA256" \
    "$RAR_EXPECTED_CASES_SHA256" \
    "$RAR_EXPECTED_BASE_FIXTURE_SHA256" \
    "$RAR_EXPECTED_FIXTURE_IMAGE_SHA256" \
    "$RAR_EXPECTED_TOOL_PINS_SHA256" \
    "$RAR_EXPECTED_ARTIFACT_NONCE"; do
    nonzero_sha "$value" || fail 'trusted digest or nonce malformed'
done
[ "$(sha_file "$cases")" = "$RAR_EXPECTED_CASES_SHA256" ] ||
    fail 'case catalog differs from trusted identity'

plan=$scratch/expected-blobs.v0
ledger=$scratch/verified-blobs.v0
encoded=$scratch/current.base64
chunk=$scratch/current.chunk
decoded=$scratch/current.blob
pass_one=$scratch/pass-one.fields
pass_two=$scratch/pass-two.fields
preimage=$scratch/normalized.preimage
field_raw=$scratch/current.field
semantic_expected=$scratch/semantic.expected
case_row_file=$scratch/case.row
receipt_keys=$scratch/receipt.keys
semantic_payload=$scratch/semantic.payload
retained_receipt=$scratch/retained.receipt
receipt_seen=0
candidate_receipt_raw=$scratch/candidate.receipt
candidate_receipt_seen=0
cleanup() {
    rc=$?
    trap - EXIT HUP INT TERM
    /bin/rm -f -- "$plan" "$ledger" "$encoded" "$chunk" "$decoded" "$pass_one" "$pass_two" "$preimage" "$field_raw" "$semantic_expected" "$case_row_file" "$receipt_keys" "$semantic_payload" "$retained_receipt" "$candidate_receipt_raw"
    [ -z "$(/usr/bin/find "$scratch" -mindepth 1 -maxdepth 1 -print -quit)" ] || rc=1
    exit "$rc"
}
trap cleanup EXIT HUP INT TERM

{
    printf '%s\n' \
        'B000001|clean-success-pass-1|RUN' \
        'B000002|clean-success-pass-2|RUN'
    /usr/bin/awk -F '|' '
    BEGIN { blob=2; logical=0; runtime=0; residual=0 }
    /^case\|[VQX][0-9][0-9][0-9]\|/ {
        logical++
        expected=(logical<=147 ? sprintf("V%03d",logical) :
            logical<=197 ? sprintf("Q%03d",logical-147) :
            sprintf("X%03d",logical-197))
        if ($2!=expected) exit 1
        if ($3 ~ /-residual$/) {
            residual++; blob++
            printf "B%06d|residual-source-proof|%s\n",blob,$2
        } else {
            runtime++
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
    END {
        if (logical!=209 || runtime!=166 || residual!=43 || blob!=709) exit 1
    }
    ' "$cases"
} > "$plan" || fail 'case catalog cannot derive exact blob allocation'
: > "$ledger"
: > "$pass_one"
: > "$pass_two"

line_number=0
IFS= read -r header < "$evidence" || fail 'header unavailable'
line_number=1
[ "${#header}" -le 8192 ] || fail 'header oversized'
oldifs=$IFS
IFS='|' read -r \
    tag schema repository controller source run_id run_attempt receipt_sha \
    subject verification validation dispositions templates precedence faults case_sha \
    base_fixture fixture_image tool_pins nonce runtime_count residual_count logical_count \
    blob_count chunk_count decoded_total normalized_count failed_count verdict extra <<EOF
$header
EOF
IFS=$oldifs
[ -z "${extra-}" ] || fail 'header has extra field'
[ "$tag" = H ] &&
    [ "$schema" = rar-alpha-controller-helper-closure-verifier-evidence-v0 ] ||
    fail 'header schema invalid'
[ "$repository" = "$RAR_EXPECTED_REPOSITORY" ] &&
    [ "$controller" = "$RAR_EXPECTED_CONTROLLER_SHA" ] &&
    [ "$source" = "$RAR_EXPECTED_SOURCE_SHA" ] &&
    [ "$run_id" = "$RAR_EXPECTED_RUN_ID" ] &&
    [ "$run_attempt" = "$RAR_EXPECTED_RUN_ATTEMPT" ] &&
    [ "$receipt_sha" = "$RAR_EXPECTED_VERIFICATION_RECEIPT_SHA256" ] ||
    fail 'trusted header identity mismatch'
[ "$subject" = "$RAR_EXPECTED_SUBJECT_SHA256" ] &&
    [ "$verification" = "$RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256" ] &&
    [ "$validation" = "$RAR_EXPECTED_VALIDATION_SHA256" ] &&
    [ "$dispositions" = "$RAR_EXPECTED_DISPOSITIONS_SHA256" ] &&
    [ "$templates" = "$RAR_EXPECTED_TEMPLATES_SHA256" ] &&
    [ "$precedence" = "$RAR_EXPECTED_PRECEDENCE_SHA256" ] &&
    [ "$faults" = "$RAR_EXPECTED_FAULTS_SHA256" ] &&
    [ "$case_sha" = "$RAR_EXPECTED_CASES_SHA256" ] &&
    [ "$base_fixture" = "$RAR_EXPECTED_BASE_FIXTURE_SHA256" ] &&
    [ "$fixture_image" = "$RAR_EXPECTED_FIXTURE_IMAGE_SHA256" ] &&
    [ "$tool_pins" = "$RAR_EXPECTED_TOOL_PINS_SHA256" ] &&
    [ "$nonce" = "$RAR_EXPECTED_ARTIFACT_NONCE" ] ||
    fail 'trusted digest header mismatch'
[ "$runtime_count" = 166 ] && [ "$residual_count" = 43 ] &&
    [ "$logical_count" = 209 ] && [ "$blob_count" = 709 ] &&
    [ "$normalized_count" = 209 ] && [ "$failed_count" = 0 ] ||
    fail 'header fixed count mismatch'
bounded_unsigned "$chunk_count" 206741 &&
    bounded_unsigned "$decoded_total" 296816640 ||
    fail 'header aggregate bound malformed'
case "$verdict" in
    mechanically-verified-not-reviewed-not-ready|normalized-not-ready) ;;
    *) fail 'verdict invalid' ;;
esac

blob_seen=0
chunk_seen=0
decoded_chunk_seen=0
encoded_seen=0
normalized_seen=0
current=
current_kind=
current_case=
current_decoded=
current_chunks=
current_sha=
current_chunk_seen=0
current_chunk_bytes=0

expected_fields_for() {
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
raw_field() { case "$1" in input-bytes|stdout-bytes|stderr-bytes|output-bytes) return 0 ;; *) return 1 ;; esac; }
case_binding() {
    if [ "$current_case" = RUN ]; then
        binding_row_sha=$RAR_EXPECTED_CASES_SHA256
        binding_oracle_sha=$RAR_EXPECTED_SUBJECT_SHA256
        return 0
    fi
    /usr/bin/awk -F '|' -v c="$current_case" '$1=="case" && $2==c { print; found++ } END { if(found!=1) exit 1 }' "$cases" > "$case_row_file" || fail 'case row is not unique'
    binding_row_sha=$(sha_file "$case_row_file")
    binding_oracle=$(/usr/bin/awk -F '|' '{ print $8 }' "$case_row_file")
    [ -n "$binding_oracle" ] || fail 'case oracle missing'
    printf '%s' "$binding_oracle" > "$semantic_expected"
    binding_oracle_sha=$(sha_file "$semantic_expected")
}
validate_projection() {
    name=$1
    raw=$2
    observation=$(observation_for "$name") || fail "field $name has no semantic rule"
    case_binding
    semantic_kind=$current_kind
    case "$semantic_kind" in clean-success-pass-1|clean-success-pass-2) semantic_kind=clean-success-pass ;; esac
    exec 4< "$raw"
    IFS= read -r p_schema <&4 || fail 'semantic schema missing'
    IFS= read -r p_kind <&4 || fail 'semantic kind missing'
    IFS= read -r p_case <&4 || fail 'semantic case missing'
    IFS= read -r p_field <&4 || fail 'semantic field missing'
    IFS= read -r p_row <&4 || fail 'semantic row binding missing'
    IFS= read -r p_oracle <&4 || fail 'semantic oracle binding missing'
    IFS= read -r p_observation <&4 || fail 'semantic observation missing'
    IFS= read -r p_bytes_line <&4 || fail 'semantic payload length missing'
    IFS= read -r p_sha_line <&4 || fail 'semantic payload digest missing'
    IFS= read -r p_data <&4 || fail 'semantic payload marker missing'
    [ "$p_schema" = schema=rar-c3v-semantic-field-v0 ] &&
        [ "$p_kind" = "kind=$semantic_kind" ] &&
        [ "$p_case" = "case_id=$current_case" ] &&
        [ "$p_field" = "field=$name" ] &&
        [ "$p_row" = "catalog_row_sha256=$binding_row_sha" ] &&
        [ "$p_oracle" = "oracle_sha256=$binding_oracle_sha" ] &&
        [ "$p_observation" = "observation=$observation" ] ||
        fail "semantic field $name binding mismatch"
    p_bytes=${p_bytes_line#payload_bytes=}
    [ "$p_bytes_line" = "payload_bytes=$p_bytes" ] &&
        bounded_unsigned "$p_bytes" 16777216 && [ "$p_bytes" -gt 0 ] ||
        fail "semantic field $name payload length malformed"
    p_sha=${p_sha_line#payload_sha256=}
    [ "$p_sha_line" = "payload_sha256=$p_sha" ] && nonzero_sha "$p_sha" ||
        fail "semantic field $name payload digest malformed"
    [ "$p_data" = payload ] || fail "semantic field $name payload marker invalid"
    /bin/dd bs=1 count="$p_bytes" <&4 of="$semantic_payload" 2>/dev/null
    [ "$(size_file "$semantic_payload")" -eq "$p_bytes" ] &&
        [ "$(sha_file "$semantic_payload")" = "$p_sha" ] ||
        fail "semantic field $name payload mismatch"
    if IFS= read -r p_extra <&4; then fail "semantic field $name extension bytes"; fi
    exec 4<&-
    if [ "$current_case" = RUN ]; then
        case "$name" in
            tool-inventory)
                [ "$(/usr/bin/wc -l < "$semantic_payload" | /usr/bin/tr -d ' ')" -eq 8 ] || fail 'tool inventory line count mismatch'
                [ "$(/usr/bin/sed -n '1p' "$semantic_payload")" = schema=rar-alpha-controller-helper-closure-verifier-tools-v0 ] || fail 'tool inventory schema mismatch'
                [ "$(/usr/bin/sed -n '8p' "$semantic_payload")" = status=reviewed-for-candidate-verification-only ] || fail 'tool inventory status mismatch'
                receipt_find=$(/usr/bin/sed -n 's/^find_sha256=//p' "$semantic_payload")
                receipt_sort=$(/usr/bin/sed -n 's/^sort_sha256=//p' "$semantic_payload")
                receipt_wc=$(/usr/bin/sed -n 's/^wc_sha256=//p' "$semantic_payload")
                receipt_stat=$(/usr/bin/sed -n 's/^stat_sha256=//p' "$semantic_payload")
                receipt_cmp=$(/usr/bin/sed -n 's/^cmp_sha256=//p' "$semantic_payload")
                receipt_id=$(/usr/bin/sed -n 's/^id_sha256=//p' "$semantic_payload")
                for tool_digest in "$receipt_find" "$receipt_sort" "$receipt_wc" "$receipt_stat" "$receipt_cmp" "$receipt_id"; do nonzero_sha "$tool_digest" || fail 'tool inventory digest malformed'; done
                printf '%s\n' \
                    'schema=rar-alpha-controller-helper-closure-verifier-tools-v0' \
                    "find_sha256=$receipt_find" "sort_sha256=$receipt_sort" "wc_sha256=$receipt_wc" \
                    "stat_sha256=$receipt_stat" "cmp_sha256=$receipt_cmp" "id_sha256=$receipt_id" \
                    'status=reviewed-for-candidate-verification-only' > "$semantic_expected"
                /usr/bin/cmp -s "$semantic_payload" "$semantic_expected" || fail 'tool inventory is not canonical bytes'
                receipt_tool_pins=$p_sha
                [ "$receipt_tool_pins" = "$RAR_EXPECTED_TOOL_PINS_SHA256" ] || fail 'tool inventory is not trusted-header bound'
                ;;
            domain-header) receipt_observer=$p_sha; [ "$receipt_observer" = "$RAR_EXPECTED_OBSERVER_SHA256" ] || fail 'observer source is not independently trusted' ;;
            fixture-inventory)
                receipt_candidate=$p_sha
                candidate_receipt_seen=$((candidate_receipt_seen + 1))
                case "$candidate_receipt_seen" in 1) /bin/cp "$semantic_payload" "$candidate_receipt_raw" ;; 2) /usr/bin/cmp -s "$semantic_payload" "$candidate_receipt_raw" || fail 'candidate receipt differs between passes' ;; *) fail 'unexpected candidate receipt copy' ;; esac
                ;;
            canonical-manifest)
                /usr/bin/awk 'length($0)>450 || $0 !~ /^[0-9a-f]{64}  [A-Za-z0-9._+:/-]{1,384}$/ { exit 1 } { if(path[$2]++) exit 1; count++ } END { if(count<1) exit 1 }' "$semantic_payload" || fail 'canonical manifest payload malformed'
                /usr/bin/awk 'NR>1 && ("x" $2) <= ("x" previous) { exit 1 } { previous=$2 }' "$semantic_payload" || fail 'canonical manifest payload order invalid'
                receipt_manifest=$p_sha
                receipt_second=$p_sha
                receipt_manifest_entries=$(/usr/bin/wc -l < "$semantic_payload" | /usr/bin/tr -d ' ')
                receipt_manifest_bytes=$(size_file "$semantic_payload")
                ;;
            topology) receipt_topology=$p_sha ;;
        esac
    fi
    case "$name" in
        observed-event) validate_observed_result "$semantic_payload" ;;
        timeout-termination|residual-proof)
            printf '%s' "$binding_oracle" > "$semantic_expected"
            /usr/bin/cmp -s "$semantic_payload" "$semantic_expected" ||
                fail "$name does not retain exact catalog oracle bytes"
            ;;
        mutation-schedule|mutation-trigger|mutation-acknowledgement|residual-source)
            [ "$current_case" != RUN ] && /usr/bin/cmp -s "$semantic_payload" "$case_row_file" ||
                fail "$name does not retain exact catalog row bytes"
            ;;
    esac
}

validate_observed_result() {
    raw=$1
    receipt_state=
    oldifs=$IFS
    IFS='|' read -r row_marker row_case row_kind row_source row_left row_right row_binding row_oracle <<EOF
$(/bin/cat "$case_row_file")
EOF
    IFS=$oldifs
    case "$row_kind" in
        disposition)
            primary=${row_oracle%%@*}
            case "$row_oracle" in *+normal-exit-status-1+*) ;; *) fail 'disposition oracle termination unsupported' ;; esac
            termination=exit-1
            controller_exit=1
            ;;
        precedence)
            case "$row_oracle" in first-error-E[0-9][0-9][0-9]) ;; *) fail 'precedence oracle unsupported' ;; esac
            primary=${row_oracle#first-error-}
            termination=exit-1
            controller_exit=1
            receipt_state=not-specified-by-precedence-oracle
            ;;
        fault)
            primary=$row_source
            case "$row_oracle" in
                signal-25+controller-exit-map-153+*) termination=signal-25; controller_exit=153 ;;
                exit-1+*) termination=exit-1; controller_exit=1 ;;
                *) fail 'fault oracle termination unsupported' ;;
            esac
            ;;
        *) fail 'runtime observed result used for residual row' ;;
    esac
    if [ -z "$receipt_state" ]; then
        case "$row_oracle" in
            *no-valid-final-receipt*) receipt_state=no-valid-final-receipt ;;
            *no-receipt*) receipt_state=no-receipt ;;
            *) fail 'catalog oracle does not define receipt state' ;;
        esac
    fi
    {
        printf '%s\n' \
            'schema=rar-c3v-observed-result-v0' \
            "case_id=$row_case" \
            "catalog_kind=$row_kind" \
            "primary=$primary" \
            "termination=$termination" \
            "controller_exit=$controller_exit" \
            "receipt_state=$receipt_state"
    } > "$semantic_expected"
    /usr/bin/cmp -s "$raw" "$semantic_expected" ||
        fail 'observed result does not mechanically satisfy catalog oracle'
}

validate_candidate_receipt() {
    raw=$1
    [ "$(size_file "$raw")" -le 4096 ] && [ "$(/usr/bin/wc -l < "$raw" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'candidate receipt size or line count invalid'
    printf '%s\n' schema status controller_sha source_sha repository ref event run_id run_attempt runner_os runner_image_version oci_image closure_root generator_sha256 find_sha256 sort_sha256 manifest_sha256 manifest_entries manifest_bytes helper_compiled helper_executed target_compiled readiness > "$receipt_keys"
    ordinal=0
    while IFS= read -r candidate_line; do
        ordinal=$((ordinal + 1)); key=${candidate_line%%=*}; value=${candidate_line#*=}
        [ "$candidate_line" != "$key" ] && [ -n "$value" ] || fail 'candidate receipt key/value malformed'
        [ "$key" = "$(/usr/bin/sed -n "${ordinal}p" "$receipt_keys")" ] || fail 'candidate receipt key order mismatch'
        case "$key" in
            schema) [ "$value" = rar-alpha-controller-helper-closure-observation-v0 ] ;;
            status) [ "$value" = observed-not-reviewed-not-ready ] ;;
            controller_sha|source_sha) [ "$value" = "$RAR_EXPECTED_CONTROLLER_SHA" ] ;;
            repository) [ "$value" = "$RAR_EXPECTED_REPOSITORY" ] ;;
            ref) [ "$value" = refs/heads/main ] ;;
            event) [ "$value" = push ] ;;
            run_id) [ "$value" = "$RAR_EXPECTED_RUN_ID" ] ;;
            run_attempt) [ "$value" = "$RAR_EXPECTED_RUN_ATTEMPT" ] ;;
            runner_os) [ "$value" = ubuntu24 ] ;;
            runner_image_version) case "$value" in ''|.*|*..*|*.|*[!0-9.]*) false ;; *) [ "${#value}" -le 64 ] ;; esac ;;
            oci_image) [ "$value" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] ;;
            closure_root) [ "$value" = /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu ] ;;
            generator_sha256) [ "$value" = "$receipt_observer" ] ;;
            find_sha256) [ "$value" = "$receipt_find" ] ;;
            sort_sha256) [ "$value" = "$receipt_sort" ] ;;
            manifest_sha256) [ "$value" = "$receipt_manifest" ] ;;
            manifest_entries) [ "$value" = "$receipt_manifest_entries" ] ;;
            manifest_bytes) [ "$value" = "$receipt_manifest_bytes" ] ;;
            helper_compiled|helper_executed|target_compiled|readiness) [ "$value" = false ] ;;
            *) false ;;
        esac || fail "candidate receipt value invalid: $key"
    done < "$raw"
}

validate_receipt() {
    raw=$1
    [ "$(size_file "$raw")" -le 8192 ] || fail 'verification receipt oversized'
    [ "$(/usr/bin/tail -c 1 "$raw" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'verification receipt terminal LF missing'
    if /usr/bin/od -An -tx1 "$raw" | /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)'; then fail 'verification receipt NUL or CR present'; fi
    [ "$(/usr/bin/wc -l < "$raw" | /usr/bin/tr -d ' ')" -eq 31 ] || fail 'verification receipt line count mismatch'
    printf '%s\n' schema status controller_sha source_sha repository run_id run_attempt runner_os runner_image_version oci_image closure_root verifier_sha256 observer_sha256 tool_pins_sha256 find_sha256 sort_sha256 wc_sha256 stat_sha256 cmp_sha256 id_sha256 candidate_receipt_sha256 candidate_manifest_sha256 recomputed_manifest_sha256 manifest_entries manifest_bytes topology_sha256 second_pass_sha256 helper_compiled helper_executed target_compiled readiness > "$receipt_keys"
    ordinal=0; manifest_candidate=; manifest_recomputed=
    while IFS= read -r receipt_line; do
        ordinal=$((ordinal + 1)); key=${receipt_line%%=*}; value=${receipt_line#*=}
        [ "$receipt_line" != "$key" ] && [ -n "$value" ] || fail 'verification receipt key/value malformed'
        expected_key=$(/usr/bin/sed -n "${ordinal}p" "$receipt_keys")
        [ "$key" = "$expected_key" ] || fail 'verification receipt key order mismatch'
        case "$key" in
            schema) [ "$value" = rar-alpha-controller-helper-closure-verification-v0 ] ;;
            status) [ "$value" = candidate-exact-set-verified-not-reviewed-not-ready ] ;;
            controller_sha|source_sha) [ "$value" = "$RAR_EXPECTED_CONTROLLER_SHA" ] ;;
            repository) [ "$value" = "$RAR_EXPECTED_REPOSITORY" ] ;;
            run_id) [ "$value" = "$RAR_EXPECTED_RUN_ID" ] ;;
            run_attempt) [ "$value" = "$RAR_EXPECTED_RUN_ATTEMPT" ] ;;
            runner_os) [ "$value" = ubuntu24 ] ;;
            runner_image_version) case "$value" in ''|.*|*..*|*.|*[!0-9.]*) false ;; *) [ "${#value}" -le 64 ] ;; esac ;;
            oci_image) [ "$value" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] ;;
            closure_root) [ "$value" = /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu ] ;;
            verifier_sha256) [ "$value" = "$RAR_EXPECTED_SUBJECT_SHA256" ] ;;
            tool_pins_sha256) [ "$value" = "$RAR_EXPECTED_TOOL_PINS_SHA256" ] && [ "$value" = "$receipt_tool_pins" ] ;;
            observer_sha256) [ "$value" = "$receipt_observer" ] ;;
            find_sha256) [ "$value" = "$receipt_find" ] ;;
            sort_sha256) [ "$value" = "$receipt_sort" ] ;;
            wc_sha256) [ "$value" = "$receipt_wc" ] ;;
            stat_sha256) [ "$value" = "$receipt_stat" ] ;;
            cmp_sha256) [ "$value" = "$receipt_cmp" ] ;;
            id_sha256) [ "$value" = "$receipt_id" ] ;;
            candidate_receipt_sha256) [ "$value" = "$receipt_candidate" ] ;;
            topology_sha256) [ "$value" = "$receipt_topology" ] ;;
            second_pass_sha256) [ "$value" = "$receipt_second" ] ;;
            candidate_manifest_sha256) [ "$value" = "$receipt_manifest" ] && manifest_candidate=$value ;;
            recomputed_manifest_sha256) [ "$value" = "$receipt_manifest" ] && manifest_recomputed=$value ;;
            manifest_entries) [ "$value" = "$receipt_manifest_entries" ] ;;
            manifest_bytes) [ "$value" = "$receipt_manifest_bytes" ] ;;
            helper_compiled|helper_executed|target_compiled|readiness) [ "$value" = false ] ;;
            *) false ;;
        esac || fail "verification receipt value invalid: $key"
    done < "$raw"
    [ "$manifest_candidate" = "$manifest_recomputed" ] || fail 'verification receipt manifest recomputation mismatch'
}
validate_semantic_field() {
    name=$1; raw=$2
    case "$name" in
        verification-receipt-inputs)
            receipt_seen=$((receipt_seen + 1))
            case "$receipt_seen" in
                1) /bin/cp "$raw" "$retained_receipt" ;;
                2) /usr/bin/cmp -s "$raw" "$retained_receipt" || fail 'clean-pass receipt bytes differ' ;;
                *) fail 'unexpected verification receipt copy' ;;
            esac
            ;;
        event-bytes|resource-bytes) [ "$current_case" = RUN ] || fail "$name outside clean-success"; validate_projection "$name" "$raw" ;;
        *) if raw_field "$name"; then :; else validate_projection "$name" "$raw"; fi ;;
    esac
}

parse_envelope() {
    target_ledger=
    case "$current" in
        B000001) target_ledger=$pass_one ;;
        B000002) target_ledger=$pass_two ;;
    esac
    exec 3< "$decoded"
    IFS= read -r envelope_magic <&3 || fail 'envelope magic missing'
    IFS= read -r envelope_kind <&3 || fail 'envelope kind missing'
    IFS= read -r envelope_case <&3 || fail 'envelope case missing'
    IFS= read -r envelope_count <&3 || fail 'envelope field count missing'
    [ "$envelope_magic" = rar-c3v-envelope-v0 ] || fail 'envelope magic invalid'
    [ "$envelope_kind" = "kind=$current_kind" ] || fail 'envelope kind mismatch'
    [ "$envelope_case" = "case_id=$current_case" ] || fail 'envelope case mismatch'
    field_count=${envelope_count#field_count=}
    [ "$envelope_count" = "field_count=$field_count" ] &&
        bounded_unsigned "$field_count" 11 && [ "$field_count" -gt 0 ] ||
        fail 'envelope field count malformed'
    expected_fields=$(expected_fields_for "$current_kind") ||
        fail 'envelope kind has no field schema'
    set -- $expected_fields
    [ "$field_count" -eq "$#" ] || fail 'envelope field count mismatch'
    field_ordinal=0
    for expected_name do
        field_ordinal=$((field_ordinal + 1))
        nn=$(/usr/bin/printf '%02d' "$field_ordinal")
        IFS= read -r field_name_line <&3 || fail 'field name missing'
        IFS= read -r field_bytes_line <&3 || fail 'field byte count missing'
        IFS= read -r field_sha_line <&3 || fail 'field digest missing'
        IFS= read -r field_data_line <&3 || fail 'field data marker missing'
        [ "$field_name_line" = "field.$nn.name=$expected_name" ] ||
            fail 'field name or order mismatch'
        field_bytes=${field_bytes_line#field.$nn.bytes=}
        [ "$field_bytes_line" = "field.$nn.bytes=$field_bytes" ] &&
            bounded_unsigned "$field_bytes" 16777216 ||
            fail 'field byte count malformed'
        field_sha=${field_sha_line#field.$nn.sha256=}
        [ "$field_sha_line" = "field.$nn.sha256=$field_sha" ] &&
            nonzero_sha "$field_sha" || fail 'field digest malformed'
        [ "$field_data_line" = "field.$nn.data" ] || fail 'field data marker invalid'
        /bin/dd bs=1 count="$field_bytes" <&3 of="$field_raw" 2>/dev/null
        [ "$(size_file "$field_raw")" -eq "$field_bytes" ] || fail 'field decoded length mismatch'
        actual_field_sha=$(sha_file "$field_raw")
        [ "$actual_field_sha" = "$field_sha" ] || fail 'field decoded digest mismatch'
        validate_semantic_field "$expected_name" "$field_raw"
        IFS= read -r field_terminator <&3 || fail 'field terminator missing'
        [ -z "$field_terminator" ] || fail 'field length framing mismatch'
        if [ -n "$target_ledger" ]; then
            printf '%s|%s|%s|%s\n' "$nn" "$expected_name" "$field_bytes" "$field_sha" >> "$target_ledger"
        fi
        if [ "$expected_name" = verification-receipt-inputs ]; then
            [ "$field_sha" = "$RAR_EXPECTED_VERIFICATION_RECEIPT_SHA256" ] ||
                fail 'verification receipt field is not trusted-header bound'
        fi
        /bin/rm -f -- "$field_raw" "$semantic_expected" "$case_row_file" "$receipt_keys" "$semantic_payload"
    done
    if IFS= read -r envelope_extra <&3; then
        fail 'envelope has extension bytes'
    fi
    exec 3<&-
}

finalize_blob() {
    [ -n "$current" ] || return 0
    [ "$current_chunk_seen" -eq "$current_chunks" ] ||
        fail 'blob chunk count mismatch'
    actual_bytes=$(size_file "$decoded")
    actual_sha=$(sha_file "$decoded")
    [ "$actual_bytes" -eq "$current_decoded" ] ||
        fail 'blob decoded length mismatch'
    [ "$actual_sha" = "$current_sha" ] || fail 'blob digest mismatch'
    parse_envelope
    printf '%s|%s|%s|%s|%s\n' \
        "$current" "$current_kind" "$current_case" "$actual_bytes" "$actual_sha" >> "$ledger"
    /bin/rm -f -- "$encoded" "$chunk" "$decoded"
    current=
}

while IFS= read -r line; do
    line_number=$((line_number + 1))
    prefix=${line%%|*}
    case "$prefix" in
        B)
            [ "${#line}" -le 256 ] || fail 'blob header oversized'
            [ "$normalized_seen" -eq 0 ] || fail 'blob follows normalized records'
            finalize_blob
            oldifs=$IFS
            IFS='|' read -r btag bid bkind bcase bbytes bchunks bsha bextra <<EOF
$line
EOF
            IFS=$oldifs
            [ -z "${bextra-}" ] || fail 'blob header has extra field'
            blob_seen=$((blob_seen + 1))
            expected=$(/usr/bin/sed -n "${blob_seen}p" "$plan")
            [ "$bid|$bkind|$bcase" = "$expected" ] ||
                fail 'blob allocation or order mismatch'
            canonical_positive "$bbytes" && bounded_unsigned "$bbytes" 16777216 ||
                fail 'blob byte count malformed'
            canonical_positive "$bchunks" && bounded_unsigned "$bchunks" 206741 ||
                fail 'blob chunk count malformed'
            nonzero_sha "$bsha" || fail 'blob digest malformed'
            current=$bid
            current_kind=$bkind
            current_case=$bcase
            current_decoded=$bbytes
            current_chunks=$bchunks
            current_sha=$bsha
            current_chunk_seen=0
            current_chunk_bytes=0
            : > "$decoded"
            ;;
        C)
            [ "${#line}" -le 2048 ] || fail 'chunk record oversized'
            [ -n "$current" ] || fail 'chunk without open blob'
            oldifs=$IFS
            IFS='|' read -r ctag cbid cid payload cextra <<EOF
$line
EOF
            IFS=$oldifs
            [ -z "${cextra-}" ] || fail 'chunk has extra field'
            current_chunk_seen=$((current_chunk_seen + 1))
            expected_cid=$(/usr/bin/printf 'C%06d' "$current_chunk_seen")
            [ "$cbid" = "$current" ] && [ "$cid" = "$expected_cid" ] ||
                fail 'chunk identity or order mismatch'
            case "$payload" in
                ''|*[!A-Za-z0-9+/=]*) fail 'Base64 alphabet invalid' ;;
            esac
            [ $(("${#payload}" % 4)) -eq 0 ] || fail 'Base64 length invalid'
            [ "${#payload}" -le 1916 ] || fail 'chunk payload oversized'
            printf '%s' "$payload" > "$encoded"
            /usr/bin/base64 --decode "$encoded" > "$chunk" 2>/dev/null ||
                fail 'Base64 decode failed'
            canonical_payload=$(/usr/bin/base64 -w 0 "$chunk")
            [ "$canonical_payload" = "$payload" ] || fail 'Base64 spelling is not canonical'
            chunk_bytes=$(size_file "$chunk")
            bounded_unsigned "$chunk_bytes" 1436 && [ "$chunk_bytes" -gt 0 ] ||
                fail 'decoded chunk length invalid'
            if [ "$current_chunk_seen" -lt "$current_chunks" ]; then
                [ "$chunk_bytes" -eq 1436 ] || fail 'nonfinal chunk is not full'
                case "$payload" in
                    *==|*[!A-Za-z0-9+/]=) fail 'full chunk padding invalid' ;;
                    *=) ;;
                    *) fail 'full chunk lacks canonical padding' ;;
                esac
            fi
            current_chunk_bytes=$(add_bounded "$current_chunk_bytes" "$chunk_bytes" 16777216) ||
                fail 'blob decoded accumulation overflow'
            chunk_seen=$(add_bounded "$chunk_seen" 1 206741) ||
                fail 'chunk count accumulation overflow'
            decoded_chunk_seen=$(add_bounded "$decoded_chunk_seen" "$chunk_bytes" 296816640) ||
                fail 'decoded total accumulation overflow'
            encoded_seen=$(add_bounded "$encoded_seen" "${#payload}" 396032120) ||
                fail 'Base64 payload accumulation overflow'
            /bin/cat "$chunk" >> "$decoded"
            /bin/rm -f -- "$encoded" "$chunk"
            ;;
        N)
            [ "${#line}" -le 2048 ] || fail 'normalized record oversized'
            finalize_blob
            oldifs=$IFS
            IFS='|' read -r ntag ordinal case_id catalog_kind result raw_ids normalized_sha nextra <<EOF
$line
EOF
            IFS=$oldifs
            [ -z "${nextra-}" ] || fail 'normalized record has extra field'
            normalized_seen=$((normalized_seen + 1))
            [ "$ordinal" = "$(/usr/bin/printf '%03d' "$normalized_seen")" ] ||
                fail 'normalized ordinal mismatch'
            expected_case=$(
                /usr/bin/awk -F '|' -v n="$normalized_seen" '
                    /^case\|[VQX][0-9][0-9][0-9]\|/ { i++; if(i==n){print $2"|"$3; exit} }
                ' "$cases"
            )
            [ "$case_id|$catalog_kind" = "$expected_case" ] ||
                fail 'normalized case or kind mismatch'
            [ "$result" = pass ] || fail 'normalized result is not pass'
            expected_ids=$(
                /usr/bin/awk -F '|' -v c="$case_id" '
                    $3==c {
                        if(out!="") out=out","
                        out=out $1
                    }
                    END { print out }
                ' "$ledger"
            )
            [ -n "$expected_ids" ] && [ "$raw_ids" = "$expected_ids" ] ||
                fail 'normalized raw blob references mismatch'
            {
                printf '%s\n' \
                    'rar-c3v-normalized-v0' \
                    "repository=$repository" \
                    "controller_sha=$controller" \
                    "source_sha=$source" \
                    "run_id=$run_id" \
                    "run_attempt=$run_attempt" \
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
            derived=$(sha_file "$preimage")
            nonzero_sha "$normalized_sha" && [ "$normalized_sha" = "$derived" ] ||
                fail 'normalized digest is not derived from retained raw blobs'
            /bin/rm -f -- "$preimage"
            ;;
        *) fail 'unknown or reordered record type' ;;
    esac
done < "$evidence"
finalize_blob
[ "$blob_seen" -eq 709 ] && [ "$chunk_seen" -eq "$chunk_count" ] &&
    [ "$normalized_seen" -eq 209 ] ||
    fail 'final record count mismatch'
[ "$decoded_chunk_seen" -eq "$decoded_total" ] ||
    fail 'decoded chunk total mismatch'
declared_decoded_sum=$(
    /usr/bin/awk -F '|' '{ sum += $4 } END { print sum+0 }' "$ledger"
)
[ "$declared_decoded_sum" -eq "$decoded_total" ] ||
    fail 'declared blob decoded total mismatch'
[ "$(size_file "$pass_one")" -gt 0 ] && /usr/bin/cmp -s "$pass_one" "$pass_two" ||
    fail 'clean-success field projections differ'
[ "$receipt_seen" -eq 2 ] || fail 'clean-success receipt count mismatch'
[ "$candidate_receipt_seen" -eq 2 ] || fail 'clean-success candidate receipt count mismatch'
validate_candidate_receipt "$candidate_receipt_raw"
validate_receipt "$retained_receipt"
[ "$encoded_seen" -le 396032120 ] || fail 'Base64 payload total exceeded'
printf '%s\n' 'controller-helper closure verifier evidence structure validated: not runtime evidence, not reviewed, not ready'
