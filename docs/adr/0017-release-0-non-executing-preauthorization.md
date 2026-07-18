# ADR 0017: Release 0 Non-Executing Pre-Authorization Phase

Status: Accepted — 2026-07-18

## Context

Prompt 7 could not audit an exact first-boot closure because no static artifact, complete emulator/firmware pin, immutable disk binding, or production authority design existed. Constructing those inputs does not require guest execution.

## Decision drivers

- Preserve the separate owner authorization for first execution.
- Produce reviewable exact bytes and hashes before asking for execution authority.
- Avoid implementing later Release 0 mechanisms before platform evidence exists.

## Considered options

- **Authorize a first boot before construction:** rejected because the closure would be incomplete.
- **Keep all construction after first authorization:** rejected because static construction itself needs no execution authority.
- **Add a non-executing Prompt 7A phase:** selected.

## Decision

Prompt 7A may acquire and verify pinned host inputs, statically build one x86-64 R0-002-compatible candidate twice, prepare certification records, and implement host-control schemas and refusal/test-double routes. No target, QEMU, firmware, emulator, VM, guest, device, or AWS authority executes or is accessed. Prompt 7 is rerun afterward and remains the only path to precise first-boot authorization.

## Consequences

Prompt 7 can review immutable evidence rather than plans. Other architecture profiles remain uncertified. Prompt 8 remains blocked.

## Security and data impact

The phase adds no execution or device authority. Outputs remain repository-confined and disposable.

## Compatibility and migration

This rephases construction only; it does not change R0-002 or a target ABI.

## Validation

CI emits explicit `not-attempted` statements for every execution class and refuses all launch routes.

## Replacement path

Prompt 7A ends after a reviewed candidate closure exists. Later releases may replace its host tooling without changing target contracts.
