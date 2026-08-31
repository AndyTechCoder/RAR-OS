#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
readme=$root/spec/alpha/lab/README.md

fail() {
    printf 'controller-helper closure verifier operator-inventory source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$inventory" "$templates" "$readme"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$inventory")" = 422950c5a3973399a6e5b7da903be0f8cd8855dd6edeb37f1cb5924115197051 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$templates")" = 4950a2c5cbe5cddaca5bd5a829d889310585a6197630e87e9afdb48ce778ae20 ] || fail 'case-template bytes escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'primary_family_count=34' \
    'repair_token_count=8' \
    'coverage_rule=every-template-primary-normalizes-to-exactly-one-listed-family+every-template-repair-equals-one-listed-token+no-unlisted-family-or-repair-is-permitted' \
    'parameter_rule=shape-checking-is-lexical-only;it-does-not-decode+apply+or-prove-a-mutation' \
    'semantic_status=exact-target+precondition+postcondition+deterministic-derivation+resource-feasibility+repair-independence-are-bound-by-owned-semantics-contract-set' \
    'activation_rule=blocked;separate-reviewed-semantics+base-binding+controller+runtime-precedence+fault+evidence+verdict-contracts-remain-required' \
    'consumer_rule=this-inventory-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+lexical-structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$inventory" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$inventory" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'inventory lacks one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$inventory" >/dev/null; then fail 'inventory contains a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$inventory" >/dev/null; then fail 'inventory contains CR'; fi
[ "$(grep -Ec '^P[0-9][0-9][0-9]\|[a-z0-9-]+\|[A-Za-z0-9+-]+\|opaque$' "$inventory")" -eq 34 ] || fail 'primary-family rows are malformed'
[ "$(grep -Ec '^R[0-9][0-9][0-9]\|[^| ]+\|(opaque|no-repair)$' "$inventory")" -eq 8 ] || fail 'repair-token rows are malformed'

number=1
while [ "$number" -le 34 ]; do
    id=$(printf 'P%03d' "$number")
    [ "$(grep -c "^$id|" "$inventory")" -eq 1 ] || fail "primary family ID is missing or duplicated: $id"
    number=$((number + 1))
done
number=1
while [ "$number" -le 8 ]; do
    id=$(printf 'R%03d' "$number")
    [ "$(grep -c "^$id|" "$inventory")" -eq 1 ] || fail "repair token ID is missing or duplicated: $id"
    number=$((number + 1))
done

expected_families=$(/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { x=$6; sub(/^[^=]*=/,"",x); if (x ~ /^[0-9]+$/) x="decimal-literal"; else sub(/:.*/,"",x); print x }' "$templates" | /usr/bin/sort -u)
actual_families=$(/usr/bin/awk -F '|' '/^P[0-9][0-9][0-9]\|/ { print $2 }' "$inventory" | /usr/bin/sort)
[ "$actual_families" = "$expected_families" ] || fail 'primary family inventory differs from templates'

expected_repairs=$(/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { print $7 }' "$templates" | /usr/bin/sort -u)
actual_repairs=$(/usr/bin/awk -F '|' '/^R[0-9][0-9][0-9]\|/ { print $2 }' "$inventory" | /usr/bin/sort)
[ "$actual_repairs" = "$expected_repairs" ] || fail 'repair token inventory differs from templates'

/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { print $1 "|" $6 }' "$templates" |
while IFS='|' read -r id primary; do
    rhs=${primary#*=}
    case "$rhs" in
        absent | base-plus-1 | bounded-private-tmpfs | empty-directory | invalid-separator-one-space | remove | \
        replace-same-bytes-new-inode | set-other-lower-sha256 | swap-first-two-records | toggle-first-digest-nibble | toggle-owner-execute) ;;
        * )
            if printf '%s\n' "$rhs" | grep -Eq '^[0-9]+$'; then :
            elif printf '%s\n' "$rhs" | grep -Eq '^(hex|append-hex|append-unterminated-hex|add-newline-alias-set-hex|add-newline-duplicate-set-hex|record-digest-hex|rename-raw-hex|single-record-relative-hex):([0-9A-F][0-9A-F])+$'; then :
            elif printf '%s\n' "$rhs" | grep -Eq '^(append-repeat-hex|repeat-hex):([0-9A-F][0-9A-F])+:[1-9][0-9]*$'; then :
            elif printf '%s\n' "$rhs" | grep -Eq '^(attach-safe-relative-path-length|canonical-record-line-bytes|delete-line|exact-all-paths-bytes|exact-descendant-count|exact-regenerated-manifest-bytes|set-nlink):[1-9][0-9]*$'; then :
            elif printf '%s\n' "$rhs" | grep -Eq '^toggle-byte:[0-9]+$'; then :
            elif printf '%s\n' "$rhs" | grep -Eq '^(add-hardlink|replace-with-symlink-to):[A-Za-z0-9._+:-]+$'; then :
            elif printf '%s\n' "$rhs" | grep -Eq '^symlink-to:/[A-Za-z0-9._/+:-]+$'; then :
            elif [ "$rhs" = 'swap-to:canonical-manifest-1048577-plus-bytes' ]; then :
            else fail "primary parameter shape is outside the closed vocabulary: $id:$rhs"
            fi
            ;;
    esac
done
grep -Fqx '`controller-helper-closure-verifier-operator-inventory-v0` closes the lexical vocabulary used by those templates to 34 primary families and eight repair tokens.' "$readme" || fail 'README repair-token count is stale'

if grep -R -Fq 'controller-helper-closure-verifier-operator-inventory-v0' "$root/.github/workflows"; then
    fail 'inactive operator inventory is wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier operator vocabulary and semantics are closed, source-only, inactive, and directly unwired'
