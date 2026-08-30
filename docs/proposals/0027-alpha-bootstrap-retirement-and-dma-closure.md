# ADR 0027 Proposal: Alpha Bootstrap Retirement and DMA Closure

Status: Proposed — 2026-08-29
Recommended decision: Alternative B

Approval context: the owner stated, "If it's safe, I approve." This proposal
records the smallest independently recommended enforceable design. That
statement is conditional: this document grants no implementation, build,
execution, provisioning, merge, or activation authority unless the exact
decision survives architecture, correctness, and security review and is
recorded through the repository's normal accepted-ADR process.

## Context

Accepted ADR 0023 fixes deterministic Recovery-owned slots, but it does not fix
the lifetime of the firmware-loaded Root image, Root stack, or Root page tables.
Retaining firmware-selected Root pages would make the final R0 memory map depend
on firmware placement. Reusing them before the one-way transfer completes would
risk corruption.

The draft Alpha source contract also asserts that boot-storage DMA writes have
stopped without defining a guest-enforced mechanism. A descriptor bit or host
claim cannot prove that staged bytes remain unchanged after validation.

## Decision drivers

- Preserve deterministic Recovery and R0 bytes without requiring Root to
  self-relocate.
- Enforce source immutability inside the guest trust boundary.
- Keep the Alpha mechanism narrow, q35-specific, and replaceable.
- Avoid prematurely implementing a general IOMMU or storage stack.
- Fail closed on unexpected hardware, timeouts, or unverifiable state.

## Considered options

### A. Fixed-address Root relocation plus an Alpha IOMMU domain

Root relocates itself into a fixed arena and confines the storage controller
with an IOMMU. This offers strong isolation but introduces substantially more
early-boot and hardware policy than the Alpha needs.

### B. Firmware placement with deterministic retirement and verified AHCI closure

Reserve the complete fixed Recovery bootstrap arena before reading payloads.
Allow UEFI to choose Root's initial image placement and record only the exact
loaded-image base and size obtained through the UEFI Loaded Image Protocol.
Before the final file read, Root allocates its own stack and page tables from
fixed private arena slots. After `ExitBootServices`, Root switches to those
tables and stack. Firmware-global stack and page-table memory is never claimed,
zeroed, or released by RAR; its boot-services descriptors are normalized by the
total UEFI conversion contract like all other firmware memory.

Root performs the one-way transfer to Recovery. Recovery retires and unmaps the
known Root loaded-image range and Root's fixed private stack/page-table slots
before producing the canonical final memory map. No retired range is retained,
reused, or described as available until that transition completes.

Pin the Alpha boot path to one exact q35 PCI-function inventory and one exact
AHCI controller profile. Root is the sole closure actor. After the final UEFI
file read and `ExitBootServices`, and before parsing Recovery or any other
payload, Root enumerates that fixed inventory through its reviewed PCI-config
access, rejects any missing or extra function, stops every AHCI command engine,
waits for the bounded idle predicate, clears PCI command-register bus mastering
on every enumerated bus-master-capable function, and reads back every disabled
state. The profile permits no unenumerated DMA-capable function.

Root then re-hashes every staged source, parses and loads Recovery, rechecks the
complete disabled-state vector immediately before entry, and transfers no PCI
configuration or controller capability. Recovery rechecks the immutable
closure record and source digests before Nucleus handoff. An unexpected
function, unavailable range, timeout, active command engine, failed read-back,
enabled bus master, or post-closure digest mismatch rejects before Nucleus
receives authority. This option is recommended.

### C. Trust host/QMP evidence of quiescence

The controller or firmware states that DMA has stopped. This is smallest, but
the guest cannot enforce the claim and a mutable source could change after its
digest check.

## Decision

Select Alternative B for the experimental q35 Alpha only. It resolves Root
lifetime deterministically and makes DMA closure an enforceable transition
without creating a production IOMMU promise.

## Consequences

- Root gains a temporary, exact post-firmware closure responsibility that ends
  permanently before Recovery entry.
- The private Alpha machine profile must enumerate every PCI function and
  rejects any topology drift.
- Alpha remains intentionally dependent on exact q35/AHCI emulation behavior
  and makes no general hardware-support claim.
- A future IOMMU-backed path replaces this closure rather than extending it.

## Required contract details

The reviewed byte and machine-profile contracts must fix:

- the complete reserved bootstrap arena, the UEFI-reported Root image range,
  Root's fixed private stack/page-table slots, and firmware-global ranges RAR
  must never claim;
- checked range capture, private-table/stack switch, ownership, non-overlap,
  retirement, unmapping, TLB invalidation, zeroing policy, and deterministic
  final-map normalization/exclusion/release ordering;
- Root's exact post-`ExitBootServices` PCI-config authority, the complete q35
  PCI-function inventory, every bus-master-capable function, the AHCI identity
  and BAR class, command-engine stop sequence, bounded waits, bus-master-disable
  operations, read-back predicates, and failure precedence;
- the last-read, DMA-closure, source re-hash, parser-entry, and handoff order;
- immediate pre-parse and pre-entry rechecks proving no writable alias or
  enabled DMA path remains, without claiming continuous trapping after Root
  permanently withholds the relevant device capabilities; and
- deterministic negative fixtures and no-effect traces for every failure.

No contract may generalize this profile into arbitrary AHCI hardware support.

## Security and data impact

No user data is introduced. Root memory is retired before canonical handoff,
and immutable source bytes are revalidated after guest-enforced DMA closure.
Failure grants no mapping, capability, thread, or device authority. The design
does not authorize raw host disks, passthrough, networking, local target work,
or execution on the Mac.

## Compatibility and migration

The addresses, AHCI sequence, and evidence are private Alpha behavior. A later
hardware profile may replace them with reviewed IOMMU, driver, discovery, and
production boot contracts. No migration interprets these fields as production
device authority. R0-002 remains unchanged.

## Validation

- Prove exact boot-device-path-to-BDF binding, duplicate-controller rejection,
  and the complete fixed PCI inventory.
- Exercise every command-engine timeout and bus-master read-back failure.
- Mutate a staged source after its first digest and prove the post-closure
  re-hash rejects with an empty mapping/capability/thread effect log.
- Prove Root-owned stack/page-table switching, firmware-range non-ownership,
  deterministic retirement, and identical final memory-map bytes.
- Prove no PCI/controller authority reaches Recovery, Nucleus, Core, or apps.

## Replacement path

Accept a later hardware-isolation ADR, add its independently reviewed machine
profile and migration evidence, reject the Alpha closure version, and remove
the private q35/AHCI mechanism without changing R0-002.
