# ADR 0001: Staged Releases

Status: Accepted — 2026-07-16

## Context

RAR OS has a universal multi-tier vision whose later subsystems depend on evidence and contracts produced by earlier foundations. Attempting one undifferentiated delivery would force interfaces to be guessed before their prerequisites exist.

## Decision drivers

- Preserve the complete product direction without claiming unbuilt behavior.
- Produce reviewable evidence and migration points early.
- Keep later features from bypassing security, data, recovery, and interface gates.
- Allow parallel work only where ownership and dependencies are explicit.

## Considered options

- **One complete implementation milestone:** rejected because it hides dependency order and delays useful evidence.
- **Independent product editions:** rejected because it conflicts with one cumulative RAR OS.
- **Evidence-gated staged releases:** selected because each release can establish contracts required by the next.

## Decision

Use Releases 0–7 from `release-roadmap.md`. Each release is independently buildable, documented, testable, and useful to the next release. Later architecture may be anticipated in specifications, but later features cannot bypass earlier contracts or enter a release merely to demonstrate progress.

## Consequences

- Working evidence appears before consumer-facing completeness.
- Interface mistakes can be corrected before broad ecosystem dependence.
- Some visible product features arrive well after architecture work begins.
- Gate closure and task-packet generation become recurring release work.

## Security and data impact

Security, recovery, privacy, and data-preservation claims are introduced only with their required evidence. A later release cannot weaken an earlier gate or treat future recovery behavior as already implemented.

## Compatibility and migration

Each release records its public contracts, limitations, and next-release migration notes. Regrouping work requires an ADR that preserves explicit dependencies and an upgrade path from the previous accepted gate.

## Validation

- Every task maps to one release packet and owned path set.
- Gate evidence maps each promise to a passing check or explicit limitation.
- Integration review rejects later-release implementation outside the active scope.
- The previous known-good release remains reproducible.

## Replacement path

Milestone contents may be regrouped through an ADR if dependencies, acceptance evidence, compatibility, and migration remain explicit.
