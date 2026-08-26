# Experimental Alpha x86-64 Boot Contract v0

Status: incomplete candidate under accepted ADR 0021; not implementation authority

## Purpose

This draft is working toward the minimum authentic Alpha chain:

`UEFI firmware → RAR Root → RAR Recovery → RAR Nucleus`

It is deliberately blocked until a focused decision fixes the complete
byte-producing GPT/FAT rules, final R0 source placement and ownership, total
UEFI attribute conversion, authoritative timer provenance, and x86 NX/WP entry
state. Implementers must not invent those details from this draft.

Root is the only UEFI application. Recovery and Nucleus are separate
freestanding RAR-owned ELF64 binaries. Root exits UEFI boot services exactly
once successfully; Recovery and Nucleus receive no firmware pointer and make no
firmware call. Recovery alone validates and loads Nucleus and constructs the
unchanged R0-002 Nucleus entry.

The candidate FAT32/GPT image, ELF container, fixed paths, and
Root-to-Recovery entry are
private Alpha contracts. They are not the future RAR package, executable,
filesystem, A/B Root, or production recovery format.

## Fixed image and payloads

The unsigned 64 MiB raw image uses a protective MBR, deterministic GPT, and one
FAT32 EFI System Partition. Root is `\EFI\BOOT\BOOTX64.EFI`; Recovery is
`\RAR\ALPHA\RECOVERY.ELF`; Nucleus is `\RAR\ALPHA\NUCLEUS.ELF`. Only ASCII 8.3
directory entries are emitted. Allocation, directory ordering, timestamps,
padding, GUIDs, and unused bytes must eventually be fully fixed by
`alpha-boot-v0.fields`. The current zero-fill rule does not substitute for the
missing required or computed standard fields.

Root accepts only the listed paths. It bounds both payloads before allocation,
fully validates and loads Recovery, stages Nucleus as inert bytes, hashes the
exact Nucleus file bytes with RAR-owned SHA-256, obtains the final UEFI memory
map, and exits boot services. Root does not parse Nucleus program headers.

## Root-to-Recovery boundary

Root passes only `RDI = 0x01800000` and `RSI = total_bytes`. The bounded entry
blob contains a fixed header, normalized copies of the final UEFI descriptors,
the exact staged Nucleus file bytes, a public fixed Alpha entropy fixture, and a
bounded trace buffer. Every section starts on a 4096-byte boundary, ranges are
checked and pairwise disjoint, padding is zero, and the entire blob fits in the
fixed 16 MiB slot.

Recovery starts in x86-64 long mode with interrupts disabled, direction flag
clear, known x87/SSE state, W^X page tables owned by Recovery, and a guarded
16-byte-aligned stack. Initial mappings contain only Recovery code/data, its
stack and page tables, the entry sections with their exact rights, and the
serial/APIC windows required for bounded failure output. Recovery never returns.

Recovery validates entry framing and sections before interpreting Nucleus. It
recomputes the Nucleus SHA-256, then validates every ELF header and segment,
allocates only the fixed Nucleus slot, copies file bytes, zeroes BSS, installs
W^X mappings, and confirms the entry point lies in one executable segment.

## R0-002 production

Recovery converts the final firmware map using the total mapping table in the
field contract, then carves all Root, Recovery, Nucleus, entry, page-table,
stack, trace, and device ranges. It sorts, splits, and merges deterministically
and assigns region IDs after canonicalization. The RHD memory records are byte-
semantic mirrors of that map.

The q35 Alpha RHD uses the existing approved R0-002 x86 values: one boot CPU,
one APIC controller, one 100 MHz architectural timer, one 16550 serial device,
one boot-volume source, APIC MMIO at `0xfee00000..0xfee01000`, and serial I/O at
`0x3f8..0x400`. Descriptive records grant no access; exact R0-002 device
authority descriptors grant the two windows.

Before Nucleus entry, Recovery owns every handoff, map, RHD, entropy, and trace
source. It completes all writes, revokes producer/DMA writes, marks source
descriptors immutable as required, and declares producer `Recovery` for every
named source. Nucleus receives the unchanged `BootEntryV1` in RDI/RSI under the
approved x86-64 adapter state. Root is never an implicit R0-002 producer.

## Failure and limits

All arithmetic is checked. Failure emits one bounded serial line
`boot:error:<stage>:<code>` and halts with interrupts disabled; it never tries a
different path, payload, mapping, or firmware service. The declarative cases in
`cases.v0` are mandatory implementation fixtures, not waived documentation.

The fixed entropy bytes are public test data and provide no production entropy
claim. SHA-256 here establishes deterministic payload identity, not publisher
authenticity. Alpha has no production boot signatures, rollback counter, A/B
activation, persistent user data, update compatibility, or physical-hardware
support claim.

Framebuffer, keyboard, and pointer device authority are deliberately not
smuggled through R0-002. Their smallest Alpha-only authority extension requires
a separate reviewed architecture decision before Milestone E.
