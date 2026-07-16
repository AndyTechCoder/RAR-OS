# ADR 0004: Components, Capabilities, and RID

Status: Accepted — 2026-07-16

## Context

Applications, services, drivers, and agents need one replaceable unit and one explicit authority model. Clients must not bind to process identities or private types.

## Decision drivers

- Prevent ambient authority.
- Make behavior independently replaceable and testable.
- Support logical endpoint rebinding.
- Generate consistent cross-language validation.

## Considered options

- **Path, user, or process identity as authority:** rejected because it grants implicit access.
- **Implementation-specific APIs and direct process addressing:** rejected because they create hidden coupling.
- **Components, capabilities, logical endpoints, and RID:** selected.

## Decision

Use components as the primary deployment/replacement unit, unforgeable capabilities as the authority model, logical service endpoints for routing, and RID as the language-neutral interface source.

Components declare dependencies, capabilities, budgets, state, health, and lifecycle. No component receives ambient access because of installation location or process identity.

## Consequences

- Apps, drivers, services, and agents share one security and lifecycle model.
- Interface design and generated validation become foundational tooling.
- Endpoint rebinding enables replacement without client rewrites.

## Security and data impact

Delegation cannot increase rights, handles are process-local, and state access remains separately authorized. Messages and shared buffers are bounded and validated.

## Compatibility and migration

RID and capability-transport versions are independent from implementations. Breaking contracts require major versions, adapters or coordinated migration, and rollback support.

## Validation

- Forged, stale, cross-process, and over-privileged handles fail safely.
- Delegation is rights-reducing.
- Generated bindings agree on validation outcomes.
- Endpoint replacement redirects new calls without changing clients.

## Replacement path

Component implementations, RID tooling, routing, and capability transport may be replaced independently behind versioned conformance suites.
