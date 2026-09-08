# Modern Settings trial entry

This source integration implements the application side of the existing
Modern-v1 TrialReady/Healthy/Active protocol, not kernel candidate creation,
executable staging, process cutover or M4.2 acceptance.

Normal active entry still starts the existing application role. A valid trial
bootstrap may only identify Settings with its one-shot health grant. Trial entry
constructs and checks the same bounded initial Settings view/line encoding used
by the active application, without IPC, surfaces, devices or heap allocation.
It then calls TRIAL_READY with the exact bootstrap health handle and token.

The existing kernel syscall consumes health and blocks the candidate. No trial
Settings GUI operation is attempted before that call. Only a future complete
kernel cutover may resume the healthy context. On return, the application takes
a fresh volatile snapshot of the fixed kernel-owned read-only Boot mapping and
requires a valid ACTIVE Settings bootstrap with the same full64-bit incarnation
and entry address. The old health token/grant must be gone and the exact active
grant graph present. A premature wake, stale/truncated identity, missing grant,
retained health authority or changed entry fails closed. Abort/fault/timeout must
destroy the candidate, never resume its discarded context.

Volatile reads use an aligned initialized mapping and retain no Rust reference
across the trap. The kernel may replace bootstrap contents only while this
process is not running, and must finish publication before scheduling it. This
is a kernel integration obligation, not an atomicity guarantee supplied by
volatile access. Receiver checks are consistency checks, not security authority.

The pure cloud harness tests bootstrap transition refusal cases and exact
Settings view/encoding. Existing cloud checks compile the actual service entry
to an object. Neither proves that a real trial ran or a kernel cutover happened.
Actual stage/map/seal, signed16KiB stack, timer budget, W^X/revocation, peer/surface
handover and durable System publication remain required. Normal-boot GUI bytes
are unchanged. No ABI layout, syscall number, signature policy, device grant,
target dependency or owner-data format changes.
