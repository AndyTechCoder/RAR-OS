# Replaceability and Rewriteability

Status: Gate 0 approved direction — 2026-07-16

## Invariant

Every substantial RAR subsystem must be replaceable without requiring unrelated components or intact user data to be discarded. Replacement may involve a controlled restart for the Nucleus, Root, or Recovery Seed; it must never rely on undocumented coupling.

## Required subsystem contract

Every subsystem specification and implementation must provide:

1. A single documented responsibility and explicit non-goals.
2. Versioned interfaces, messages, errors, timeouts, and cancellation behavior.
3. Declared capabilities, dependencies, resource budgets, and hardware assumptions.
4. A separately versioned persistent-state schema.
5. Export, import, validation, migration, and rollback behavior.
6. Health checks and provisional activation criteria.
7. Conformance tests independent from the implementation.
8. Observability sufficient to compare old and replacement implementations.
9. A failure-containment and recovery-escalation path.
10. Documentation explaining how to build and substitute an implementation.

## Replacement modes

- **Cold replacement:** stop the old component, replace it, and restart its dependents.
- **Warm replacement:** preserve serialized state, restart the implementation, and rebind endpoints.
- **Live replacement:** run old and new implementations together, migrate state, redirect traffic, validate, and retire the old component.
- **Shadow replacement:** feed recorded or duplicated safe inputs to a candidate without granting production authority.
- **Foundation replacement:** use A/B system or recovery slots and a controlled restart.

The subsystem specification declares which modes it supports. Live replacement is not mandatory when it would weaken integrity.

## Interface rules

- Components communicate through RID-defined contracts rather than internal types.
- Callers address logical services, not process IDs or memory addresses.
- Unknown optional fields are ignored; unknown critical fields fail safely.
- Breaking changes require a new major interface version and an adapter or coordinated migration.
- Experimental interfaces are explicitly marked and cannot store irreplaceable state without an exit migration.
- Shared memory carries validated data structures with ownership and lifetime rules.

## State rules

- Executable code never becomes the only place where state meaning is defined.
- Migrations write a new version copy-on-write and retain the previous verified version until commit.
- Failed migration cannot make the original state unreadable.
- Downgrade behavior is declared before an update is accepted.
- Removing a layer leaves dependent data dormant unless the user explicitly deletes it.
- Replacement code receives only state it is authorized to access.

## Implementation independence

Public interfaces must not expose Rust layouts, C compiler ABI details, allocator identities, kernel object addresses, endianness-dependent structs, or private filesystem structures. Native components use a stable RAR ABI and generated bindings.

## Provisional deployment

Every privileged replacement follows:

1. Verify package, publisher, metadata, compatibility, and resources.
2. Install without replacing the active version.
3. Start with reduced or simulated capabilities.
4. Run conformance and component health checks.
5. Migrate a copy of state.
6. Route a controlled workload or shadow traffic.
7. Promote only after policy and health approval.
8. Preserve an automatic rollback path.

## Rewrite acceptance

A full rewrite is acceptable when it passes the same interface, security, state, failure, recovery, and performance conformance suites. A rewrite may intentionally change the contract only through the documented version and migration process.

## Enforcement

- CI rejects public components without RID contracts and conformance tests.
- Architecture review rejects undocumented cross-component imports or storage access.
- Stable components may not read another component's private state directly.
- System inspection must show implementation, interface versions, state schemas, capabilities, and rollback candidate.
- Every Architecture Decision Record includes a future replacement path.

## Exceptions

Hardware and early boot may force tightly coupled code. Each exception must identify why isolation is impossible, the exact trusted boundary, tests, risk, and the event that would allow later separation.
