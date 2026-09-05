# Modern-v0 lifecycle mechanism

Status: experimental candidate model contract; no kernel runtime activation.
This is separate from Desktop-v0 and stable RID. See modern-system-v0.md for
signed layer policy. No disk or executable mapping is performed by this model.

## Ownership and identities

The nucleus owns all mutable process/capability/queue/binding state. Public Rust
methods here are kernel-internal, not syscalls. The real trap dispatcher must
derive caller slot from its current process, never an untrusted argument.
No userspace service may call grant, destroy, fault, preempt or choose a peer
handle table. The eventual ABI must validate pointers and copy bounded inputs.

16 physical slots and10 logical principals are reserved. Initial protected
principals0..6,8,9 map to matching physical slots at incarnation1. Slot7 is
vacant, not a preconstructed spare process. Slots10..15 remain unused. Only
Settings principal5 currently has a replacement path; slots5 and7 alternate.
Principal8 is the lifecycle manager; principal9 is planned System storage.
This model's bootstrap graph includes only IPC/lifecycle handles. Device,
framebuffer, filesystem authority and the complete application grants are not
implemented or granted here.

Every replacement consumes a new global64-bit incarnation and64-bit trial
token. Both counters use checked addition; exhaustion refuses before mutation.
Cap handles remain caller-local, with upper32 bits slot generation and lower32
bits one-based index. Revocation clears object/rights and advances generation;
overflow permanently retires that cap slot. Process reuse preserves the cap
table's revocation history; private memory must separately be zeroed by runtime.

## Capability types and routing

- NamedSend(principal): SEND only. This intentionally follows an authorized
  logical binding change. It never grants receive, manage or device rights.
- Receive(slot,incarnation): RECEIVE only; only its exact live owner redeems it.
- TrialHealth(slot,incarnation,token): one-shot HEALTH only.
- Manager: MANAGE only, provisioned solely to principal8.
- No numeric object request or manifest field creates a capability.

Each table has12 entries. Self receive index0, Settings shell send1, compositor
send2, trial health9 and manager10 are reserved. Shell holds its named Settings
send at index5. The real kernel alone fills a process's read-only bootstrap with
the permitted handles; the model handle() accessor is not a userspace operation.

Messages contain logical sender principal,64-bit sender incarnation, length and
128-byte zero-filled bounded payload. These are stamped by the kernel, not
copied from caller metadata. Queues hold four messages, at most two from one
sender incarnation. Full/empty/stale returns are explicit and bounded.
Service protocols must provide bounded retries/correlation where required.
This model does not promise delivery across a lifecycle cutover.

## Prepare, health and cutover

Only an active principal8 with its Manager handle may begin, abort or cut over.
Begin refuses concurrent trials, unknown image seals, unavailable slots,
invalid authenticated budgets, stale previous bindings, recovery-required state
or exhausted counters. The caller supplies no execution budget. It reserves a
vacant Settings slot, advances incarnation/token, clears its queue and grants
only the exact one-shot health capability. It grants no production receive/
send, surface, input, storage, port, framebuffer or management access.

A seal must match a private kernel-owned staged record binding digest,
generation and signed trial budget. Begin derives all three from that record
and consumes it; an arbitrary nonzero ID grants nothing. There is intentionally
no production record-insertion API yet: only model tests create fixtures.
The pending staging bridge must bind immutable, fully staged bytes to verified
metadata. The policy service verifies those exact bytes; the kernel enforces
the copy/mapping boundary. This private model gate is not an implemented seal.
A real candidate is constructed unschedulable with zeroed private memory,
guarded stack and final W^X mappings; writable aliases must be removed first.

TrialReady must come from the exact candidate slot/incarnation, present the
current trial token and redeem its one-shot health handle. It consumes that
handle and moves the process to Healthy. Healthy is NOT runnable: it waits for
manager cutover with no production authority. Trial execution is bounded by
1..100 actual timer preemptions. Timeout destroys candidate authority; health
does not grant any extension. Real runtime must also bound wall-clock/liveness
and treat faults through the kernel fault path.

Cutover rechecks exact trial token, Healthy state/incarnation, unchanged prior
binding and prior liveness. It prepares the candidate's Receive and named
shell/compositor grants in a temporary cap table before any shared mutation.
Grant failure leaves active state/binding untouched. Thereafter, under one
single-CPU IF-disabled transaction, it:

1. Revokes every old-process capability, destroys its receive queue, and removes
   every outbound old-principal/incarnation message from all queues.
2. Clears the candidate queue, installs phase-appropriate production caps and
   marks it active as logical principal5.
3. Rebinds the named endpoint to the candidate and consumes the trial record.

This is atomic in-memory publication only. The model does not itself copy
executable bytes, schedule CPU contexts, retain surfaces, write or flush a
System selector, or decide policy-level signed fallback. Cross-service/device
commit ordering requires the integrated lifecycle/storage state machine.

## Faults, cancellation and recovery

Abort, candidate fault and trial timeout revoke candidate caps, clear its state,
consume the trial and preserve the previous active binding. A stale token never
redeems. Fault events carry the exact slot/incarnation endpoint; timer events
also carry the trial token. Delayed events from an earlier occupant cannot kill
or charge the current process in a reused slot.

A lifecycle-manager fault cancels any Trial or Healthy candidate, clears staged
state and marks controlled recovery required. The prior active Settings remains
untouched. No replacement manager or new authority is implicitly created;
further trial starts fail closed. Runtime-controlled recovery remains pending.

If the old active instance dies during a trial, cutover fails stale;
the manager must abort/reconcile, not silently reuse a changed binding.

After active Settings faults, its binding becomes unavailable and all messages
from its old incarnation are purged. Recovery must separately reverify the exact
committed fallback manifest/payload and health-test a cleanly constructed new
process. That process receives a fresh incarnation, even when reusing slot5.
A killed process is never resurrected. Other principals remain unchanged.

The real compositor must keep the last committed view while resetting only the
new Settings incarnation's staging/version namespace after authorized handover.
Shell and service clients must accept kernel-authenticated incarnation changes;
they must not continue Desktop's hard-coded generation1 rule. This UI change is
pending. Data storage and recovery permissions remain outside this IPC model.

## Tests, migration and remaining integration

Tests cover zero/wrong/stale handles and actors, concurrent trial refusal,
production-access denial during health, one-shot ready, exact queue purge,
named-send continuity, empty new receive queue, old-cap rejection, timeout,
abort, faults before/after cutover, fresh recovery incarnation, independent
principal preservation, counter exhaustion and queue/cap retirement. Additional
cases cover authenticated staging budgets, consumed seals, delayed events after
slot reuse, and manager failure during Trial and Healthy.

These are model tests, not a boot or isolation proof. Positive signed loading,
actual sealing/W^X/private-memory clearing, timer/fault assembly integration,
Modern ABI, UI handover and durable journal orchestration remain required before
activation. The runtime must use these mechanisms or prove conformance; a
passing standalone model cannot substitute for real paths.

No persistent state or stable ABI is introduced here. Desktop-v0 remains
unchanged. Replacement of this model requires the same state/capability/IPC
conformance behavior and explicit migration for any future public ABI.
