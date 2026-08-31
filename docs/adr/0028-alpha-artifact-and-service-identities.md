# ADR 0028: Alpha Artifact and Service Identities

Status: Accepted — 2026-08-30
Decision: Alternative A

Approval basis: after the exact B/A/B decision set and its plain-language
safety effect were presented, the owner approved continuing on 2026-08-30.
Acceptance selects experimental Alpha specification work only. It grants no
target build, image, launch, execution, provisioning, or production authority.

The complete considered alternatives remain in the
[historical proposal](../proposals/0028-alpha-artifact-and-service-identities.md).

## Context

Alpha transports artifact and state-service digests, but authority cannot rely
on circular self-identities, display names, or producer-selected recipients.

## Decision drivers

- Make every identity deterministic, non-circular, and role-bound.
- Bind authority to reviewed immutable bytes rather than names or metadata.
- Preserve replacement through explicit version and domain separation.
- Avoid pulling production signing and package machinery into Alpha.

## Considered options

- Alternative A: domain-separated identities over exact immutable preimages.
  Selected as the bounded integrity mechanism.
- Alternative B: identities derived from textual names. Rejected because names
  do not bind the executable receiving authority.
- Alternative C: signed production-style manifests. Rejected for Alpha because
  it prematurely imports package, signing, rollback, and trust-root work.

## Decision

Every identity is SHA-256 over a versioned ASCII domain tag, fixed-length
framing, exact role, exact canonical contract identity where applicable, and
exact immutable artifact bytes. Identity fields are excluded from their own
preimage.

Root retains the exact `BOOTX64.EFI` and `RECOVERY.ELF` source bytes required by
their reviewed checks. It re-hashes staged bytes after DMA closure. Root's
self-identity is descriptive evidence and never an authority input. Root is the
sole pre-load Recovery verifier and loader and compares Recovery's identity with
the reviewed literal before parsing, mapping, or loading it. Recovery
independently performs the post-entry secondary check and retires its file-byte
source before Nucleus. Recovery validates outer framing, fixed roles, reviewed
contract identities, source containment, and digest; it does not parse the
component bundle. Nucleus receives only the authenticated identity and compares
it with its reviewed literal.

Core's loader remains the sole component-bundle parser. Each state-service
identity binds its distinct role, executable-contract
identity, and exact executable payload bytes. The trusted order is services,
literal table, Nucleus, then final envelope/image. Nucleus hashes only a
capability-contained byte slice selected by the fixed request and compares the
computed, compiled, and transported identities; it does not parse the component
bundle or choose a service.

## Consequences

Recovery retains its source bytes only through the secondary check. The trusted
build order becomes services, literal table, Nucleus, then final envelope. A
service-byte change requires a reviewed Alpha transition, and Nucleus remains a
bounded hasher rather than a component parser.

## Security and data impact

Metadata cannot choose its authority recipient. A wrong or stale identity fails
before executable mapping or state-slot attachment. No secret, owner data,
signing key, host identity, or production attestation is introduced.

## Compatibility and migration

All domains are private Alpha v0 identities. Production package and signed
component identities replace them through a new versioned contract; old Alpha
values are rejected rather than reinterpreted. R0-002 remains unchanged.

## Validation

Contracts must define every preimage byte, producer, immutable source,
transport, verifier, expected literal, comparison stage, retirement point,
build order, and mismatch effect. Independent vectors and negative fixtures
must cover wrong domains, roles, contracts, slices, literals, outer values,
stale bytes, framing, and circular fields with empty effect logs.

## Replacement path

Introduce reviewed signed package and component identities, migrate through a
new envelope version, and reject every Alpha identity domain without silent
reinterpretation.
