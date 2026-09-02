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
- [Sprint Alpha accepted-evidence publication/recovery task packet](tasks/sprint-alpha-accepted-evidence-publication.md)
- [Sprint Alpha boot/platform contract integration task packet](tasks/sprint-alpha-boot-platform-contract-integration.md)
- [Sprint Alpha compact PCI BDF integration task packet](tasks/sprint-alpha-compact-bdf-integration.md)
- [Sprint Alpha ADR 0024 controller/helper integration packet](tasks/sprint-alpha-controller-helper-integration.md)
- [Sprint Alpha ADR 0024 C1 contract-closure packet](tasks/sprint-alpha-controller-helper-c1-contracts.md)
- [Sprint Alpha ADR 0024 C2 observer-discovery packet](tasks/sprint-alpha-controller-helper-c2-observer.md)
- [Sprint Alpha ADR 0024 C3V exact-set verification packet](tasks/sprint-alpha-controller-helper-c3v-verifier.md)
- [Initial Codex Handoff Prompt](handoff-prompt.md)
- [V1 Alpha Codex Execution Runbook](v1-alpha-execution.md)
- [GitHub Actions account-unblock runbook](runbooks/github-actions-account-unblock.md)
- [Sprint Alpha 0.1](sprint-alpha.md)
- [Sprint Alpha Dashboard](sprint-alpha-dashboard.md)
- [Security Remediation Status](security-remediation-status.md)
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
- [ADR 0022: Alpha Graphics and Input Authority](adr/0022-alpha-graphics-input-authority.md)
- [ADR 0023: Alpha Boot Determinism and Entry State](adr/0023-alpha-boot-determinism-and-entry-state.md)
- [ADR 0024: Alpha Controller Helper Build Trust](adr/0024-alpha-controller-helper-build-trust.md)
- [ADR 0025: Alpha Pre-GUI Evidence Input and Continuity Sequencing](adr/0025-alpha-gui-continuity-evidence-sequencing.md)
- [ADR 0026: Alpha Platform Payload and State Sources](adr/0026-alpha-platform-payload-and-state-sources.md)
- [ADR 0027: Alpha Bootstrap Retirement and DMA Closure](adr/0027-alpha-bootstrap-retirement-and-dma-closure.md)
- [ADR 0028: Alpha Artifact and Service Identities](adr/0028-alpha-artifact-and-service-identities.md)
- [ADR 0029: Alpha State Ticket Lifecycle](adr/0029-alpha-state-ticket-lifecycle.md)
- [ADR 0031: Alpha Compact PCI BDF Encoding](adr/0031-alpha-compact-pci-bdf-encoding.md)

Gate 0 approval covers ADRs 0001–0016 and the Release 0 task packet. Later
indexed ADRs are authoritative only through their separately recorded approval
dates. No approval prevents ADR-governed evolution or makes an implementation
permanently irreplaceable.

## Sprint Alpha decision history and integration

- [Recorded Alpha owner choices](proposals/alpha-owner-choice-brief.md)
- [Historical proposal 0022](proposals/0022-alpha-graphics-input-authority.md)
- [Historical proposal 0023](proposals/0023-alpha-boot-determinism-and-entry-state.md)
- [Historical proposal 0024](proposals/0024-alpha-controller-helper-build-trust.md)
- [Historical proposal 0025](proposals/0025-alpha-gui-continuity-evidence-sequencing.md)
- [Historical proposal 0026](proposals/0026-alpha-platform-payload-and-state-sources.md)
- [Historical proposal 0027](proposals/0027-alpha-bootstrap-retirement-and-dma-closure.md)
- [Historical proposal 0028](proposals/0028-alpha-artifact-and-service-identities.md)
- [Historical proposal 0029](proposals/0029-alpha-state-ticket-lifecycle.md)
- [Historical boot follow-up choice brief](proposals/alpha-boot-followup-choice-brief.md)
- [Open proposal 0030: Alpha Accepted-Evidence Publication and Recovery](proposals/0030-alpha-accepted-evidence-publication-recovery.md)
- [Historical proposal 0031](proposals/0031-alpha-compact-pci-bdf-encoding.md)
- [Alpha Decision Integration Plan](proposals/alpha-decision-integration-plan.md)
- [Sprint Alpha Completion Evidence Map](tasks/sprint-alpha-completion-evidence-map.md)
