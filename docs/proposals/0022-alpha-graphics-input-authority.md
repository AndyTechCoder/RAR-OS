# ADR 0022: Alpha Graphics and Input Authority

Status: Proposed — owner decision required
Decision: Undecided

## Context

Sprint Alpha Milestone E requires a real framebuffer GUI plus keyboard and
pointer input. The fixed q35 profile instantiates standard VGA, xHCI, a USB
keyboard, and a USB tablet, but a host launcher selecting devices does not grant
guest authority to use them.

R0-002 intentionally defines authority only for typed RHD v1 APIC/GIC, timer,
and serial register windows. Its compatible-minor rules skip unknown records
without constructing authority. It cannot honestly convey VGA framebuffer,
PCI/xHCI, USB, keyboard, or pointer authority, and its stable semantics cannot
be silently repurposed.

ADR 0021 also fixes direct `BootEntryV1` delivery in RDI/RSI for the Alpha
Nucleus entry. Adding an outer Alpha-only authority envelope changes that
private boot boundary and therefore requires an explicit decision.

## Decision drivers

- Produce genuine guest-rendered GUI and guest-consumed input.
- Preserve the unchanged R0-002 byte contract, validation order, and errors.
- Grant no authority merely from framebuffer metadata or a QEMU profile.
- Keep raw device access out of applications and ordinary services.
- Avoid pulling a production PCI, USB, HID, IOMMU, or RHD v2 design into Alpha.
- Keep the shortcut bounded, testable, and replaceable.

## Alternatives

### A. Define production Boot Entry and RHD v2 now

Add stable framebuffer, bus, controller, input, DMA, and device authority
records. This is architecturally complete but expands Alpha into a major public
hardware contract and delays the visible system.

### B. Keep xHCI/USB and add a private Alpha peripheral grant

Recovery passes exact framebuffer and xHCI authority beside `BootEntryV1`.
Nucleus confines xHCI and exports bounded input events. This preserves R0-002,
but requires substantial PCI/xHCI/USB/HID/DMA implementation. Without an IOMMU,
the controller must remain inside a trusted platform adapter.

### C. Add a private Alpha envelope and prefer non-DMA input

Recovery produces `AlphaPlatformEntryV0`, containing byte-exact unchanged
`BootEntryV1` plus one separately framed `AlphaPeripheralGrantV0`. The grant is
bound to the exact machine-profile digest. It carries checked framebuffer
geometry/range/write authority and exact input transport ranges, interrupts,
rights, and any unavoidable DMA bounds.

The Nucleus Alpha adapter copies and validates the outer envelope, runs the
unchanged R0-002 validator with identical outcomes, then constructs no
peripheral capability until the separate grant validates. Framebuffer authority
is attenuated to the graphics service. Raw input authority remains in the
minimal platform adapter; the input service receives only a bounded event
endpoint, and applications receive only surface/input handles.

The machine profile changes to a pinned, proven QMP-injectable non-DMA input
transport if the selected QEMU/firmware combination can provide one. If not,
Alpha retains xHCI/USB but confines it as in Alternative B. QMP remains a
host-only evidence mechanism and grants no guest authority. This option is
proposed.

## Proposed direction

Select Alternative C. Before implementation, the pinned QEMU candidate must
prove which non-DMA transport is available. A reviewed experimental contract
then fixes the exact envelope bytes, profile digest, framebuffer format,
address spaces, ranges, rights, interrupt semantics, validation order, and
failure codes. No GUI/input target code may assume this proposal before owner
acceptance.

## Consequences if accepted

- Milestone E gains an honest, minimal device-authority path without changing
  stable R0-002 semantics.
- The Alpha Nucleus entry receives a private outer envelope rather than direct
  `BootEntryV1`; the embedded R0-002 bytes and validator remain unchanged.
- Graphics and input services receive attenuated capabilities, never ambient
  hardware access.
- USB/DMA complexity is avoided when the pinned platform proves a non-DMA input
  option; otherwise it remains confined to a trusted Alpha adapter.

## Validation

- Standalone R0-002 conformance remains byte-for-byte unchanged.
- Malformed R0 input fails before any peripheral grant is inspected or created.
- Wrong profile digest, overflow, overlap, executable framebuffer, excess
  rights, wrong interrupt, unbounded DMA, duplicate grant, and unknown-critical
  records fail without mapping or device access.
- Graphics cannot read input hardware; input cannot map the framebuffer; apps
  cannot access either raw device.
- Scripted QMP input reaches the guest input service and guest pixels reach the
  captured framebuffer with trace correlation.

## Replacement path

A later reviewed hardware-discovery and RHD major-version contract replaces the
Alpha envelope. Services retain their surface/input interfaces while the
platform adapter and grant disappear. No Alpha grant is accepted as a stable or
production device authority.

