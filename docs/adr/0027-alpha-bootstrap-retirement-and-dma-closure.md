# ADR 0027: Alpha Bootstrap Retirement and DMA Closure

Status: Accepted — 2026-08-30
Decision: Alternative B

Approval basis: after the exact B/A/B decision set and its plain-language
safety effect were presented, the owner approved continuing on 2026-08-30.
Acceptance selects experimental Alpha specification work only. It grants no
target build, image, launch, execution, provisioning, or production authority.

The complete considered alternatives remain in the
[historical proposal](../proposals/0027-alpha-bootstrap-retirement-and-dma-closure.md).

## Context

Root's firmware-selected image, private stack, and private page tables must not
make the final R0 memory map nondeterministic. Staged boot bytes also require a
guest-enforced DMA closure rather than a host assertion.

## Decision drivers

- Preserve deterministic Recovery and R0 bytes without Root self-relocation.
- Enforce staged-source immutability inside the guest trust boundary.
- Keep Alpha q35-specific, bounded, fail-closed, and replaceable.
- Avoid prematurely adding a general IOMMU or storage stack.

## Considered options

- Alternative A: fixed-address Root relocation plus an Alpha IOMMU domain.
  Rejected because it introduces substantially broader early hardware policy.
- Alternative B: firmware placement, deterministic retirement, and verified
  q35/AHCI closure. Selected as the narrow enforceable Alpha mechanism.
- Alternative C: trust host or QMP quiescence evidence. Rejected because the
  guest cannot enforce source immutability.

## Decision

Reserve the complete fixed Recovery bootstrap arena before reading payloads.
UEFI may place Root initially; Root records only the Loaded Image Protocol range
and switches after `ExitBootServices` to fixed private stack and page-table
slots. Firmware-global stack and page-table memory is never claimed, cleared,
or released by RAR; its boot-services descriptors are normalized by the total
UEFI conversion contract. Recovery deterministically retires, unmaps,
invalidates, and normalizes the Root image and private ranges before producing
the final memory map.

Pin Alpha to one exact q35 PCI-function inventory and AHCI profile. After the
last UEFI file read and `ExitBootServices`, and before parsing Recovery or any
other payload, Root stops every AHCI command engine, waits for bounded idle,
disables and reads back PCI bus mastering for every declared capable function,
rejects topology drift, then re-hashes every staged source. Root rechecks the
entire disabled vector immediately before Recovery entry and transfers no PCI
or controller authority. Recovery revalidates the immutable closure record and
source digests before Nucleus handoff.

Missing or extra hardware, unavailable ranges, timeout, active engines, failed
readback, enabled bus mastering, or a changed source rejects before Nucleus
receives authority and with no mapping, thread, or capability effects.

## Consequences

Root gains one temporary q35-specific closure duty that ends permanently before
Recovery. The private machine profile rejects topology drift. Alpha remains
dependent on exact emulated q35/AHCI behavior and gains no general hardware or
IOMMU claim.

## Security and data impact

Root memory is retired before canonical handoff, and staged sources are
revalidated after guest-enforced DMA closure. Failure grants no mapping,
capability, thread, or device authority. No owner data, host disk, passthrough,
networking, or local target execution is introduced.

## Compatibility and migration

The addresses, AHCI sequence, and evidence are private Alpha behavior. Later
profiles reject this version and replace it under reviewed hardware contracts;
R0-002 remains unchanged.

## Validation

Contracts must bind the arena and private ranges, switch/retirement order,
firmware memory exclusions, total UEFI normalization, exact q35 inventory,
boot-device-to-BDF relation, AHCI stop sequence, bounded waits, bus-master
readback, re-hash order, first-failure precedence, and no-effect traces.
Retained machine evidence must prove the selected emulated firmware and
topology before readiness. No contract may generalize this profile into broad
AHCI or hardware support.

## Replacement path

Accept a later hardware-isolation ADR, add its independently reviewed profile
and migration evidence, reject the Alpha closure version, and remove the
private q35/AHCI mechanism without reinterpreting Alpha authority.
