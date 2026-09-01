#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-closure-observer-test-v0.fields
fail() { printf '%s\n' 'rar-alpha-controller-helper-closure-observer-test-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = a4633dffa6727ace25cdd69705d3c006709a085726d93e6a2c42f287bccb1238 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-observer-test-v0' "$subject" || fail 'schema changed'
grep -Fqx 'case_count=21' "$subject" || fail 'case count declaration changed'
[ "$(grep -Ec '^case\|O[0-9][0-9][0-9]\|' "$subject")" -eq 21 ] || fail 'observer cases incomplete'
grep -Fqx 'effect_rule=no-case-may-update-lock+inventory+profile+gate+readiness+workflow+GitHub' "$subject" || fail 'observer effect denial changed'
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-observer-test-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-observer-test-v0 is complete, source-only, inactive, and unwired'
