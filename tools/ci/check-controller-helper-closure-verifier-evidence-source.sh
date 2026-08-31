#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields
fail() { printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-evidence-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = perl: warning: Setting locale failed.
perl: warning: Please check that your locale settings:
	LC_ALL = "C.UTF-8",
	LC_CTYPE = "C.UTF-8",
	LANG = "C.UTF-8"
    are supported and installed on your system.
perl: warning: Falling back to the standard locale ("C").
panic: locale.c: 4486: Could not change LC_CTYPE locale to C.UTF-8, errno=9 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-evidence-v0' "$subject" || fail 'schema changed'
for line in \
 'case_count_rule=147-disposition+50-precedence+12-fault=209,each-required-ID-exactly-once,canonical-kind-then-numeric-order' \
 'anti_replay_rule=run-nonce+root+controller+source+fixture-image+tool-pins-tuple-never-reused,no-cross-revision-evidence' \
 'no_success_effect=lock+inventory+profile+gate+readiness+workflow+GitHub+compiler-use+helper-build-unchanged'; do grep -Fqx "$line" "$subject" || fail "missing invariant: $line"; done
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-verifier-evidence-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-evidence-v0 is complete, source-only, inactive, and unwired'
