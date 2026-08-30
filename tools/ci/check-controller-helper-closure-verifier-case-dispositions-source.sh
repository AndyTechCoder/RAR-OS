#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
catalog=$root/spec/alpha/lab/controller-helper-closure-verifier-case-dispositions-v0
errors=$root/spec/alpha/lab/controller-helper-closure-verifier-errors-v0
validation=$root/spec/alpha/lab/controller-helper-closure-verifier-validation-v0.fields
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
subject=$root/tools/ci/verify-controller-helper-closure-candidate.sh

fail() {
    printf 'controller-helper closure verifier case-disposition source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$catalog" "$errors" "$validation" "$domain" "$subject"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$catalog")" = 873265794ef134a1b6f7d545ccad514649efac54b5d9e9c5094e1f3c96d8a734 ] || fail 'case-disposition bytes escaped review'
[ "$(sha_file "$errors")" = 9370f2e29e3932f42826441568baed629d2d2ab8fd107f50b6fb58e1d1637b4f ] || fail 'error catalog escaped review'
[ "$(sha_file "$validation")" = 1958c06a458cca81d4c5914f2664d4e70e0575ef2d7e260407638485c1727f2f ] || fail 'validation contract escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input-domain contract escaped review'
[ "$(sha_file "$subject")" = 3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c ] || fail 'verifier subject escaped review'

for required in \
    'status=experimental-inactive-source-only' \
    'execution_authority=none' \
    'disposition_count=147' \
    'disposition_set=fixture+synchronized-mutation+fault-only+source-proof+source-dominated+cryptographic-residual+domain-extension' \
    'coverage_rule=every-error-catalog-class+occurrence-stage-pair-appears-exactly-once;class-only-coverage-is-insufficient' \
    'scratch_exclusion_rule=before-verifier-start-controller-closes-every-path+file-descriptor+backing-handle+mount-namespace-handle+proc-write+ptrace-path-to-/tmp-and-/tmp/rar-controller-helper-closure-verification;primary-mutations+repairs+retained-handles-must-target-only-nonscratch-fixture-backing;controller-cannot-enter-verifier-mount-namespace-after-start;syscall+tool+scratch-output-failures-belong-only-to-fault-contract' \
    'source_proof_rule=exactly-E106+E107+E125+E126-from-the-pinned-validation-contract;not-runtime-cases' \
    'source_dominated_rule=additional-exact-subject-occurrence-is-unreachable-under-current-private-scratch+validated-tool-output+single-primary-mutation-model;distinct-from-pinned-source-proof-class-set' \
    'source_dominated_E072=record-line-limit-precedes-path-limit+64-byte-digest+2-byte-separator+385-byte-path-equals-451-bytes' \
    'source_dominated_E085=384-byte-relative-path+fixed-type+bounded-Linux-stat-fields-cannot-reach-1024-byte-topology-line;unexpected-tool-output-belongs-to-fault-contract' \
    'source_dominated_E090=empty-regular-path-snapshot-fails-E123-before-regeneration+removal-after-nonempty-snapshot-fails-E087' \
    'source_dominated_E105=first-manifest-is-private-scratch+first-comparison-binds-its-bytes-to-the-original-manifest-digest' \
    'source_proof_E106_E107=staged-receipt-is-exactly-31-bounded-lines-from-validated-fields' \
    'source_proof_E125_E126=all-output-digests-come-from-sha-file-or-earlier-require-sha-validation' \
    'effect_rule=no-row-is-a-fixture+controller+fault-injector+runtime-oracle+test-result+acceptance-evidence+workflow-authority' \
    'consumer_rule=this-catalog-does-not-authorize-controller-implementation+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$catalog" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$catalog" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'catalog lacks one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$catalog" >/dev/null; then fail 'catalog contains a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$catalog" >/dev/null; then fail 'catalog contains CR'; fi
[ "$(grep -Ec '^D[0-9][0-9][0-9]\|E[0-9][0-9][0-9]\|[a-z0-9-]+\|(fixture|synchronized-mutation|fault-only|source-proof|source-dominated|cryptographic-residual|domain-extension)\|[A-Za-z0-9-]+$' "$catalog")" -eq 147 ] || fail 'catalog rows are incomplete or malformed'
[ "$(grep -Ec '^D[0-9][0-9][0-9]\|' "$catalog")" -eq 147 ] || fail 'catalog row count is not 147'

number=1
while [ "$number" -le 147 ]; do
    id=$(printf 'D%03d' "$number")
    [ "$(grep -c "^$id|" "$catalog")" -eq 1 ] || fail "catalog ID is missing or duplicated: $id"
    number=$((number + 1))
done

expected=$(/usr/bin/awk -F '|' 'NR > 2 && $1 ~ /^E[0-9][0-9][0-9]$/ { count=split($2, stages, "+"); for (i=1; i<=count; i++) print $1 "|" stages[i] }' "$errors" | /usr/bin/sort)
actual=$(sed -n 's/^D[0-9][0-9][0-9]|\(E[0-9][0-9][0-9]|[a-z0-9-]*\)|[^|]*|[^|]*$/\1/p' "$catalog" | /usr/bin/sort)
[ "$actual" = "$expected" ] || fail 'catalog does not classify every exact error occurrence once'

[ "$(grep -c '|source-proof|' "$catalog")" -eq 4 ] || fail 'source-proof occurrence count changed'
[ "$(grep -c '|source-dominated|' "$catalog")" -eq 6 ] || fail 'source-dominated occurrence count changed'
[ "$(grep -c '|cryptographic-residual|' "$catalog")" -eq 1 ] || fail 'cryptographic residual count changed'
[ "$(grep -c '|domain-extension|' "$catalog")" -eq 2 ] || fail 'domain-extension count changed'
source_proofs=$(grep '|source-proof|' "$catalog" | sort)
expected_source_proofs=$(printf '%s\n' \
    'D141|E106|receipt-publication|source-proof|unreachable-under-bound-subject' \
    'D142|E107|receipt-publication|source-proof|unreachable-under-bound-subject' \
    'D145|E125|output-digest-validation|source-proof|unreachable-under-bound-subject' \
    'D146|E126|output-digest-validation|source-proof|unreachable-under-bound-subject' | sort)
[ "$source_proofs" = "$expected_source_proofs" ] || fail 'source-proof occurrence set changed'
grep -Fqx 'source_proof_classes=E106+E107+E125+E126' "$errors" || fail 'error-catalog source-proof class set changed'
grep -Fqx 'source_proof_rule=E106+E107+E125+E126-are-defensive-source-proven-guards+not-runtime-case-oracles;E127-for-freshly-hashed-values-is-an-explicit-SHA-256-zero-digest-residual-risk+not-a-practical-fixture-oracle;fault-induced-variants-belong-only-to-the-future-fault-contract' "$validation" || fail 'validation source-proof rule changed'
source_dominated=$(grep '|source-dominated|' "$catalog" | sort)
expected_source_dominated=$(printf '%s\n' \
    'D086|E072|candidate-manifest|source-dominated|dominated-by-record-line-bound' \
    'D108|E085|closure-first-pass|source-dominated|unreachable-under-bound-subject' \
    'D109|E085|closure-second-pass|source-dominated|unreachable-under-bound-subject' \
    'D122|E090|closure-first-pass|source-dominated|dominated-by-empty-regular-path-snapshot' \
    'D123|E090|closure-second-pass|source-dominated|dominated-by-empty-regular-path-snapshot' \
    'D140|E105|closure-stability|source-dominated|dominated-by-private-first-manifest' | sort)
[ "$source_dominated" = "$expected_source_dominated" ] || fail 'source-dominated occurrence set changed'

source_line() {
    /usr/bin/grep -nF -- "$1" "$subject" | sed -n '1s/:.*//p'
}

source_line_last() {
    /usr/bin/grep -nF -- "$1" "$subject" | tail -n 1 | sed 's/:.*//'
}

assert_before() {
    left=$(source_line "$1")
    right=$(source_line "$2")
    [ -n "$left" ] && [ -n "$right" ] && [ "$left" -lt "$right" ] || fail "source-order proof failed: $1 before $2"
}

assert_before_last_left() {
    left=$(source_line_last "$1")
    right=$(source_line "$2")
    [ -n "$left" ] && [ -n "$right" ] && [ "$left" -lt "$right" ] || fail "source-order proof failed: final $1 before $2"
}

assert_before '[ "${#line}" -le 450 ]' '[ "${#relative}" -le 384 ]'
assert_before_last_left '[ "${#relative}" -le 384 ]' '[ "${#topology_line}" -le 1023 ]'
assert_before 'bound_snapshot "$regular_paths" 65536' 'exec 6> "$regenerated"'
assert_before '"$comparator" -s -- "$manifest" "$scratch/first.manifest"' '"$comparator" -s -- "$scratch/first.manifest" "$scratch/second.manifest"'
assert_before '[ "$manifest_sha" = "$(sha_file "$manifest")" ]' '[ "$recomputed_sha" = "$manifest_sha" ]'
grep -Fq '|E076|baseline-mount-topology|fixture|pre-start-negative-mount' "$catalog" || fail 'baseline mount disposition changed'
grep -Fq '|E076|final-mount-topology|synchronized-mutation|mount-added-after-baseline-check' "$catalog" || fail 'final mount disposition changed'

if grep -R -Fq 'controller-helper-closure-verifier-case-dispositions-v0' "$root/.github/workflows"; then
    fail 'inactive case-disposition catalog is wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier case dispositions are complete, inactive, and directly unwired'
