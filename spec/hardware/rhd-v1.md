# Release 0 RAR Hardware Description

Status: Draft implementation contract for R0-002

Source manifest: `rhd-v1.fields`

## Purpose and non-goals

RHD v1 gives x86-64 and AArch64 one normalized description of processors, memory, interrupts, timers, serial output, and boot source. Platform discovery produces RHD; portable Nucleus code consumes it without firmware, profile, board, or emulator names.

It does not define drivers, storage, networking, firmware callbacks, packages, a post-boot discovery service, Tier 0 layout, integrity, or authenticity.

## Encoding and bounds

Integers are unsigned little-endian wire values. Addresses are 64-bit physical byte addresses. Public format is bytes, never a Rust/C layout. The blob is 32 through 65,536 bytes, has at most 256 records, and is aligned to 8 bytes. Every addition and multiplication is checked before access. Reserved bytes are zero.

The 32-byte header is:

| Offset | Type | Field | Rule |
| ---: | --- | --- | --- |
| 0 | `[u8; 8]` | magic | `RARRHD\0\0` |
| 8 | `u16` | major | `1` |
| 10 | `u16` | minor | `0` |
| 12 | `u16` | header bytes | `32` |
| 14 | `u16` | record-header bytes | `16` |
| 16 | `u32` | total bytes | bounded, multiple of 8 |
| 20 | `u16` | record count | 1 through 256 |
| 22 | `u16` | flags | zero |
| 24 | `u16` | architecture | 1 x86-64, 2 AArch64 |
| 26 | `u8` | address bits | 32 through 64 |
| 27 | `u8` | page shift | 12 through 30 |
| 28 | `u32` | reserved | zero |

Every record begins with a 16-byte prefix: `kind:u16`, `flags:u16`, `record_bytes:u32`, `record_id:u32`, `reserved:u32`. Flag bit 0 means critical; other bits and reserved are zero. Record size includes the prefix and is a nonzero multiple of 8. Records are canonically ordered by `(kind, record_id)` and IDs are unique within a kind. Unknown non-critical records are range-checked then skipped; unknown critical records fail.

## Required record payloads

- CPU, total 48 bytes: `cpu_id:u32`, `flags:u32`, `hardware_id:u64`, `interrupt_controller_id:u32`, `timer_id:u32`, `reserved[8]`. Exactly one CPU has boot flag bit 0, and its ID equals the handoff `boot_cpu_id`.
- Memory, total 48 bytes: `base:u64`, `length:u64`, `kind:u16`, `attributes:u16`, `owner:u16`, `reserved0:u16`, `region_id:u32`, `reserved1:u32`. It is semantically identical to `MemoryRegionV1` in the handoff map. Entries are sorted by base, nonempty, non-wrapping, and non-overlapping.
- Interrupt controller, total 48 bytes: `controller_id:u32`, `model:u16`, `flags:u16`, `register_base:u64`, `register_bytes:u32`, `global_interrupt_base:u32`, `interrupt_count:u32`, `reserved:u32`. Models are 1 x86 APIC and 2 ARM GICv3 and must match architecture.
- Timer, total 48 bytes: `timer_id:u32`, `model:u16`, `flags:u16`, `frequency_hz:u64`, `register_base:u64`, `interrupt:u32`, `interrupt_controller_id:u32`. Frequency is nonzero; models are 1 x86 architectural timer and 2 ARM generic timer.
- Serial, total 48 bytes: `serial_id:u32`, `model:u16`, `flags:u16`, `register_base:u64`, `register_bytes:u16`, `interrupt:u16`, `interrupt_controller_id:u32`, `input_clock_hz:u64`. Models are 1 16550-compatible and 2 PL011-compatible.
- Boot source, total 24 bytes: `kind:u16`, `flags:u16`, `stable_source_id:u32`. It carries no firmware callback, executable pointer, or retained firmware address.

At least one record of every required kind exists. References identify records, never memory addresses. Dangling references fail. The authoritative dynamic ownership view is the boot memory map; RHD memory records must normalize to the same ranges, kinds, attributes, owners, and IDs or validation returns `inconsistent-description`.

## Validation, ownership, and architecture equivalence

The adapter validates the RHD candidate address using the boot contract, bounded-copies the complete blob, then parses only the owned copy. Order is fixed header, scalar/bounds, record framing/order, record values/references, memory-map equivalence, then architecture consistency. A rejected value is never used for MMIO or pointer access.

x86-64 and AArch64 use different model values but produce identical semantic categories and reference relationships. The valid baseline fixtures carry the same semantic identity. Portable code branches on normalized values, never QEMU machine names.

## Versioning, failure, and replacement

Minor versions may add only ignorable non-critical records. Changed meaning, required content, byte order, or bounds requires a new major version and parallel decoder. Failure uses the stable boot validation codes and enters the R0 invalid-handoff recovery halt. Platform adapters and decoders are replaceable when they produce equal owned values and pass the corpus. Platform evidence may require a versioned correction; v1 is never silently reinterpreted.
