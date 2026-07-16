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
- [Initial Codex Handoff Prompt](handoff-prompt.md)
- [V1 Alpha Codex Execution Runbook](v1-alpha-execution.md)
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

Gate 0 approval makes the indexed direction and the Release 0 task packet implementation contracts. It does not approve later release-specific interfaces early, prevent ADR-governed evolution, or make a technical implementation permanently irreplaceable.
