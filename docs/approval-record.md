# Gate 0 Owner Approval Record

Status: Approved

## Product commitments presented for approval

- RAR OS is a from-scratch operating system, not based on Linux or another existing OS.
- Work is delivered through staged releases while preserving the complete long-term vision.
- Four cumulative tiers are used: Micro, Device, Personal, and Compute; profiles remain separate.
- RAR Root, Recovery Seed, System Store, and Data Vault are isolated.
- Components, state, and interfaces are designed for replacement and complete rewrites.
- The normal experience is simple; deep inspection and developer replacement remain available.
- Virtual machines and RAR Lab are initial test environments, not the product target.
- Physical-device claims require later physical validation.
- This Mac is source/build storage only; RAR OS execution is restricted to separately owner-authorized certified VM profiles and never replaces or modifies macOS.
- The final branded design system is deferred; accessible provisional UI primitives remain required.
- Pal is prepared for through agent interfaces but is not part of Release 0.

## Technical recommendations presented for approval

- Capability-based hybrid microkernel.
- `no_std` Rust plus limited assembly for trusted target code.
- Stable RAR ABI and RID contracts with Rust and C SDKs.
- User-space drivers by default.
- Custom RAR filesystem and state system.
- RAR-owned implementations of external standards where practical.
- Established cryptography implemented and validated by RAR; no casual custom primitives.
- Native RAR applications first; optional compatibility layers only later.

## Approval effect

Approval authorizes Release 0 implementation to follow the listed specifications and accepted ADRs. It does not approve later release interfaces before their release-specific task packets are reviewed, and it does not make any implementation permanently irreplaceable.

Owner approval statement to record:

> I approve the Gate 0 product commitments and technical direction as the basis for Release 0 implementation. Material changes must follow the ADR and stop-condition process.

Approval: approved
Approver: Andy / RAR project owner
Date: 2026-07-16

Approval source: The owner explicitly approved proceeding after confirmation
that the repository handoff, execution runbook, model roles, review gates, and
host-safety policy were configured.

## Approved Gate 0 contract set

The approval applies to the foundation, architecture, safety, process, Release 0 task packet, and initial ADR documents listed in `docs/README.md` at repository publication. Their Gate 0 status approves the documented direction and Release 0 boundaries, not unreviewed later-release interface details.

ADRs 0009 and 0010 formalize the already-present approved commitments to four cumulative tiers with separate profiles and to staged self-hosting in Release 6. They do not add a tier, change a tier meaning, move a release commitment, or authorize target execution.

## Prompt 7A non-executing sequencing approval

Approval: approved
Approver: Andy / RAR project owner
Decision date: 2026-07-18

The owner approved a non-executing pre-authorization phase before Prompt 7 is rerun. It may statically compile and reproducibly hash one x86-64 R0-002-compatible candidate, acquire and verify the exact Debian snapshot closure inside repository-confined output, prepare certification records, and implement repository-side controls for a future external one-shot authority. The approved base is the Rust 1.95.0 OCI index digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`; the selected Debian packages are `lld-19=1:19.1.7-3+b1`, `qemu-system-x86=1:10.0.8+ds-0+deb13u1+b2`, and `ovmf=2025.02-8+deb13u1`, with their complete no-recommends closure.

The future authorization authority is an owner-approved AWS DynamoDB conditional-transition ledger with KMS signing and CloudTrail integrity evidence, authenticated from an owner-approved GitHub environment using short-lived OIDC. This approval covers schemas, state-machine logic, synthetic clients, and test doubles only. It does not authorize AWS provisioning or calls, credentials, QEMU or firmware execution, target execution, VM or guest launch, device access, macOS installation or modification, or first guest execution. Prompt 7 must be rerun with fresh architecture and security review, followed by a separate exact one-shot owner authorization, before Prompt 8 may execute anything.
