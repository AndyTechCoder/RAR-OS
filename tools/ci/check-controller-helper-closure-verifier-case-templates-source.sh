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
[ "$(sha_file "$templates")" = a5ae29ea7053dc200147901b89db67271cd895013769865910314070744835a3 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$dispositions")" = 3e693cf86851164cb07577e71b7ff256a17201df1a844c7b9355b135e0e0ba61 ] || fail 'case dispositions escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain escaped review'
[ "$(sha_file "$subject")" = 3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c ] || fail 'verifier subject escaped review'

for required in \
    'status=experimental-incomplete-inactive-source-only' \
    'execution_authority=none' \
    'base_fixture_status=absent' \
    'template_count=125' \
    'template_exactness=structural-only;primary+repair-tokens-are-opaque-identifiers+not-yet-decodable-operators' \
    'operator_semantics_status=absent' \
    'repair_semantics_status=absent' \
    'operator_binding_requirement=separate-reviewed-versioned-operator+repair-semantics-contract-must-define-every-token+exact-target+precondition+postcondition+derivation+bound+independence-before-base-binding' \
    'coverage_rule=exactly-one-template-for-every-disposition-row-marked-fixture-or-synchronized-mutation+no-template-for-fault-only+source-proof+source-dominated+cryptographic-residual+domain-extension' \
    'scratch_exclusion=controller-has-no-post-start-path+FD+backing+namespace+proc+ptrace+mutation+repair-access-to-/tmp-or-verifier-scratch' \
    'termination_oracle=normal-process-exit-status-exactly-1;signal+core+timeout+resource-limit+shell+tool+other-fault-termination-are-rejected+reserved-for-separate-fault-contract' \
    'runtime_state=no-base-fixture+no-fixture-image+no-controller+no-harness+no-run+no-evidence+no-verdict' \
    'ordinary_output_oracle=/verification-empty-except-E053-receipt-collision' \
    'E053_output_oracle=preexisting-/verification/controller-helper-closure-verification.receipt-bytes+device+inode+mode+uid+gid+size+mtime+ctime+link-count-remain-exactly-unchanged+no-other-verification-entry' \
    'acceptance_rule=blocked;templates-are-not-executable-instances-until-the-operator+repair-semantics+exact-base+phase-instrumentation+controller+fault+evidence+verdict-contracts-are-reviewed' \
    'consumer_rule=this-catalog-does-not-authorize-fixture-creation+controller-implementation+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$templates" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$templates" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'templates lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$templates" >/dev/null; then fail 'templates contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$templates" >/dev/null; then fail 'templates contain CR'; fi
[ "$(grep -Ec '^C[0-9][0-9][0-9]\|D[0-9][0-9][0-9]\|E[0-9][0-9][0-9]\|[a-z0-9-]+\|[a-z0-9-]+\|[^| ]+\|[^| ]+\|E[0-9][0-9][0-9]@[a-z0-9-]+\+normal-exit-status-1\+no-valid-final-receipt$' "$templates")" -eq 125 ] || fail 'template rows are incomplete or malformed'

number=1
while [ "$number" -le 125 ]; do
    id=$(printf 'C%03d' "$number")
    [ "$(grep -c "^$id|" "$templates")" -eq 1 ] || fail "template ID is missing or duplicated: $id"
    number=$((number + 1))
done

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
        none | repair-tool-pin-env-sha256 | base-controller-uid=1000 | create-hidden-same-inode-link | \
        rebuild-observation-canonical | repair-observation-manifest-fields | repair-observation-manifest-digest-and-bytes | \
        repair-observation-manifest-digest | repair-manifest-if-pre-start) ;;
        *) fail "unreviewed repair token: $id:$repairs" ;;
    esac
done

[ "$(grep -c '|pre-start|' "$templates")" -eq 89 ] || fail 'fixture template count changed'
[ "$(grep -c '|synchronized-mutation|' "$dispositions")" -eq 36 ] || fail 'synchronized disposition count changed'
[ "$(grep -c '|pre-start|' "$templates")" -lt 125 ] || fail 'no synchronized phase templates exist'
grep -Fqx 'C055|D058|E053|input-identity|pre-start|file./verification/controller-helper-closure-verification.receipt=hex:58|none|E053@input-identity+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'E053 collision oracle binding changed'
if grep -R -Fq 'controller-helper-closure-verifier-case-templates-v0' "$root/.github/workflows"; then
    fail 'inactive case templates are wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier case templates are complete relative to an absent base, inactive, and directly unwired'
