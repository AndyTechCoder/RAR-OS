#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
PATH=/usr/bin:/bin
export PATH

evidence=${1-}
cases=${2-}
scratch=${3-}
fail() { printf 'controller-helper closure verifier evidence rejected: %s\n' "$1" >&2; exit 1; }
sha_file() { /usr/bin/sha256sum -- "$1" | /usr/bin/awk '{ print $1 }'; }
size_file() { /usr/bin/stat -c %s "$1"; }
links_file() { /usr/bin/stat -c %h "$1"; }
canonical_decimal() {
    case "${1-}" in ''|0*|*[!0-9]*) return 1 ;; esac
    [ "${#1}" -le 20 ]
}
nonzero_sha() {
    case "${1-}" in ''|*[!0-9a-f]*) return 1 ;; esac
    [ "${#1}" -eq 64 ] && [ "$1" != 0000000000000000000000000000000000000000000000000000000000000000 ]
}
revision() {
    case "${1-}" in ''|*[!0-9a-f]*) return 1 ;; esac
    [ "${#1}" -eq 40 ] && [ "$1" != 0000000000000000000000000000000000000000 ]
}

[ -f "$evidence" ] && [ ! -L "$evidence" ] && [ -s "$evidence" ] || fail 'evidence missing, symbolic, or empty'
[ -f "$cases" ] && [ ! -L "$cases" ] && [ -s "$cases" ] || fail 'case catalog missing, symbolic, or empty'
[ "$(links_file "$evidence")" -eq 1 ] || fail 'evidence is hardlinked'
[ -d "$scratch" ] && [ ! -L "$scratch" ] || fail 'scratch root missing or symbolic'
[ -z "$(/usr/bin/find "$scratch" -mindepth 1 -maxdepth 1 -print -quit)" ] || fail 'scratch root not empty'
[ "$(size_file "$evidence")" -le 440401920 ] || fail 'evidence file oversized'
[ "$(/usr/bin/tail -c 1 "$evidence" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'terminal LF missing'
if /usr/bin/od -An -tx1 "$evidence" | /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)'; then fail 'NUL or CR present'; fi
if /usr/bin/grep -n '[^ -~]' "$evidence" >/dev/null; then fail 'non-ASCII byte present'; fi
if /usr/bin/grep -n '^$' "$evidence" >/dev/null; then fail 'blank line present'; fi
[ "$(/usr/bin/wc -l < "$evidence" | /usr/bin/tr -d ' ')" -le 207660 ] || fail 'logical record bound exceeded'

for name in RAR_EXPECTED_CONTROLLER_SHA RAR_EXPECTED_SOURCE_SHA RAR_EXPECTED_SUBJECT_SHA256     RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256 RAR_EXPECTED_VALIDATION_SHA256     RAR_EXPECTED_DISPOSITIONS_SHA256 RAR_EXPECTED_TEMPLATES_SHA256 RAR_EXPECTED_PRECEDENCE_SHA256     RAR_EXPECTED_FAULTS_SHA256 RAR_EXPECTED_CASES_SHA256 RAR_EXPECTED_BASE_FIXTURE_SHA256     RAR_EXPECTED_FIXTURE_IMAGE_SHA256 RAR_EXPECTED_TOOL_PINS_SHA256     RAR_EXPECTED_RUN_NONCE RAR_EXPECTED_ROOT_IDENTITY; do
    eval "value=\${$name-}"
    [ -n "$value" ] || fail "trusted expectation missing: $name"
done
revision "$RAR_EXPECTED_CONTROLLER_SHA" || fail 'controller revision malformed'
revision "$RAR_EXPECTED_SOURCE_SHA" || fail 'source revision malformed'
[ "$RAR_EXPECTED_CONTROLLER_SHA" = "$RAR_EXPECTED_SOURCE_SHA" ] || fail 'controller/source revision mismatch'
for value in "$RAR_EXPECTED_SUBJECT_SHA256" "$RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256"     "$RAR_EXPECTED_VALIDATION_SHA256" "$RAR_EXPECTED_DISPOSITIONS_SHA256"     "$RAR_EXPECTED_TEMPLATES_SHA256" "$RAR_EXPECTED_PRECEDENCE_SHA256"     "$RAR_EXPECTED_FAULTS_SHA256" "$RAR_EXPECTED_CASES_SHA256"     "$RAR_EXPECTED_BASE_FIXTURE_SHA256" "$RAR_EXPECTED_FIXTURE_IMAGE_SHA256"     "$RAR_EXPECTED_TOOL_PINS_SHA256" "$RAR_EXPECTED_RUN_NONCE" "$RAR_EXPECTED_ROOT_IDENTITY"; do
    nonzero_sha "$value" || fail 'trusted digest or identity malformed'
done
[ "$(sha_file "$cases")" = "$RAR_EXPECTED_CASES_SHA256" ] || fail 'case catalog differs from trusted identity'

plan=$scratch/expected-blobs.v0
ledger=$scratch/verified-blobs.v0
encoded=$scratch/current.base64
decoded=$scratch/current.raw
cleanup() {
    rc=$?
    trap - EXIT HUP INT TERM
    /bin/rm -f -- "$plan" "$ledger" "$encoded" "$decoded"
    [ -z "$(/usr/bin/find "$scratch" -mindepth 1 -maxdepth 1 -print -quit)" ] || rc=1
    exit "$rc"
}
trap cleanup EXIT HUP INT TERM

/usr/bin/awk -F '|' '
BEGIN { blob=2; logical=0; runtime=0; residual=0 }
/^case\|/ {
    logical++
    expected=(logical<=147 ? sprintf("V%03d",logical) : logical<=197 ? sprintf("Q%03d",logical-147) : sprintf("X%03d",logical-197))
    if ($2!=expected) exit 1
    if ($3 ~ /-residual$/) {
        residual++; blob++; printf "B%06d|residual-source-proof|%s\n",blob,$2
    } else {
        runtime++
        names[1]="pre-input-topology"; names[2]="stdout"; names[3]="stderr"; names[4]="post-output-event-resource"
        for (i=1;i<=4;i++) { blob++; printf "B%06d|%s|%s\n",blob,names[i],$2 }
    }
}
END { if (logical!=209 || runtime!=166 || residual!=43 || blob!=709) exit 1 }
' "$cases" > "$plan" || fail 'case catalog cannot derive exact blob allocation'
{
    printf '%s\n' 'B000001|clean-success-pass-1|S000' 'B000002|clean-success-pass-2|S000'
    /bin/cat "$plan"
} > "$encoded"
/bin/mv "$encoded" "$plan"
: > "$ledger"

IFS= read -r header < "$evidence" || fail 'header unavailable'
oldifs=$IFS
IFS='|' read -r tag schema controller source subject verification validation dispositions templates precedence faults case_sha base_fixture fixture_image tool_pins nonce root_identity runtime_count residual_count logical_count blob_count chunk_count decoded_total failed_count verdict extra <<EOF
$header
EOF
IFS=$oldifs
[ -z "${extra-}" ] || fail 'header has extra field'
[ "$tag" = H ] && [ "$schema" = rar-alpha-controller-helper-closure-verifier-evidence-v0 ] || fail 'header schema invalid'
[ "$controller" = "$RAR_EXPECTED_CONTROLLER_SHA" ] && [ "$source" = "$RAR_EXPECTED_SOURCE_SHA" ] || fail 'header revision mismatch'
[ "$subject" = "$RAR_EXPECTED_SUBJECT_SHA256" ] || fail 'subject identity mismatch'
[ "$verification" = "$RAR_EXPECTED_VERIFICATION_CONTRACT_SHA256" ] || fail 'verification contract identity mismatch'
[ "$validation" = "$RAR_EXPECTED_VALIDATION_SHA256" ] || fail 'validation identity mismatch'
[ "$dispositions" = "$RAR_EXPECTED_DISPOSITIONS_SHA256" ] || fail 'disposition identity mismatch'
[ "$templates" = "$RAR_EXPECTED_TEMPLATES_SHA256" ] || fail 'template identity mismatch'
[ "$precedence" = "$RAR_EXPECTED_PRECEDENCE_SHA256" ] || fail 'precedence identity mismatch'
[ "$faults" = "$RAR_EXPECTED_FAULTS_SHA256" ] && [ "$case_sha" = "$RAR_EXPECTED_CASES_SHA256" ] || fail 'fault/case identity mismatch'
[ "$base_fixture" = "$RAR_EXPECTED_BASE_FIXTURE_SHA256" ] && [ "$fixture_image" = "$RAR_EXPECTED_FIXTURE_IMAGE_SHA256" ] || fail 'fixture identity mismatch'
[ "$tool_pins" = "$RAR_EXPECTED_TOOL_PINS_SHA256" ] || fail 'tool-pin identity mismatch'
[ "$nonce" = "$RAR_EXPECTED_RUN_NONCE" ] && [ "$root_identity" = "$RAR_EXPECTED_ROOT_IDENTITY" ] || fail 'run nonce/root mismatch'
[ "$runtime_count" = 166 ] && [ "$residual_count" = 43 ] && [ "$logical_count" = 209 ] && [ "$blob_count" = 709 ] && [ "$failed_count" = 0 ] || fail 'header fixed count mismatch'
canonical_decimal "$chunk_count" && canonical_decimal "$decoded_total" || fail 'header totals are not canonical'
[ "$chunk_count" -le 206741 ] && [ "$decoded_total" -le 296816640 ] || fail 'header total bound exceeded'
case "$verdict" in mechanically-verified-not-reviewed-not-ready|normalized-not-ready) ;; *) fail 'verdict invalid' ;; esac
[ "${#header}" -le 8192 ] || fail 'header oversized'

blob_seen=0
chunk_seen=0
decoded_seen=0
normalized_seen=0
current=
current_kind=
current_case=
current_decoded=
current_chunks=
current_sha=
current_chunk_seen=0
success_one_sha=
success_two_sha=
success_one_bytes=
success_two_bytes=

finalize_blob() {
    [ -n "$current" ] || return 0
    [ "$current_chunk_seen" -eq "$current_chunks" ] || fail 'blob chunk count mismatch'
    /usr/bin/base64 --decode "$encoded" > "$decoded" 2>/dev/null || fail 'Base64 decode failed'
    actual_bytes=$(size_file "$decoded")
    actual_sha=$(sha_file "$decoded")
    [ "$actual_bytes" -eq "$current_decoded" ] || fail 'blob decoded length mismatch'
    [ "$actual_sha" = "$current_sha" ] || fail 'blob digest mismatch'
    printf '%s|%s|%s|%s|%s\n' "$current" "$current_kind" "$current_case" "$actual_bytes" "$actual_sha" >> "$ledger"
    case "$current" in
        B000001) success_one_sha=$actual_sha; success_one_bytes=$actual_bytes ;;
        B000002) success_two_sha=$actual_sha; success_two_bytes=$actual_bytes ;;
    esac
    /bin/rm -f -- "$encoded" "$decoded"
    current=
}

while IFS= read -r line; do
    [ "$line" != "$header" ] || continue
    prefix=${line%%|*}
    case "$prefix" in
        B)
            [ "$normalized_seen" -eq 0 ] || fail 'blob follows normalized records'
            finalize_blob
            oldifs=$IFS
            IFS='|' read -r btag bid bkind bcase bbytes bchunks bsha bextra <<EOF
$line
EOF
            IFS=$oldifs
            [ -z "${bextra-}" ] || fail 'blob header has extra field'
            blob_seen=$((blob_seen+1))
            expected=$(/usr/bin/sed -n "${blob_seen}p" "$plan")
            [ "$bid|$bkind|$bcase" = "$expected" ] || fail 'blob allocation/order mismatch'
            canonical_decimal "$bbytes" || [ "$bbytes" = 0 ] || fail 'blob byte count malformed'
            canonical_decimal "$bchunks" || fail 'blob chunk count malformed'
            [ "$bbytes" -le 16777216 ] && [ "$bchunks" -le 206741 ] || fail 'blob bound exceeded'
            nonzero_sha "$bsha" || fail 'blob digest malformed'
            current=$bid; current_kind=$bkind; current_case=$bcase
            current_decoded=$bbytes; current_chunks=$bchunks; current_sha=$bsha
            current_chunk_seen=0
            : > "$encoded"
            ;;
        C)
            [ -n "$current" ] || fail 'chunk without open blob'
            oldifs=$IFS
            IFS='|' read -r ctag cbid cid payload cextra <<EOF
$line
EOF
            IFS=$oldifs
            [ -z "${cextra-}" ] || fail 'chunk has extra field'
            current_chunk_seen=$((current_chunk_seen+1)); chunk_seen=$((chunk_seen+1))
            expected_cid=$(/usr/bin/printf 'C%06d' "$current_chunk_seen")
            [ "$cbid" = "$current" ] && [ "$cid" = "$expected_cid" ] || fail 'chunk identity/order mismatch'
            [ "${#payload}" -le 1916 ] || fail 'chunk payload oversized'
            case "$payload" in ''|*[!A-Za-z0-9+/=]*) fail 'Base64 alphabet invalid' ;; esac
            if [ "$current_chunk_seen" -lt "$current_chunks" ]; then
                [ "${#payload}" -eq 1912 ] || fail 'nonfinal chunk encoded length invalid'
                case "$payload" in *=*) fail 'nonfinal chunk is padded' ;; esac
            else
                [ $((${#payload}%4)) -eq 0 ] || fail 'final Base64 length invalid'
                case "$payload" in *===*|*=*=*|=*|*==?*) fail 'Base64 padding invalid' ;; esac
            fi
            printf '%s' "$payload" >> "$encoded"
            ;;
        N)
            finalize_blob
            oldifs=$IFS
            IFS='|' read -r ntag ordinal case_id kind result raw_ids normalized_sha nextra <<EOF
$line
EOF
            IFS=$oldifs
            [ -z "${nextra-}" ] || fail 'normalized record has extra field'
            normalized_seen=$((normalized_seen+1))
            [ "$ordinal" = "$(/usr/bin/printf '%03d' "$normalized_seen")" ] || fail 'normalized ordinal mismatch'
            expected_case=$(/usr/bin/awk -F '|' -v n="$normalized_seen" '/^case\|/ { i++; if(i==n){print $2"|"$3; exit}}' "$cases")
            [ "$case_id|$kind" = "$expected_case" ] || fail 'normalized case/kind mismatch'
            [ "$result" = pass ] || fail 'normalized result is not pass'
            expected_ids=$(/usr/bin/awk -F '|' -v c="$case_id" '$3==c { if(out!="")out=out","; out=out $1 } END{print out}' "$ledger")
            [ "$raw_ids" = "$expected_ids" ] || fail 'normalized raw blob references mismatch'
            projection=$(/usr/bin/awk -F '|' -v c="$case_id" -v k="$kind" 'BEGIN{printf "%s|%s|pass",c,k} $3==c {printf "|%s:%s",$1,$5} END{printf "\n"}' "$ledger")
            derived=$(printf '%s' "$projection" | /usr/bin/sha256sum | /usr/bin/awk '{print $1}')
            [ "$normalized_sha" = "$derived" ] || fail 'normalized digest is not derived from retained raw blobs'
            ;;
        *) fail 'unknown record type' ;;
    esac
done < "$evidence"
finalize_blob
[ "$blob_seen" -eq 709 ] && [ "$chunk_seen" -eq "$chunk_count" ] && [ "$normalized_seen" -eq 209 ] || fail 'final record count mismatch'
decoded_sum=$(/usr/bin/awk -F '|' '{sum+=$4} END{print sum+0}' "$ledger")
[ "$decoded_sum" -eq "$decoded_total" ] || fail 'decoded total mismatch'
[ "$success_one_bytes" = "$success_two_bytes" ] && [ "$success_one_sha" = "$success_two_sha" ] || fail 'clean-success passes differ'
printf '%s\n' 'controller-helper closure verifier evidence validated: mechanically-verified-not-reviewed-not-ready'
