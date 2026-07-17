# Release 0 Boot Entry and Handoff

Status: R0-002 implementation contract; ADRs 0013–0015 accepted 2026-07-17

Source schema: `handoff-v1.fields`

## Trusted entry boundary

RAR Root or Recovery supplies one contiguous immutable `BootEntryV1` byte slice through the architecture entry ABI. On x86-64 its physical address and byte length are in `RDI` and `RSI`; on AArch64 they are in `x0` and `x1`. This fixed register pair is the only pre-descriptor read authority. The architecture adapter already knows its expected architecture, bounds the external length to 64 through 4,096 bytes, checks 8-byte alignment and address-width fit, copies the slice exactly once, and then parses only the owned copy. Root/Recovery guarantees the slice is immutable and DMA-revoked from before transfer until the copy completes.

The entry header is 64 bytes followed inline by 32-byte window descriptors. `total_bytes` must equal `64 + descriptor_count * 32` using checked arithmetic; descriptor count is 1 through 126. Header fields are magic `RARENTRY`, version 1.0, header size 64, descriptor size 32, architecture, address bits 32 through 64, zero flags/reserved bytes, and a nonzero snapshot generation.

Each descriptor contains `base:u64`, `length:u64`, `purpose:u16`, `rights:u16`, `producer:u16`, `transfer:u16`, `owner_kind:u16`, `flags:u16`, and `owner_id:u32`. Ranges are nonempty checked half-open ranges. Purposes are handoff, memory map, RHD, entropy, trace, device MMIO, and device I/O port. Rights are read, write, execute, and device. Producers are Root or Recovery only. Transfer modes are snapshot, exclusive, authority, and clear-after-snapshot. Source descriptors require immutable and DMA-revoked flags; executable source metadata is forbidden.

Exactly one descriptor exists for handoff, map, RHD, entropy, and trace. Source descriptors are pairwise disjoint. Handoff/map/RHD use read-only snapshot transfer. Entropy uses read-only snapshot or read/write clear-after-snapshot consistently with the handoff flag. Trace uses read/write exclusive transfer. Device descriptors use read/write/device authority, identify an owning RHD `(kind,id)`, and do not grant access until RHD and memory-map cross-validation succeeds.

## Boot handoff and memory map

`BootHandoffV1` remains exactly 128 little-endian bytes with the offsets in the source schema. It carries magic/version, architecture, page shift, boot-source kind, memory-map address/count, RHD address/bytes, entropy range/flags, trace identity/range, and boot CPU ID. Its embedded ranges and exact lengths must equal the unique entry descriptors. The adapter snapshots map, RHD, and entropy once, requires embedded and external lengths to agree, parses only owned copies, and never revisits source addresses.

Each 32-byte memory-map entry is `base:u64`, `length:u64`, `kind:u16`, `attributes:u16`, `owner:u16`, `reserved0:u16`, `region_id:u32`, `reserved1:u32`. Entries are ordered by base, nonempty, non-wrapping, and non-overlapping. Kinds are usable, firmware, boot-owned, Nucleus, MMIO, and reserved. Attribute bits are read, write, execute, cacheable, and device. Owners are none, Root, Recovery, Nucleus, firmware, and device. Unknown values/bits and nonzero reserved fields fail.

System-memory register windows must fit exactly one MMIO entry with read/write/device, no execute, and device ownership. I/O-port windows are not physical-memory entries and must fit the 16-bit port space. In both spaces, the entry authority descriptor must contain the complete range and match the parent record kind and ID.

## Snapshot and ownership sequence

1. Bound and copy the entry slice; compare its architecture with the adapter.
2. Validate every descriptor and its checked range before reading a described window.
3. Verify source immutability and DMA revocation for the advertised generation.
4. Snapshot handoff, map, RHD, and entropy exactly once into bounded owned storage.
5. Require descriptor, handoff, and embedded RHD/map lengths to agree and consume all bytes exactly.
6. Validate owned values according to the total predicate table.
7. Only after success transfer trace exclusivity and construct device authorities. Clear entropy source only under clear-after-snapshot transfer.

Any generation change, producer/DMA write, short copy, or re-read attempt returns `snapshot-violation`. Rejection grants no mapping, executable, device, trace, or recovery authority.

## Deterministic validation

`handoff-v1.fields` is the canonical total predicate table. Predicates run strictly in ascending table order; a decoder returns the code of the first failing row and performs only the access named by that row after all prerequisites passed. Unknown minor versions are rejected. Numeric code order is not precedence. Changing row order or an existing code meaning requires a new major contract.

## Compatibility and replacement

The approved trust, identity, and precedence decisions revise the unmerged v1 draft. After merge, changes to entry authority, descriptor semantics, required RHD identity/window rules, or predicate order require a new major version and parallel decoder. Platform adapters remain replaceable when they produce identical owned values, access no rejected range, and pass the complete corpus.
