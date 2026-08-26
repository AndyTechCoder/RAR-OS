# ADR 0021: Alpha Boot Payload and Handoff Boundary

Status: Accepted — 2026-08-26
Decision: Alternative C

Approval basis: explicit owner approval after a plain-language explanation on
2026-08-26 of the Root → Recovery → Nucleus boot responsibilities and the
Alpha-only, replaceable FAT/ELF boundary.

## Context

Milestone A requires three authentic RAR-owned execution stages—Root, Recovery,
and Nucleus—but the approved contracts define only the final R0-002 entry into
Nucleus. They do not define the Alpha boot-volume paths, executable container,
Root-to-Recovery entry, firmware-service lifetime, or which stage owns loading
Nucleus. Inventing those details in code would silently create a boot format
and trust boundary.

This decision is Alpha-only. It must not establish the eventual RAR executable,
package, filesystem, Root A/B, or production recovery format.

## Decision drivers

- Execute distinct Root, Recovery, and Nucleus binaries rather than relabeling
  functions in one program.
- Keep firmware and third-party target code outside the RAR trusted runtime.
- Use standards where compatibility requires them and RAR-owned parsing and
  validation everywhere else.
- Make the cloud-built image deterministic, bounded, inspectable, and easy to
  replace after Alpha.
- Preserve R0-002 as the sole Nucleus entry contract.
- Leave room for signed A/B Root and Recovery layouts without pretending Alpha
  already implements them.

## Considered options

### A. One UEFI application containing all three stages

This is the smallest image, but Root, Recovery, and Nucleus would not have real
binary or authority boundaries. It cannot honestly demonstrate the intended
boot chain.

### B. Chain-load three UEFI applications through firmware

Each stage is distinct and PE/COFF is standardized, but firmware remains the
loader and execution coordinator after Root. Recovery and Nucleus inherit UEFI
application assumptions that should not leak into RAR internals.

### C. UEFI Root loads bounded ELF64 Recovery and Nucleus payloads

Root is the only UEFI application. It uses minimal RAR-owned UEFI bindings to
read two fixed files from the Alpha FAT boot volume. Root validates and loads
Recovery's bounded ELF64 image, but treats the Nucleus file only as bounded
staged bytes. Root obtains the final firmware memory map, exits boot services
once, and transfers the staged bytes and memory ownership to Recovery through
an explicitly experimental bounded entry value.

Recovery makes no firmware call. It independently validates and loads the
Nucleus ELF64 image, constructs every R0-002 source as producer `Recovery`,
establishes immutable/DMA-revoked source preconditions, constructs the approved
R0-002 entry, and enters Nucleus. No R0-002 descriptor uses a mixed or implicit
Root producer. This option is selected.

### D. Implement the production partition, A/B, and RAR filesystem layout now

This best resembles the final architecture but pulls persistent storage,
updates, migration, and recovery policy into Milestone A before a first boot.

## Decision

Alternative C is selected. A reviewed experimental specification remains
mandatory before target implementation or an image recipe assumes the
Alpha-only boundary.

The incomplete candidate preimplementation contract is in `spec/alpha/boot/`.
It currently records the image geometry, stage paths, ELF acceptance rules,
Root-to-Recovery entry candidate, fixed Alpha fixtures, and negative cases. Its
explicit blocked status prevents implementation from inventing the remaining
byte-layout, memory-attribute, timer, and x86 control-state details. It does not
authorize target compilation, image creation, or execution on the Mac.

Before target implementation, a reviewed experimental specification must be
committed. It must define:

- the exact fixed Alpha-only paths for Root, Recovery, and Nucleus;
- accepted ELF64 class, machine, type, segment, alignment, relocation, bounds,
  overlap, entry-point, W^X permissions, canonical/reserved ranges, BSS zeroing,
  checked arithmetic, and rejection rules;
- one bounded, versioned, little-endian Root-to-Recovery byte layout and
  deterministic errors, with generated language-neutral representations rather
  than Rust ABI;
- the x86-64 Recovery entry state: long mode, interrupts disabled, direction
  flag clear, known FPU/SIMD state, Recovery-owned page tables with W^X mappings
  and a guarded 16-byte-aligned writable stack, RDI/RSI as the sole initial
  pointer/length authority, non-returning failure behavior, and no firmware
  pointer or callable service;
- ownership of the firmware memory map, payload pages, trace buffer, stack, and
  the single ExitBootServices transition;
- Recovery's independent Nucleus ELF validation and exact identity check before
  mapping or entry;
- Recovery ownership of every named handoff, memory-map, RHD, entropy, and trace
  source; their R0-002 descriptors all declare producer `Recovery` after writes
  and DMA access are revoked;
- the total deterministic UEFI-memory-type/attribute to MemoryRegionV1 mapping,
  canonical split/merge/order and region-ID rules, checked ownership changes,
  and rejection of unrepresentable firmware ranges;
- deterministic q35 CPU/APIC/timer/16550/boot-source RHD construction plus the
  exact device MMIO/I/O descriptors and containment that authorize each window;
- bounded entropy and trace allocation, initialization, provenance, size,
  lifetime, ownership, and write/DMA revocation, including how Root's earlier
  marker is retained under Recovery producer ownership;
- the pinned firmware's allowed pre-ExitBootServices protocols, returned-data
  validation, retry bounds, and residual bootstrap trust;
- construction of the unchanged R0-002 BootEntryV1 passed in RDI/RSI, including
  exact containment of every Recovery-produced source;
- reproducible FAT image ordering, timestamps, padding, capacity, and hashes;
- negative fixtures for malformed/truncated/overlapping payloads and failed
  firmware-map retry; and
- explicit Alpha limitations: no production boot trust, persistent format, or
  update compatibility claim.

## Consequences

- Root alone contains UEFI-facing code; Recovery and Nucleus are freestanding.
- Root validates Recovery, while Recovery—not Root—owns Nucleus validation and
  the complete R0-002 producer boundary.
- UEFI, FAT, PE/COFF for Root, and ELF64 payloads are external standards, not
  imported target implementations.
- RAR implements the minimal UEFI bindings, ELF validation/loading, deterministic
  image packer, stage entries, and all target behavior itself.
- Milestone A can prove real stage transitions while postponing production A/B,
  signatures, and RAR filesystem work to their approved milestones.
- The private Alpha Root-to-Recovery entry is replaceable and never becomes the
  stable RAR ABI by accident.

## Security and data impact

Firmware authority ends once Root successfully exits boot services. Recovery
receives no callable firmware pointer, independently validates Nucleus, and
owns every source described to Nucleus through R0-002. Checked bounds, W^X,
explicit ownership transfer, immutable source bytes, and DMA revocation prevent
the bootstrap shortcut from granting implicit authority. This Alpha boot format
contains no user data and makes no production boot-trust claim.

## Compatibility and migration

The FAT paths, ELF payloads, and Root-to-Recovery entry are explicitly
experimental Alpha contracts. A later signed RAR image/package and A/B storage
design replaces them through a reviewed migration ADR. The approved R0-002
Nucleus entry contract remains unchanged across that replacement.

## Validation

- Separate symbol/file identities and exact guest markers prove execution of
  Root, Recovery, and Nucleus in order.
- A clean cloud build produces byte-identical target images twice.
- Malformed ELF headers, segments, ranges, alignments, overlaps, and entry
  points fail before transfer of control.
- No Recovery/Nucleus code calls UEFI or receives a firmware pointer; entry
  tests assert the required register, flags, stack, and initial-memory limits.
- Nucleus receives only an R0-002-conforming owned entry and rejects the fixed
  malformed-input case without granting authority.
- Dependency inspection reports no target-linked third-party code.

## Replacement path

A later accepted boot/storage ADR replaces the Alpha FAT paths, ELF payloads,
and private Recovery entry with signed RAR executable/package formats, A/B
activation, and the RAR filesystem. R0-002 can remain unchanged across that
replacement.
