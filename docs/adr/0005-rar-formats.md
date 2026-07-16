# ADR 0005: RAR-Owned Interfaces and Formats

Status: Accepted — 2026-07-16

## Context

RAR OS needs deterministic native contracts for interfaces, signed metadata, components, portable execution, layers, packages, and installed composition.

## Decision drivers

- Canonical hashing and signing.
- Strict bounds and malformed-input handling.
- Compiler and language independence.
- Controlled evolution with migration and rollback.

## Considered options

- **Use another OS's formats as native:** rejected because their assumptions would define RAR architecture.
- **Use compiler-native layouts:** rejected because they are unstable persistent contracts.
- **Create bounded, canonical RAR-owned formats:** selected.

## Decision

Create RID, deterministic RME metadata, RCI component images, RBC portable bytecode, RLM layer manifests, RPK transport packages, and RSM system manifests. Public formats are bounded, canonical, versioned, fuzzed, and compiler-independent.

## Consequences

- RAR controls its application and update model.
- Format tooling and compatibility tests must exist before broad component work.
- Existing executable/package formats may be consumed by tools but do not define native RAR contracts.

## Security and data impact

Security metadata rejects duplicates, truncation, oversize values, unknown critical fields, invalid signatures, and rollback generations. System manifests do not embed user secrets.

## Compatibility and migration

Each persistent format defines reader coexistence, canonical-hash versioning, migration, downgrade policy, and rollback before a breaking change.

## Validation

- Independent parsers agree on canonical bytes, hashes, and rejection outcomes.
- Valid, boundary, malformed, truncated, duplicate, oversized, signature-invalid, and rollback corpora pass.
- No compiler memory layout becomes a public format.

## Replacement path

Readers, writers, compilers, and runtimes may be replaced independently when they pass the same versioned conformance and migration requirements.
