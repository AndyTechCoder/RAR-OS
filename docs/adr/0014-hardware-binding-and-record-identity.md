# ADR 0014: Hardware Binding and Record Identity

Status: Accepted — 2026-07-17

Approval basis: explicit owner approval of the recommended decision on 2026-07-17.

## Context

The draft RHD assigns both a record-header ID and payload-specific IDs without defining equality or reference namespaces. It also represents every register location as an untyped physical address, which cannot distinguish x86 I/O ports from MMIO or describe role-separated regions such as GICv3 distributor and redistributor windows.

This decision changes public RHD fields and device authority semantics before the R0-002 draft is frozen.

## Decision drivers

- Every reference must resolve to exactly one validated object.
- Portable code must not infer address space, interrupt namespace, register role, or access width from a machine name.
- The model must describe q35-class x86-64 and ARM `virt` hardware without platform-specific branches in common code.
- Future devices must be extensible through versioned records rather than overloaded zero fields.
- Invalid or unknown model-specific combinations must have deterministic failure behavior.

## Considered options

### A. Keep header and payload IDs, requiring equality

The current duplicate IDs remain on wire and validators reject mismatches.

- Advantage: smallest textual change.
- Cost: permanent redundancy, more malformed states, and ambiguous cross-kind references.

### B. Use the record-header ID as the sole canonical identity within a typed namespace

Payload IDs are removed. References are `(record_kind, record_id)` or use a field whose target kind is fixed by schema. IDs are unique within kind; references declare their target kind.

- Advantage: one identity source, compact records, deterministic lookup, and clear duplicate rules.
- Cost: revises the current draft layouts and generated types.

### C. Use one globally unique ID across every record kind

All records share a single global namespace.

- Advantage: references need only one integer.
- Cost: unnecessary coupling between unrelated record families and harder composition of independently produced tables.

For hardware windows:

- **Single model-specific scalar window:** compact but cannot express split regions or I/O ports safely.
- **Typed register-window subrecords:** explicit address space, role, base, length, access width, stride, and byte order; extensible but larger.
- **Opaque model payloads:** flexible but pushes public semantics into platform-specific parsers.

## Decision

Use alternative B for identity and typed register-window subrecords for hardware binding.

- `record_id` is the sole identity and is unique within `record_kind`.
- A reference field has a schema-fixed target kind; ambiguous generic references encode both kind and ID.
- Payload-specific duplicate IDs are removed from the draft v1 layouts.
- Register windows declare address space (`system-memory` or `x86-io-port` initially), role, checked base/length, access width, stride, and little-endian byte order.
- Model specifications declare required and forbidden window roles. GICv3 requires distinct distributor and redistributor roles; 16550 may use either MMIO or x86 I/O-port space as declared.
- Interrupt references use a controller-relative namespace plus a declared trigger/polarity model; global numbering is derived only by the validated controller binding.
- Every window is cross-checked against the authority decision in ADR 0013 before access.

## Consequences

- The draft RHD wire layouts and generated Rust types change before freeze.
- Record lookup becomes deterministic and duplicate/mismatch states disappear.
- Hardware records become slightly larger or use bounded child records.
- Platform adapters gain explicit model validation but portable consumers avoid machine-name branches.
- Fixtures must cover duplicate IDs, dangling references, unknown roles, wrong address spaces, missing/extra windows, invalid widths/strides, and controller-relative interrupt boundaries.

## Security and data impact

One canonical identity prevents validators and consumers from resolving different privileged objects. Typed windows prevent port/MMIO confusion and make containment checks enforceable. No persistent user data is introduced.

## Compatibility and migration

This decision revises unmerged RHD v1. After freeze, adding optional roles may use a compatible minor version only when old readers can ignore them safely; identity, address-space, or required-role changes require a major version.

## Validation

- Independent decoders resolve every valid reference to the same typed record.
- q35/16550 and ARM `virt`/GICv3 fixture pairs normalize to equivalent semantic device categories.
- No accepted fixture requires a QEMU machine-name branch.
- Wrong-space, wrong-role, duplicate, dangling, overflow, and unauthorized-window cases fail before register access.

## Replacement path

Hardware discovery producers and platform consumers may be replaced independently when they emit or consume the same typed RHD values and pass the versioned corpus.
