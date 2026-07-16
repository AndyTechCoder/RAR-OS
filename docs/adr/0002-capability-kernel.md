# ADR 0002: Capability-Based Hybrid Microkernel

Status: Accepted — 2026-07-16

## Context

RAR OS needs isolation, component replacement, recovery, portability, and measured performance while keeping its privileged boundary small.

## Decision drivers

- Contain driver and service failures.
- Enforce authority independently of component importance.
- Keep policy-rich services replaceable.
- Permit only evidence-backed privileged optimizations.

## Considered options

- **Monolithic kernel:** rejected because it expands shared failure and replacement boundaries.
- **Pure microkernel with no exceptions:** rejected because some hardware paths may need narrowly reviewed optimization.
- **Capability-based hybrid microkernel:** selected.

## Decision

Keep memory protection, threads, scheduling, interrupts, capability enforcement, IPC primitives, DMA mediation, tracing hooks, and recovery entry in the Nucleus. Run drivers and policy-rich services outside it by default. Co-location or privileged fast paths require measured benefit, documented invariants, reviewed manifests, and unchanged public contracts.

## Consequences

- Driver and service failures are containable.
- IPC and boundary validation become critical.
- Hardware exceptions require explicit justification.

## Security and data impact

Capabilities, not stack position, grant authority. Privileged exceptions expand the trusted base and require security review, focused tests, and a removal condition.

## Compatibility and migration

Components depend on RID and RAR ABI contracts, not Nucleus internals. Boundary changes require versioned adapters or coordinated migration.

## Validation

- Isolation and forged-capability tests pass.
- Failures remain contained on x86-64 and ARM64.
- Each privileged exception has measured benefit and tested invariants.

## Replacement path

The boundary may change through versioned contracts, conformance evidence, migration where needed, and correctness plus security review.
