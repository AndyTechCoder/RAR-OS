# ADR 0007: RAR Implementations of Established Cryptography

Status: Accepted — 2026-07-16

## Context

Cryptography protects identity, signing, privacy, updates, and recovery. RAR needs standard interoperability without casual custom primitives.

## Decision drivers

- Use reviewed standards and official vectors.
- Avoid undeclared target-linked code.
- Preserve algorithm agility without downgrade.
- Match claims to evidence.

## Considered options

- **Invent RAR primitives:** rejected as outside normal OS work.
- **Adopt an external target library by default:** rejected by dependency policy.
- **Implement established standards under stronger review:** selected.

## Decision

RAR may implement established public cryptographic standards internally. New or modified primitives are excluded. Sensitive use requires official vectors, independent interoperability, constant-time analysis, fuzzing, specialist review, and external audit before production claims. Protocol choices require follow-up ADRs.

## Consequences

- RAR retains source ownership and integration control.
- Review cost is high.
- The VM alpha cannot claim audited production cryptography.

## Security and data impact

Secret-dependent code requires constant-time analysis. Keys, algorithms, protocols, and policy remain separated, and claims cannot exceed evidence.

## Compatibility and migration

Protocol ADRs define coexistence, key migration, revocation, and downgrade refusal.

## Validation

- Official vectors and independent implementations agree.
- Parsers and state machines are fuzzed.
- Sensitive paths receive specialist review and audit before production claims.

## Replacement path

Versioned interfaces permit implementation replacement and controlled key migration without silent downgrade.
