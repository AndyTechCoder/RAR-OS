# Modern Settings update transaction — integration decision

Status: implementation design under focused review. No syscall, process, disk
layout, image, or runtime authority is activated by this document. It refines
ADR0034 inside the existing M4 implementation PR; it is not a new permission
packet or a separate release gate.

## Decisions and alternatives

Keep signature, compatibility and rollback policy in the trusted core manager.
The nucleus supplies copying, immutable sealing, mapping, scheduling and
revocation mechanisms. Putting signature policy in the nucleus or accepting an
arbitrary caller's declared budget is rejected.

Boot must verify selected System bytes before Settings executes. Use a distinct bootstrap-construction state containing only System9, manager8
and physical idle. Settings and all other desktop principal bindings, grants
and published Boot mappings are initially absent; do not activate them and
revoke them afterward. Prepare the remaining initial processes unscheduled. Do not briefly execute an
older factory Settings image while checking a newer committed generation.
After the selected package passes verification and isolated health with no
previous Settings endpoint, finish every fallible desktop process/map/Boot/cap
preparation step. Publish all initial bindings and read-only Boot snapshots
under one IF=0 transaction, then mark processes runnable last. Preparation
failure leaves only bootstrap services, never a partially active desktop.
This initial boot barrier is distinct from live replacement, where all unrelated
apps and services must remain scheduled.

If the selected package fails signature/content/compatibility verification or
isolated health, boot may attempt exactly one explicit authorized fallback only
when the intact selected record names a prior package, the System transport and
journal are still usable, and that exact prior manifest/generation/payload can
be read and independently verified. Health-test it in a fresh incarnation,
durably commit the legal fallback selector preserving the high-water, then
publish the initial desktop. Any failure halts unavailable; no second fallback,
retry, implicit floor1 image or repeated boot-time attempt loop is permitted.
An I/O failure that makes the transport/journal sticky-failed or indeterminate
halts directly; it does not authorize alternate reads or another write. A
selected fallback record with no prior cannot fall back again. A separate
immutable factory copy cannot bypass these exact prior/high-water conditions.

A known virgin laboratory System image is explicitly provisioned from reviewed
factory package and selector bytes by the cloud fixture builder. The guest does
not infer permission to initialize media from zero bytes. Unknown or corrupt
media is not formatted. The factory/recovery package promised as immutable resides on the separate
read-only boot attachment, independently bound to reviewed boot inputs and the
public laboratory signing root. Provisioned System A/B package slots and
selector sectors are writable and reusable, not immutable recovery material.
Their initial contents cannot substitute for the separate boot copy.

A failed or indeterminate durable publication cannot be resolved from an absent
ACK. Close the transaction, refuse further mutation, and enter an explicit
reconcile-required controlled halt/reboot. The cloud test destroys the whole VM
and reboots the same retained disks; it does not reset RAM or inject recovered
state. A post-publication cutover invariant failure uses the same path. Retrying,
rewriting the selector back, or resurrecting a dead process is rejected.

## Authority and immutable staging

System9 alone owns System device access. Manager8 alone owns verification and
lifecycle decisions. Neither can select Data through a request field. Staging
requires a distinct narrow System-only copy authority, not overloading Device
or giving System the Manager capability. Their bounded protocol must have one
outstanding transaction, kernel-stamped identities and full-width monotonic
correlation IDs; late, duplicate, reordered or mismatched replies cannot unlock
a later transaction.

The kernel chooses and reserves the vacant Settings physical slot; no caller
supplies a slot, address, page table or executable pointer. It also owns one
bounded staging buffer. Copies are exact-length, strictly sequential and reject
gaps/overlaps; all bytes outside the declared package length are initialized.
A fresh nonreusable seal binds the reserved slot, package length and exact
immutable bytes. The reserved slot must be recorded in staged state: begin_trial
must not independently choose another vacant slot.

Only after copying is complete and all writable aliases are removed does the
manager receive a fixed read-only view. It invokes the existing manifest verifier
on that view using the correct intact committed generation policy. Manager-only
binding of verified metadata references the exact seal; no substitution of a
different mutable buffer is allowed. The kernel independently validates PE
mapping/W^X/resource geometry, not publisher policy.

The sealed package has exactly 384 manifest bytes followed by a PE payload of
at most 2,097,152 bytes: maximum logical length 2,097,536 bytes. A single
page-rounded buffer therefore needs 513 pages (2,101,248 bytes), plus separate
unmapped guard pages. Any disk-slot padding is outside this logical sealed
package and must be independently checked by the System codec. The present
36 MiB arena has no spare region sufficient for this maximum package. Implementation must reserve a separately guarded Modern-only region
and increase only the Modern allocation as needed. Do not overlap the existing
2 MiB process strides, page tables, stacks, Boot pages or framebuffer. Exact
addresses, chunk framing and syscall numbers must be specified alongside their
implementation and boundary tests, before that implementation is accepted.

Release/abort removes verifier views and every executable/user alias before
zeroing or reuse. The non-executable staging view is unmapped from manager before trial scheduling.
A slot remains unavailable until revocation, TLB invalidation and zeroing are
complete. Validate arena bounds, all aliases and page-table capacity under the
Modern-only memory extension; keep historical profile allocation unchanged. Full-u64 incarnation/seal exhaustion never wraps.

## One transaction, durable publication before destructive cutover

1. System reads intact selectors and verifies their legal chain. Manager verifies
   the referenced active/prior signed identities; checksums are not authority.
2. System writes only the inactive package, flushes and reads it back exactly.
   Active package and current selector remain unchanged.
3. Kernel copies/seals those bytes. Manager verifies framing, signature, payload,
   ABI/profile, state compatibility, generation and signed resource limits.
4. Kernel constructs a fresh unschedulable W^X process, then publishes TRIAL
   Boot with only its one-shot health grant and exact seal-derived identity.
   The stack has four mapped pages (16 KiB) and unmapped guards. Registers, SIMD,
   BSS, private memory and kernel stack are initialized.
5. The running IRQ0 path charges every trial preemption against the authenticated
   budget. Fault, timeout, abort or manager loss revokes logical and physical
   state. The verified prior component remains active. Healthy stays blocked.
6. Prepare all potentially fallible ACTIVE mappings, Boot contents, capability
   changes and handover validation while preserving the active component.
7. System commits the legal selector successor through Journal: alternate
   selector write, flush, both-record readback, exact protected-old check.
   Any failure locks mutation; Indeterminate never means not committed.
8. Only after durable ACK may the kernel perform the bounded IF=0 cutover:
   revoke old endpoint/caps/queued messages and physical mappings, publish the
   new binding/ACTIVE Boot, and schedule the candidate last. No fallible
   allocation or permission grant may remain after destructive publication.
9. Report install success only after durable publication and runtime cutover.
   Post-cutover fallback verifies and health-tests the permitted prior bytes in
   a new incarnation, publishes a legal fallback selector without lowering the
   high-water mark, and never reuses a killed process.

Cross-service messages bind transaction ID, exact seal, manifest digest, signed
generation, selected slots and selector sequence as applicable. Their concrete
wire codec and refusal tests accompany the implementation; an uncorrelated
boolean success message is insufficient.

## Live desktop handover

Shell and compositor require a kernel-authenticated, fixed-Settings binding
query or equivalent narrow notification. They must not accept arbitrary newer
generation values from an application. Named sends continue following the
logical Settings endpoint; old queued/in-flight messages remain rejected.

On an authenticated full-u64 Settings incarnation change, the compositor retains
last committed Settings pixels, discards only Settings uncommitted/staged state,
and starts a fresh per-incarnation version namespace. The shell updates only
its Settings expectation. Files, Terminal, storage, input, shell and compositor
are not restarted, and unrelated UI/storage state is unchanged.

## Recovery and proof limits

Recovery has immutable laboratory input plus System authority and no Data-write
authority. Repair only identified damaged System units and prove exact unchanged
Data bytes. Preserve every intact authoritative generation constraint. If both
selectors are corrupt and no trusted high-water can be recovered, stay
unavailable/read-only; do not silently reset to floor1. Whole-disk rollback
detection still requires an independent trust anchor and is not an Alpha claim.

Required tests include complete initial boot selection, real different Settings
code, active peers continuing during update, failed health, every durable
publication cut, fresh-incarnation fallback, stale seal/handle/queue rejection,
full-width binding changes, physical revocation/zero-before-reuse, staged view
immutability, exact stack bounds, interrupted repair and unchanged Data hashes.
Include failure at each initial desktop preparation step (no partial logical or
runnable publication), repeated selected-package health failure (halt rather
than recovery loops), and both-selector corruption (unavailable, no floor reset).
Source/model tests alone do not prove these runtime properties.
