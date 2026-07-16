# Documentation and Decision Policy

Status: Gate 0 approved direction — 2026-07-16

## Definition of documented

Every accepted subsystem includes purpose, non-goals, architecture, public interfaces, persistent formats, security/privacy assumptions, resource behavior, update/migration/rollback, failure/recovery, examples, tests, limitations, and replacement path.

## Document classes

- Constitution and glossary
- Product/release and tier specifications
- Architecture and subsystem specifications
- Threat models and security protocols
- RID/API and format references generated from source
- Architecture Decision Records
- Developer, application, driver, recovery, and porting guides
- Tutorials and executable examples
- Test plans, release evidence, limitations, and migration notes

## Change rules

- Public behavior changes update docs and tests in the same change.
- Persistent-format changes include migration and rollback notes.
- Security-boundary changes update the threat model.
- New dependencies update provenance and exception records.
- Examples compile/run in CI.
- Generated references identify their source and are never hand-edited.
- Documentation has an owner and review status.

## ADR template

Each important decision records status, context, decision drivers, considered options, selected choice, consequences, security/data impact, compatibility/migration, validation, and future replacement path.

Required initial ADRs cover architecture style, Rust/assembly, capability security, RID/RME, filesystem design, cryptographic protocols, virtual device model, tier model, and self-hosting toolchain. Gate 0 records these decisions in ADRs 0001–0010; combined decisions remain separate where their replacement paths differ.

## Review levels

- Draft: internally coherent but not approved.
- Reviewed: technically checked by the owning workstream.
- Approved: accepted as an implementation contract.
- Superseded: retained for history and linked to its replacement.

## Handoff rule

Implementation agents receive approved specifications plus explicit work ownership and tests. They do not convert unresolved prose into permanent interfaces without an ADR and architecture approval.
