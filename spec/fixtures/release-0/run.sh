#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
fixtures=$root/spec/fixtures/release-0/cases.v1
failures=0
count=0
x86_semantic=
arm_semantic=

validate() {
    if [ "$handoff_bytes" -lt 128 ]; then result=truncated
    elif [ "$handoff_total" -gt 4096 ] || [ "$rhd_total" -gt 65536 ] || [ "$rhd_bytes" -gt 65536 ]; then result=oversized
    elif [ $((rhd_address % 8)) -ne 0 ] || [ $((record_offset % 8)) -ne 0 ]; then result=bad-alignment
    elif [ "$rhd_address" -lt "$trusted_base" ] || [ "$rhd_total" -gt $((trusted_base + trusted_size - rhd_address)) ]; then result=invalid-pointer-range
    elif [ "$unknown_kind" -ne 0 ] && [ $((unknown_flags & 1)) -ne 0 ]; then result=unknown-critical
    elif [ "$memory_a_base" -lt $((memory_b_base + memory_b_length)) ] && [ "$memory_b_base" -lt $((memory_a_base + memory_a_length)) ]; then result=overlap
    elif [ "$handoff_arch" -ne "$rhd_arch" ]; then result=architecture-mismatch
    else result=ok
    fi
}

while IFS='|' read -r id expected handoff_bytes handoff_total handoff_arch rhd_arch rhd_bytes rhd_total rhd_address trusted_base trusted_size record_offset unknown_kind unknown_flags memory_a_base memory_a_length memory_b_base memory_b_length semantic_id; do
    case "$id" in
        schema=*) [ "$id" = schema=rar-r0-contract-fixtures-v1 ] || { echo "fixture schema mismatch" >&2; exit 1; }; continue ;;
        id|'') continue ;;
    esac
    count=$((count + 1))
    validate
    if [ "$result" != "$expected" ]; then
        echo "$id: expected $expected, got $result" >&2
        failures=$((failures + 1))
    fi
    case "$id" in
        valid-x86_64) x86_semantic=$semantic_id ;;
        valid-aarch64) arm_semantic=$semantic_id ;;
    esac
done < "$fixtures"

[ "$count" -ge 12 ] || { echo "fixture corpus is incomplete" >&2; exit 1; }
[ -n "$x86_semantic" ] && [ "$x86_semantic" = "$arm_semantic" ] || { echo "architecture semantic fixtures diverge" >&2; exit 1; }
[ "$failures" -eq 0 ] || exit 1
echo "R0-002 conformance passed: $count fixtures"
