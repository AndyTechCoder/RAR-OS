# ADR 0031: Alpha Compact PCI BDF Encoding

Status: Accepted — 2026-08-31
Decision: Alternative A

Approval basis: Codex presented the exact Alternative A approval sentence,
confirmed that the choice is safe within its documented experimental limits,
and repeated the sentence for machine-valid approval. The owner then replied,
"I approve." This acceptance records approval only of that immediately
preceding exact sentence and grants no broader authority.

Acceptance selects experimental Alpha specification bytes only. It grants no
target build, image, launch, execution, provisioning, hardware, persistence,
or production authority.

The complete considered alternatives remain in the
[historical proposal](../proposals/0031-alpha-compact-pci-bdf-encoding.md).

## Context

The pending P0 machine-closure candidate stores PCI inventory BDFs as an
explicit `u32`, but its disabled-function vector has a `u16 bdf` field. Root
and Recovery require one total mapping to reconstruct identical closure bytes
and digest independently.

## Decision drivers

- Preserve every PCI bus, device, and function bit without collision.
- Match the fixed Alpha `00:1f.2` literal `0x00fa`.
- Keep the candidate 4-byte disabled-function record unchanged.
- Make reconstruction checked, language-neutral, and little-endian.

## Considered options

- Alternative A: compact `(bus << 8) | (device << 3) | function` into a
  little-endian `u16`. Selected because it preserves the complete 8/5/3-bit
  tuple and matches `00:1f.2 → 0x00fa`.
- Alternative B: truncate the inventory formula
  `(bus << 16) | (device << 11) | (function << 8)` to 16 bits. Rejected because
  it maps AHCI to `0xfa00` and loses bus bits.
- Alternative C: enlarge the disabled-function BDF to `u32`. Rejected because
  it changes the candidate record size, vector length, offsets, and framing
  without an Alpha benefit.

## Decision

For private experimental Alpha v0 closure framing:

`bdf_u16 = (bus << 8) | (device << 3) | function`

Inputs are checked before shifting: `bus <= 255`, `device <= 31`, and
`function <= 7`. The result is serialized as little-endian `u16`. Out-of-range
input, overflow, truncation, endian reversal, missing/extra/duplicate/reordered
functions, or disagreement between independent reconstructions rejects before
authority transfer with no device or mapping effect.

The declared bus-master-disable order has exact values
`0x0008,0x00d0,0x00d1,0x00d2,0x00d7,0x00e8,0x00e9,0x00ea,0x00ef,0x00fa`
and exact little-endian byte pairs
`08 00,d0 00,d1 00,d2 00,d7 00,e8 00,e9 00,ea 00,ef 00,fa 00`.

This compact value is not the existing `u32` PCI-inventory encoding, a general
PCI identifier, permission to enumerate PCI, or device authority.

## Consequences

The 136-byte disabled-vector preimage and 512-byte closure record retain their
sizes. Root and Recovery can independently reconstruct one collision-free
function vector. The P0 contract and trusted checker still require separate
integration, reviews, guarded validation, merge, and exact-main evidence.

## Security and data impact

The total mapping prevents one PCI function from being confused with another
during the final bus-master-disabled recheck. It introduces no host path, owner
data, storage, persistence, target execution, or production behavior.

## Compatibility and migration

The encoding is private Alpha v0. A production machine-discovery contract may
define another versioned PCI identifier, but it must migrate explicitly and
must never silently reinterpret Alpha v0 bytes.

## Validation

Conformance must include min/max and independent bus/device/function basis
vectors, every fixed Alpha function, checked range rejection, overflow,
ordering, duplicates, endian reversal, shifted-u32 truncation, the fixed AHCI
value, two independent preimage implementations, complete mutation coverage,
and exact-head plus exact-main guarded evidence.

## Replacement path

Accept a new versioned machine-discovery ADR and contract, migrate every
producer and verifier explicitly, preserve old-version rejection, and retire
Alpha v0 only after reviewed replacement evidence.
