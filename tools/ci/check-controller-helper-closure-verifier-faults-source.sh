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
[ "$actual" = 43f38cc2e75567a9e473c8f2c3d5f8ed6fffc03b8a33a70622bcdd1521901a00 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-faults-v0' "$subject" || fail 'schema changed'
grep -Fqx 'fault_count=12' "$subject" || fail 'fault count declaration changed'
[ "$(grep -Ec '^fault\|F[0-9][0-9][0-9]\|' "$subject")" -eq 12 ] || fail 'fault rows incomplete'
grep -Fqx 'no_success_effect=lock+inventory+profile+gate+readiness+workflow+GitHub+candidate-acceptance-unchanged' "$subject" || fail 'fault effect denial changed'
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-verifier-faults-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-faults-v0 is complete, source-only, inactive, and unwired'
