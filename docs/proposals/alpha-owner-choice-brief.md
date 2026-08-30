# Alpha Owner Choice Brief

Status: Historical decision aid — canonical decisions recorded elsewhere

This page preserves the plain-language choice presented before acceptance.
Canonical decisions are `../adr/0022-*.md` through `../adr/0026-*.md`, bound by
`../approval-record.md`; the retained proposal files are historical only. This
page grants no execution authority and cannot make a blocked contract ready.

## Decisions presented before the bootable Alpha

### ADR 0023 — How exact should the temporary Alpha boot design be?

Recommended: **Alternative C**.

RAR would define one exact, replaceable Alpha-only boot layout and implement it
with RAR-owned code. That includes deterministic disk-image bytes, fixed memory
areas, complete firmware-memory conversion, one verified virtual timer source,
and enforced non-executable/write-protected memory rules.

- Benefit: implementers receive one secure, reproducible contract instead of
  inventing hidden boot behavior in code.
- Cost: more specification work before Milestone A starts.
- It does not make this temporary layout a production standard.

### ADR 0024 — Where should the trusted controller helper be built?

Recommended for Alpha: **Alternative A**.

An approved isolated Linux cloud runner would verify one exact compiler bundle,
build the small host-side helper twice with networking disabled, and accept it
only when both binaries are identical. The helper is Development Lab plumbing;
it is not part of RAR OS and never enters an OS image.

- Benefit: the smallest route to trustworthy helper evidence for the Alpha.
- Cost: the cloud runner temporarily holds tightly bounded compiler authority.
- Production can later move this build into the more isolated controller-tool
  image described by Alternative C without changing RAR OS.

These two decisions are independent: ADR 0023 defines guest boot behavior;
ADR 0024 defines how a cloud-only host helper is produced. Both are required
before Milestone A, but neither authorizes Mac execution or a VM launch.

### ADR 0026 — How do components, apps, and recovery data reach the OS?

Recommended: **Alternative C**.

Root would stage one minimal Core-bootstrap image, one component bundle, and
separate immutable initial system/preserved-data images before Nucleus starts.
Recovery would pass exact bounded memory sources in a private Alpha envelope.
Nucleus would only map and start the fixed Core bootstrap; Core could then load
real isolated components without putting app or lifecycle policy inside
Nucleus. Recovery could rebuild mutable system state from the retained
read-only source without ambient disk access or preserved-data write authority.

- Benefit: one honest delivery/state boundary supports C–F without repeatedly
  rewriting the A boot chain.
- Cost: the private envelope and four source formats must be fully
  specified and reviewed before A.
- Alpha remains memory-backed at runtime and makes no promise that changes
  persist after VM shutdown; production storage comes later.

ADR 0026 is required before the final Milestone A boot contract becomes ready.
It authorizes no execution or production storage claim.

## Decision presented for the pre-Milestone B gate

### ADR 0025 — How should tests work before keyboard and GUI support exist?

Recommended: **Alternative B**.

RAR would create a new reviewed test-plan version. Milestones B–D would start
their deterministic tests automatically, in strict order, instead of pretending
to consume keyboard shortcuts before an input driver exists. The post-crash GUI
continuity check would stay in the same sequence but become mandatory from
Milestone E, when a real GUI exists.

- Benefit: early milestones remain honest and testable without hidden keyboard
  or GUI code, while later milestones still prove the GUI survives a component
  crash.
- Cost: the trusted controller and evidence verifiers must bind a new exact
  plan version and reject the old version for every new run.
- The active plan remains unchanged until this decision is accepted and the
  replacement passes independent review.

ADR 0025 is required before Milestone B. It does not authorize target code,
cloud execution, or input/GUI implementation.

## Decision presented for the later graphics/input gate

### ADR 0022 — How should the Alpha GUI receive real device authority?

Recommended: **Alternative C**.

RAR would keep the stable R0-002 boot information unchanged and wrap it in a
private, disposable Alpha envelope. The envelope grants only exact framebuffer
and input authority bound to the exact reviewed Alpha machine-profile digest. Graphics,
input services, and apps receive reduced capabilities rather than ambient raw
device access.

- Benefit: genuine guest-rendered graphics and guest-consumed input without
  prematurely defining a production hardware standard.
- Cost: the exact input transport still depends on reviewed, pinned Development
  Lab QEMU capability evidence.
- This decision does not block Milestones A–D; it is required before Milestone E.

## Recorded owner decision

The owner accepted the exact five-choice sentence below on 2026-08-29. This
authorizes the team to finish and independently review the matching
experimental specifications and source implementation. It would not by itself
authorize cloud provisioning, credentials, builds, target execution, VM launch,
Mac execution, merging, production compatibility claims, or a `ready` status.
Every existing evidence and safety gate would remain in force.

Accepted sentence:

`Approve ADR 0022 Alternative C, ADR 0023 Alternative C, ADR 0024 Alternative A, ADR 0025 Alternative B, and ADR 0026 Alternative C.`

The authoritative approval fields are in `../approval-record.md`. ADRs 0023,
0024, and 0026 are required before A; ADR 0025 is required before B.
ADR 0022 may be implemented at E, but accepting it before A permits the common
envelope and ownership consequences to be reviewed together. Any different
selection must name the ADR and alternative exactly.
