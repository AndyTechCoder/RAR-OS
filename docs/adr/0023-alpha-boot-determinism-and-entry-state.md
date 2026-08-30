# ADR 0023: Alpha Boot Determinism and Entry State

Status: Accepted — 2026-08-29
Decision: Alternative C

Approval basis: explicit owner approval of the repository's exact five-choice
sentence on 2026-08-29. Acceptance selects one private deterministic Alpha
profile; it does not complete its field contract or authorize target builds,
images, provisioning, or execution.

## Context

Accepted ADR 0021 selects the authentic Alpha chain UEFI Root → freestanding
Recovery → freestanding Nucleus. Its first candidate field contract correctly
fixed the broad boundary but an independent architecture, correctness, and
security review found five details that implementation must not invent:

- every mandatory and computed protective-MBR, GPT, FAT32, directory, and CRC
  byte needed for reproducible images;
- fixed placement, ceilings, ownership changes, and exhaustion behavior for the
  final R0-002 entry and all Recovery-produced source bytes;
- a total UEFI memory-type and attribute conversion;
- one authenticated timer-frequency source; and
- fail-closed x86 NX capability plus exact CR0/CR4/EFER W^X state.

The boot draft is marked `draft-incomplete` until this decision is accepted and
the exact contract passes fresh review.

## Decision drivers

- Keep the end-of-week Alpha path small and authentic.
- Produce byte-identical images without third-party target code.
- Preserve unchanged R0-002 semantics and Recovery-only source production.
- Make W^X an enforced CPU/page-table property, not prose.
- Avoid turning Alpha bootstrap choices into production compatibility promises.
- Give implementers one total contract instead of hidden discretionary choices.

## Considered options

### A. Allow the implementation to choose the missing details

This is fastest initially, but different implementations can emit different
images, memory rights, ownership, timers, and page protections. It silently
creates security and format policy inside code and is not acceptable under the
existing constitution.

### B. Delegate image and boot details to third-party libraries

An existing image builder, boot library, or runtime can decide most standard
bytes and platform state. This reduces RAR code, but imports ready-made target
behavior, weakens reproducibility control, and conflicts with the from-scratch
direction unless separately excepted.

### C. Fix one private Alpha profile and implement it with RAR-owned code

Create a reviewed Alpha Boot Contract v0 and Machine Profile v2 with:

- a byte-producing UEFI 2.10 protective-MBR/GPT/FAT32 table, including exact
  mandatory fields, CRC algorithm and coverage, cluster chains, entries,
  timestamps, padding, and computed free/next values;
- fixed page-aligned Recovery scratch slots for `BootEntryV1`, `BootHandoffV1`,
  the normalized map, RHD, entropy, and trace, with exact descriptor rights,
  transfers, write/DMA revocation, trace retention, and exhaustion failures;
- a known-bit, conflict-rejecting UEFI attribute table whose type/attribute
  combinations either map to an exact R0 kind/rights/owner tuple or reject;
- a digest-bound QEMU CPU/profile value that requires NX and one fixed virtual
  TSC frequency; provisioning must prove the pinned QEMU accepts and reports
  that model before the profile can become active; and
- Root and Recovery checks for CPUID NX, `CR0.WP`, `CR4.PAE`, and
  `IA32_EFER.NXE`, with NX on every non-executable mapping and negative fixtures
  for missing/cleared protections.

All standard serialization and validation remains RAR-owned. UEFI/FAT/GPT/ELF
are compatibility standards, not imported implementations. This option is
selected.

## Decision

Select Alternative C. Generate language-neutral fixture bytes from the reviewed
field contract and require two independent RAR-owned implementations—the image
packer and a read-only inspector—to agree on the final image. Keep the existing
v1 machine profile inactive; v2 becomes usable only after exact cloud-side QEMU
model-expansion evidence is reviewed.

This choice authorizes specification and source implementation only. It does
not authorize Mac target compilation or execution, cloud credentials,
provisioning, deployment, VM launch, or production compatibility claims.

## Consequences

- Milestone A receives a deterministic, enforceable boot contract.
- The private Alpha image and fixed physical slots are intentionally disposable.
- A pinned emulated CPU profile makes the Alpha timer and W^X behavior explicit.
- Unsupported firmware attributes or CPU features fail closed rather than being
  guessed.
- Production boot, storage, discovery, timer calibration, and hardware support
  still require later designs.

## Security and data impact

No user data is introduced. Recovery remains the sole R0 source producer.
Exact source placement and ownership prevent aliasing; total attribute mapping
prevents accidental rights inflation; NX/WP requirements enforce W^X; the fixed
timer value is accepted only from the digest-bound emulated profile. The Mac
remains storage-only and no target execution is authorized.

The ready boot contract must prove required protections before parsing any
untrusted payload/source bytes, remove writable aliases before executable
mapping, and recheck the required state across Root → Recovery → Nucleus.

## Compatibility and migration

Every format and physical address in this ADR is experimental Alpha-only. A
later signed RAR package/image, A/B Root, production Recovery, hardware
discovery, and calibrated time design replaces it. R0-002 remains unchanged.

## Validation

- Two independent RAR-owned image implementations emit/inspect identical bytes.
- GPT/FAT golden fixtures cover every mandatory and computed field.
- Every R0 slot boundary, ceiling, overlap, ownership, and exhaustion case is
  exercised.
- Every supported UEFI type/attribute class and every rejection class is fixed.
- QEMU model expansion proves NX and the fixed virtual TSC frequency before use.
- NX unavailable, NXE clear, WP clear, writable+executable pages, and executable
  stack/data fail before Nucleus receives authority.

## Replacement path

The Alpha field contract and Machine Profile v2 are removed when the production
boot/storage and hardware contracts are accepted. Service and R0 interfaces do
not depend on the Alpha image layout or fixed physical slots.
