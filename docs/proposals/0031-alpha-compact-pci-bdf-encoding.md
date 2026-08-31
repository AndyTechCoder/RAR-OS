# ADR 0031: Alpha Compact PCI BDF Encoding

Status: Historical proposal — superseded on 2026-08-31
Decision: Undecided at proposal publication

Canonical decision: [ADR 0031](../adr/0031-alpha-compact-pci-bdf-encoding.md).
This file preserves the considered alternatives and is not an authority source.

This proposal selects experimental Alpha specification bytes only. It grants no
target build, image, launch, execution, provisioning, hardware, or production
authority.

## Context

The pending P0 machine-closure candidate stores PCI inventory BDFs as an explicit
`u32`, but the disabled-function vector has only a `u16 bdf` field. The latter
must have one exact mapping so Root and Recovery independently reconstruct the
same digest. Leaving it implicit makes the public closure preimage ambiguous.

## Decision drivers

- Preserve all PCI bus, device, and function bits without collision.
- Match the fixed Alpha `00:1f.2` literal `0x00fa`.
- Keep the candidate 4-byte disabled-function record unchanged.
- Make independent reconstruction language-neutral and little-endian.

## Alternatives

- Alternative A: encode `(bus << 8) | (device << 3) | function` as a
  little-endian `u16`. This preserves 8 bus bits, 5 device bits, and 3 function
  bits and maps `00:1f.2` to `0x00fa`.
- Alternative B: truncate the inventory formula
  `(bus << 16) | (device << 11) | (function << 8)` to 16 bits. This maps the
  fixed AHCI function to `0xfa00`, contradicts `0x00fa`, and loses bus bits.
- Alternative C: enlarge the disabled-function BDF to `u32`. This is explicit
  but changes the candidate record size, vector length, offsets, and digest
  framing for no Alpha benefit.

## Proposed decision

Select Alternative A. Define the field as:

`bdf = (bus << 8) | (device << 3) | function`, serialized as little-endian
`u16`, with checked ranges `bus <= 255`, `device <= 31`, and `function <= 7`.

The exact Alpha disabled-vector digest then uses compact BDF values
`0008,d0,d1,d2,d7,e8,e9,ea,ef,fa` in the already declared function order.

## Consequences

The 136-byte disabled-vector preimage and 512-byte closure record remain the
same size. Root and Recovery gain one collision-free reconstruction rule. The
encoding is private Alpha v0 and must not be silently reused as a production
PCI identifier.

## Security and data impact

The explicit mapping prevents different functions from being confused during
the final bus-master-disabled recheck. It introduces no host path, owner data,
device authority, target execution, or persistence behavior.

## Validation

Conformance must cover minimum and maximum bus/device/function values, every
fixed Alpha function, out-of-range rejection, order changes, duplicate
functions, endian reversal, shifted-u32 truncation, and the fixed AHCI value
`0x00fa`.

## Replacement path

A production machine-discovery contract may define a different versioned PCI
identifier. It must migrate explicitly and never reinterpret Alpha v0 bytes.

## Historical approval prompt

`I approve ADR 0031 Alternative A for experimental Alpha compact PCI BDF encoding under the documented safety limits.`

The owner approved this immediately preceding exact sentence on 2026-08-31;
the canonical ADR and approval record are the only authority sources. Approval
does not by itself make P0 mergeable. The rule still requires contract/checker
integration, affected digests, mutations, reviews, guarded validation, and
exact-main evidence.
