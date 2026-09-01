#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
dispositions=$root/spec/alpha/lab/controller-helper-closure-verifier-case-dispositions-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
subject=$root/tools/ci/verify-controller-helper-closure-candidate.sh

fail() {
    printf 'controller-helper closure verifier case-template source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$templates" "$dispositions" "$domain" "$subject"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$templates")" = 8df14525a37c49df99d669b638efd316c8993906d69e063625445f431ab204f8 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$dispositions")" = 0284cbb3edc56c28971b6e5d151237165aa63a5321cea44772b0a20fbe8c3565 ] || fail 'case dispositions escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain escaped review'
[ "$(sha_file "$subject")" = 3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c ] || fail 'verifier subject escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'base_fixture_status=specified-by-controller-helper-closure-verifier-cases-v0;instance-not-created-until-C3V' \
    'template_count=117' \
    'template_exactness=bound-to-owned-operator+repair-semantics-contract-set+controller-helper-closure-verifier-cases-v0' \
    'operator_semantics_status=complete-across-owned-semantics-contract-set' \
    'repair_semantics_status=complete-across-owned-semantics-contract-set' \
    'operator_binding_requirement=separate-reviewed-versioned-operator+repair-semantics-contract-must-define-every-token+exact-target+precondition+postcondition+derivation+bound+independence-before-base-binding' \
    'coverage_rule=exactly-one-template-for-every-disposition-row-marked-fixture-or-synchronized-mutation+no-template-for-fault-only+source-proof+source-dominated+cryptographic-residual+domain-extension' \
    'scratch_exclusion=controller-has-no-post-start-path+FD+backing+namespace+proc+ptrace+mutation+repair-access-to-/tmp-or-verifier-scratch' \
    'termination_oracle=normal-process-exit-status-exactly-1;signal+core+timeout+resource-limit+shell+tool+other-fault-termination-are-rejected+reserved-for-separate-fault-contract' \
    'runtime_state=contracts-complete;C3V-base+fixture-image+controller+harness+run+evidence+verdict-instances-not-created' \
    'ordinary_output_oracle=/verification-empty-except-E053-receipt-collision' \
    'E053_output_oracle=preexisting-/verification/controller-helper-closure-verification.receipt-bytes+device+inode+mode+uid+gid+size+mtime+ctime+link-count-remain-exactly-unchanged+no-other-verification-entry' \
    'acceptance_rule=blocked;templates-remain-nonexecuting-until-C3V-instances+controller+wiring+exact-main-validation-are-reviewed' \
    'consumer_rule=this-catalog-does-not-authorize-fixture-creation+controller-implementation+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$templates" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$templates" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'templates lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$templates" >/dev/null; then fail 'templates contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$templates" >/dev/null; then fail 'templates contain CR'; fi
[ "$(grep -Ec '^C[0-9][0-9][0-9]\|D[0-9][0-9][0-9]\|E[0-9][0-9][0-9]\|[a-z0-9-]+\|[a-z0-9-]+\|[^| ]+\|[^| ]+\|E[0-9][0-9][0-9]@[a-z0-9-]+\+normal-exit-status-1\+no-valid-final-receipt$' "$templates")" -eq 117 ] || fail 'template rows are incomplete or malformed'

/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { if (seen[$1]++) exit 1; count++ } END { if (count != 117) exit 1 }' "$templates" ||
    fail 'template IDs are duplicated or the sparse reviewed set is incomplete'
domain_extensions=$(/usr/bin/awk -F '|' '$4 == "domain-extension" { print $1 }' "$dispositions" | /usr/bin/paste -sd, -)
[ "$domain_extensions" = 'D027,D028,D029,D055,D057,D104,D105,D112,D113,D114,D115,D133' ] ||
    fail 'domain-extension disposition set changed'

expected=$(/usr/bin/awk -F '|' '$4 == "fixture" || $4 == "synchronized-mutation" { print $1 "|" $2 "|" $3 }' "$dispositions" | /usr/bin/sort)
actual=$(/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { print $2 "|" $3 "|" $4 }' "$templates" | /usr/bin/sort)
[ "$actual" = "$expected" ] || fail 'templates do not cover exactly every constructible disposition'

/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { print }' "$templates" |
while IFS='|' read -r id disposition class stage phase primary repairs oracle; do
    row=$(grep "^$disposition|$class|$stage|" "$dispositions")
    [ -n "$row" ] || fail "disposition binding is absent: $id"
    kind=$(printf '%s\n' "$row" | cut -d '|' -f4)
    case "$kind:$phase" in
        fixture:pre-start) ;;
        synchronized-mutation:pre-start) fail "synchronized template uses pre-start phase: $id" ;;
        synchronized-mutation:*) ;;
        *) fail "template binds a nonconstructible disposition: $id" ;;
    esac
    [ "$oracle" = "$class@$stage+normal-exit-status-1+no-valid-final-receipt" ] || fail "oracle differs from class occurrence: $id"
    case "$repairs" in
        none | repair-tool-pin-env-sha256 | \
        rebuild-observation-canonical | repair-observation-manifest-fields | repair-observation-manifest-digest-and-bytes | \
        repair-observation-manifest-digest | repair-manifest-if-pre-start) ;;
        *) fail "unreviewed repair token: $id:$repairs" ;;
    esac
done

[ "$(grep -c '|pre-start|' "$templates")" -eq 83 ] || fail 'fixture template count changed'
[ "$(grep -c '|synchronized-mutation|' "$dispositions")" -eq 34 ] || fail 'synchronized disposition count changed'
[ "$(grep -c '|pre-start|' "$templates")" -lt 117 ] || fail 'no synchronized phase templates exist'
grep -Fqx 'C055|D058|E053|input-identity|pre-start|file./verification/controller-helper-closure-verification.receipt=hex:58|none|E053@input-identity+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'E053 collision oracle binding changed'
if grep -R -Fq 'controller-helper-closure-verifier-case-templates-v0' "$root/.github/workflows"; then
    fail 'inactive case templates are wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier case templates are complete relative to the C1 case contract, inactive, and directly unwired'
