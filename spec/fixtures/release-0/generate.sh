#!/bin/sh
set -eu

destination=${1-}
[ -n "$destination" ] && [ -d "$destination" ] || {
    echo "usage: generate.sh EXISTING-DIRECTORY" >&2
    exit 64
}

byte() { octal=$(printf '%03o' "$1"); printf "\\$octal"; }
u16() { value=$1; byte $((value % 256)); byte $(((value / 256) % 256)); }
u32() { value=$1; byte $((value % 256)); byte $(((value / 256) % 256)); byte $(((value / 65536) % 256)); byte $(((value / 16777216) % 256)); }
u64() { value=$1; u32 $((value % 4294967296)); u32 $((value / 4294967296)); }
zeros() { remaining=$1; while [ "$remaining" -gt 0 ]; do byte 0; remaining=$((remaining - 1)); done; }

record_header() { u16 "$1"; u16 "$2"; u32 "$3"; u32 "$4"; u32 0; }

descriptor() {
    u64 "$1"; u64 "$2"; u16 "$3"; u16 "$4"; u16 1; u16 "$5"; u16 "$6"; u16 "$7"; u32 "$8"
}

emit_entry_header() {
    printf 'RARENTRY'
    u16 1; u16 0; u16 64; u16 32
    u32 "$entry_total"; u16 "$entry_arch"; byte "$entry_address_bits"; byte 0
    u16 "$descriptor_count_field"; u16 0; u32 0; zeros 32
}

emit_entry() {
    if [ "$entry_blob_length" -eq 63 ]; then
        printf 'RARENTRY'
        u16 1; u16 0; u16 64; u16 32
        u32 63; u16 "$entry_arch"; byte 48; byte 0
        u16 0; u16 0; u32 0; zeros 31
        return
    fi
    emit_entry_header
    descriptor 4096 128 1 1 1 0 3 0
    if [ "$overlap_entry" -eq 1 ]; then map_address=4096; else map_address=8192; fi
    descriptor "$map_address" "$map_length" 2 1 1 0 3 0
    descriptor 12288 "$rhd_length" 3 1 1 0 3 0
    descriptor 16384 32 4 1 1 0 3 0
    descriptor 20480 4096 5 3 2 0 3 0
    if [ "$entry_arch" -eq 1 ]; then
        descriptor "$apic_descriptor_base" 4096 6 11 3 3 0 1
        descriptor 1016 8 7 11 3 5 0 1
    else
        descriptor 134217728 65536 6 11 3 3 0 1
        descriptor 134873088 16121856 6 11 3 3 0 1
        descriptor 150994944 4096 6 11 3 5 0 1
    fi
    if [ "$entry_blob_length" -eq 4097 ]; then zeros 3809; fi
}

memory_entry() {
    u64 "$1"; u64 "$2"; u16 "$3"; u16 "$4"; u16 "$5"; u16 0; u32 "$6"; u32 0
}

emit_map() {
    memory_entry 4096 "$boot_length" 3 11 1 1
    memory_entry 1048576 524288 1 11 0 2
    if [ "$entry_arch" -eq 1 ]; then
        memory_entry 4276092928 4096 5 19 5 3
    else
        memory_entry 134217728 65536 5 19 5 3
        memory_entry 134873088 16121856 5 19 5 4
        memory_entry 150994944 4096 5 19 5 5
    fi
}

emit_handoff() {
    printf 'RARBOOT\000'
    u16 1; u16 0; u16 128; u16 0; u32 128
    u16 "$entry_arch"; byte 48; byte 12; u16 1; u16 0; u32 0
    u64 8192; u32 "$map_count"; u16 32; u16 1
    u64 12288; u32 "$rhd_length"; u16 8; u16 0
    u64 16384; u32 32; u32 0
    u32 1; u16 1; u16 0; u64 20480; u32 4096; u32 0
    u32 1; u32 0; zeros 16
}

emit_memory_record() {
    record_header 2 0 48 "$1"
    u64 "$2"; u64 "$3"; u16 "$4"; u16 "$5"; u16 "$6"; u16 0; zeros 8
}

emit_window() {
    record_header 7 0 56 "$1"
    u16 "$2"; u16 "$3"; u32 "$4"; u16 "$5"; byte "$6"; byte 1
    u16 "$7"; u16 1; u64 "$8"; u64 "$9"; u16 "${10}"; u16 0; u32 0
}

emit_rhd() {
    printf 'RARRHD\000\000'
    u16 1; u16 "$rhd_minor"; u16 32; u16 16; u32 "$rhd_length"; u16 "$record_count"; u16 0
    u16 "$rhd_arch"; byte 48; byte 12; u32 0

    record_header 1 0 48 1
    u32 1; u32 0; u64 16; u32 "$cpu_interrupt_id"; u32 1; zeros 8

    record_header 1 0 48 2
    u32 0; u32 0; u64 "$cpu2_hardware_id"; u32 1; u32 1; zeros 8

    emit_memory_record 1 4096 "$boot_length" 3 11 1
    emit_memory_record 2 "$rhd_usable_base" 524288 1 11 0
    if [ "$entry_arch" -eq 1 ]; then
        emit_memory_record 3 4276092928 4096 5 19 5
    else
        emit_memory_record 3 134217728 65536 5 19 5
        emit_memory_record 4 134873088 16121856 5 19 5
        emit_memory_record 5 150994944 4096 5 19 5
    fi

    record_header 3 0 48 1
    if [ "$entry_arch" -eq 1 ]; then u16 1; else u16 2; fi
    u16 0; u32 "$interrupt_base"; u32 "$interrupt_count"; zeros 20

    record_header 4 0 48 1
    if [ "$entry_arch" -eq 1 ]; then u16 1; else u16 2; fi
    u16 0; u32 0; u64 100000000; u32 30; u32 1; byte 1; byte 1; zeros 6

    record_header 5 0 48 1
    if [ "$entry_arch" -eq 1 ]; then u16 1; else u16 2; fi
    u16 0; u32 "$serial_interrupt_index"; u32 1; u32 0; u64 1843200; byte 1; byte 1; zeros 6

    if [ "$unknown_critical" -eq 1 ]; then record_header 99 1 24 1; zeros 8
    else record_header 6 0 24 1; u16 1; u16 0; u32 0
    fi

    if [ "$entry_arch" -eq 1 ]; then
        emit_window 1 3 1 1 "$apic_space" 4 "$apic_stride" 4276092928 4096 "$apic_authority_purpose"
        emit_window "$second_window_id" 5 5 1 2 1 1 1016 8 7
    else
        emit_window 1 3 2 1 1 4 4 134217728 65536 6
        emit_window "$second_window_id" 3 3 1 1 4 4 134873088 16121856 6
        emit_window 3 5 5 1 1 4 4 150994944 4096 6
    fi
    if [ "$optional_window_role" -eq 1 ]; then
        record_header 7 "$optional_window_critical" 56 99
        u16 0; u16 99; u32 0; zeros 32
    fi
}

emit_bundle() {
    printf 'R0FXBIN\000'; u16 1; u16 "$expected_code"
    u32 "$entry_blob_length"; u32 128; u32 "$map_length"; u32 "$rhd_length"; u32 0; u32 "$copy_fault"
    u16 "$entry_arch"; byte 48; byte 0; u64 268435456; u64 4096; u16 8; u16 16; u32 0
    emit_entry; emit_handoff; emit_map; emit_rhd
}

for case_id in \
    valid-x86_64 valid-aarch64 truncated-entry oversized-entry misaligned-window \
    overlapping-entry unknown-critical duplicate-id bad-reference invalid-memory-map \
    interrupt-out-of-range invalid-entry snapshot-violation invalid-register-window \
    wrong-address-space unauthorized-device-window map-rhd-inconsistent architecture-inconsistent \
    entry-address-bits-65 duplicate-cpu-hardware interrupt-overflow compatible-window-role critical-window-role; do
    expected_code=0; entry_arch=1; rhd_arch=1; entry_blob_length=288; entry_total=288
    descriptor_count_field=7; map_count=3; map_length=96; rhd_length=552; record_count=11
    copy_fault=0; overlap_entry=0; apic_descriptor_base=4276092928
    boot_length=20480; rhd_usable_base=1048576; unknown_critical=0; cpu_interrupt_id=1
    serial_interrupt_index=4; apic_space=1; apic_stride=16; apic_authority_purpose=6
    second_window_id=2
    entry_address_bits=48; rhd_minor=0; cpu2_hardware_id=17
    interrupt_base=32; interrupt_count=224; optional_window_role=0; optional_window_critical=0
    case "$case_id" in
        valid-aarch64)
            entry_arch=2; rhd_arch=2; entry_blob_length=320; entry_total=320
            descriptor_count_field=8; map_count=5; map_length=160; rhd_length=704; record_count=14
            ;;
        truncated-entry) expected_code=1; entry_blob_length=63; entry_total=63; descriptor_count_field=0 ;;
        oversized-entry) expected_code=2; entry_blob_length=4097; entry_total=4097 ;;
        misaligned-window) expected_code=12; apic_descriptor_base=4276092929 ;;
        overlapping-entry) expected_code=13; overlap_entry=1 ;;
        unknown-critical) expected_code=15; unknown_critical=1 ;;
        duplicate-id) expected_code=16; second_window_id=1 ;;
        bad-reference) expected_code=17; cpu_interrupt_id=99 ;;
        invalid-memory-map) expected_code=20; boot_length=0 ;;
        interrupt-out-of-range) expected_code=24; serial_interrupt_index=224 ;;
        invalid-entry) expected_code=30; descriptor_count_field=6 ;;
        snapshot-violation) expected_code=31; copy_fault=1 ;;
        invalid-register-window) expected_code=32; apic_stride=4 ;;
        wrong-address-space) expected_code=32; apic_space=2 ;;
        unauthorized-device-window) expected_code=33; apic_authority_purpose=7 ;;
        map-rhd-inconsistent) expected_code=29; rhd_usable_base=1572864 ;;
        architecture-inconsistent) expected_code=18; rhd_arch=2 ;;
        entry-address-bits-65) expected_code=30; entry_address_bits=65 ;;
        duplicate-cpu-hardware) expected_code=21; cpu2_hardware_id=16 ;;
        interrupt-overflow) expected_code=22; interrupt_base=4294967295; interrupt_count=2 ;;
        compatible-window-role) rhd_minor=1; rhd_length=608; record_count=12; optional_window_role=1 ;;
        critical-window-role) expected_code=15; rhd_minor=1; rhd_length=608; record_count=12; optional_window_role=1; optional_window_critical=1 ;;
    esac
    emit_bundle > "$destination/$case_id.bin"
done
