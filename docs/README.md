# RAR OS Specifications

Status: Gate 0 approved on 2026-07-16

## Foundation

- [Constitution](constitution.md)
- [Glossary](glossary.md)
- [From-Scratch and Dependency Policy](from-scratch-policy.md)
- [Replaceability](replaceability.md)
- [Simplicity Principles](simplicity-principles.md)

## Product and architecture

- [Release Roadmap](release-roadmap.md)
- [Tiers and Profiles](tiers-and-profiles.md)
- [System Architecture](architecture.md)
- [Security and Recovery](security-and-recovery.md)
- [Interfaces and Formats](interfaces-and-formats.md)
- [RAR Lab](rar-lab.md)
- [Host Mac Safety Policy](host-safety.md)

## Process

- [Documentation Policy](documentation-policy.md)
- [Implementation Handoff](handoff.md)
- [Gate 0 Approval Record](approval-record.md)
- [Initial Publication Record](publication-record.md)
- [Release 0 Task Packets](tasks/release-0.md)
- [Sprint Alpha Vertical Implementation Packet](tasks/sprint-alpha-vertical.md)
- [Sprint Alpha Milestone A execution map](tasks/sprint-alpha-milestone-a-execution-map.md)
- [Sprint Alpha Milestones B–G execution and evidence map](tasks/sprint-alpha-milestones-b-g-execution-map.md)
- [Initial Codex Handoff Prompt](handoff-prompt.md)
- [V1 Alpha Codex Execution Runbook](v1-alpha-execution.md)
- [Sprint Alpha 0.1](sprint-alpha.md)
- [Durable Sprint Status](../SPRINT_STATUS.md)
- [Project Backlog](../BACKLOG.md)

## Architecture decisions

- [ADR 0001: Staged Releases](adr/0001-staged-releases.md)
- [ADR 0002: Hybrid Microkernel](adr/0002-capability-kernel.md)
- [ADR 0003: Rust, Assembly, and ABI](adr/0003-rust-assembly-abi.md)
- [ADR 0004: Components, Capabilities, and RID](adr/0004-components-capabilities-rid.md)
- [ADR 0005: RAR Formats](adr/0005-rar-formats.md)
- [ADR 0006: Storage and Recovery](adr/0006-storage-recovery.md)
- [ADR 0007: Cryptography](adr/0007-cryptography.md)
- [ADR 0008: RAR Lab and Hardware](adr/0008-rar-lab-hardware.md)
- [ADR 0009: Cumulative Tiers and Profiles](adr/0009-cumulative-tiers-and-profiles.md)
- [ADR 0010: Staged Self-Hosting Toolchain](adr/0010-staged-self-hosting-toolchain.md)
- [ADR 0011: Release 0 Reproducibility Gate Phasing](adr/0011-release-0-reproducibility-gate-phasing.md)
- [ADR 0012: Release 0 Host Bootstrap Trust and Snapshot](adr/0012-release-0-host-bootstrap-trust-and-snapshot.md)
- [ADR 0013: Pre-Copy Trust Boundary and MMIO Authority](adr/0013-pre-copy-trust-and-mmio-authority.md)
- [ADR 0014: Hardware Binding and Record Identity](adr/0014-hardware-binding-and-record-identity.md)
- [ADR 0015: Deterministic Validation Precedence](adr/0015-deterministic-validation-precedence.md)
- [ADR 0016: Release 0 Entry Validation and Authority Closure](adr/0016-release-0-entry-validation-and-authority-closure.md)
- [ADR 0017: Sprint Alpha 0.1 and Cloud Development Lab](adr/0017-sprint-alpha-development-lab.md)
- [ADR 0018: End-of-Week Alpha Demonstrator](adr/0018-end-of-week-demonstrator.md)
- [ADR 0019: Alpha Layer Signing Profile](adr/0019-alpha-layer-signing.md)
- [ADR 0020: Alpha Reference-Oracle Isolation](adr/0020-alpha-reference-oracle-isolation.md)
- [ADR 0021: Alpha Boot Payload and Handoff Boundary](adr/0021-alpha-boot-payload-boundary.md)

Gate 0 approval makes the indexed direction and the Release 0 task packet implementation contracts. It does not approve later release-specific interfaces early, prevent ADR-governed evolution, or make a technical implementation permanently irreplaceable.

## Open Alpha architecture decision

- [Plain-language Alpha owner choice brief](proposals/alpha-owner-choice-brief.md)
- [Proposed ADR 0022: Alpha Graphics and Input Authority](proposals/0022-alpha-graphics-input-authority.md)
- [Proposed ADR 0023: Alpha Boot Determinism and Entry State](proposals/0023-alpha-boot-determinism-and-entry-state.md)
- [Proposed ADR 0024: Alpha Controller Helper Build Trust](proposals/0024-alpha-controller-helper-build-trust.md)
- [Proposed ADR 0025: Alpha GUI Continuity Evidence Sequencing](proposals/0025-alpha-gui-continuity-evidence-sequencing.md)
