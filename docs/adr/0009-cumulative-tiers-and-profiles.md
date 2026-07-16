# ADR 0009: Cumulative Tiers and Separate Profiles

Status: Accepted — 2026-07-16

## Context

RAR OS must span constrained sensors through development systems without becoming incompatible editions or pretending every device has the same resources.

## Decision drivers

- Preserve one operating system and lower-tier compatibility.
- State meaningful minimum platform contracts.
- Keep role-specific composition separate from compatibility.
- Require explicit capabilities and honest reduced-assurance behavior.

## Considered options

- **Separate editions by device class:** rejected because components and system foundations would diverge.
- **Capabilities with no tier contracts:** rejected because compatibility floors and ecosystem expectations would be unclear.
- **Cumulative tiers plus independent profiles and capability checks:** selected.

## Decision

Use four cumulative tiers: Micro, Device, Personal, and Compute. A higher tier includes lower-tier contracts. Components declare a minimum tier and specific capabilities; capability checks decide whether they can run. Profiles compose layers, defaults, resources, and device behavior without becoming editions. Tier changes are verified transactions, and removal preserves dependent state as dormant unless the user explicitly deletes it.

## Consequences

- Lower-tier components can move upward when required capabilities exist.
- Profiles may overlap or change without redefining RAR OS.
- Tier 0 must declare which guarantees cannot be enforced and expose the resulting reduced assurance.
- Resource costs remain visible rather than hidden behind a tier label.

## Security and data impact

Tier membership grants no authority. Capabilities remain explicit, reduced assurance is visible, and tier removal cannot silently delete intact state.

## Compatibility and migration

Higher tiers support lower-tier RID contracts directly or through specified adapters. Cross-architecture movement serializes versioned state and restarts compatible code; it never copies raw process memory.

## Validation

- Conformance tests exercise every cumulative contract and transition.
- Components handle capability disappearance and dormant state.
- Portable code does not branch on profile names.
- Unsupported security or resource requirements prevent activation.

## Replacement path

Tier contracts or names may evolve only through an ADR with compatibility mapping, affected-profile review, migration, and updated conformance tests.
