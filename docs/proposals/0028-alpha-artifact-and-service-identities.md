# ADR 0028: Alpha Artifact and Service Identities

Status: Historical proposal — superseded on 2026-08-30
Decision: Undecided at proposal publication

Canonical decision: [ADR 0028](../adr/0028-alpha-artifact-and-service-identities.md).
This file preserves the considered alternatives and is not an authority source.

## Context

The Alpha draft transports Root, Recovery, contract, and state-reader digests,
but it does not define the exact preimages, producers, transports, or
verification stages. A digest embedded in the bytes it identifies would be
circular. A producer-selected reader digest would let untrusted metadata choose
the service authorized to receive state.

## Decision drivers

- Make every identity deterministic and non-circular.
- Bind authority to reviewed executable bytes and role, not a display name.
- Avoid introducing production signing and package machinery into Alpha.
- Preserve replaceability through explicit domain separation and versions.

## Considered options

### A. Domain-separated identities over exact immutable preimages

Define each identity as SHA-256 over a versioned domain tag, fixed-length
framing, exact role, exact contract identity where applicable, and exact
immutable artifact bytes. Identity fields are never part of their own preimage.

The image packer and independent inspector compute the Root identity from exact
`BOOTX64.EFI` file bytes as build evidence. Root separately reads that exact
fixed path into a retained immutable buffer before DMA closure, re-hashes that
buffer after DMA closure, and emits the same descriptive identity. The buffer
is retired only after that re-hash. No authority decision trusts Root's
self-reported identity in this unsigned Alpha; if the retained-buffer rule
cannot be met, the pre-closure value is untrusted diagnostic evidence only.

Root retains exact `RECOVERY.ELF` file bytes in a fixed immutable source slot,
computes the Recovery identity over those file bytes after DMA closure, and
compares it with the reviewed expected literal before parsing, mapping, or
loading Recovery. Root then transports both the bounded read-only source
descriptor and verified identity through Root-to-Recovery. After entry,
Recovery independently recomputes the identity from that retained file-byte
source as a secondary chain check, transports the verified identity through
Recovery-to-Nucleus, and retires the file-byte slot before Nucleus entry. The
slot is an existing Root-to-Recovery boot payload, not a fifth ADR 0026
platform source or a Nucleus capability. Nucleus compares the
Recovery-authenticated identity with its reviewed expected literal and
validates the record framing. It neither receives those file bytes nor claims
independent file-digest verification. Root remains the sole actor that
validates and loads Recovery.

Each initial state-service identity covers its distinct role, the exact
service-executable contract identity, and exact immutable executable payload
bytes. The trusted controller computes the two services first, records their
literal identities in the reviewed ready contract, and compiles the same exact
constants into the Nucleus Alpha adapter before the final artifact build. The
packer may copy but never choose or recompute those expected constants. The
outer record carries the fixed role/contract/identity tuples; Recovery validates
only its framing, fixed roles, exact reviewed contract identities, source
containment, and digest, without parsing the component bundle.

The Core loader remains the sole component-bundle parser. Its map request names
one fixed state-service role and one exact executable byte slice within Core's
immutable component-source capability. Nucleus does not parse bundle entries or
dependencies: it checks source-capability containment, hashes the requested
slice with the outer record's fixed role and contract identity, and compares the
result both with the reviewed literal compiled into Nucleus and with the
matching outer-record value before mapping or attaching a state slot. A mismatch
among any of the three rejects. Controller evidence binds the service artifact
hashes, literal table, Nucleus input, final Nucleus artifact, outer record, and
complete image to the exact source revision. This option is recommended.

### B. Hash fixed textual service and artifact names

Names are easy to reproduce but do not bind the implementation receiving
authority.

### C. Signed production-style identity manifests

Signed manifests provide publisher authenticity but pull later package,
signing, rollback, and trust-root work into the Alpha bootstrap.

## Recommended decision

Recommend Alternative A. These values are deterministic integrity and identity
bindings only; they do not claim publisher authenticity or production trust.

## Consequences

- Recovery retains its exact input file bytes only through its post-entry
  secondary chain check and retires them before Nucleus entry.
- The trusted build order becomes service artifacts, literal identity table,
  Nucleus artifact, then final outer record/image.
- A state-service byte change requires a reviewed Alpha envelope transition.
- Nucleus verifies bounded mapped bytes without becoming a component parser.

## Required contract details

The experimental contracts must specify:

- ASCII domain tags, version, byte order, length framing, included fields, and
  exact excluded self-referential fields for every identity;
- producer, immutable preimage source, transport record, verifier, expected
  literal, comparison stage, and mismatch outcome for Root build evidence,
  Root's descriptive self-check, Recovery's Root-enforced pre-load check and
  post-entry secondary check, contract, component, and both state-service
  identities;
- exact executable contract identities as hashes of reviewed canonical
  contract bytes;
- the controller-bound literal expected-service-identity table, its one-way
  service-first/Nucleus-second build order, the matching outer record, and a
  Nucleus verification operation limited to hashing a capability-contained
  byte slice, never bundle discovery, dependency resolution, or lifecycle
  policy;
- re-hash points after copy, after DMA closure, and before executable mapping or
  authority binding; and
- valid, wrong-domain, wrong-role, wrong-contract, stale-byte, transport-change,
  and circular-field negative fixtures with empty effect logs.

## Security and data impact

Metadata cannot choose its own authority recipient. Wrong or stale identities
fail before executable mapping, ticket creation, or capability transfer. No
secret, owner data, signing key, host identity, or production attestation is
introduced.

## Compatibility and migration

All identity domains are private Alpha v0 domains. Any service executable-byte
change creates a new identity and requires a newly reviewed outer-envelope
transition; Alpha provides no identity migration shortcut. Production package
and signed component identities replace these domains through a new reviewed
contract; old Alpha identities are rejected rather than reinterpreted. R0-002
is unchanged.

## Validation

- Recompute Root build evidence and Recovery identity from the exact available
  file-byte preimages with two independent RAR-owned implementations.
- Prove Recovery file-byte retirement before Nucleus and absence from the four
  ADR 0026 platform-source capabilities.
- Exercise wrong domain, role, contract, source slice, literal table, outer
  record, stale bytes, and identity-cycle cases with empty effect logs.
- Prove the controller binds the service-first build order and every referenced
  artifact/hash to one exact source revision.
- Prove Nucleus rejects before mapping or slot attachment when any of the
  computed, compiled, or transported identities differs.

## Replacement path

Introduce reviewed signed package/component identities, migrate through a new
versioned envelope, and reject every Alpha identity domain without silent
reinterpretation.
