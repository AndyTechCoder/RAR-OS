# Release 0 RAR Hardware Description

Status: Approved implementation contract for R0-002

Source manifest: `rhd-v1.fields`

## Purpose and non-goals

RHD v1 gives x86-64 and AArch64 one normalized description of processors, memory, interrupts, timers, serial output, boot source, and typed register windows. Platform discovery produces RHD; portable Nucleus code consumes it without firmware, profile, board, emulator, or machine names. RHD describes hardware but grants no register-access authority.

It does not define drivers, storage, networking, firmware callbacks, packages, a post-boot discovery service, Tier 0 layout, integrity, or authenticity.

## Encoding, framing, and identity

Integers are unsigned little-endian wire values. Addresses are physical byte addresses in the explicitly declared address space. Public format is bytes, never a Rust/C layout. The blob is 32 through 65,536 bytes, has at most 256 records, and is aligned to 8 bytes. Every addition and multiplication is checked before access. Reserved bytes are zero.

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

Every record begins with `kind:u16`, `flags:u16`, `record_bytes:u32`, `record_id:u32`, `reserved:u32`. `record_id` is the sole identity and is unique within its kind; payloads never repeat it. References name a schema-fixed target kind. Flag bit 0 means critical; other bits and reserved fields are zero. Record size includes the prefix and is a nonzero multiple of 8. Records are canonically ordered by `(kind, record_id)`. Unknown non-critical records are range-checked then skipped; unknown critical records fail.

## Required records

- CPU, 48 bytes: `flags:u32`, reserved `u32`, `hardware_id:u64`, `interrupt_controller_id:u32`, `timer_id:u32`, reserved 8 bytes. Exactly one CPU has boot flag bit 0; its header ID equals handoff `boot_cpu_id`.
- Memory, 48 bytes: `base:u64`, `length:u64`, `kind:u16`, `attributes:u16`, `owner:u16`, reserved 10 bytes. Its header ID equals the corresponding memory-map `region_id`; all other semantic fields match exactly. Entries are sorted, nonempty, non-wrapping, and non-overlapping.
- Interrupt controller, 48 bytes: `model:u16`, `flags:u16`, `global_interrupt_base:u32`, `interrupt_count:u32`, reserved 20 bytes. Models are x86 APIC (1) and ARM GICv3 (2).
- Timer, 48 bytes: `model:u16`, `flags:u16`, reserved `u32`, `frequency_hz:u64`, controller-relative `interrupt_index:u32`, `interrupt_controller_id:u32`, `interrupt_trigger:u8`, `interrupt_polarity:u8`, reserved 6 bytes. Models are x86 architectural timer (1) and ARM generic timer (2); both use architectural/system registers and forbid register-window children.
- Serial, 48 bytes: `model:u16`, `flags:u16`, controller-relative `interrupt_index:u32`, `interrupt_controller_id:u32`, reserved `u32`, `input_clock_hz:u64`, `interrupt_trigger:u8`, `interrupt_polarity:u8`, reserved 6 bytes. Models are 16550-compatible (1) and PL011-compatible (2).
- Boot source, 24 bytes: `kind:u16`, `flags:u16`, reserved `u32`. It carries no firmware callback, executable pointer, retained firmware address, or separate payload identity.
- Register window, 56 bytes: `parent_kind:u16`, `role:u16`, `parent_id:u32`, `address_space:u16`, `access_width:u8`, `byte_order:u8`, `stride:u16`, `flags:u16`, `base:u64`, `length:u64`, `authority_descriptor_index:u16`, and reserved fields. `parent_kind` must be interrupt (3), timer (4), or serial (5); the ID must resolve uniquely in that kind.

At least one record of every core kind exists and at least the model-required register windows exist. Dangling references, duplicate IDs within a kind, ambiguous targets, and forbidden windows fail.

CPU payload flag bit 0 identifies the boot CPU and all other CPU flag bits are zero. Interrupt, timer, serial, and boot-source payload flags are zero. Release 0 boot-source kind 1 means the validated boot volume and must equal handoff `boot_source_kind`. All known payload reserved fields are zero and every known record has its exact declared size.

## Typed register binding

Address spaces are system memory (1) and x86 I/O port (2). Roles are APIC (1), GIC distributor (2), GIC redistributor (3), timer (4), and serial (5). Byte order is explicitly little-endian (1). Access width and stride are bytes and must be nonzero, powers of two, no larger than the window, and model-exact:

| Parent model | Required windows | Space / width / stride |
| --- | --- | --- |
| x86 APIC | one APIC | system-memory / 4 / 16 |
| ARM GICv3 | one distributor and one redistributor | system-memory / 4 / 4 |
| x86 architectural timer | none | windows forbidden |
| ARM generic timer | none | windows forbidden |
| 16550 serial | one serial | system-memory or x86-I/O-port / 1 / 1 |
| PL011 serial | one serial | system-memory / 4 / 4 |

Unknown roles, model-incompatible roles or spaces, missing or extra required windows, overflow, and invalid width/stride return `invalid-register-window`. I/O-port ranges must end at or below 65,536 and are valid only for x86-64.

Each window names exactly one Boot Entry device descriptor by zero-based `authority_descriptor_index`. Its full checked range, space, parent kind/ID, and read/write/device rights must agree. System-memory windows must also lie within one device-owned MMIO memory-map entry. An RHD record or memory-map entry alone never grants access; a mismatch returns `unauthorized-device-window` before any register access.

## References and equivalence

CPU references target interrupt and timer kinds. Timer and serial controller IDs target interrupt records. Interrupt indices are controller-relative and strictly less than the referenced controller's `interrupt_count`; global numbering is derived only after validation. Every timer and serial interrupt declares trigger (edge 1 or level 2) and polarity (active-high 1 or active-low 2); consumers never infer them from the platform name.

The boot memory map is the authoritative dynamic ownership view. RHD memory records must normalize to identical ranges, kinds, attributes, owners, and header IDs or validation returns `inconsistent-description`.

x86-64 and AArch64 use different models and required windows but yield the same semantic categories and reference relationships. Portable code branches on validated normalized values, never on QEMU or board names.

## Validation, versioning, and replacement

Validation follows the total predicate table in `../boot/handoff-v1.fields`. Parsing occurs only from the owned snapshot established by the Boot Entry contract. Rejected values are never used for MMIO, port I/O, or pointer access.

Minor versions may add only safely ignorable non-critical records or optional roles. Changed identity, required content, address-space meaning, required role, byte order, or bounds requires a new major version and parallel decoder. Platform discovery producers and consumers are replaceable when they produce equal owned values and pass the versioned corpus. Failure enters the R0 invalid-handoff recovery halt.
