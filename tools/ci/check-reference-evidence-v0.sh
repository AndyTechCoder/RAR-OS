#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

evidence=${1-}
transcript=${2-}
inventory=${3-}
harness=${4-}
fail() { printf 'reference evidence rejected: %s\n' "$1" >&2; exit 1; }

for file in "$evidence" "$transcript" "$inventory" "$harness"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
done

size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }
hex_at() { /usr/bin/od -An -tx1 -v -j "$2" -N "$3" "$1" | /usr/bin/tr -d ' \n'; }
u16_at() {
    bytes=$(/usr/bin/od -An -tu1 -j "$2" -N 2 "$1"); set -- $bytes
    [ "$#" -eq 2 ] || fail 'truncated u16'
    printf '%s' "$(( $1 + ($2 << 8) ))"
}
u32_at() {
    bytes=$(/usr/bin/od -An -tu1 -j "$2" -N 4 "$1"); set -- $bytes
    [ "$#" -eq 4 ] || fail 'truncated u32'
    printf '%s' "$(( $1 + ($2 << 8) + ($3 << 16) + ($4 << 24) ))"
}
sha_file() { /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
sha_slice() { /bin/dd if="$1" bs=1 skip="$2" count="$3" 2>/dev/null | /usr/bin/shasum -a 256 | /usr/bin/awk '{ print $1 }'; }
require_zero() { case "$1" in '' | *[!0]*) fail "$2 is nonzero or truncated" ;; esac; }

transcript_size=$(size_of "$transcript")
[ "$transcript_size" -ge 96 ] && [ "$transcript_size" -le 1048576 ] || fail 'transcript size is outside bounds'
[ "$(hex_at "$transcript" 0 8)" = 524152434d500000 ] || fail 'transcript magic mismatch'
[ "$(u16_at "$transcript" 8)" -eq 0 ] && [ "$(u16_at "$transcript" 10)" -eq 0 ] || fail 'transcript version mismatch'
[ "$(u16_at "$transcript" 12)" -eq 32 ] && [ "$(u16_at "$transcript" 14)" -eq 64 ] || fail 'transcript fixed size mismatch'
[ "$(u32_at "$transcript" 16)" -eq "$transcript_size" ] || fail 'transcript total size mismatch'
record_count=$(u32_at "$transcript" 20)
[ "$record_count" -ge 1 ] && [ "$record_count" -le 512 ] || fail 'transcript record count invalid'
[ "$(u32_at "$transcript" 24)" -eq 0 ] && [ "$(u32_at "$transcript" 28)" -eq 0 ] || fail 'transcript flags or reserved bytes nonzero'
transcript_table_end=$((32 + record_count * 64))
[ "$transcript_table_end" -le "$transcript_size" ] || fail 'transcript table exceeds file'

evidence_size=$(size_of "$evidence")
[ "$evidence_size" -ge 384 ] && [ "$evidence_size" -le 1048576 ] || fail 'evidence size is outside bounds'
[ "$(hex_at "$evidence" 0 8)" = 5241525245464556 ] || fail 'evidence magic mismatch'
[ "$(u16_at "$evidence" 8)" -eq 0 ] && [ "$(u16_at "$evidence" 10)" -eq 0 ] || fail 'evidence version mismatch'
[ "$(u16_at "$evidence" 12)" -eq 128 ] && [ "$(u16_at "$evidence" 14)" -eq 256 ] || fail 'evidence fixed size mismatch'
[ "$(u32_at "$evidence" 16)" -eq "$evidence_size" ] || fail 'evidence total size or EOF mismatch'
evidence_count=$(u32_at "$evidence" 20)
[ "$evidence_count" -eq "$record_count" ] || fail 'evidence/transcript count mismatch'
[ "$evidence_size" -eq $((128 + evidence_count * 256)) ] || fail 'evidence checked size arithmetic mismatch'
[ "$(u32_at "$evidence" 24)" -eq 0 ] && [ "$(u32_at "$evidence" 28)" -eq 0 ] || fail 'evidence flags or reserved bytes nonzero'
transcript_sha=$(sha_file "$transcript")
inventory_sha=$(sha_file "$inventory")
harness_sha=$(sha_file "$harness")
[ "$(hex_at "$evidence" 32 32)" = "$transcript_sha" ] || fail 'evidence transcript binding mismatch'
[ "$(hex_at "$evidence" 64 32)" = "$inventory_sha" ] || fail 'evidence inventory binding mismatch'
[ "$(hex_at "$evidence" 96 32)" = "$harness_sha" ] || fail 'evidence harness binding mismatch'

previous_case=0
payload_cursor=$transcript_table_end
index=0
while [ "$index" -lt "$record_count" ]; do
    transcript_record=$((32 + index * 64))
    evidence_record=$((128 + index * 256))
    operation=$(u16_at "$transcript" "$transcript_record")
    case_id=$(u32_at "$transcript" $((transcript_record + 4)))
    [ "$case_id" -gt "$previous_case" ] || fail 'transcript case order is not strict'
    previous_case=$case_id
    [ "$(u16_at "$transcript" $((transcript_record + 2)))" -eq 0 ] || fail 'transcript record flags nonzero'
    require_zero "$(hex_at "$transcript" $((transcript_record + 56)) 8)" 'transcript record reserved bytes'
    input_offset=$(u32_at "$transcript" $((transcript_record + 8)))
    input_bytes=$(u32_at "$transcript" $((transcript_record + 12)))
    target_offset=$(u32_at "$transcript" $((transcript_record + 16)))
    target_bytes=$(u16_at "$transcript" $((transcript_record + 20)))
    target_status=$(u16_at "$transcript" $((transcript_record + 22)))
    [ "$input_offset" -eq "$payload_cursor" ] || fail 'transcript payload is not canonical contiguous input'
    [ "$target_offset" -eq $((input_offset + input_bytes)) ] || fail 'transcript target output does not follow input'
    [ $((target_offset + target_bytes)) -le "$transcript_size" ] || fail 'transcript payload exceeds file'
    [ "$input_bytes" -le 1060 ] || fail 'transcript input exceeds operation ceiling'
    case "$operation" in
        1) expected_output=32; [ "$input_bytes" -le 1024 ] || fail 'sha256 input exceeds bound' ;;
        2) expected_output=64; [ "$input_bytes" -le 1024 ] || fail 'sha512 input exceeds bound' ;;
        3) expected_output=32; [ "$input_bytes" -eq 32 ] || fail 'ed25519 public input size invalid' ;;
        4) expected_output=64; [ "$input_bytes" -ge 36 ] || fail 'ed25519 sign input truncated'; message_bytes=$(u32_at "$transcript" $((input_offset + 32))); [ "$message_bytes" -le 1024 ] && [ "$input_bytes" -eq $((36 + message_bytes)) ] || fail 'ed25519 sign message framing invalid' ;;
        5) expected_output=1; [ "$input_bytes" -ge 100 ] || fail 'ed25519 verify input truncated'; message_bytes=$(u32_at "$transcript" $((input_offset + 96))); [ "$message_bytes" -le 1024 ] && [ "$input_bytes" -eq $((100 + message_bytes)) ] || fail 'ed25519 verify message framing invalid' ;;
        *) fail 'unsupported transcript operation' ;;
    esac
    [ "$target_bytes" -eq "$expected_output" ] || fail 'target output size disagrees with operation'
    [ "$target_status" -eq 0 ] || [ "$target_status" -eq 1 ] || fail 'target status invalid'
    payload_sha=$(sha_slice "$transcript" "$input_offset" $((input_bytes + target_bytes)))
    [ "$(hex_at "$transcript" $((transcript_record + 24)) 32)" = "$payload_sha" ] || fail 'transcript payload hash mismatch'

    [ "$(u32_at "$evidence" "$evidence_record")" -eq "$case_id" ] || fail 'evidence case mismatch'
    [ "$(u16_at "$evidence" $((evidence_record + 4)))" -eq "$operation" ] || fail 'evidence operation mismatch'
    [ "$(u16_at "$evidence" $((evidence_record + 6)))" -eq "$target_status" ] || fail 'evidence target status mismatch'
    [ "$(u16_at "$evidence" $((evidence_record + 8)))" -eq "$target_status" ] || fail 'reference 1 status mismatch'
    [ "$(u16_at "$evidence" $((evidence_record + 10)))" -eq "$target_status" ] || fail 'reference 2 status mismatch'
    [ "$(u16_at "$evidence" $((evidence_record + 12)))" -eq "$target_bytes" ] || fail 'evidence output size mismatch'
    [ "$(u16_at "$evidence" $((evidence_record + 14)))" -eq 0 ] || fail 'evidence record flags nonzero'
    if [ "$target_bytes" -lt 64 ]; then
        require_zero "$(hex_at "$evidence" $((evidence_record + 16 + target_bytes)) $((64 - target_bytes)))" 'reference 1 output tail'
        require_zero "$(hex_at "$evidence" $((evidence_record + 80 + target_bytes)) $((64 - target_bytes)))" 'reference 2 output tail'
    fi
    require_zero "$(hex_at "$evidence" $((evidence_record + 240)) 16)" 'evidence record reserved bytes'
    target_output=$(hex_at "$transcript" "$target_offset" "$target_bytes")
    reference_1=$(hex_at "$evidence" $((evidence_record + 16)) "$target_bytes")
    reference_2=$(hex_at "$evidence" $((evidence_record + 80)) "$target_bytes")
    [ "$reference_1" = "$target_output" ] && [ "$reference_2" = "$target_output" ] || fail 'reference output disagreement'
    reference_1_sha=$(sha_slice "$evidence" $((evidence_record + 16)) "$target_bytes")
    reference_2_sha=$(sha_slice "$evidence" $((evidence_record + 80)) "$target_bytes")
    target_output_sha=$(sha_slice "$transcript" "$target_offset" "$target_bytes")
    [ "$(hex_at "$evidence" $((evidence_record + 144)) 32)" = "$reference_1_sha" ] || fail 'reference 1 output hash mismatch'
    [ "$(hex_at "$evidence" $((evidence_record + 176)) 32)" = "$reference_2_sha" ] || fail 'reference 2 output hash mismatch'
    [ "$(hex_at "$evidence" $((evidence_record + 208)) 32)" = "$target_output_sha" ] || fail 'target output hash mismatch'
    payload_cursor=$((target_offset + target_bytes))
    index=$((index + 1))
done
[ "$payload_cursor" -eq "$transcript_size" ] || fail 'transcript has trailing or unreferenced bytes'

printf 'reference evidence validated: records=%s\n' "$record_count"
