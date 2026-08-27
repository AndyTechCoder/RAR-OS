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

## Sprint Alpha 0.1 rebaseline approval

Status: Approved

The owner explicitly approved the Sprint Alpha 0.1 roadmap overlay and the
cloud-only Development Lab trust-boundary change on 2026-08-20. ADR 0017 records
the decision. The approval permits automated target builds and isolated guest
execution only in the bounded repository-approved cloud Development Lab. It
does not permit local target compilation, linking, image creation, firmware
loading, QEMU, emulator, VM, or guest execution on the owner's Mac.

The approval preserves the long-term Releases 0–7 plan, target isolation,
signing, rollback, recovery, and data-separation commitments. Production
one-shot authorization, physical hardware, production cloud authority, guest
networking, host integration, passthrough, raw devices, elevated execution,
and unrelated external access remain unauthorized.

Sprint approval: approved
Sprint approver: Andy / RAR project owner
Rebaseline date: 2026-08-20

## End-of-week demonstrator approval

Status: Approved

On 2026-08-25, the owner directed the sprint to deliver a working bootable GUI
version by the end of the week while retaining the previously approved RAR OS
requirements and long-term expansion plan. ADR 0018 narrows Alpha completeness
to an authentic, minimal vertical demonstration without converting experimental
later-release prototypes into production-complete claims.

The owner also clarified that GitHub is the durable source of truth; SSD
repository/worktree data is temporary working state. The exact RAR OS subtree
may be used, but unrelated, irreplaceable SSD data must not be read or changed.

Sprint adjustment: approved
Approver: Andy / RAR project owner
Adjustment date: 2026-08-25

## Sprint Alpha architecture decisions

Status: Approved

On 2026-08-26, after receiving plain-language explanations, the owner approved
Alternative C for ADR 0020 and Alternative C for ADR 0021. ADR 0020 isolates
host-only cryptographic references from untrusted target builds in a separate
cloud role; production cloud-service wiring remains later work. ADR 0021 uses
the authentic Alpha boot chain Root → Recovery → Nucleus with an explicitly
replaceable FAT/ELF bootstrap boundary.

ADR 0020 approval: approved
ADR 0020 approver: Andy / RAR project owner
ADR 0020 date: 2026-08-26
ADR 0020 decision: Alternative C
ADR 0021 approval: approved
ADR 0021 approver: Andy / RAR project owner
ADR 0021 date: 2026-08-26
ADR 0021 decision: Alternative C
