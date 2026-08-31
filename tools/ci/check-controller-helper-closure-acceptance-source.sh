#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-closure-acceptance-v0.fields
fail() { printf '%s\n' 'rar-alpha-controller-helper-closure-acceptance-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = 9447da52e1253267ce886aeed85a75f5544d6b520d737342dde9cdce0d0aa261 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-acceptance-v0' "$subject" || fail 'schema changed'
grep -Fqx 'status=experimental-schema-only-no-instance' "$subject" || fail 'schema gained an instance'
grep -Fqx 'authority=C3A-separately-reviewed-instance-only' "$subject" || fail 'C3A authority changed'
grep -Fqx 'runtime_rule=complete-observer-cases+117-runtime-dispositions+37-runtime-precedence+12-runtime-faults=166-runtime-cases,failed-count-zero,normalized-verdict-exact' "$subject" || fail 'runtime acceptance set changed'
grep -Fqx 'residual_rule=30-disposition-residual-proofs+13-precedence-residual-proofs=43-reviewed-nonexecuting-proofs;all-209-logical-relationships-accounted-for;no-residual-observed-exit+stdout+stderr+filesystem-result' "$subject" || fail 'residual acceptance set changed'
grep -Fqx 'rejection_rule=missing+zero+stale+cross-revision+self-attested+mutable+incomplete+replayed+aliased+mechanical-exact-set-only-reject' "$subject" || fail 'acceptance rejection set changed'
grep -Fqx 'effect_rule=no-lock+inventory+profile+gate+readiness+workflow+compiler-use+helper-build-update' "$subject" || fail 'schema effect denial changed'
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-acceptance-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-acceptance-v0 is complete, source-only, inactive, and unwired'
