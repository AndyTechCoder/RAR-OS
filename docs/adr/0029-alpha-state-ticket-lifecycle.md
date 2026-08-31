# ADR 0029: Alpha State Ticket Lifecycle

Status: Accepted — 2026-08-30
Decision: Alternative B

Approval basis: after the exact B/A/B decision set and its plain-language
safety effect were presented, the owner approved continuing on 2026-08-30.
Acceptance selects experimental Alpha specification work only. It grants no
target build, image, launch, execution, provisioning, or production authority.

The complete considered alternatives remain in the
[historical proposal](../proposals/0029-alpha-state-ticket-lifecycle.md).

## Context

Initial state bytes must remain unreadable to Core while their two owning
services receive deterministic, restartable, and revocable authority.

## Decision drivers

- Keep state bytes completely unreadable to Core.
- Bind authority to independently verified service identities.
- Make redemption atomic, deterministic, restartable, and revocable.
- Avoid a permanent broad policy broker or ambient state authority.

## Considered options

- Alternative A: consume authority when a process handle is delivered. Rejected
  because a pre-import crash could permanently lose initial-state authority.
- Alternative B: redeem revocable Nucleus-held identity-bound slots. Selected
  because it supports same-identity restart without exposing state to Core.
- Alternative C: permanent Nucleus source broker. Rejected because it creates a
  wider continuing policy and confused-deputy surface.

## Decision

Nucleus holds exactly two state-authority slots, one per approved role, each
owning the sole read capability to its immutable source and the exact expected
service identity. Core receives only two fixed-position opaque selectors.
Selectors are non-readable, non-clonable, non-delegable, and role-bound.

Core may use a selector only in a bounded create-service request. After Nucleus
verifies the executable under ADR 0028, Nucleus injects a distinct,
nondelegable, incarnation-bound redeem token directly into that service. Core
never possesses or observes the token. One atomic transition gives the first
matching incarnation a derived read-only source capability; concurrent attempts
have exactly one winner. Wrong, malformed, or stale attempts grant nothing and
do not consume an available slot.

On service exit or crash, Nucleus revokes the derived handle and permits only
the same verified identity to rebind. Only verified source-integrity failure or
a reviewed reconstruction-teardown transition may terminally revoke the slot.
Terminal revocation destroys future slot, selector, token, and derived-handle
authority and quarantines the service's mutable destination without claiming
erasure of information already observed.

## Consequences

Core receives two selectors but no readable state. Each verified service gets
its token directly from Nucleus. Crash recovery rebinds only the same identity,
and terminal revocation also quarantines the mutable destination without an
information-erasure claim.

## Security and data impact

Core and unrelated services cannot read either source. Revocation removes all
future capability and address-space access but does not claim to erase bytes an
authorized service already observed. No owner data, host path, or persistence
promise is introduced.

## Compatibility and migration

Slots, selectors, tokens, and handles are private Alpha mechanisms. Production
state and lifecycle versions replace them explicitly; Alpha handles are never
reused or silently reinterpreted.

## Validation

Contracts must define object kinds and rights, fixed selector positions,
direct token injection, states `unredeemed`, `bound`, `rebindable`, and
`revoked`, every event transition, deterministic precedence, concurrency,
crash/restart, terminal causes, quarantine, stale handling, and audit events.
Tests must prove Core and unrelated services never map, read, clone, delegate,
retarget, redeem, or revoke either source.

## Replacement path

Accept production state and lifecycle contracts, migrate without reusing Alpha
handles, and remove private slots only after reviewed source/destination
migration or destruction.
