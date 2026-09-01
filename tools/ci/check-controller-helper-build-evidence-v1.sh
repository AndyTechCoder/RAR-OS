#!/bin/sh
set -eu
LC_ALL=C LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=$root/spec/alpha/lab/controller-helper-build-evidence-v1.fields
fail(){ printf '%s\n' "controller-helper build evidence v1 failed: $1" >&2; exit 1; }
sha(){ env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{print $1}'; }
line(){ [ "$(grep -Fxc "$2" "$1")" -eq 1 ] || fail "missing or duplicate field: $2"; }
is_sha(){ printf '%s\n' "$1" | grep -Eq '^[0-9a-f]{64}$' && [ "$1" != 0000000000000000000000000000000000000000000000000000000000000000 ]; }
[ -f "$contract" ] && [ ! -L "$contract" ] || fail 'contract unavailable'
[ "$(sha "$contract")" = c2d174bf2d793079f98a7467f4b7bd12ceddc00e0c67d247a4b0531f66999f97 ] || fail 'contract bytes escaped review'
line "$contract" 'schema=rar-alpha-controller-helper-build-evidence-v1'
line "$contract" 'test_rule=consume-exactly-one-controller-helper-test-evidence-v1-instance,127-cases,failed-count-0,no-v0-or-13-case-substitution'
[ "$#" -ne 0 ] || { printf '%s\n' 'controller-helper build evidence v1 contract is byte-bound'; exit 0; }
[ "$#" -eq 11 ] || fail 'usage: evidence controller source acceptance compiler closure test-evidence-sha build-plan golden receipt-1 receipt-2'
evidence=$1 controller=$2 source=$3 acceptance=$4 compiler=$5 closure=$6 test_sha=$7 build_plan=$8 golden=$9 receipt_1=${10} receipt_2=${11}
[ -f "$evidence" ] && [ ! -L "$evidence" ] || fail 'evidence unavailable'
[ "$(wc -c < "$evidence" | tr -d ' ')" -le 65536 ] || fail 'evidence oversized'
[ "$(wc -l < "$evidence" | tr -d ' ')" -eq 27 ] || fail 'evidence line count changed'
expected_fields=$(printf '%s\n' schema controller_sha source_sha closure_acceptance_sha compiler_sha compiler_closure_sha test_evidence_v1_sha build_plan_sha golden_sha build_1_receipt_sha build_2_receipt_sha build_1_log_sha build_2_log_sha build_count network credential build_1_exit build_2_exit build_1_binary_sha build_2_binary_sha final_binary_sha build_1_job_nonce build_2_job_nonce build_1_root build_2_root binary_bytes status)
actual_fields=$(/usr/bin/awk -F '=' '{ print $1 }' "$evidence")
[ "$actual_fields" = "$expected_fields" ] || fail 'fields are missing, extra, duplicated, or reordered'
for value in "$controller" "$source" "$acceptance" "$compiler" "$closure" "$test_sha" "$build_plan" "$golden" "$receipt_1" "$receipt_2"; do is_sha "$value" || fail 'zero or malformed context identity'; done
line "$evidence" 'schema=rar-alpha-controller-helper-build-evidence-v1'
line "$evidence" "controller_sha=$controller"; line "$evidence" "source_sha=$source"
line "$evidence" "closure_acceptance_sha=$acceptance"; line "$evidence" "compiler_sha=$compiler"
line "$evidence" "compiler_closure_sha=$closure"; line "$evidence" "test_evidence_v1_sha=$test_sha"
line "$evidence" "build_plan_sha=$build_plan"; line "$evidence" "golden_sha=$golden"
line "$evidence" "build_1_receipt_sha=$receipt_1"; line "$evidence" "build_2_receipt_sha=$receipt_2"
[ "$receipt_1" != "$receipt_2" ] || fail 'build receipts alias'
line "$evidence" 'build_count=2'; line "$evidence" 'network=none'; line "$evidence" 'credential=none'
line "$evidence" 'build_1_exit=0'; line "$evidence" 'build_2_exit=0'; line "$evidence" 'status=accepted'
b1=$(sed -n 's/^build_1_binary_sha=//p' "$evidence"); b2=$(sed -n 's/^build_2_binary_sha=//p' "$evidence"); final=$(sed -n 's/^final_binary_sha=//p' "$evidence")
is_sha "$b1" && [ "$b1" = "$b2" ] && [ "$b1" = "$final" ] || fail 'outputs are not byte-identical'
j1=$(sed -n 's/^build_1_job_nonce=//p' "$evidence"); j2=$(sed -n 's/^build_2_job_nonce=//p' "$evidence"); r1=$(sed -n 's/^build_1_root=//p' "$evidence"); r2=$(sed -n 's/^build_2_root=//p' "$evidence")
for value in "$j1" "$j2" "$r1" "$r2"; do is_sha "$value" || fail 'fresh build identity malformed'; done
[ "$j1" != "$j2" ] && [ "$r1" != "$r2" ] && [ "$j1" != "$r1" ] && [ "$j2" != "$r2" ] || fail 'build identities replayed or aliased'
log1=$(sed -n 's/^build_1_log_sha=//p' "$evidence"); log2=$(sed -n 's/^build_2_log_sha=//p' "$evidence")
is_sha "$log1" && is_sha "$log2" && [ "$log1" != "$log2" ] || fail 'complete distinct build logs are not bound'
size=$(sed -n 's/^binary_bytes=//p' "$evidence"); case "$size" in ''|*[!0-9]*) fail 'binary size malformed';; esac
[ "$size" -ge 1 ] && [ "$size" -le 16777216 ] || fail 'binary size outside bound'
printf '%s\n' 'controller-helper build evidence v1 accepted'
