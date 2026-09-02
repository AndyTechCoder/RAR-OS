#!/bin/sh
set -eu
LC_ALL=C
LANG=C
PATH=/usr/bin:/bin
export LC_ALL LANG PATH

[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] && [ "${RUNNER_OS-}" = Linux ] || {
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
: > "$ledger"
: > "$body"

{
    printf '%s\n' \
        'B000001|clean-success-pass-1|RUN' \
        'B000002|clean-success-pass-2|RUN'
    /usr/bin/awk -F '|' '
    BEGIN { blob=2 }
    /^case\|/ {
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

chunk_total=0
decoded_total=0
while IFS='|' read -r blob kind case_id; do
    fields=$(fields_for "$kind") || fail "unknown planned kind $kind"
    set -- $fields
    {
        printf '%s\n' \
            'rar-c3v-envelope-v0' \
            "kind=$kind" \
            "case_id=$case_id" \
            "field_count=$#"
        ordinal=0
        for name do
            ordinal=$((ordinal + 1))
            nn=$(/usr/bin/printf '%02d' "$ordinal")
            printf '%s\n' \
                "field.$nn.name=$name" \
                "field.$nn.bytes=0" \
                "field.$nn.sha256=$empty_sha" \
                "field.$nn.data" \
                ''
        done
    } > "$envelope"
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
            "verification_receipt_sha256=$empty_sha" \
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
        "$revision" "$revision" "$empty_sha" \
        "$digest" "$digest" "$digest" "$digest" "$digest" "$digest" "$digest" "$cases_sha" \
        "$digest" "$digest" "$digest" "$nonce" "$chunk_total" "$decoded_total"
    /bin/cat "$body"
} > "$valid"

validator_scratch=$work/validator
/bin/mkdir "$validator_scratch"
/bin/sh "$validator" "$valid" "$cases" "$validator_scratch" >/dev/null ||
    fail 'complete canonical evidence was rejected'

mutate() {
    id=$1
    input=$2
    output=$3
    case "$id" in
        EP001) : > "$output" ;;
        EP002)
            /usr/bin/sed '0,/^B|B000003|/s//B|B000003|runtime-pre-input-topology|V002|0|/' "$input" > "$output" ;;
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
        EP009)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="B"{$5=$5+1;done=1}{print}' "$input" > "$output" ;;
        EP010)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="B"{$7="2222222222222222222222222222222222222222222222222222222222222222";done=1}{print}' "$input" > "$output" ;;
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
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="N"{$6="N001";done=1}{print}' "$input" > "$output" ;;
        EP019)
            /usr/bin/awk -F '|' 'BEGIN{OFS="|"} !done && $1=="N"{$7="2222222222222222222222222222222222222222222222222222222222222222";done=1}{print}' "$input" > "$output" ;;
        EP020)
            second=$(/usr/bin/awk -F '|' '$1=="N" && $3=="V002"{print $6;exit}' "$input")
            /usr/bin/awk -F '|' -v ids="$second" 'BEGIN{OFS="|"} !done && $1=="N"{$6=ids;done=1}{print}' "$input" > "$output" ;;
        *) fail "unknown policy case $id" ;;
    esac
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
printf '%s\n' 'C3VA evidence policy tests passed: complete=1 rejected=20 runtime=none'
