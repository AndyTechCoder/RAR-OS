# ADR 0016: Release 0 Entry Validation and Authority Closure

Status: Accepted — 2026-07-17

Approval basis: explicit owner approval of the consolidated exact-head remediation decisions on 2026-07-17.

## Context

Exact-head review of the first implementation of ADRs 0013–0015 found that a global validation order could inspect nested handoff/RHD bytes before their descriptors were authorized, that the fixture-only snapshot receipt was not representable by the entry ABI, and that alias, memory-authority, version, and architecture-entry rules were incomplete.

## Decision drivers

- No nested source byte may be inspected before its descriptor is validated and the source is copied into owned storage.
- The public ABI must not claim a mutation detector it cannot transport.
- Address-space aliasing and memory ownership must have one fail-closed interpretation.
- Compatible minor versions must follow the approved format policy without making unknown authority mandatory.
- Architecture adapters need sufficient entry-state preconditions to implement the same boundary.

## Considered options

### A. Keep the global order and add an out-of-band mutation counter

This preserves the first draft but adds a third trusted architecture input and still makes artifact-specific safe-access order difficult to express.

### B. Use staged artifact-scoped validation and a trusted immutable-source precondition

The adapter validates and copies the entry first, validates descriptors without following them, relies on Root/Recovery's mandatory producer/DMA revocation, copies each source once, then applies artifact-local framing and semantic predicates to owned bytes. The ABI carries no separate mutation counter.

### C. Copy every advertised range before descriptor validation

This simplifies ordering but converts attacker-controlled addresses and lengths into read authority and is unacceptable.

## Decision

Use alternative B.

Validation is staged: architecture entry bounds and entry framing; descriptor arithmetic, address width, alignment, purpose, rights and full alias matrix; immutable/DMA-revoked precondition; exactly one bounded copy of each source; owned handoff/map/RHD framing; semantic/reference/model validation; authority construction and transfers. A stage may inspect only the bytes named by its access budget.

The architecture-adapter conformance tuple is expected architecture, external entry address and length, address width, page size, entry alignment, and stack alignment. Source descriptors are uniquely selected by `(purpose,owner_kind,owner_id)` and never by position. Authority descriptors may repeat that owner selector when their ranges do not overlap; each RHD window must have exactly one semantic match by selector, address space, rights, transfer, and checked containment. The provider records every copy and can produce bounded short reads and faults. A separate effect sink remains empty on rejection and commits entropy clearing, trace transfer, and device authority only after acceptance.

`snapshot_generation` and the fixture-only observed receipt are removed from v1. Root/Recovery's immutable and DMA-revoked flags are a trusted precondition, not a Nucleus-detected fact. Short copies, copy-provider faults, or any adapter-observed stability failure return `snapshot-violation`; the contract does not claim detection of a malicious trusted producer.

Within system-memory space, the external entry slice and every descriptor range are pairwise disjoint. Device I/O-port descriptors are pairwise disjoint in their separate space. RHD register windows may be contained only in their one named device authority descriptor; windows belonging to different `(kind,id)` owners may not overlap.

Memory-map combinations are exact:

- usable: read/write/cacheable, owner none;
- firmware: read/cacheable, owner firmware;
- boot-owned: read/write/cacheable, owner Root or Recovery;
- Nucleus: read/write/cacheable, owner Nucleus;
- MMIO: read/write/device, owner device;
- reserved: no attributes, owner none.

Execute is forbidden in the Release 0 handoff map. Source descriptor ranges must be contained in boot-owned ranges of their declared producer. Reclaim and ownership transitions happen only after complete validation and are not encoded as accepted input states.

Same-major higher minors are accepted only when fixed sizes remain supported and every addition is bounded, safely skippable, and explicitly optional/non-critical. Unknown non-critical additions are range-checked and skipped; unknown critical RHD records, critical entry descriptors, flags, required roles, or changed fixed sizes fail. Existing precedence and meaning do not change in a minor version.

Boot Entry has one allocated descriptor compatibility result. `handoff-v1.fields` defines the exact inert form: entry minor is greater than supported, purpose is unallocated, producer is Root or Recovery, and base, length, rights, transfer, owner kind/id, and flags are zero. It is excluded from binding, aliasing, acquisition, and authority. Any deviation, including invalid producer or critical flags, returns `unsupported-minor`. RHD retains distinct `unknown-critical` and `unsupported-minor` outcomes as consecutive whole-table predicates; validators collect all framed-record facts and reduce them in that fixed order rather than returning during wire-order traversal.

The single memory-map predicate includes all entry framing, checked range arithmetic, identity, canonical ordering, overlap, ownership/attribute semantics, and containment of every acquired source descriptor in a boot-owned region of its declared producer. Every such failure returns `invalid-memory-map`; no generic arithmetic or pointer-range code escapes that stage.

For register roles specifically, a compatible higher minor uses the existing register-window record and its record-header criticality bit: an unknown role with bit 0 clear participates in framing, identity, and canonical ordering but is then skipped without reference resolution, descriptor consumption, or authority construction; an unknown role with bit 0 set returns `unknown-critical`. This makes optional-role compatibility executable rather than prose-only.

x86-64 entry requires long mode, interrupts disabled, direction flag clear, a 16-byte-aligned writable adapter stack, and a Root/Recovery-controlled translation that maps only the entry slice for the initial copy. AArch64 entry requires EL1, interrupts masked, a 16-byte-aligned writable adapter stack, coherent entry bytes, and MMU-off physical addressing for the initial copy. Other source mappings are constructed only after descriptor validation.

## Consequences

- The validation table becomes stage-aware and may repeat a failure code for different artifact-local predicates.
- Fixtures must execute every predicate and adjacent edge while recording source identity, exact requested base/length, and side effects.
- The producer precondition is smaller and honest, but a malicious trusted Root/Recovery producer remains outside structural validation.
- The strict Release 0 memory matrix may require a later major contract for executable or shared map states.

## Security and data impact

The decision prevents pre-authority reads, removes a fictitious mutation signal, prevents overlapping authority and contradictory map states, and grants no new device, persistence, or user-data behavior.

## Compatibility and migration

The affected v1 contracts are unmerged and are revised in place. After merge, changing stages, alias rules, the memory matrix, or entry state requires a major version. Compatible optional minor additions follow the criticality rules above.

## Validation

- Every table predicate has an executable focused byte fixture.
- Every adjacent edge has an executable compound-invalid fixture returning the earlier failure.
- Access logs prove no nested read before source acquisition and no rejected transfer, entropy clear, trace activation, or device-authority construction.
- Reordered descriptors, short providers, overflow, all alias classes, contradictory map states, and model/parent cardinality are covered.

## Replacement path

Architecture adapters, snapshot providers, and decoders may be replaced independently when they obey the same entry-state assumptions, access budgets, owned-byte semantics, and conformance corpus.
