#!/bin/sh
set -eu

destination=${1-}
[ -n "$destination" ] && [ -d "$destination" ] || {
    echo "usage: generate.sh EXISTING-DIRECTORY" >&2
    exit 64
}

byte() {
    octal=$(printf '%03o' "$1")
    printf "\\$octal"
}

u16() {
    value=$1
    byte $((value % 256))
    byte $(((value / 256) % 256))
}

u32() {
    value=$1
    byte $((value % 256))
    byte $(((value / 256) % 256))
    byte $(((value / 65536) % 256))
    byte $(((value / 16777216) % 256))
}

u64() {
    value=$1
    u32 $((value % 4294967296))
    u32 $((value / 4294967296))
}

zeros() {
    remaining=$1
    while [ "$remaining" -gt 0 ]; do
        byte 0
        remaining=$((remaining - 1))
    done
}

record_header() {
    u16 "$1"
    u16 "$2"
    u32 "$3"
    u32 "$4"
    u32 0
}

emit_map() {
    u64 1048576
    u64 524288
    u16 1
    u16 3
    u16 0
    u16 0
    u32 1
    u32 0

    if [ "$case_id" = overlapping-memory ]; then second_base=1310720; else second_base=2097152; fi
    u64 "$second_base"
    u64 524288
    u16 1
    u16 3
    u16 0
    u16 0
    u32 2
    u32 0
}

emit_handoff() {
    printf 'RARBOOT\000'
    u16 1
    u16 0
    u16 128
    u16 0
    u32 "$handoff_total"
    u16 "$handoff_arch"
    byte 48
    byte 12
    u16 1
    u16 0
    u32 0
    u64 8192
    u32 2
    u16 32
    u16 1
    u64 "$rhd_address"
    u32 "$rhd_claimed"
    u16 8
    u16 0
    u64 16384
    u32 32
    u32 0
    u32 1
    u16 1
    u16 0
    u64 20480
    u32 4096
    u32 0
    u32 1
    u32 0
    if [ "$handoff_length" -eq 128 ]; then zeros 16; else zeros 15; fi
}

emit_memory_record() {
    record_header 2 0 48 "$1"
    u64 "$2"
    u64 524288
    u16 1
    u16 3
    u16 0
    u16 0
    u32 "$1"
    u32 0
}

emit_rhd() {
    printf 'RARRHD\000\000'
    u16 1
    u16 0
    u16 32
    u16 16
    u32 "$rhd_total"
    u16 "$record_count"
    u16 0
    u16 "$rhd_arch"
    byte 48
    byte 12
    u32 0

    record_header 1 0 "$cpu_record_bytes" 1
    u32 1
    u32 1
    u64 16
    u32 1
    u32 1
    zeros 8

    emit_memory_record 1 1048576
    if [ "$case_id" = overlapping-memory ]; then second_base=1310720; else second_base=2097152; fi
    emit_memory_record 2 "$second_base"

    record_header 3 0 48 1
    u32 1
    if [ "$rhd_arch" -eq 2 ]; then u16 2; else u16 1; fi
    u16 0
    if [ "$rhd_arch" -eq 2 ]; then u64 134217728; else u64 4276092928; fi
    u32 4096
    u32 32
    u32 224
    u32 0

    record_header 4 0 48 1
    u32 1
    if [ "$rhd_arch" -eq 2 ]; then u16 2; else u16 1; fi
    u16 0
    u64 100000000
    u64 0
    u32 30
    u32 1

    record_header 5 0 48 1
    u32 1
    if [ "$rhd_arch" -eq 2 ]; then u16 2; else u16 1; fi
    u16 0
    if [ "$rhd_arch" -eq 2 ]; then u64 150994944; else u64 1016; fi
    u16 8
    u16 4
    u32 1
    u64 1843200

    record_header 6 0 24 1
    u16 1
    u16 0
    u32 1

    if [ "$case_id" = unknown-critical ]; then
        record_header 99 1 16 1
    elif [ "$case_id" = valid-max-rhd ]; then
        record_header 99 0 65192 1
        zeros 65176
    fi
}

emit_bundle() {
    printf 'R0FXBIN\000'
    u16 1
    u16 "$expected_code"
    u32 "$handoff_length"
    u32 64
    u32 "$rhd_blob_length"
    u32 0
    u64 4096
    u64 "$trusted_size"
    emit_handoff
    emit_map
    emit_rhd
}

for case_id in \
    valid-x86_64 valid-aarch64 valid-max-rhd truncated-handoff \
    oversized-handoff oversized-rhd misaligned-rhd misaligned-record \
    invalid-pointer unknown-critical overlapping-memory architecture-inconsistent; do
    expected_code=0
    handoff_length=128
    handoff_total=128
    handoff_arch=1
    rhd_arch=1
    rhd_address=12288
    rhd_total=344
    rhd_claimed=344
    rhd_blob_length=344
    record_count=7
    cpu_record_bytes=48
    trusted_size=65536

    case "$case_id" in
        valid-aarch64) handoff_arch=2; rhd_arch=2 ;;
        valid-max-rhd)
            rhd_total=65536; rhd_claimed=65536; rhd_blob_length=65536
            record_count=8; trusted_size=131072
            ;;
        truncated-handoff) expected_code=1; handoff_length=127 ;;
        oversized-handoff) expected_code=2; handoff_total=4097 ;;
        oversized-rhd)
            expected_code=2; rhd_total=65544; rhd_claimed=65544
            ;;
        misaligned-rhd) expected_code=9; rhd_address=12289 ;;
        misaligned-record) expected_code=9; cpu_record_bytes=49 ;;
        invalid-pointer) expected_code=12; rhd_address=69632 ;;
        unknown-critical)
            expected_code=15; rhd_total=360; rhd_claimed=360
            rhd_blob_length=360; record_count=8
            ;;
        overlapping-memory) expected_code=13 ;;
        architecture-inconsistent) expected_code=18; rhd_arch=2 ;;
    esac

    emit_bundle > "$destination/$case_id.bin"
done
