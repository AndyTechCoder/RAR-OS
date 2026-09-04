# ADR 0032: Fast-Track Alpha Milestone Governance

Status: Accepted — 2026-09-04
Decision: Alternative A

## Context

Alpha work accumulated nested packet, activation, verifier and retirement
chains. Those chains protected important boundaries but made preparation the
unit of progress, created self-referential merge-first validation, and delayed
authentic OS implementation. The owner requires a fast, safe path to a real
bootable foundation without weakening RAR OS commitments.

## Decision drivers

- Measure progress by working, retained evidence rather than prompt numbers.
- Preserve from-scratch target code and replaceable subsystem boundaries.
- Keep all local Mac and SSD target execution forbidden.
- Prevent pull-request code from acquiring workflow, credential or host authority.
- Reduce repeated reviews while retaining deep review at security boundaries.
- Keep documentation and tests coupled to public behavior.

## Considered options

### Alternative A — Five vertical milestones with isolated proposal validation

Use Foundation, Platform, Usable Alpha, Modern Architecture and Expansion as the
active sequence. Use quick continuous checks, a small number of vertical pull
requests, one integrated independent review near each milestone close, and one
consolidated remediation pass where practical. Permit changed proposal
validators to execute only inside a trusted fixed sandbox.

### Alternative B — Preserve the existing authorization chains

Continue creating separate packet, activation, verifier and retirement changes
for each narrow step.

### Alternative C — Remove governance and validation gates

Permit direct implementation and merging without milestone contracts or
evidence gates.

## Decision

Adopt Alternative A. Historic chains remain evidence but no longer authorize or
block new Alpha work. The active milestone contract is
[`Fast-Track Alpha Milestone 1`](../tasks/fast-track-alpha-milestone-1.md).

The trusted default-branch workflow remains the outer controller. A canonical
same-repository proposal whose executable validation closure differs from the
base receives `isolated-proposal` status. It may run only in the workflow's
fixed pinned container with no credentials, network, capabilities, elevation,
host devices, Docker socket or persistent state, under explicit resource and
time bounds. Proposal content cannot change the outer commands or sandbox.
Exact-main validation remains regression evidence after merge, not a
precondition that prevents reviewing the validator change itself.

The owner approves one reviewed transition merge from the previous controller
semantics. Target implementation cannot merge until the resulting exact-main
Specifications run passes.

## Consequences

- Progress is reported by the five milestone contracts and runtime evidence.
- Low-risk changes no longer receive repeated full audits.
- Security-critical boundaries still receive focused review.
- A malicious proposal can waste only bounded disposable runner resources; it
  cannot obtain repository write authority or host access through this workflow.
- Historic documents may contain obsolete sequencing language and are labeled
  as historic instead of being rewritten.

## Security and data impact

No constitutional or technical trust boundary is weakened. Signing, isolation,
recovery, rollback, validation, dependency controls and System Store/Data Vault
separation remain mandatory. The Mac and SSD remain non-execution environments.
For the current session they are also read-only: repository mutation occurs
through GitHub only.

The certified cloud guest has no networking, credentials, host filesystem,
raw-disk access, device passthrough or elevated privileges. Its storage and
firmware are disposable inputs with recorded identities.

## Compatibility and migration

Existing commits, branches, evidence and accepted ADRs remain intact. Existing
public formats and tier meanings do not change. Historic A–G and prompt
documents remain readable records. New work starts at Milestone 1 and may reuse
reviewed contracts only when they match the active milestone.

## Validation

- Static policy proves the trusted workflow remains the outer authority.
- Policy mutation tests cover exact-closure and isolated-proposal decisions.
- Controller-changing proposals execute in the fixed isolated container.
- The first resulting `main` run must complete primary and mutation validation.
- Each implementation milestone requires its own runtime evidence contract.
- Documentation must distinguish specification, build and boot evidence.

## Replacement path

A later accepted ADR may replace the milestone sequence or sandbox design after
documenting migration, security consequences and retained evidence. It may not
silently relax the constitution or persistent-data promises.
