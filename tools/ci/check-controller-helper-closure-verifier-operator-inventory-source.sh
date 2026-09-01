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
[ "$(sha_file "$inventory")" = c018c7fee8e1c70145a4c6adae852ef310ab20fd9d6b0cfad320521cdd76f062 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$templates")" = 8df14525a37c49df99d669b638efd316c8993906d69e063625445f431ab204f8 ] || fail 'case-template bytes escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'primary_family_count=34' \
    'active_primary_family_count=32' \
    'deferred_primary_family_count=2' \
    'repair_token_count=8' \
    'active_repair_token_count=7' \
    'deferred_repair_token_count=1' \
    'coverage_rule=every-template-primary-normalizes-to-exactly-one-active-listed-family+every-template-repair-equals-one-active-listed-token+deferred-P015+P016+R001-are-unused-by-templates+preserved-only-for-explicit-residuals+no-unlisted-family-or-repair-is-permitted' \
    'parameter_rule=lexical-shape-check-precedes-one-pass-decode;semantic-rows-fix-target+precondition+operation+postcondition+bound' \
    'semantic_status=exact-target+precondition+postcondition+deterministic-derivation+resource-feasibility+repair-independence-are-bound-by-owned-semantics-contract-set' \
    'activation_rule=blocked;base-instance+controller+runtime-evidence+C3V+C3A+workflow-wiring-remain-required' \
    'consumer_rule=this-inventory-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+lexical-structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$inventory" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$inventory" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'inventory lacks one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$inventory" >/dev/null; then fail 'inventory contains a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$inventory" >/dev/null; then fail 'inventory contains CR'; fi
/usr/bin/awk -F '|' '
    /^P[0-9][0-9][0-9]\|/ {
        expected=(($1 == "P015" || $1 == "P016") ? "deferred-domain-extension:" $1 : "defined:" $1)
        if (NF != 4 || $4 != expected || primary[$1]++) exit 1
        state[$1]=($4 ~ /^deferred-/ ? "deferred" : "active")
        primary_family[$1]=$2
        primary_count++
    }
    /^semantic\|P[0-9][0-9][0-9]\|/ {
        if (NF != 7 || active_semantics[$2]++ || state[$2] != "active") exit 1
        active_count++
    }
    /^semantic_residual\|P[0-9][0-9][0-9]\|/ {
        if (NF != 7 || residual_semantics[$2]++ || state[$2] != "deferred") exit 1
        residual_count++
    }
    END {
        if (primary_count != 34 || active_count != 32 || residual_count != 2) exit 1
        for (id in state) {
            if (state[id] == "active" && !(id in active_semantics)) exit 1
            if (state[id] == "deferred" && !(id in residual_semantics)) exit 1
        }
    }
' "$inventory" || fail 'primary-family active/deferred semantic rows are malformed, duplicated, or incomplete'
/usr/bin/awk -F '|' '
    /^R[0-9][0-9][0-9]\|/ {
        expected=($1 == "R001" ? "deferred-domain-extension:R001" : ($1 == "R002" ? "no-repair" : "defined:" $1))
        if (NF != 3 || $3 != expected || repair[$1]++) exit 1
        state[$1]=($3 ~ /^deferred-/ ? "deferred" : "active")
        repair_token[$1]=$2
        repair_count++
    }
    /^repair_semantic\|R[0-9][0-9][0-9]\|/ {
        if (NF != 7 || active_semantics[$2]++ || state[$2] != "active" || $3 != repair_token[$2]) exit 1
        active_count++
    }
    /^repair_residual\|R[0-9][0-9][0-9]\|/ {
        if (NF != 7 || residual_semantics[$2]++ || state[$2] != "deferred" || $3 != repair_token[$2]) exit 1
        residual_count++
    }
    END {
        if (repair_count != 8 || active_count != 7 || residual_count != 1) exit 1
        for (id in state) {
            if (state[id] == "active" && !(id in active_semantics)) exit 1
            if (state[id] == "deferred" && !(id in residual_semantics)) exit 1
        }
    }
' "$inventory" || fail 'repair-token active/deferred semantic rows are malformed, duplicated, mismatched, or incomplete'

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
actual_families=$(/usr/bin/awk -F '|' '/^P[0-9][0-9][0-9]\|/ && $4 !~ /^deferred-/ { print $2 }' "$inventory" | /usr/bin/sort)
[ "$actual_families" = "$expected_families" ] || fail 'primary family inventory differs from templates'

expected_repairs=$(/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ { print $7 }' "$templates" | /usr/bin/sort -u)
actual_repairs=$(/usr/bin/awk -F '|' '/^R[0-9][0-9][0-9]\|/ && $3 !~ /^deferred-/ { print $2 }' "$inventory" | /usr/bin/sort)
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
grep -Fqx '`controller-helper-closure-verifier-operator-inventory-v0` records 32 active and two deferred primary families plus seven active and one deferred repair token.' "$readme" || fail 'README repair-token count is stale'

if grep -R -Fq 'controller-helper-closure-verifier-operator-inventory-v0' "$root/.github/workflows"; then
    fail 'inactive operator inventory is wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier operator vocabulary and semantics are closed, source-only, inactive, and directly unwired'
