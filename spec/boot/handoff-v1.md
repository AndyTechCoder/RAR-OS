# Release 0 Boot Entry and Handoff

Status: R0-002 implementation contract; ADRs 0013–0016 accepted 2026-07-17

Source schema: `handoff-v1.fields`

## Trusted entry boundary

RAR Root or Recovery supplies one contiguous immutable `BootEntryV1` byte slice through the architecture entry ABI. On x86-64 its physical address and byte length are in `RDI` and `RSI`; on AArch64 they are in `x0` and `x1`. This fixed register pair is the only pre-descriptor read authority. The architecture adapter conformance tuple is expected architecture, external entry address and length, address width, page size, entry alignment, and stack alignment. The adapter bounds the external length to 64 through 4,096 bytes, validates the tuple with checked arithmetic, copies the slice exactly once, and then parses only the owned copy. Root/Recovery guarantees the slice is immutable and DMA-revoked from before transfer until the copy completes.

x86-64 enters in long mode with interrupts disabled, direction flag clear, a 16-byte-aligned writable adapter stack, and Root/Recovery-controlled translation mapping only the entry slice for the initial copy. AArch64 enters at EL1 with interrupts masked, a 16-byte-aligned writable adapter stack, coherent entry bytes, and MMU-off physical addressing for the initial copy. The adapter maps no other source until its descriptor passes.

The entry header is 64 bytes followed inline by 32-byte window descriptors. `total_bytes` must equal `64 + descriptor_count * 32` using checked arithmetic; descriptor count is 1 through 126. Header fields are magic `RARENTRY`, version 1.0, header size 64, descriptor size 32, architecture, address bits 32 through 64, and zero flags/reserved bytes. Immediately after the owned header is framed, its architecture and address width must equal the already validated trusted adapter tuple; mismatch or an out-of-domain entry value returns `invalid-entry` before any descriptor value influences bounds, acquisition, shifts, or parsing.

Each descriptor contains `base:u64`, `length:u64`, `purpose:u16`, `rights:u16`, `producer:u16`, `transfer:u16`, `owner_kind:u16`, `flags:u16`, and `owner_id:u32`. Ranges are nonempty checked half-open ranges. Purposes are handoff, memory map, RHD, entropy, trace, device MMIO, and device I/O port. Rights are read, write, execute, and device. Producers are Root or Recovery only. Transfer modes are snapshot, exclusive, authority, and clear-after-snapshot. Source descriptors require immutable and DMA-revoked flags; executable source metadata is forbidden.

Exactly one descriptor exists for handoff, map, RHD, entropy, and trace. Descriptor order is immaterial. The external entry slice and every system-memory descriptor are pairwise disjoint; I/O-port descriptors are pairwise disjoint in their separate space. Handoff/map/RHD use read-only snapshot transfer. Entropy uses read-only snapshot or read/write clear-after-snapshot consistently with the handoff flag. Trace uses read/write exclusive transfer. Device descriptors use read/write/device authority, identify an owning RHD `(kind,id)`, and do not grant access until RHD and memory-map cross-validation succeeds. Multiple non-overlapping authority descriptors may share `(purpose, owner_kind, owner_id)` when one device has split windows; this tuple is a selector, not a globally unique key, and descriptor position carries no authority meaning.

## Boot handoff and memory map

`BootHandoffV1` remains exactly 128 little-endian bytes with the offsets in the source schema. It carries magic/version, architecture, page shift, boot-source kind, memory-map address/count, RHD address/bytes, entropy range/flags, trace identity/range, and boot CPU ID. Its embedded ranges and exact lengths must equal the unique entry descriptors. The adapter snapshots map, RHD, and entropy once, requires embedded and external lengths to agree, parses only owned copies, and never revisits source addresses.

Release 0 boot-source kind is 1 and must match the sole RHD boot-source record. Entropy flag bit 0 selects clear-after-snapshot and must match a read/write clear-after descriptor; zero selects a read-only snapshot descriptor. Other entropy bits are zero. Trace version is exactly 1.0, trace flags are zero, and the trace buffer remains inactive until full validation succeeds.

Each 32-byte memory-map entry is `base:u64`, `length:u64`, `kind:u16`, `attributes:u16`, `owner:u16`, `reserved0:u16`, `region_id:u32`, `reserved1:u32`. Entries are ordered by base, nonempty, non-wrapping, and non-overlapping. Exact accepted `(kind,attributes,owner)` combinations are usable `(read|write|cacheable, none)`, firmware `(read|cacheable, firmware)`, boot-owned `(read|write|cacheable, Root or Recovery)`, Nucleus `(read|write|cacheable, Nucleus)`, MMIO `(read|write|device, device)`, and reserved `(none, none)`. Execute is forbidden. Source ranges lie in boot-owned entries of their producer. Any other combination or reserved value fails.

System-memory register windows must fit exactly one MMIO entry with read/write/device, no execute, and device ownership. I/O-port windows are not physical-memory entries and must fit the 16-bit port space. In both spaces, each RHD window resolves to exactly one authority descriptor matching owner identity, address space, rights, authority transfer, and checked full-range containment. Zero or multiple matches fail, and lookup never depends on descriptor order.

## Snapshot and ownership sequence

1. Under the architecture preconditions in ADR 0016, bound and copy only the entry slice; compare its architecture with the adapter.
2. Validate every descriptor and its checked range before reading a described window.
3. Verify the trusted immutable and DMA-revoked precondition; no independent mutation receipt is claimed.
4. Snapshot handoff, map, RHD, and entropy exactly once into bounded owned storage.
5. Require descriptor, handoff, and embedded RHD/map lengths to agree and consume all bytes exactly.
6. Validate owned values according to the total predicate table.
7. Only after success transfer trace exclusivity and construct device authorities. Clear entropy source only under clear-after-snapshot transfer.

A short copy, copy-provider fault, adapter-observed stability failure, or re-read attempt returns `snapshot-violation`. A malicious producer that violates the trusted precondition is outside structural validation. Rejection grants no mapping, executable, device, trace, or recovery authority.

## Deterministic validation

`handoff-v1.fields` is the canonical artifact-qualified staged predicate table and defines the exact inter-artifact first-error order. Adapter predicates run before the entry copy; entry predicates bind the owned entry to that trusted adapter before descriptor binding; acquisition follows; owned handoff, map, and RHD framing and semantics run only afterward; effects commit last. Descriptor range arithmetic is a whole-table pass, followed by whole-table minor compatibility: an exact inert noncritical higher-minor descriptor is skipped, while any other unknown descriptor addition, including critical, returns the row's sole `unsupported-minor` code. The whole-table binding pass returns `invalid-pointer-range` for zero length, address-width, alignment, selector/cardinality, rights, producer, transfer, or flag failures. The whole memory-map stage returns only `invalid-memory-map` for framing, checked arithmetic, identity, canonical ordering, overlap, ownership/attribute semantics, or failure to contain an acquired source descriptor in a boot-owned region belonging to its producer. RHD framing, critical extension, minor support, identity, references, model/cardinality, canonical order, and cross-artifact checks are likewise global stages. Each row states its required prior fact and access budget. Same-major higher minors are accepted only when fixed sizes remain supported and bounded additions are explicitly optional and non-critical; unknown critical additions fail. Numeric code order is not precedence.

The conformance oracle uses a descriptor-keyed source provider and an effect sink. It records every permitted entry/source copy, faults or truncates selected sources, rejects a second copy, and records entropy clearing, trace transfer, and device-authority construction only after complete acceptance. Rejected input must leave the effect log empty.

## Compatibility and replacement

The approved trust, identity, and precedence decisions revise the unmerged v1 draft. After merge, changes to entry authority, descriptor semantics, required RHD identity/window rules, or predicate order require a new major version and parallel decoder. Platform adapters remain replaceable when they produce identical owned values, access no rejected range, and pass the complete corpus.
