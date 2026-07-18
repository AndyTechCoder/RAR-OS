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

A future owner-provisioned AWS authority stores one record per authorization. Issuance creates `issued`; revocation conditionally changes `issued` to `revoked`; consumption conditionally changes `issued` to `consumed` exactly once while binding the complete prepared identity graph, certification, profile, command, artifact, disk record and bytes, both firmware images, closure, execution host, resolver, spawner, irreversible consumption key, nonce, issue/expiry times, and exact GitHub identity. The accepted identity is repository `AndyTechCoder/RAR-OS`, workflow `AndyTechCoder/RAR-OS/.github/workflows/first-boot.yml@refs/heads/main`, ref `refs/heads/codex/r0-prompt7a-preauth`, protected environment `rar-r0-first-boot`, issuer `https://token.actions.githubusercontent.com`, audience `sts.amazonaws.com`, and subject `repo:AndyTechCoder/RAR-OS:environment:rar-r0-first-boot`.

The authority record is the canonical LF-terminated field order in `spec/lab/preauth/authority-v1.fields`. Its self-digest covers every field through `transition_version`; the KMS signature input additionally includes that self-digest. The strict parsed record itself—not a flattened or hand-built subset—is the ledger issue and consume input. Every authority-bearing field is cross-checked by typed name against the prepared identity graph before mutation. KMS uses the named key, `RSASSA_PSS_SHA_256`, and the exact context digest. Every DynamoDB transition is conditional on the prior state and version. Validation completes before issuance has any effect. A rejected issue, consume, revoke, reissue, replay, race, partial response, or uncertain result grants no authority. CloudTrail evidence includes a prior-evidence digest and must maintain an independently verified integrity chain.

Prompt 7A implements schemas, deterministic state transitions, request/response validation, and synthetic clients only. It makes no AWS call and accesses no credential.

## Consequences

First execution depends on an external control plane and host HTTPS availability. Guest networking remains disabled.

## Security and data impact

AWS roles have only the exact conditional-item, signing, and evidence-read permissions. Uncertain transitions fail closed and permanently require reconciliation; they never restore authority.

## Compatibility and migration

Authority schema changes require a new major schema and fresh authorization. Existing records are never reinterpreted.

## Validation

Synthetic tests cover canonical parsing and signature input, every OIDC/KMS/context binding, issuance side-effect freedom, stale/duplicate/replay/racing consume, revocation, terminal-state reissue refusal, partial/uncertain transitions, confused-deputy substitutions, crash recovery, and evidence-chain mismatch.

## Replacement path

Another authority may replace AWS only if it provides equivalent conditional monotonic state, signing, short-lived identity, and independently integrity-protected audit evidence.
