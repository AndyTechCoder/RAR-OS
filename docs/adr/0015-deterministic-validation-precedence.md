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

### C. Artifact-qualified staged predicate tables with one inter-artifact order

The canonical schema assigns every validation predicate to a named artifact and stage, states which bytes or windows may be accessed after it passes, and defines one exact order between artifacts. Generated decoders and conformance fixtures derive from that table.

- Advantage: deterministic, auditable, and safe by construction.
- Cost: more specification work and deliberate compatibility management when predicates change.

## Decision

Use alternative C.

The canonical schema contains artifact-qualified staged predicates plus an exact inter-artifact first-error order, not a single incomplete global sequence and not merely code numbers. It begins with adapter bounds, arithmetic, address-width, and alignment; copies and frames the entry; validates descriptors and alias rules; acquires each descriptor-keyed source exactly once; then validates owned handoff, map, and RHD framing and semantics before cross-artifact consistency and commit-only effects. No predicate may inspect an artifact before its validated acquisition.

Each row defines:

- predicate identity and stable returned code;
- required previously validated facts;
- exact bytes/windows it may inspect;
- whether failure permits evidence fields beyond the code;
- compatibility behavior for unknown minor, optional, and critical fields.

The entry-framing row binds the owned entry architecture and address width to the already validated trusted adapter tuple before descriptor arithmetic or acquisition. The descriptor binding row is a whole-table predicate that deliberately collapses zero length, address-width, alignment, selector/cardinality, rights, producer, transfer, and flag failures to `invalid-pointer-range`. RHD rows are likewise whole-table stages: framing precedes compatibility, identity, references, CPU, interrupt, timer, serial, boot source, canonical order, and cross-artifact checks. Reordering descriptors or records cannot select a different first error within those stages.

Changing precedence or the meaning of an existing failure is a breaking public-contract change. Adding a new predicate requires an explicitly allocated position and compatibility analysis.

## Consequences

- The prior global prose order is replaced by one machine-readable staged table with explicit artifact and access-budget columns.
- Generated decoders, Rust enums, documentation, and fixtures share the same source.
- Compound-invalid fixtures are required for every adjacent precedence edge and security-sensitive non-adjacent pair.
- Some existing numeric codes may remain out of numeric order because safe access order controls precedence.

## Security and data impact

The recommendation prevents decoders from reading unvalidated locations merely to discover a nominally earlier error. Deterministic failures improve recovery and cross-architecture evidence without weakening rejection.

## Compatibility and migration

This decision revises the unmerged v1 draft. After freeze, changing precedence or code meaning requires a new major validation contract; old and new decoders may coexist only with version-selected corpora.

## Validation

- One staged table is the source for prose, decoder stubs, and test expectations.
- Every single predicate has a focused fixture.
- Every precedence edge has a dual-fault fixture proving the earlier result.
- Provider and effect-sink instrumentation proves no predicate reads outside its declared access budget and no rejected case commits a side effect.
- x86-64, AArch64, and the host reference oracle return identical codes.

## Replacement path

Any decoder may be replaced when it consumes the same table and passes all single-fault, dual-fault, boundary, and access-budget tests.
