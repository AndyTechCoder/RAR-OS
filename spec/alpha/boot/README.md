# Experimental Alpha x86-64 Boot Contract v0

Status: experimental P0 contract pending independent review; not implementation authority

## Purpose

This draft is working toward the minimum authentic Alpha chain:

`UEFI firmware → RAR Root → RAR Recovery → RAR Nucleus`

Accepted ADRs 0023 and 0026–0029 fix the private Alpha choices. This P0 contract
set remains blocked until architecture, correctness, security, mutation, merge,
and exact-main validation pass, and the machine profile remains inactive until
retained cloud evidence matches its exact firmware and topology. Implementers
must not infer authority from pending-review bytes.

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
padding, GUIDs, and unused bytes are fixed by `alpha-boot-v0.fields` and its
canonical fixture manifest.

Root accepts only the seven fixed source paths. It reserves the complete
bootstrap arena before the first read and stages every source into a distinct
bounded slot. After the final read and `ExitBootServices`, Root switches to its
private tables and stack, closes the exact q35/AHCI DMA path, re-hashes every
source, and only then validates and loads Recovery. Root does not parse
Nucleus, component, or state inner formats.

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

`alpha-machine-closure-v0.fields` is the sole private q35/AHCI closure
authority. Root rejects topology drift, stops every implemented AHCI engine
with bounded waits, disables and reads back every declared bus-master bit, and
rechecks the complete disabled vector immediately before entry. No PCI, AHCI,
boot-device, or DMA authority crosses into Recovery or later software.

Recovery performs the independent post-entry Recovery identity check, retires
the retained Recovery file source, then unmaps, invalidates, zeroes, and
normalizes Root's private ranges in the fixed order. Firmware-global stack and
table memory is not claimed or cleared; its descriptor follows the total UEFI
normalization contract.

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

## Platform and state boundary

The private `AlphaPlatformEntryV0` wraps the unchanged `BootEntryV1` and four
fixed immutable sources. Recovery validates outer framing only. Nucleus maps
one fixed Core bootstrap and creates two identity-bound state slots. Core gets
one component-source capability and two opaque selectors; it never gets state
readability or a redeem token. Core alone parses the component bundle, and each
state service alone parses its matching source.

Identity is SHA-256 over versioned, domain-separated, fixed-framed exact bytes.
Root self-identity is descriptive. All validators share
`../platform/alpha-validation-v0.fields` as their sole first-failure authority.
A rejection commits no mapping, thread, capability, slot, device, or mutable-
state effect.

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
