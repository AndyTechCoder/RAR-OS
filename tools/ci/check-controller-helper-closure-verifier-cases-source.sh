#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-closure-verifier-cases-v0
fail() { printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-cases-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = 8a5564e07810f300d2a65bd0cd7a5c76513ef80f36f1ade4b87bf1f8a3b64897 ] || fail 'subject bytes escaped review'
for required in \
    'schema=rar-alpha-controller-helper-closure-verifier-cases-v0' \
    'disposition_case_count=147' \
    'disposition_runtime_count=117' \
    'disposition_residual_count=30' \
    'precedence_case_count=50' \
    'precedence_runtime_count=37' \
    'precedence_residual_count=13' \
    'fault_case_count=12' \
    'runtime_case_count=166' \
    'residual_case_count=43' \
    'total_case_count=209' \
    'precedence_instance_rule=37-executable-runtime-relations+13-catalog-only-residual-relations;residual-relations-authorize-no-mutation+no-observed-runtime-result' \
    'precedence_projection_rule=precedence-rows-bind-two-exact-primary+repair-tuples-on-one-fresh-base;precedence-residual-rows-bind-only-reviewed-catalog-sides+reason+no-runtime-mutation+no-observed-result' \
    'ordering_rule=logical-order-is-exactly-V001-through-V147+Q001-through-Q050+X001-through-X012;runtime+residual-projections-preserve-logical-order' \
    'unrepresentable_rule=source-proof+source-dominated+cryptographic-residual+domain-extension-remain-explicit-rows,never-silently-skipped'; do
    grep -Fqx "$required" "$subject" || fail "required invariant is missing: $required"
done
[ "$(grep -Ec '^case\|V[0-9][0-9][0-9]\|disposition\|' "$subject")" -eq 117 ] || fail 'runtime disposition cases incomplete'
[ "$(grep -Ec '^case\|V[0-9][0-9][0-9]\|disposition-residual\|' "$subject")" -eq 30 ] || fail 'residual disposition proofs incomplete'
[ "$(grep -Ec '^case\|Q[0-9][0-9][0-9]\|precedence\|' "$subject")" -eq 37 ] || fail 'runtime precedence cases incomplete'
[ "$(grep -Ec '^case\|Q[0-9][0-9][0-9]\|precedence-residual\|' "$subject")" -eq 13 ] || fail 'residual precedence proofs incomplete'
[ "$(grep -Ec '^case\|X[0-9][0-9][0-9]\|' "$subject")" -eq 12 ] || fail 'fault cases incomplete'
/usr/bin/awk -F '|' '
    /^case\|V/ { if (v[$2]++) exit 1; vc++ }
    /^case\|Q/ { if (q[$2]++) exit 1; qc++ }
    /^case\|X/ { if (x[$2]++) exit 1; xc++ }
    END { if (vc != 147 || qc != 50 || xc != 12) exit 1 }
' "$subject" || fail 'logical case IDs are missing or duplicated'
/usr/bin/awk -F '|' '
    /^case\|V[0-9][0-9][0-9]\|disposition-residual\|/ {
        if (NF != 8 || $7 !~ /^explicit-(fault-only|source-proof|source-dominated|cryptographic-residual|domain-extension):/ ||
            $8 != "source-proof-or-residual+no-runtime-omission") exit 1
    }
    /^case\|Q[0-9][0-9][0-9]\|precedence\|/ {
        if (NF != 8 || $5 !~ /^left=D[0-9][0-9][0-9]:.+:.+$/ ||
            $6 !~ /^right=D[0-9][0-9][0-9]:.+:.+$/ || $8 !~ /^first-error-E[0-9][0-9][0-9]$/) exit 1
    }
    /^case\|Q[0-9][0-9][0-9]\|precedence-residual\|/ {
        if (NF != 8 ||
            $5 !~ /^left=(runtime|residual)-D[0-9][0-9][0-9]:E[0-9][0-9][0-9]:[a-z-]+$/ ||
            $6 !~ /^right=(runtime|residual)-D[0-9][0-9][0-9]:E[0-9][0-9][0-9]:[a-z-]+$/ ||
            ($5 !~ /^left=residual-/ && $6 !~ /^right=residual-/) ||
            $7 != "catalog-relation-only+no-runtime-mutation" ||
            $8 != "residual-no-observed-exit-or-result") exit 1
    }
    /^case\|X/ { if (NF != 8 || $7 == "" || $8 == "") exit 1 }
' "$subject" || fail 'runtime or residual case row is malformed'
if grep -Fq 'direct-E' "$subject"; then fail 'unconstructible direct precedence placeholder remains'; fi
if grep -Eq '^case\|[QX].*\|opaque(\||$)' "$subject"; then fail 'case instance retains opaque semantics'; fi
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-verifier-cases-v0' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-cases-v0 accounts for 166 runtime cases and 43 residual proofs, remains inactive, and is unwired'
