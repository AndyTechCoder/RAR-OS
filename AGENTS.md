# RAR OS Agent Instructions

## Before changing code

Read, in order:

1. `docs/constitution.md`
2. `docs/from-scratch-policy.md`
3. `docs/release-roadmap.md`
4. `docs/architecture.md`
5. `docs/security-and-recovery.md`
6. `docs/interfaces-and-formats.md`
7. `docs/handoff.md`
8. The active release task packet
9. `docs/host-safety.md`
10. `docs/v1-alpha-execution.md`

## Current scope

No OS implementation exists. The first implementation handoff is Release 0 in `docs/tasks/release-0.md`, and it begins only after `docs/approval-record.md` records owner approval.

Do not implement later-release filesystems, networking, GUI, agents, package systems, or applications during Release 0.

The physical Mac is source/build storage only. Never execute RAR OS natively or modify macOS. Before explicit owner authorization of the first certified VM boot, do not execute RAR target code even in a VM. Follow `docs/host-safety.md` without exception.

Routine repository work is intentionally autonomous. `.codex/config.toml` permits automatic review of Git/GitHub and repository-confined operations while denying effects outside this repository. Do not request manual approval merely for ordinary repository edits, tests, commits, pushes, PR readiness changes, or evidence-gated merges.

The owner authorized automatic merging on 2026-07-16. Merge only at the final
review/remediation or release-gate step for a change, after all required tests,
independent reviews, acceptance evidence, and documentation pass with no
blocking findings. Never merge an implementation PR early merely because its
author believes it is ready. Stop if GitHub reports conflicts, failing required
checks, missing evidence, or unresolved review findings.

Use the specialist roles in `.codex/agents/` as assigned by the execution
runbook. Parallel writers must have disjoint owned paths; reviewers remain
read-only and must not approve their own implementation.

## Mandatory rules

- RAR OS is not built on Linux or another OS.
- No third-party code may be linked into target images without an approved Dependency Exception Record.
- Host tools are allowed only under the from-scratch policy and must be pinned.
- Public contracts come from approved specifications; do not invent stable formats inside implementation code.
- Update documentation and tests in the same change as public behavior.
- Preserve replaceability: no undocumented cross-subsystem internals.
- Unsafe Rust and assembly require documented invariants and focused tests.
- Do not weaken isolation, validation, signing, rollback, or acceptance tests to make a milestone pass.
- Never use raw host disks, boot/firmware changes, system extensions, physical-device passthrough, direct QEMU launch, elevated VM execution, or unapproved VM networking.
- Preserve user-authored and unrelated changes in the workspace.

## Stop and escalate

Stop before changing a constitutional principle, trust boundary, persistent-data promise, tier meaning, dependency policy, public format, native application model, or release commitment. Propose an ADR with alternatives and consequences.

## Completion report

Report owned paths, behavior, tests, evidence commands, documentation, unsafe/security review, limitations, and remaining risks.
