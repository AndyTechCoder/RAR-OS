#!/bin/sh
set -eu
LC_ALL=C LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=$root/spec/alpha/lab/controller-helper-test-evidence-v1.fields
cases=$root/spec/alpha/lab/controller-helper-runtime-cases.v0
fail(){ printf '%s\n' "controller-helper test evidence v1 failed: $1" >&2; exit 1; }
sha(){ env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{print $1}'; }
line(){ [ "$(grep -Fxc "$2" "$1")" -eq 1 ] || fail "missing or duplicate field: $2"; }
is_sha(){ printf '%s\n' "$1" | grep -Eq '^[0-9a-f]{64}$' && [ "$1" != 0000000000000000000000000000000000000000000000000000000000000000 ]; }
[ -f "$contract" ] && [ ! -L "$contract" ] || fail 'contract unavailable'
[ "$(sha "$contract")" = 62e0b135014a854d1187ef6ff9b24da9de07be50cd6e2e9b14cd9d59a0508ad7 ] || fail 'contract bytes escaped review'
line "$contract" 'schema=rar-alpha-controller-helper-test-evidence-v1'
line "$contract" 'case_set=97-inherited-attempt-cases+30-runtime-cases,127-total,exactly-once,canonical-order'
[ -f "$cases" ] && [ ! -L "$cases" ] || fail 'runtime cases unavailable'
[ "$(grep -Ec '^[AR][0-9][0-9][0-9]\|' "$cases")" -eq 127 ] || fail 'runtime cases incomplete'
[ "$#" -ne 0 ] || { printf '%s\n' 'controller-helper test evidence v1 contract is byte-bound'; exit 0; }
[ "$#" -eq 11 ] || fail 'usage: evidence controller source helper acceptance runtime attempt cases fixture previous-nonce previous-root'
evidence=$1; shift
[ -f "$evidence" ] && [ ! -L "$evidence" ] || fail 'evidence unavailable'
[ "$(wc -c < "$evidence" | tr -d ' ')" -le 8388608 ] || fail 'evidence oversized'
[ "$(wc -l < "$evidence" | tr -d ' ')" -eq 143 ] || fail 'evidence line count changed'
expected_header=$(printf '%s\n' schema controller_sha source_sha helper_sha closure_acceptance_sha runtime_contract_sha attempt_contract_sha cases_sha fixture_sha run_nonce root_identity case_count failed_count network credential status)
actual_header=$(/usr/bin/sed -n '1,16p' "$evidence" | /usr/bin/awk -F '=' '{ print $1 }')
[ "$actual_header" = "$expected_header" ] || fail 'header fields are missing, extra, duplicated, or reordered'
/usr/bin/awk 'NR > 16 && $0 !~ /^case\|/ { exit 1 }' "$evidence" || fail 'non-case data follows the header'
for value in "$@"; do case "$value" in *[!0-9a-f]*|'') fail 'context identity malformed';; esac; done
controller=$1 source=$2 helper=$3 acceptance=$4 runtime=$5 attempt=$6 case_sha=$7 fixture=$8 previous_nonce=$9 previous_root=${10}
for value in "$controller" "$source" "$helper" "$acceptance" "$runtime" "$attempt" "$case_sha" "$fixture" "$previous_nonce" "$previous_root"; do is_sha "$value" || fail 'zero or malformed context identity'; done
line "$evidence" 'schema=rar-alpha-controller-helper-test-evidence-v1'
line "$evidence" "controller_sha=$controller"
line "$evidence" "source_sha=$source"
line "$evidence" "helper_sha=$helper"
line "$evidence" "closure_acceptance_sha=$acceptance"
line "$evidence" "runtime_contract_sha=$runtime"
line "$evidence" "attempt_contract_sha=$attempt"
line "$evidence" "cases_sha=$case_sha"
line "$evidence" "fixture_sha=$fixture"
line "$evidence" 'case_count=127'
line "$evidence" 'failed_count=0'
line "$evidence" 'network=none'
line "$evidence" 'credential=none'
line "$evidence" 'status=accepted'
nonce=$(sed -n 's/^run_nonce=//p' "$evidence"); root_id=$(sed -n 's/^root_identity=//p' "$evidence")
is_sha "$nonce" && is_sha "$root_id" || fail 'fresh identity malformed'
[ "$nonce" != "$root_id" ] && [ "$nonce" != "$previous_nonce" ] && [ "$root_id" != "$previous_root" ] || fail 'replayed or aliased identity'
[ "$(grep -Ec '^case\|[AR][0-9][0-9][0-9]\|' "$evidence")" -eq 127 ] || fail 'case row count changed'
expected=$(awk -F '|' '/^[AR][0-9][0-9][0-9]\|/ {print $1 "|" $2 "|" $3 "|" $4}' "$cases")
actual=$(awk -F '|' '/^case\|[AR][0-9][0-9][0-9]\|/ {print $2 "|" $3 "|" $4 "|" $5}' "$evidence")
[ "$actual" = "$expected" ] || fail 'case set missing, duplicate, or reordered'
awk -F '|' '
    BEGIN { zero="0000000000000000000000000000000000000000000000000000000000000000" }
    /^case\|/ {
        if (NF != 12 || $6 !~ /^-?[0-9]+$/ || $7 !~ /^(none|[0-9]+)$/ ||
            $8 !~ /^[0-9a-f]{64}$/ || $9 !~ /^[0-9a-f]{64}$/ ||
            $10 !~ /^[0-9a-f]{64}$/ || $11 !~ /^[0-9a-f]{64}$/ ||
            $8 == zero || $9 == zero || $10 == zero || $11 == zero ||
            $12 != "pass") exit 1
    }
' "$evidence" || fail 'case result malformed, zero, or incomplete'
printf '%s\n' 'controller-helper test evidence v1 accepted'
