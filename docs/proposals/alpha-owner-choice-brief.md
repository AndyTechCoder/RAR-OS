# Alpha Owner Choice Brief

Status: Explanatory only — no decision is recorded by this document

This page summarizes the three open Alpha proposals in plain language. The
proposal files remain authoritative. Creating or reading this page does not
accept an ADR, authorize execution, or make a blocked contract ready.

## Decisions needed before the bootable Alpha

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

## Decision needed later, before graphics and input

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

## What accepting the recommended set would authorize

It would authorize the team to finish and independently review the matching
experimental specifications and source implementation. It would not by itself
authorize cloud provisioning, credentials, builds, target execution, VM launch,
Mac execution, merging, production compatibility claims, or a `ready` status.
Every existing evidence and safety gate would remain in force.

When the owner is available, an unambiguous approval can be recorded as:

`Approve ADR 0023 Alternative C, ADR 0024 Alternative A, and ADR 0022 Alternative C.`

The owner may approve only ADRs 0023 and 0024 first and defer ADR 0022 until
Milestone E. Any different selection must name the ADR and alternative exactly.
