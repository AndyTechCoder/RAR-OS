#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
work=$root/.generate-tmp
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
/bin/rm -rf "$work"
/bin/mkdir -m 700 "$work"

hex_to_bin() { /usr/bin/printf '%s' "$1" | /usr/bin/xxd -r -p; }
u16() { /usr/bin/printf '%02x%02x' "$(( $1 & 255 ))" "$(( ($1 >> 8) & 255 ))" | /usr/bin/xxd -r -p; }
u32() { /usr/bin/printf '%02x%02x%02x%02x' "$(( $1 & 255 ))" "$(( ($1 >> 8) & 255 ))" "$(( ($1 >> 16) & 255 ))" "$(( ($1 >> 24) & 255 ))" | /usr/bin/xxd -r -p; }
zeros() { /bin/dd if=/dev/zero bs=1 count="$1" 2>/dev/null; }
digest() { /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }

/usr/bin/printf 'abc' > "$work/input"
hex_to_bin ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad > "$work/target-output"
/bin/cat "$work/input" "$work/target-output" > "$work/payload"
payload_sha=$(digest "$work/payload")
target_output_sha=$(digest "$work/target-output")

transcript=$root/comparison-transcript.v0
{
    hex_to_bin 524152434d500000
    u16 0; u16 0; u16 32; u16 64; u32 131; u32 1; u32 0; u32 0
    u16 1; u16 0; u32 1; u32 96; u32 3; u32 99; u16 32; u16 0
    hex_to_bin "$payload_sha"; zeros 8
    /bin/cat "$work/input" "$work/target-output"
} > "$transcript"

transcript_sha=$(digest "$transcript")
inventory_sha=$(digest "$root/reference-inventory.v0")
harness_sha=$(digest "$root/reference-harness.v0")
evidence=$root/comparison-evidence.v0
{
    hex_to_bin 5241525245464556
    u16 0; u16 0; u16 128; u16 256; u32 384; u32 1; u32 0; u32 0
    hex_to_bin "$transcript_sha"; hex_to_bin "$inventory_sha"; hex_to_bin "$harness_sha"
    u32 1; u16 1; u16 0; u16 0; u16 0; u16 32; u16 0
    /bin/cat "$work/target-output"; zeros 32
    /bin/cat "$work/target-output"; zeros 32
    hex_to_bin "$target_output_sha"; hex_to_bin "$target_output_sha"; hex_to_bin "$target_output_sha"; zeros 16
} > "$evidence"

controller_sha=$(digest "$root/controller-context.v0")
source_sha=$(digest "$root/source-context.v0")
evidence_sha=$(digest "$evidence")
accepted=$root/reference-verdict-accepted.v0
/usr/bin/printf '%s\n' \
    schema=rar-alpha-reference-verdict-v0 \
    status=accepted \
    probe=milestone-f \
    controller_sha256="$controller_sha" \
    source_sha256="$source_sha" \
    transcript_sha256="$transcript_sha" \
    reference_inventory_sha256="$inventory_sha" \
    comparison_evidence_sha256="$evidence_sha" \
    record_count=1 \
    reference_1_result=match \
    reference_2_result=match \
    target_result=match \
    reason=all-three-match > "$accepted"

not_required=$root/reference-verdict-not-required.v0
/usr/bin/printf '%s\n' \
    schema=rar-alpha-reference-verdict-v0 \
    status=not-required \
    probe=milestone-a \
    controller_sha256="$controller_sha" \
    source_sha256="$source_sha" \
    transcript_sha256="$transcript_sha" \
    reference_inventory_sha256=0000000000000000000000000000000000000000000000000000000000000000 \
    comparison_evidence_sha256=0000000000000000000000000000000000000000000000000000000000000000 \
    record_count=0 \
    reference_1_result=not-run \
    reference_2_result=not-run \
    target_result=not-evaluated \
    reason=probe-does-not-require-reference > "$not_required"

printf '%s\n' 'reference evidence fixtures generated'
