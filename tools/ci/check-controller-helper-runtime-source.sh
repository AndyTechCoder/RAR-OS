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
[ "$actual" = 20d85fcafc0b0fefd44adcef73303afc1e3232a91afcdcb8292498f2017b8ad1 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-runtime-v0' "$subject" || fail 'schema changed'
grep -Fqx 'descriptor_count=6' "$subject" || fail 'descriptor count changed'
[ "$(grep -c '^descriptor|' "$subject")" -eq 6 ] || fail 'descriptor rows incomplete'
for line in \
 'descriptor|3|helper-executable|regular,owner-controller,mode-0500,nlink-1,O_RDONLY+O_NOFOLLOW+O_CLOEXEC,exact-binary' \
 'descriptor|4|journal-root|directory,owner-controller,mode-0700,O_RDONLY+O_DIRECTORY+O_NOFOLLOW+O_CLOEXEC,CLOEXEC-cleared-only-in-stopped-child-after-parent-validation' \
 'descriptor|5|source-root|directory,owner-controller,mode-0700,O_RDONLY+O_DIRECTORY+O_NOFOLLOW+O_CLOEXEC,CLOEXEC-cleared-only-in-stopped-child-after-parent-validation' \
 'descriptor|6|destination-root|directory,owner-controller,mode-0700,O_RDONLY+O_DIRECTORY+O_NOFOLLOW+O_CLOEXEC,CLOEXEC-cleared-only-in-stopped-child-after-parent-validation' \
 'descriptor|7|manifest-root|directory,owner-controller,mode-0700,O_RDONLY+O_DIRECTORY+O_NOFOLLOW+O_CLOEXEC,CLOEXEC-cleared-only-in-stopped-child-after-parent-validation' \
 'descriptor|8|control-channel|full-duplex-seqpacket,controller-created,CLOEXEC-cleared-only-in-stopped-child-after-parent-validation' \
 'descriptor_rule=all-other-descriptors-closed-before-exec,descriptor-numbers+purposes-exact,no-ambient-root+cwd+path+proc-fd-reopen-authority' \
 'inheritance_rule=parent-opens-3-through-8-CLOEXEC+validates-exact-table;stopped-child-clears-FD_CLOEXEC-only-on-4-through-8+F_GETFD-revalidates-4-through-8-numbers+purposes+FD_CLOEXEC-absent+3-still-FD_CLOEXEC+all-other-FDs-closed;child-acks-only-after-validation;parent-authorizes-execveat-only-after-exact-ack;any-clear+validation+ack-failure-rejects-before-helper-code' \
 'start_protocol=durable-start-authorized,spawn,child-stop-before-helper-code,parent-validates-handle+descriptors,parent-releases-bounded-pre-exec,child-clears+revalidates-inherited-descriptors,child-acks-pre-exec-ready,parent-authorizes-execveat,bounded' \
 'exec_rule=execveat-helper-fd-empty-path+AT_EMPTY_PATH,argv-exact,empty-environment,no-shell+PATH+interpreter+url+credential' \
 'phase_boundary_rule=C1-source-validation-must-not-invoke-compiler+linker+helper+target+container+VM+emulator+firmware+network+credential;exec_rule-is-a-future-H1C-runtime-requirement-only+grants-no-C1-execution-authority;helper-execution-remains-forbidden-until-separately-reviewed-H1C-activation' \
 'process_rule=controller-owned-pidfd-or-equivalent-nonreusable-handle,bare-PID-never-authority,handle-closed-after-reap' \
 'recovery_rule=source-never-deleted,exclusive-attempt-roots,bounded-exact-enumeration,inventory-durable-before-delete,identity-match-before-each-delete' \
 'attempt_ceiling=3-normal-attempts+3-recovery-sessions,fourth-is-durable-blocked,no-reuse+loop'; do grep -Fqx "$line" "$subject" || fail "missing invariant: $line"; done
cases=$root/spec/alpha/lab/controller-helper-runtime-cases.v0
[ -f "$cases" ] && [ ! -L "$cases" ] || fail 'runtime cases unavailable'
[ "$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$cases" | /usr/bin/awk '{print $1}')" = df51cc29ce2ce905d9935a25f1ef9803a9cdf3706b2144b8de347fd324ba79f2 ] || fail 'runtime case bytes escaped review'
grep -Fqx 'R001|descriptor-table-exact|accept|no-readiness-update' "$cases" || fail 'valid descriptor-table oracle changed'
grep -Fqx 'R013|exact-descriptor-exec|accept|no-readiness-update' "$cases" || fail 'exact descriptor execution oracle changed'
[ "$(grep -Ec '^[AR][0-9][0-9][0-9]\\|' "$cases")" -eq 127 ] || fail 'runtime case count changed'
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-runtime-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-runtime-v0 is complete, source-only, inactive, and unwired'
