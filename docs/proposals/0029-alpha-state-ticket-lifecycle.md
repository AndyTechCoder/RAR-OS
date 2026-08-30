# ADR 0029: Alpha State Ticket Lifecycle

Status: Proposed — owner decision required
Decision: Undecided

Recommended alternative: B. This recommendation is not a decision and grants
no implementation, build, execution, provisioning, merge, or activation
authority. Exact owner approval and the normal accepted-ADR process are required.

## Context

ADR 0026 requires exact attenuated transfer from immutable state sources to the
two owning services. The draft says tickets are sealed and one-shot, but does
not define how Core receives them, what a ticket contains, whether wrong-identity
or concurrent attempts consume it, how a service restart rebinds, or how all
derived authority is revoked. The fixed Core bootstrap capability inventory
also cannot safely treat readable state bytes as ordinary Core capabilities.

## Decision drivers

- Keep state bytes completely unreadable to Core.
- Bind state authority to independently verified service identities.
- Make redemption atomic, deterministic, and revocable.
- Permit the same verified service implementation to restart after a crash.
- Avoid a permanent broad policy broker or ambient state authority.

## Considered options

### A. Consume on delivery of a process handle

Delivery is simple, but a crash before import permanently loses initial-state
authority and encourages Core to retain or recreate it.

### B. Redeem into revocable Nucleus-held identity-bound slots

After validating the outer envelope, Nucleus creates exactly two state-authority
slots, one per approved state role. Each slot owns the sole read capability to
its immutable source and stores the exact expected service identity. Core
receives only two opaque slot-selector handles in its fixed initial capability
inventory. Selectors are non-readable, non-clonable, non-delegable, role-bound,
and reveal no state bytes.

Core uses a selector only in its bounded create-service request. After Nucleus
verifies and maps the requested state-service executable as specified by ADR
0028, Nucleus injects a distinct, nondelegable, incarnation-bound redeem token
directly into that new service's initial capability table. Core never possesses
or observes the redeem token. The service presents its injected token; the
request contains no caller-supplied identity. Nucleus derives identity from the
validated executable mapping.

Under one atomic slot transition, the first matching incarnation receives a
derived read-only source capability. Concurrent attempts have exactly one
winner. Wrong identity, wrong role, malformed request, stale selector, or stale
token grants nothing and leaves an unredeemed slot available; repeated policy
abuse may be audited but does not destroy the source.

When that service incarnation exits or crashes, Nucleus revokes its derived
handle and returns the slot to rebindable state for only the same verified
identity. Only the Nucleus's fixed Alpha recovery/quarantine mechanism may
terminally revoke a slot, and only for a verified source-integrity failure or a
reviewed reconstruction-teardown transition recorded in the state-machine
contract. Terminal revocation destroys the slot's source capability,
selector, redeem token, and every live derived handle; it cannot erase bytes a
previously authorized service already copied into its separately owned mutable
region, so that region is independently quarantined and revoked by the same
transition. Core can request service creation and observe structured status but
can neither redeem, read, duplicate, delegate, revoke, nor retarget a slot.
This option is recommended.

### C. Permanent Nucleus source broker

A permanent broker can serve restarted processes, but it creates an unnecessary
continuing policy interface and a wider confused-deputy surface.

## Recommended decision

Recommend Alternative B. It preserves one-time authority binding per live service
incarnation, permits safe restart of the same verified service, and never gives
state-readable authority to Core.

## Consequences

- Core's fixed initial capability inventory contains two selectors but no
  readable state authority.
- Each verified state-service incarnation receives its redeem token directly
  from Nucleus and can never delegate it.
- Crash recovery rebinds only the same exact reviewed identity.
- Terminal revocation also quarantines the mutable destination, while making no
  claim that already observed information can be erased.

## Required state machine and contract details

The reviewed private contract must fix:

- slot, selector, incarnation-bound redeem-token, service-incarnation, and
  derived-handle object kinds;
- exact Core initial capability positions and rights for both opaque selectors
  plus the direct Nucleus-to-service token-injection rule;
- states `unredeemed`, `bound`, `rebindable`, and `revoked`, with a total event
  transition table and deterministic failure precedence;
- atomic concurrency behavior, caller-identity derivation, crash/exit cleanup,
  restart/rebind, the two exact Nucleus-authorized terminal-revocation causes,
  mutable-region quarantine, stale-selector/token behavior, and audit events;
- checked role/source/identity bindings and proof that Core never maps source
  bytes; and
- focused success, race, wrong-identity, wrong-role, crash-before-use,
  crash-during-import, repeated-restart, revocation, allocation-failure, and
  no-effect fixtures.

Mutable state is created only by the owning service in a new non-aliased region
after its inner format validates. Immutable source slots never grant write or
execute authority.

## Security and data impact

The design prevents Core or another service from reading either state source.
Capability and address-space revocation is complete for future access; it does
not claim erasure of information an authorized service already observed.
Identity-bound restart does not widen authority.
The preserved fixture follows the already approved exact public bytes `abc` in
`docs/tasks/sprint-alpha-milestones-b-g-execution-map.md` and
`spec/alpha/evidence/README.md`; this lifecycle ADR does not select or change
those bytes and corrects the rejected contract draft's conflicting 28-byte
candidate. No owner data, host path, shared folder, or persistent shutdown
claim is introduced.

## Compatibility and migration

The ticket objects and state machine are private Alpha mechanisms. Production
state, identity, lifecycle, and recovery interfaces replace them through
versioned contracts. Old Alpha handles and source bytes are rejected rather
than migrated implicitly.

## Validation

- Cover the total slot event/state table, deterministic precedence, checked
  arithmetic, and exact initial capability positions.
- Race concurrent redemption and prove exactly one winner without consuming the
  slot on wrong identity, wrong role, malformed, or stale attempts.
- Inject crash before redemption, during import, after mutable copy, and across
  repeated same-identity restart.
- Prove terminal revocation destroys future source/selector/token/derived access
  and quarantines the mutable address space.
- Prove Core and unrelated services never map, read, clone, delegate, retarget,
  or revoke either immutable state source.

## Replacement path

Accept production state/lifecycle contracts, create an explicit migration that
never reuses Alpha handles, and remove the private slots after their sources and
mutable destinations are either migrated or destroyed under reviewed policy.
