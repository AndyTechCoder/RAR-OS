# ADR 0015: Deterministic Validation Precedence

Status: Accepted — 2026-07-17

Approval basis: explicit owner approval of the recommended decision on 2026-07-17.

## Context

R0-002 assigns stable validation codes but orders only broad phases. Multi-fault inputs can therefore produce different first errors across independent or architecture-specific decoders. Numeric code order also does not consistently express safe access order.

This decision changes public failure semantics before the R0-002 draft is frozen.

## Decision drivers

- Identical bytes and trusted-window inputs must produce one result on every architecture.
- No later predicate may require a read that an earlier predicate has not made safe.
- Stable codes must remain useful for recovery evidence without leaking unchecked input.
- The order must be generated and tested, not duplicated informally in prose and code.

## Considered options

### A. Lowest numeric validation code wins

Evaluate all applicable errors conceptually and return the lowest code.

- Advantage: simple rule.
- Cost: unsafe because discovering a low-numbered later error may require reads that earlier bounds checks have not authorized.

### B. Keep phase ordering and allow implementation-defined ties

Each decoder follows the current broad phases but chooses its own order within a phase.

- Advantage: easiest implementation.
- Cost: violates deterministic cross-architecture conformance and makes compound-invalid fixtures non-portable.

### C. One total ordered predicate table with access preconditions

The canonical schema assigns every validation predicate a unique precedence and states which bytes or windows may be accessed after it passes. Generated decoders and conformance fixtures derive from that table.

- Advantage: deterministic, auditable, and safe by construction.
- Cost: more specification work and deliberate compatibility management when predicates change.

## Decision

Use alternative C.

The canonical schema contains a total predicate sequence, not merely code numbers. It begins with externally bounded fixed-header availability; then magic/version/fixed sizes; checked scalar arithmetic, address-width limits, alignment, entry semantics and descriptor binding; alias rejection and snapshot acquisition; exact embedded/external length equality; record framing, identity and references; memory and model validation; canonical order; authoritative-map consistency and device authority; and finally cross-record architecture/page consistency.

Each row defines:

- predicate identity and stable returned code;
- required previously validated facts;
- exact bytes/windows it may inspect;
- whether failure permits evidence fields beyond the code;
- compatibility behavior for unknown minor, optional, and critical fields.

Changing precedence or the meaning of an existing failure is a breaking public-contract change. Adding a new predicate requires an explicitly allocated position and compatibility analysis.

## Consequences

- The prior prose order is replaced by one machine-readable table.
- Generated decoders, Rust enums, documentation, and fixtures share the same source.
- Compound-invalid fixtures are required for every adjacent precedence edge and security-sensitive non-adjacent pair.
- Some existing numeric codes may remain out of numeric order because safe access order controls precedence.

## Security and data impact

The recommendation prevents decoders from reading unvalidated locations merely to discover a nominally earlier error. Deterministic failures improve recovery and cross-architecture evidence without weakening rejection.

## Compatibility and migration

This decision revises the unmerged v1 draft. After freeze, changing precedence or code meaning requires a new major validation contract; old and new decoders may coexist only with version-selected corpora.

## Validation

- One generated table is the source for prose, decoder stubs, and test expectations.
- Every single predicate has a focused fixture.
- Every precedence edge has a dual-fault fixture proving the earlier result.
- Instrumentation proves no predicate reads outside its declared access budget.
- x86-64, AArch64, and the host reference oracle return identical codes.

## Replacement path

Any decoder may be replaced when it consumes the same table and passes all single-fault, dual-fault, boundary, and access-budget tests.
