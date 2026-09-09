# Modern Data fault campaign integration (candidate)

Status: source implementation under validation; no fault scenario is activated.
The baseline persistence diagnostic remains separate. This does not complete
M4.1 or authorize a VM run, claim physical power-loss safety, or touch owner data.

## Observation and authority

fault_audit.record accepts only the bounded canonical ASCII JSON emitted by
the synthetic backend. Duplicate keys, nonfinite numbers, alternate whitespace,
malformed or oversized records fail through the existing owned-child stop path.
The fault plan is a fixed controller-owned operation/ordinal/effect/prefix.
scan binds every complete request and event by order, per-operation ordinal,
geometry and exact injection. Only Data may carry an injection.

For torn-cut and short-error, block_disk emits persisted_prefix_bytes only AFTER
the selected persistence prefix, fsync and descriptor identity checks succeed.
An underlying I/O error lacks that field and cannot impersonate the planned
short error. Flush prefixes are bounded by unique dirty sectors from completed
writes; successful flush clears that set. Failed devices may emit subsequent
request-only refusals, never another acknowledged operation.

observe requires exact cut terminal evidence plus exit20/problem cut/complete
pipe drain for cut effects. Error effects require no terminal record, no exit,
no transport problem and no EOF at observation. A process-poll race after a
terminal record cannot be labeled live. A cut whose pipe is still draining is
bounded to one second while the VM controller continues checking all peers.

## VM-control ordering

PlannedDataFault is a distinct, one-shot controller signal, not ValueError.
Unexpected System/Boot/Data/QEMU failures, stderr, panic/isolation markers and
invalid QMP messages remain failures. Exact pending QMP replies are consumed and
validated before the planned signal is delivered, but success cannot escape to
a screenshot/action caller before that signal. Partial unsolicited QMP records
are bounded; unknown replies/errors are not hidden. Error-mode continuation is
only for a fixed observation script; no mutation retry/remount or write unlock.
Cut delivery permits only whole-VM destruction, not further VM commands.

## Stop and frozen evidence

VM.fault_receipt accepts only its exact delivered signal and immutable tuple
fields. joined_fault checks the actual signal, performs an additional live check
for error effects, then destroys the whole QEMU before stopping all three
backends. Entry snapshots bind QEMU/peer liveness and effect-specific Data state.
QEMU must end with exact integer -9. Data cut requires20/cut/complete terminal;
error Data and peers require -9 or21/backend-failed, with complete matching
terminal records for21. Final serial drain also rejects panic markers, including
markers split across prior/final reads. Snapshot failure never skips cleanup.
No failed join/drain/identity/status grants frozen-image authority.

The caller must still freeze the retained images only after all joins, use the
independent authenticated Data oracle, verify unchanged System/Boot/header bytes,
and launch a fresh VM with fresh firmware and read-only Data. A record match or
joined receipt alone never proves old/new persistence or GUI recovery.

## Validation and unfinished integration

Pure cloud tests cover canonical parsing through the actual Backend.poll path,
real Disk.execute marker placement with inert I/O adapters, unique dirty-sector
bounds, child state/terminal races, QMP reply ordering, failure precedence,
one-shot delivery, exact stop receipts, cut and error teardown mutations, and
entry-snapshot exceptions with cleanup of every owned child. VM self-tests cover
final-drain panic fragments. Existing block and baseline regressions remain.

The launcher recipe includes the exact new helper. Changes are candidate source
only: actual fault case selection, reboot/UI/oracle scenarios, retained evidence
validation, source/controller exact binding and reviewed cloud activation are
still required. Signed live replacement, rollback and System-only recovery remain
separate M4 requirements. No crypto reference is linked into target images.

## Fixed interrupted-save scenario candidate

The unactivated `fault_scenarios.run(session, index)` API selects one of 60
controller-owned cases: each of the six CREATE/WRITE write and flush boundaries
with before/after cuts, no-success errors and 255-byte short/torn prefixes. Each publication flush contains one dirty
sector, so reversed ordering would be a no-op and is not counted as separate
coverage; multi-sector backend ordering remains a distinct test obligation. Selection is an exact integer, never a
path, command or user-controlled backend plan. Each invocation requires its own
fresh disposable cloud container and uniquely keyed public empty Data fixture.

The first guest receives the unpredictable challenge through Terminal only.
Only a typed planned-fault receipt can initiate successful scenario teardown;
unexpected errors fail and clean up. The whole first VM and all three backends
must be joined before frozen Data inspection. The independent authenticated
oracle must recover the exact complete revision, committed/burned slot lists,
next-slot position and non-exhausted classification permitted at that boundary.
A fresh second VM has read-only Data authority and receives only F1; its Files
pixels must independently match no file, an empty NOTE, or the complete value.
Data headers, System and immutable boot bytes are checked unchanged as applicable.

The orchestration tests replace all I/O and VM construction with inert objects;
they verify all fixed case paths, stop/freeze/reboot ordering, read-only observer
selection, unexpected-error rejection and failed-disk rejection. These are not
VM evidence. The existing four baseline visual scenes remain unchanged.

Still required before activation/acceptance: confirm actual baseline block
ordinals from retained cloud evidence, independent scenario/security review,
cloud source tests, retained fault-evidence validator and negative mutations,
trusted-controller integration, and actual exact-source faulted runs. The
candidate API explicitly returns `observed-not-independently-accepted` and
`milestone_complete: false`. No fault scenario is activated by this change.
