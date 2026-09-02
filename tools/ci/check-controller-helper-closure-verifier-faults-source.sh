#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-closure-verifier-faults-v0.fields
fail() { printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-faults-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = 926b4b2efe66897baa6574475f74ba6878e96cf394b8c8eab290c03cd3ea2d9a ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-faults-v0' "$subject" || fail 'schema changed'
grep -Fqx 'status=experimental-complete+C3VA-evidence-candidate-source-only-unwired' "$subject" || fail 'fault evidence state overclaims completion'
grep -Fq 'evidence_fault_binding=' "$subject" || fail 'fault evidence binding missing'
grep -Fq 'evidence_policy_binding=' "$subject" || fail 'fault evidence rejection binding missing'
grep -Fqx 'fault_count=12' "$subject" || fail 'fault count declaration changed'
[ "$(grep -Ec '^fault\|F[0-9][0-9][0-9]\|' "$subject")" -eq 12 ] || fail 'fault rows incomplete'
grep -Fqx 'fault|F011|resource-file-size|receipt-staging|RLIMIT_FSIZE=0+SIGXFSZ-unblocked|signal-25+controller-exit-map-153+no-valid-final-receipt' "$subject" || fail 'resource fault is not exact'
if grep -Fq 'memory-or-pids' "$subject"; then fail 'ambiguous resource fault returned'; fi
grep -Fqx 'no_success_effect=lock+inventory+profile+gate+readiness+workflow+GitHub+candidate-acceptance-unchanged' "$subject" || fail 'fault effect denial changed'
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-verifier-faults-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-faults-v0 is complete, source-only, inactive, and unwired'
