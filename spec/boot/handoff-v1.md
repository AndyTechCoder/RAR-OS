# Release 0 Boot Handoff

Status: Draft implementation contract for R0-002

Source manifest: `handoff-v1.fields`

## Boundary and non-goals

RAR Root/Recovery supplies one immutable handoff and separately binds readable/writable physical windows. The architecture adapter copies the fixed header without interpreting embedded addresses. Validation uses integers and window descriptors; it never dereferences a candidate address. Only validated ranges are copied into Nucleus-owned memory.

This contract does not define firmware entry, page tables, executable entry ABI, recovery implementation, target boot code, a VM profile, trace record encoding, or entropy authenticity.

## Fixed wire record

`BootHandoffV1` is exactly 128 bytes, little-endian and 8-byte aligned:

| Offset | Field | Rule |
| ---: | --- | --- |
| 0x00 | `magic[8]` | `RARBOOT\0` |
| 0x08 | `major:u16`, `minor:u16` | 1, 0 |
| 0x0c | `header_bytes:u16`, `flags:u16` | 128, zero |
| 0x10 | `total_bytes:u32` | 128 in v1; future ceiling 4,096 |
| 0x14 | `architecture:u16`, `address_bits:u8`, `page_shift:u8` | architecture 1 x86-64 or 2 AArch64; bits 32..64; shift 12..30 |
| 0x18 | `boot_source_kind:u16`, `reserved0:u16`, `reserved1:u32` | source matches RHD; reserved zero |
| 0x20 | `memory_map_paddr:u64` | aligned readable range |
| 0x28 | `memory_map_count:u32`, `entry_bytes:u16`, `map_version:u16` | 1..256, 32, 1 |
| 0x30 | `rhd_paddr:u64` | aligned readable range |
| 0x38 | `rhd_bytes:u32`, `rhd_alignment:u16`, `reserved2:u16` | 32..65,536; 8; zero |
| 0x40 | `entropy_paddr:u64` | aligned readable range |
| 0x48 | `entropy_bytes:u32`, `entropy_flags:u32` | 32..64; only bit 0 (source writable) |
| 0x50 | `trace_channel_id:u32`, `trace_major:u16`, `trace_minor:u16` | nonzero ID; version 1.0 |
| 0x58 | `trace_buffer_paddr:u64` | 64-byte aligned writable range |
| 0x60 | `trace_buffer_bytes:u32`, `trace_flags:u32` | 4 KiB..1 MiB, multiple of 64; flags zero |
| 0x68 | `boot_cpu_id:u32`, `reserved3:u32` | ID references RHD; reserved zero |
| 0x70 | `reserved4[16]` | zero |

Each 32-byte memory-map entry is `base:u64`, `length:u64`, `kind:u16`, `attributes:u16`, `owner:u16`, `reserved0:u16`, `region_id:u32`, `reserved1:u32`. Entries are sorted by base, nonempty, non-wrapping, and non-overlapping. Kinds are 1 usable, 2 firmware, 3 boot-owned, 4 nucleus, 5 MMIO, and 6 reserved. Attribute bits are read, write, execute, cache, and device. Unknown bits fail.

## Range and ownership rules

Every referenced range must fit `address_bits`, be fully contained in exactly one boot-owned, non-MMIO memory-map entry with the needed access, and be pairwise disjoint from the handoff, map, RHD, entropy, and trace ranges. Metadata and entropy must be non-executable. Trace is the only writable destination.

Root/Recovery owns all source buffers until acceptance. The Nucleus bounded-copies map, RHD, and entropy and retains values, never boot pointers. Entropy is untrusted seed input: structural acceptance makes no unpredictability or authenticity claim. If entropy flag bit 0 is set the adapter clears the source after copying; otherwise it relinquishes access. Temporary copies are cleared after seeding.

The trace channel is only a versioned bounded byte sink identified by `(id, major, minor)`; R0-002 assigns no record framing or trust semantics. Exclusive producer ownership transfers to the Nucleus on acceptance and ends at shutdown. Mutation of any bound window during validation invalidates the handoff.

## Stable validation codes and order

Codes are `u32`: 0 ok; 1 truncated; 2 oversized; 3 bad-magic; 4 unsupported-major; 5 unsupported-minor; 6 bad-header-size; 7 unsupported-flags; 8 nonzero-reserved; 9 bad-alignment; 10 range-overflow; 11 out-of-address-range; 12 invalid-pointer-range; 13 overlap; 14 bad-count-or-length; 15 unknown-critical; 16 duplicate-id; 17 bad-reference; 18 architecture-mismatch; 19 page-size-mismatch; 20 invalid-memory-map; 21 invalid-cpu-set; 22 invalid-interrupt; 23 invalid-timer; 24 invalid-serial; 25 invalid-boot-source; 26 invalid-entropy; 27 invalid-trace; 28 noncanonical-order; 29 inconsistent-description.

Return the first applicable code in this order: bounded fixed header; magic/version/header/flags/reserved; checked lengths and address-width limits; alignment and window coverage; overlap; bounded copy; memory map; RHD framing/order; references and record-specific values; map/RHD consistency; architecture/page consistency. Rejection never requires pointer or MMIO access.

## Compatibility, recovery, and replacement

Minor additions must leave the fixed prefix unchanged and be ignorable. Breaking layout, ownership, or validation meaning requires a new major version and side-by-side decoder. Invalid handoff yields a bounded code and the R0 recovery halt; it grants no mapping, device, executable, or recovery authority. A replacement adapter must pass the same fixtures and prove zero access to rejected ranges. Structural validation is not signature or authenticity verification.
