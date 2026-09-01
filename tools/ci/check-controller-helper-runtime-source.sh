#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-runtime-v0.fields
fail() { printf '%s\n' 'rar-alpha-controller-helper-runtime-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = 7cd99e8f19229c2403add420b881d37a3a7d7b4c3cc7513d13938d691ead9d33 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-runtime-v0' "$subject" || fail 'schema changed'
grep -Fqx 'descriptor_count=6' "$subject" || fail 'descriptor count changed'
[ "$(grep -c '^descriptor|' "$subject")" -eq 6 ] || fail 'descriptor rows incomplete'
for line in \
 'exec_rule=execveat-helper-fd-empty-path+AT_EMPTY_PATH,argv-exact,empty-environment,no-shell+PATH+interpreter+url+credential' \
 'phase_boundary_rule=C1-source-validation-must-not-invoke-compiler+linker+helper+target+container+VM+emulator+firmware+network+credential;exec_rule-is-a-future-H1C-runtime-requirement-only+grants-no-C1-execution-authority;helper-execution-remains-forbidden-until-separately-reviewed-H1C-activation' \
 'process_rule=controller-owned-pidfd-or-equivalent-nonreusable-handle,bare-PID-never-authority,handle-closed-after-reap' \
 'recovery_rule=source-never-deleted,exclusive-attempt-roots,bounded-exact-enumeration,inventory-durable-before-delete,identity-match-before-each-delete' \
 'attempt_ceiling=3-normal-attempts+3-recovery-sessions,fourth-is-durable-blocked,no-reuse+loop'; do grep -Fqx "$line" "$subject" || fail "missing invariant: $line"; done
cases=$root/spec/alpha/lab/controller-helper-runtime-cases.v0
[ -f "$cases" ] && [ ! -L "$cases" ] || fail 'runtime cases unavailable'
[ "$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$cases" | /usr/bin/awk '{print $1}')" = addc112fdd9c88f5dc99a0f31eef452fbe1a54f808d348d3d13b8731da3103c0 ] || fail 'runtime case bytes escaped review'
[ "$(grep -Ec '^[AR][0-9][0-9][0-9]\\|' "$cases")" -eq 127 ] || fail 'runtime case count changed'
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-runtime-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-runtime-v0 is complete, source-only, inactive, and unwired'
