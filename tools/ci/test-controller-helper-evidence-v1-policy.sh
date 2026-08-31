#!/bin/sh
set -eu
LC_ALL=C LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
tmp=${TMPDIR:-/tmp}/rar-c1-evidence-policy-$$
mkdir -m 700 "$tmp"
fail(){ printf '%s\n' "controller-helper evidence v1 policy failed: $1" >&2; exit 1; }
a=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
b=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
c=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
d=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd
e=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
f=ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
one=1111111111111111111111111111111111111111111111111111111111111111
two=2222222222222222222222222222222222222222222222222222222222222222
three=3333333333333333333333333333333333333333333333333333333333333333
four=4444444444444444444444444444444444444444444444444444444444444444
five=5555555555555555555555555555555555555555555555555555555555555555
six=6666666666666666666666666666666666666666666666666666666666666666
seven=7777777777777777777777777777777777777777777777777777777777777777
eight=8888888888888888888888888888888888888888888888888888888888888888
cases=$root/spec/alpha/lab/controller-helper-runtime-cases.v0
{
 printf '%s\n' schema=rar-alpha-controller-helper-test-evidence-v1 controller_sha=$a source_sha=$b helper_sha=$c closure_acceptance_sha=$d runtime_contract_sha=$e attempt_contract_sha=$f cases_sha=$one fixture_sha=$five run_nonce=$two root_identity=$three case_count=127 failed_count=0 network=none credential=none status=accepted
 awk -F '|' '/^[AR][0-9][0-9][0-9]\|/ {printf "case|%s|%s|%s|%s|0|none|%s|%s|%s|%s|pass\n",$1,$2,$3,$4,"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}' "$cases"
} > "$tmp/test.v1"
"$root/tools/ci/check-controller-helper-test-evidence-v1.sh" "$tmp/test.v1" $a $b $c $d $e $f $one $five $four $six >/dev/null || fail 'valid test evidence rejected'
sed 's/schema=rar-alpha-controller-helper-test-evidence-v1/schema=rar-alpha-controller-helper-test-evidence-v0/' "$tmp/test.v1" > "$tmp/bad-version"
if "$root/tools/ci/check-controller-helper-test-evidence-v1.sh" "$tmp/bad-version" $a $b $c $d $e $f $one $five $four $six >/dev/null 2>&1; then fail 'v0 substitution accepted'; fi
awk '$0 !~ /^case\|A050\|/' "$tmp/test.v1" > "$tmp/missing-case"
if "$root/tools/ci/check-controller-helper-test-evidence-v1.sh" "$tmp/missing-case" $a $b $c $d $e $f $one $five $four $six >/dev/null 2>&1; then fail 'missing case accepted'; fi
if "$root/tools/ci/check-controller-helper-test-evidence-v1.sh" "$tmp/test.v1" $four $b $c $d $e $f $one $five $four $six >/dev/null 2>&1; then fail 'stale identity accepted'; fi
if "$root/tools/ci/check-controller-helper-test-evidence-v1.sh" "$tmp/test.v1" $a $b $c $d $e $f $one $five $two $six >/dev/null 2>&1; then fail 'replayed nonce accepted'; fi
{
 printf '%s\n' schema=rar-alpha-controller-helper-build-evidence-v1 controller_sha=$a source_sha=$b closure_acceptance_sha=$c compiler_sha=$d compiler_closure_sha=$e test_evidence_v1_sha=$f build_plan_sha=$two golden_sha=$three build_1_receipt_sha=$four build_2_receipt_sha=$five build_1_log_sha=$six build_2_log_sha=$seven build_count=2 network=none credential=none build_1_exit=0 build_2_exit=0 build_1_binary_sha=$one build_2_binary_sha=$one final_binary_sha=$one build_1_job_nonce=$two build_2_job_nonce=$three build_1_root=$four build_2_root=$f binary_bytes=4096 status=accepted
} > "$tmp/build.v1"
"$root/tools/ci/check-controller-helper-build-evidence-v1.sh" "$tmp/build.v1" $a $b $c $d $e $f $two $three $four $five >/dev/null || fail 'valid build evidence rejected'
sed "s/test_evidence_v1_sha=$f/test_evidence_v1_sha=$a/" "$tmp/build.v1" > "$tmp/bad-test"
if "$root/tools/ci/check-controller-helper-build-evidence-v1.sh" "$tmp/bad-test" $a $b $c $d $e $f $two $three $four $five >/dev/null 2>&1; then fail 'wrong test identity accepted'; fi
printf '%s\n' 'controller-helper evidence v1 mutation policy passed'
