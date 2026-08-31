#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
semantics=$root/spec/alpha/lab/controller-helper-closure-verifier-scalar-repair-semantics-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
scalar=$root/spec/alpha/lab/controller-helper-closure-verifier-scalar-semantics-v0

fail() {
    printf 'controller-helper closure verifier scalar-repair-semantics source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$semantics" "$templates" "$inventory" "$domain" "$scalar"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$semantics")" = 72bc43e9e14f2967e04fbec20452cba4a4c0c640d568265ebfdaaaba3f51293a ] || fail 'scalar repair semantics bytes escaped review'
[ "$(sha_file "$templates")" = 4950a2c5cbe5cddaca5bd5a829d889310585a6197630e87e9afdb48ce778ae20 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$inventory")" = 8440a105896b8e038fd6548187b53b2af2466760ace40678fe276cdc0af146e8 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain bytes escaped review'
[ "$(sha_file "$scalar")" = 42442ffa4306fbcc83acd95a138ee4f6f5d059a671b206c4de6452ff64776ddb ] || fail 'scalar semantics bytes escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'semantic_row_count=1' \
    'repair_token_coverage=repair-tool-pin-env-sha256' \
    'covered_template_count=5' \
    'ordering_rule=complete+validate+record-the-one-primary-postcondition-first;then-apply-only-the-declared-repair;launch-only-after-the-primary+repair+combined-postconditions-pass;primary-failure+repair-failure+overlap-invalidates-before-launch' \
    'noncancellation_rule=repair-never-restores+rewrites+aliases+replaces+or-changes-identity+bytes+metadata-of-the-primary-target;all-primary-postconditions-remain-true-through-launch' \
    'tool_pin_digest_rule=after-primary-post-state-recording+compute-SHA-256-over-the-exact-complete-mutated-tool-pin-regular-file-byte-string-with-the-future-byte-pinned-controller-hasher;encode-the-32-octet-digest-as-exactly-64-lowercase-ASCII-hex-bytes-with-no-prefix+separator+LF+NUL;set-only-RAR_REVIEWED_VERIFIER_TOOLS_SHA256-to-those-bytes' \
    'failure_rule=base-proof+ordering+read-scope+digest+encoding+environment+identity+noncancellation+combined-postcondition-failure-invalidates-before-launch;never-skip+retry+coerce+fallback+repair-the-primary-target' \
    'remaining_status=6-non-none-repair-tokens+23-primary-families+pre-start+repair-coupled-links+required-path-symlinks+raw-name+path-alias+mount+tree+manifest-specific-primary-families+exact-base+controller+runtime-precedence+fault+evidence+verdict-remain-absent' \
    'activation_rule=blocked;this-slice-cannot-create-fixtures+execute-mutations+apply-repairs+or-authorize-a-controller' \
    'consumer_rule=this-contract-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$semantics" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$semantics" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'semantics lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$semantics" >/dev/null; then fail 'semantics contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$semantics" >/dev/null; then fail 'semantics contain CR'; fi
[ "$(grep -Ec '^R[0-9][0-9][0-9]\|[^| ]+\|[^| ]+\|[^| ]+\|[^| ]+$' "$semantics")" -eq 1 ] || fail 'semantic rows are malformed'
grep -Fqx 'R001|repair-tool-pin-env-sha256|env.RAR_REVIEWED_VERIFIER_TOOLS_SHA256|pre-start+primary-postcondition-recorded+mutated-tool-pin-remains-declared-single-link-regular-file+controller-hasher-identity-byte-pinned+repair-target-pre-bytes-recorded|tool_pin_digest_rule+repair-target-post-bytes-are-the-derived-64-byte-digest+preserve-primary-file-bytes+identity+metadata+all-other-environment-bindings' "$semantics" || fail 'tool-pin digest repair row changed'
if grep -Fq 'base-controller-uid=1000' "$semantics"; then fail 'removed context token returned to repair semantics'; fi

[ "$(grep -c '|base-controller-uid=1000|' "$templates")" -eq 0 ] || fail 'removed controller UID repair token returned'
[ "$(grep -c '|repair-tool-pin-env-sha256|' "$templates")" -eq 5 ] || fail 'tool-pin repair template count changed'

/usr/bin/awk -F '|' '
/^C[0-9][0-9][0-9]\|/ && $7=="repair-tool-pin-env-sha256" {
    n++
    if ($5 != "pre-start") exit 20
    target=$6; sub(/=.*/, "", target)
    if (target!="file./trusted/controller-helper-closure-verifier-tools.v0" && target!="file./trusted/controller-helper-closure-verifier-tools.v0.line-1" && target!="file./trusted/controller-helper-closure-verifier-tools.v0.line-2") exit 22
}
END { if (n != 5) exit 23 }
' "$templates" || fail 'repair template phase, target, or count changed'

for id in C036 C037 C039 C040 C041; do
    [ "$(grep -c "^$id|" "$templates")" -eq 1 ] || fail "covered repair template is missing or duplicated: $id"
done

if grep -R -Fq 'controller-helper-closure-verifier-scalar-repair-semantics-v0' "$root/.github/workflows"; then
    fail 'inactive scalar repair semantics are wired directly to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier scalar repair semantics cover one token and five templates, inactive and directly unwired'
