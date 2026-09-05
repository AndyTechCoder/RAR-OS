# Implementation Handoff

## Current owner-directed milestone — Modern Architecture

Milestone 3 was published as [v0.3.0-usable-alpha](https://github.com/AndyTechCoder/RAR-OS/releases/tag/v0.3.0-usable-alpha)
at `06ecaaad61ab40f4c90ee73df85ee3493c89ccc1`. The owner now directs Milestone 4.
Use `docs/tasks/fast-track-alpha-milestone-4.md`; ADR0034 is proposed, not yet
accepted. No new persistent cloud profile or target behavior is active yet.
Preserve the published Desktop and cloud-only/no-local-write constraints.

## Historical Milestone 3 handoff

Milestone 3 Usable Graphical Alpha is now active under
`docs/tasks/fast-track-alpha-milestone-3.md`, ADR0032 and the reviewed ADR0033
composition. Platform was published as `v0.2.0-platform-alpha` at
`61916ca3a17c2f7013205c91afb3117dbb03cbc0`; Foundation remains preserved.
Earlier current-scope wording below is historical. Desktop-v0 implementation
and limitations are in docs/desktop-implementation.md. Completion is recorded by
v0.3.0-usable-alpha only after actual cloud interaction and all release gates pass;
before that publication the implementation remains a candidate.


## Active Fast-Track Alpha handoff

ADR 0032 is the active process authority. Work proceeds by evidence milestones,
not prompt numbers or nested authorization packets. The current handoff is
[`Fast-Track Alpha Milestone 2`](tasks/fast-track-alpha-milestone-2.md).
Platform implementation and independent remediation are demonstrated in
[`the Platform evidence record`](evidence/platform-milestone-2.md). Its final-head,
merge, exact-main and publication gates define release promotion. Foundation
remains the preserved regression baseline. Usable Alpha is next, not yet started.
Keep public contracts replaceable, update tests and documentation with behavior,
and retain all security, signing, isolation, recovery, rollback, dependency and
data-separation requirements.

Status: Gate 0 approved direction — 2026-07-16

## Repository structure

Use one monorepo:

- `docs/`: constitution, specifications, ADRs, guides, and release evidence
- `spec/`: RID, formats, protocols, hardware contracts, and conformance fixtures
- `nucleus/`: portable Nucleus plus architecture ports
- `core/`: universal component, capability, identity, update, and recovery services
- `services/`: drivers, storage, networking, graphics, user, and system services
- `apps/`: shell and first-party system applications
- `sdk/`: Rust/C SDKs, examples, and future language integration
- `tools/`: build, package, signing, debug, inspection, and RAR Lab tools
- `tests/`: cross-system, fault, security, compatibility, and release scenarios

Exact internal directories may evolve, but ownership boundaries follow subsystem contracts.

## Workstreams

1. Architecture/specifications
2. Build/bootstrap/reproducibility
3. x86-64 platform
4. ARM64 and Tier 0 platforms
5. Nucleus and IPC
6. Component fabric/capabilities/identity
7. Security/Vault/signing/recovery
8. Storage/state/packages/updates
9. Hardware/drivers/power
10. Networking/Wi-Fi/device mesh
11. Graphics/input/accessibility
12. Applications/Rust+C SDKs
13. Continuity/agents
14. RAR Lab/testing/documentation

The coordinator schedules these in release-gate waves; parallel work uses approved mocks generated from RID.

## Change authority

- Workstreams own implementations, not shared contracts.
- Architecture/spec owns stable specifications after review.
- A shared-contract change requires an ADR, affected-owner review, migration plan, and conformance update.
- Security-sensitive changes require security review.
- Data-format changes require storage/recovery review.
- No agent may weaken a gate to make its implementation pass.

## Task packet

Every implementation task states objective, approved specifications, in/out of scope, owned paths, dependencies, public interfaces, failure cases, required tests, documentation, acceptance evidence, and unresolved risks.

## Completion evidence

- Reproducible commands and tool versions
- Build/test/conformance output
- Fault and negative tests
- Documentation and examples
- Security and unsafe-code review
- Performance measurements where relevant
- Migration/rollback demonstration
- Known limitations and next risks

## Integration gates

Follow Releases 0–7 in `release-roadmap.md`. A gate passes only when every required architecture target builds, conformance and fault suites pass, documentation is current, and the previous known-good system remains reproducible.

## Initial handoff sequence

1. Approve Gate 0 documents and create initial ADRs.
2. Scaffold monorepo and host toolchain lock.
3. Freeze boot, hardware-description, Nucleus, capability, IPC, RID, and trace contracts for Release 0.
4. Assign platform and Nucleus workstreams in parallel.
5. Integrate only through the pinned VM matrix and evidence requirements.

## Stop conditions

Stop and request owner/architecture direction when work would change a constitutional principle, trust boundary, persistent user-data promise, tier meaning, native application model, dependency policy, or release commitment.
