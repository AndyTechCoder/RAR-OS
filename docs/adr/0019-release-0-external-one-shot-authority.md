# ADR 0019: Release 0 External One-Shot Authority

Status: Accepted — 2026-07-18

## Context

A repository-local authorization marker can be rolled back and cannot prove irreversible one-shot consumption.

## Decision drivers

- Atomic replay-resistant consumption.
- Short-lived, attributable CI identity with no long-lived repository credential.
- Independently auditable transitions and signing.

## Considered options

- **Repository-local ledger:** rejected as rollbackable.
- **A local macOS keychain or daemon:** rejected because it modifies the development host and has no approved monotonic counter.
- **DynamoDB conditional transition, KMS signing, CloudTrail integrity, GitHub OIDC:** selected.

## Decision

A future owner-provisioned AWS authority stores one record per authorization. Issuance creates `issued`; revocation conditionally changes `issued` to `revoked`; consumption conditionally changes `issued` to `consumed` exactly once while binding certification, profile, command, artifact, disk, firmware, closure, nonce, expiry, GitHub environment, repository, workflow, ref, and OIDC subject. KMS signs canonical records. CloudTrail digest-chain evidence is required for certification review. GitHub receives short-lived credentials only from an owner-approved protected environment and exact OIDC claims.

Prompt 7A implements schemas, deterministic state transitions, request/response validation, and synthetic clients only. It makes no AWS call and accesses no credential.

## Consequences

First execution depends on an external control plane and host HTTPS availability. Guest networking remains disabled.

## Security and data impact

AWS roles have only the exact conditional-item, signing, and evidence-read permissions. Uncertain transitions fail closed and permanently require reconciliation; they never restore authority.

## Compatibility and migration

Authority schema changes require a new major schema and fresh authorization. Existing records are never reinterpreted.

## Validation

Synthetic tests cover issuance, signature binding, claim mismatch, stale/duplicate/replay consume, revocation, timeout, uncertain commit, crash recovery, and evidence-chain mismatch.

## Replacement path

Another authority may replace AWS only if it provides equivalent conditional monotonic state, signing, short-lived identity, and independently integrity-protected audit evidence.
