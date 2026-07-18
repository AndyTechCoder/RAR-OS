# ADR 0013: Pre-Copy Trust Boundary and MMIO Authority

Status: Accepted — 2026-07-17

Approval basis: explicit owner approval of the recommended decision on 2026-07-17.

## Context

R0-002 currently starts with a handoff address and separately bound windows, but does not define who authenticates those inputs, how they remain stable while copied, or what grants authority to access device registers. R0-003 and R0-004 must not invent different trust roots or treat an RHD address as permission to map MMIO.

This decision changes a boot trust boundary and public handoff contract before the R0-002 draft is frozen.

## Decision drivers

- No unverified pointer, producer-controlled length, or RHD value may be dereferenced.
- x86-64 and AArch64 must begin portable validation from equivalent trusted inputs.
- A description of device registers must not itself grant MMIO authority.
- Source mutation, DMA, overflow, aliasing, and ownership transfer must fail closed.
- The entry mechanism must remain replaceable behind a small architecture adapter.

## Considered options

### A. Trust firmware-native entry structures directly

Each platform adapter would validate UEFI or machine-specific structures and construct R0-002 inputs.

- Advantage: smallest immediate boot shim.
- Cost: firmware rules become part of each trusted boundary, semantic drift is likely, and Tier 0 or later non-UEFI ports need another model.

### B. RAR-owned boot-entry envelope plus capability-like window descriptors

Root/Recovery delivers one fixed, bounded boot-entry value containing the handoff locator, expected architecture, and a bounded list of immutable readable or exclusive writable physical-window descriptors. The adapter validates only that envelope, snapshots each source once, then parses owned bytes. MMIO access is separately authorized by containment in an authoritative device/MMIO window whose owner and access rights match the RHD binding.

- Advantage: one portable trust boundary, explicit provenance and lifetime, and no authority derived from descriptive metadata.
- Cost: adds a small public entry contract, descriptor validation, and platform-specific construction work.

### C. Copy all metadata into one fixed bootstrap arena

Root/Recovery places the handoff, map, RHD, and entropy into one fixed-size architecture-defined arena before entry.

- Advantage: simple bounds and snapshot rules.
- Cost: fixed placement leaks platform assumptions into the public contract, wastes constrained memory, and complicates future extensibility.

## Decision

Use alternative B.

`BootEntryV1` is a small RAR-owned value delivered by the architecture entry ABI. It binds the expected architecture, handoff address and exact size, and a bounded descriptor table. Each descriptor carries a physical half-open range, access purpose and rights, producer identity, and transfer mode. Root/Recovery must revoke producer and DMA writes before entry and provide the immutable snapshot guarantee defined by the contract.

The mandatory algorithm is:

1. Validate the architecture-adapter tuple—expected architecture, external entry address/length, address width, page size, entry alignment, and stack alignment—then copy the fixed entry value without following embedded addresses.
2. Validate descriptor count, checked half-open arithmetic, address width, alignment, non-aliasing, purpose, and expected architecture.
3. Before a request, enforce the canonical purpose ceiling (handoff 4096, memory map 8192, RHD 65536, entropy 64, trace 1048576 bytes); copy each readable source exactly once into bounded Nucleus-owned scratch and require external and embedded lengths to agree. Trace is a bounded destination, not a readable source.
4. Parse only owned copies, consume records exactly, and never revisit source addresses.
5. Transfer the trace destination only after all validation succeeds; clear entropy source only when its descriptor grants that one write.
6. Treat RHD register ranges as descriptions only. Grant MMIO or I/O-port access only when exactly one descriptor matches `(purpose,owner_kind,owner_id)`, address space, rights, authority transfer, and complete checked range containment. Multiple non-overlapping descriptors may share an owner selector; order is never identity.

The conformance boundary is an instrumented descriptor-keyed source provider and a commit-only effect sink. The provider records descriptor identity plus the exact requested base and byte length, returns selected short copies or faults, and rejects a second copy. The sink records entropy clearing, trace activation, and constructed device authority only after complete validation; every rejected case must have an empty effect log.

## Consequences

- R0-002 gains a new public entry record and explicit window semantics before it can freeze.
- R0-003 and R0-004 share one pre-copy algorithm but still implement architecture entry adapters.
- Root/Recovery must establish immutability or revocation before Nucleus entry.
- MMIO descriptions cannot expand authority and must be cross-checked against the authoritative map.
- Fixtures need length-mismatch, overflow, alias, wrong-owner, wrong-kind, mutation, and transfer-order cases.

## Security and data impact

The recommendation removes implicit pointer and MMIO authority, prevents torn source parsing, and makes DMA/producer mutation part of the trusted entry guarantee. It introduces no user-data or persistence behavior.

## Compatibility and migration

Because RHD/boot v1 is not merged, approval should revise the draft v1 contracts rather than silently reinterpret them. Once merged, any change to descriptor or authority meaning requires a new major version and side-by-side decoder.

## Validation

- Independent adapters produce the same owned snapshot for equivalent x86-64 and AArch64 inputs.
- Every overflow, alias, source mutation, length mismatch, unauthorized MMIO, and wrong-owner case returns a deterministic validation code before device access.
- Instrumented reference tests prove exact permitted reads, one-copy behavior, provider faults, and absence of rejected writes or authority construction.

## Replacement path

Firmware or board-specific entry adapters may be replaced independently when they produce the same validated `BootEntryV1` values and pass the same conformance corpus.
