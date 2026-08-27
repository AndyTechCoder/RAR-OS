#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=${1-$root/spec/alpha/lab/controller-handoff-attempt-v0.fields}
cases=${2-$root/spec/alpha/lab/controller-handoff-attempt-cases.v0}
fail() { printf 'controller handoff attempt contract rejected: %s\n' "$1" >&2; exit 1; }

for file in "$contract" "$cases"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
    size=$(/usr/bin/stat -f %z "$file" 2>/dev/null || /usr/bin/stat -c %s "$file")
    [ "$size" -le 32768 ] || fail "input exceeds bound: $file"
    /usr/bin/od -An -tx1 "$file" | /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)' && fail "input contains NUL or CR: $file"
done

sha_file() {
    env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}
[ "$(sha_file "$contract")" = 283c9e7ae99b0383aa6c02fea5f6bda836dc840107a892278a53bfd5d83051df ] || fail 'contract is not the exact reviewed byte set'
[ "$(sha_file "$cases")" = 69a574038d6574bae00e0be1c368bac59c3a0850d0eb5e359721950de14a72a9 ] || fail 'case table is not the exact reviewed byte set'

/usr/bin/awk -F '|' '
    NF == 1 {
        if ($0 !~ /^[a-z0-9_]+=[^[:cntrl:]]+$/) bad=1
        split($0, pair, "=")
        if (++single[pair[1]] != 1) bad=1
        next
    }
    $1 == "wire_field" {
        if (NF != 5 || $2 !~ /^Attempt[A-Za-z0-9]+V0$/ || $3 !~ /^[0-9]+$/ ||
            $4 !~ /^[a-z][a-z0-9_]*$/ || ++field[$2 SUBSEP $4] != 1) bad=1
        type=$5
        if (type ~ /^u16(:[0-9]+)?$/) width=2
        else if (type ~ /^(u32|i32)(:[0-9]+)?$/) width=4
        else if (type ~ /^(u64|i64)(:[0-9]+)?$/) width=8
        else if (type ~ /^bytes:[0-9]+/) {
            split(type, parts, ":"); width=parts[2]+0
            if (parts[3] != "" && length(parts[3]) != width) bad=1
        }
        else bad=1
        if (($3+0) != next_offset[$2]) bad=1
        next_offset[$2]=($3+0)+width
        next
    }
    { bad=1 }
    END {
        if (next_offset["AttemptActiveHeaderV0"] != 512) bad=1
        if (next_offset["AttemptRootV0"] != 128) bad=1
        if (next_offset["AttemptExpectedEntryV0"] != 256) bad=1
        if (next_offset["AttemptTransitionV0"] != 512) bad=1
        if (next_offset["AttemptRecoveryHeaderV0"] != 256) bad=1
        if (next_offset["AttemptRecoveryEntryV0"] != 192) bad=1
        exit bad ? 1 : 0
    }
' "$contract" || fail 'field grammar, uniqueness, or contiguous wire layout is invalid'

/usr/bin/awk -F '|' '
    NR == 1 { if ($0 != "schema=rar-alpha-controller-handoff-attempt-cases-v0") bad=1; next }
    NR == 2 { if ($0 != "id|contract|expected") bad=1; next }
    NR > 2 {
        if (NF != 3 || $1 !~ /^[a-z0-9][a-z0-9-]*$/ ||
            $2 !~ /^[a-z0-9][a-z0-9-]*$/ || $3 !~ /^(accept|reject)$/ ||
            ++seen[$1] != 1) bad=1
        count++
    }
    END { if (count != 97) bad=1; exit bad ? 1 : 0 }
' "$cases" || fail 'case grammar, uniqueness, or count is invalid'

require_line() {
    [ "$(/usr/bin/grep -Fxc -- "$1" "$contract")" -eq 1 ] || fail "required contract row missing or duplicated: $1"
}
for row in \
    'status=experimental-host-only' \
    'journal_root_rule=controller-created-task-owned,mode-0700,current-controller-owner,not-symbolic,exclusive,no-helper-fd,no-role-mount' \
    'active_open_rule=O_RDWR+O_CREAT+O_EXCL+O_CLOEXEC+O_NOFOLLOW,mode-0600,one-active-attempt' \
    'bound_rule=roots-3..4,expected-entries-1..999,attempt-ordinal-1..3,recovery-ordinal-0..3,transition-count-1..4096,recovery-entries-0..1998,journal-total-maximum-3145728' \
    'timeout_rule=watchdog-1..1200-seconds,termination-grace-1..30-seconds,total-controller-1..3600-seconds,all-stored-in-active-header,outer-monotonic-process-handle' \
    'active_total_rule=512+root-count*128+expected-count*256,maximum-256768,exact-EOF' \
    'path_rule=descriptor-relative-canonical-basename-only,no-stored-path,no-url,no-command,no-environment,no-credential' \
    'root_table_rule=canonical-order,source-indices-contiguous-1..N,N-in-1..2,exactly-one-destination-at-N+1,exactly-one-manifest-at-N+2,phase-role-source-count-valid' \
    'expected_rule=canonical-unique-basenames,ordinal-1..999,maximum-role-bound,flags-zero,unused-tails-zero,table-canonical-output-order,every-source-root-index-references-declared-source' \
    'transition_chain_rule=sequence-starts-1+increments-1,previous-sha256-active-sha256-at-sequence-1,then-prior-transition-sha256,no-missing+duplicate+reorder+fork' \
    'pre_running_failure_rule=prepared->blocked-for-preauthorization-policy-only,start-authorized->blocked-for-spawn-failure-or-exited-success-or-exited-failure-or-stop-requested-or-blocked,observed-pre-running-exit-zero->exited-success,observed-pre-running-exit-nonzero->exited-failure' \
    'spawn_failure_field_rule=blocked+exit-status--2147483648+error-spawn+receipt-zero+inventory-zero,no-process-and-no-recovery-cleanup-authority' \
    'failure_state_rule=running-observed->exited-failure-or-stop-requested-or-blocked,stop-requested->stopped-observed-or-blocked,exited-success+validation-failure->recovery-required,exited-failure-or-stopped-observed->recovery-required,recovery-required->recovery-inventory-durable->discarded-or-blocked,any-journal-or-policy-uncertainty->blocked' \
    'takeover_pre_recovery_rule=new-session+matching-descriptors+independently-observed-stopped-helper-may-enter-recovery-required-from-running-observed+exited-failure+stop-requested+stopped-observed+exited-success+outputs-validated,never-resume-validation-or-commit,otherwise-blocked' \
    'takeover_field_rule=running-observed-or-stop-requested-without-observed-exit-retains-exit-sentinel+receipt-zero+uses-error-recovery,exited-failure-or-stopped-observed-retains-exact-exit+receipt+cause,exited-success-or-outputs-validated-retains-zero-exit+receipt+uses-error-recovery' \
    'transition_field_rule=phase+role+active+helper-match-active,recovery-ordinal-0-before-recovery+1..3-during-recovery,session-nonce-exact-per-session-rule,reserved-zero,monotonic-nanoseconds-nondecreasing' \
    'exit_status_rule=-2147483648-before-observed-exit-or-no-process-spawn-failure,0-only-exited-success,observed-nonzero-exited-failure,observed-status-stopped-observed,recovery+terminal-states-retain-last-observed-status' \
    'error_code=0:none,1:spawn,2:timeout,3:cancelled,4:exit-nonzero,5:journal,6:validation,7:recovery,8:identity,9:sync,10:policy' \
    'error_rule=zero-for-normal-pre-exit+exited-success+outputs-validated+committed,validation-failure-after-exited-success-uses-error-validation,nonzero-for-exited-failure+stop-requested+stopped-observed+recovery-required+blocked,discarded-retains-recovery-cause' \
    'digest_state_rule=receipt-sha256-nonzero-only-from-first-observed-exit-and-immutable-thereafter,inventory-sha256-zero-through-recovery-required-even-when-existing-inventory-was-validated-out-of-band+first-nonzero-at-recovery-inventory-durable+immutable-thereafter,unused-digests-all-zero' \
    'running_rule=live-nonreusable-controller-owned-process-handle-required,pid-never-authority' \
    'commit_rule=committed-durable-before-active-removal,active-remove-after-device+inode-match+journal-root-fsync,next-phase-requires-both' \
    'inventory_open_rule=descriptor-relative-journal-root-fd,O_RDWR+O_CREAT+O_EXCL+O_CLOEXEC+O_NOFOLLOW,mode-0600' \
    'inventory_publication_rule=absent-inventory-created-once-and-header-sequence+previous-transition-sha256-exactly-bind-origin-durable-recovery-required,write-exact+same-FD-parse+identity-recheck+EOF+fdatasync+close+journal-root-fsync-before-recovery-inventory-durable-or-delete' \
    'inventory_reuse_rule=existing-inventory-open-readonly+CLOEXEC+NOFOLLOW,regular+owner+mode+nlink+exact-EOF+same-FD-parse+identity-recheck+sha256+origin-binding,never-replace+extend+delete+republish,recovery-inventory-durable-and-all-later-transitions-repeat-exact-inventory-sha256' \
    'inventory_presence_rule=absent-through-recovery-required-means-zero-digest+may-create-once,present-after-crash-before-recovery-inventory-durable-means-out-of-band-validate+repeat-recovery-required-with-zero-digest+append-exactly-one-recovery-inventory-durable-with-real-digest,at-or-after-recovery-inventory-durable-means-present+same-nonzero-digest,unexpected-presence-or-absence-blocks' \
    'inventory_header_rule=origin-recovery-ordinal-1..3+origin-session-nonce-equal-durable-origin-recovery-required-transition,sequence+previous-transition-sha256+active+attempt+phase+role+roots+expected-table-exact-origin-binding,flags-absent,all-reserved-zero' \
    'inventory_rule=descriptor-relative-exact-enumeration,destination+manifest-only,canonical-order,regular,current-controller-owner,mode-0600,nlink-1,size-bounded,digest-readable,entry-flags-zero,all-reserved-zero' \
    'recovery_rule=source-never-deleted,inventory-durable-before-delete,remove-only-inventory-entry-after-device+inode+type+owner+mode+links+size+sha256+mtime+ctime-match,missing-means-idempotently-removed,changed-means-blocked' \
    'root_discard_rule=fsync-empty-destination+manifest-roots,retain-empty-attempt-root-directories,no-parent-fd,no-root-directory-unlink' \
    'terminal_finalize_rule=restart-with-durable-committed-or-discarded+active-present-performs-identity-checked-active-unlink+journal-root-fsync-only,no-work-rerun,active-absent-before-root-fsync-requires-root-fsync-finalization,next-phase-blocked-until-complete' \
    'restart_bound_rule=recovery-ordinal-maximum-3,each-distinct-resumed-session-increments-exactly-once,fourth-recovery-restart-durable-blocked,no-unbounded-retry' \
    'session_rule=prepared-through-original-session-transitions-equals-active-controller-session-nonce,normal-session-nonce-never-changes,recovery-takeover-only-under-takeover-pre-recovery-rule-or-after-durable-recoverable-state+new-nonzero-nonce-distinct-from-all-prior+ordinal-exactly-prior-plus-one,first-takeover-transition-enters-recovery-required-from-allowed-pre-recovery-state-or-repeats-latest-recovery-required-or-recovery-inventory-durable-state,all-later-transitions-in-that-session-retain-takeover-nonce+ordinal' \
    'restart_rule=session-mismatch-never-resume-normal-work,recovery-only-with-matching-resupplied-descriptors+independently-observed-stopped-helper+unchanged-origin-transition+inventory-presence-rule,otherwise-blocked' \
    'failure_rule=blocked-retains-active-marker+bounded-journal,no-next-phase,no-publication,no-release,no-merge,no-signing-acceptance' \
    'activation_rule=source-contract-only,no-helper-spawn,no-process-FD-protocol,no-cloud-command,no-ready-identity,no-Mac-execution'; do
    require_line "$row"
done

for required_case in \
    'canonical-commit|state|accept' \
    'canonical-recovered-discard|recovery|accept' \
    'spawn-failure-with-observed-status|start|reject' \
    'zero-exit-before-running-observed|exit|accept' \
    'validation-failure-after-success|validation|accept' \
    'second-active-attempt|active|reject' \
    'recovery-helper-may-run|recovery|reject' \
    'inventory-entry-identity-changed|cleanup|reject' \
    'source-root-deletion|cleanup|reject' \
    'partial-recovery-inventory|inventory|reject' \
    'inventory-entry-size-changed|cleanup|reject' \
    'inventory-entry-content-changed|cleanup|reject' \
    'stale-active-after-committed|active|accept' \
    'stale-active-after-discarded|active|accept' \
    'crash-after-active-unlink-before-root-sync|restart|accept' \
    'crash-after-inventory-before-durable-transition|restart|accept' \
    'pre-inventory-takeover-zero-digest|state|accept' \
    'existing-inventory-repeated-recovery-required-zero-digest|state|accept' \
    'existing-inventory-repeated-recovery-required-nonzero-digest|state|reject' \
    'exact-inventory-reuse-new-session|restart|accept' \
    'attempted-second-inventory|inventory|reject' \
    'inventory-wrong-origin-binding|inventory|reject' \
    'session-nonce-change-mid-recovery|binding|reject' \
    'crash-after-running-observed|restart|accept' \
    'crash-after-exited-failure|restart|accept' \
    'crash-after-stop-requested|restart|accept' \
    'crash-after-stopped-observed|restart|accept' \
    'crash-after-exited-success-before-validation|restart|accept' \
    'crash-after-outputs-validated|restart|accept' \
    'recovery-ordinal-skip|restart|reject' \
    'new-entry-after-inventory|cleanup|reject' \
    'fourth-attempt|bound|reject' \
    'fourth-recovery-restart|bound|reject' \
    'next-phase-after-discarded|progression|reject' \
    'next-phase-after-blocked|progression|reject'; do
    [ "$(/usr/bin/grep -Fxc -- "$required_case" "$cases")" -eq 1 ] || fail "required case missing or changed: $required_case"
done

printf '%s\n' 'controller handoff attempt contract validated: states=13 cases=97 activation=forbidden'
