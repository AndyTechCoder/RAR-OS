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

Sprint Alpha 0.1 is the owner-approved exception recorded by ADRs 0017 and
0018. Its immediate release target is the bounded end-of-week demonstrator in
`docs/sprint-alpha.md`. Implement only the minimum authentic vertical slice
needed for that acceptance demonstration; do not mistake prototype breadth for
stable completion of later release gates.

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
- Never compile, link, object-copy, package, or image RAR target code on the
  Mac. Direct compiler/linker/image commands are forbidden; reviewed local
  scripts are limited to host-only policy, specification, lint, and refusal
  tests. Target work occurs only in the approved cloud Development Lab.
- Preserve user-authored and unrelated changes in the workspace.
- The SSD is shared with irreplaceable owner data. Before and after every local
  write-producing phase, run `tools/ci/check-workspace-budget.sh`; stop below
  10 GiB SSD free, above 8 GiB total RAR OS workspace, or above 512 MiB combined
  repository `out/` data. Local scripts must set a per-file or aggregate bound;
  never create unbounded logs, caches, artifacts, downloads, or build outputs.

## Sprint workspace and continuity rules

- GitHub `AndyTechCoder/RAR-OS` is the durable source of truth.
- On the owner's Mac, RAR OS work is confined to
  `/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha`. The repository clone,
  worktrees, scratch space, and artifacts there are disposable working state,
  not the only copy of accepted work.
- Never read, enumerate, modify, move, permission-change, or delete any sibling
  or parent content on `/Volumes/Z Slim`. Never run cleanup against the volume,
  `Andy’s folder`, or `Codex`; operate only on an exact RAR OS path.
- Current owner directive: delete nothing in the RAR OS workspace.
- No-deletion scope: files, directories, scratch, artifacts, and worktrees.
- No-overwrite scope: moving or copying over an existing path is forbidden.
- Duration: this remains in force after merge until explicitly lifted by the owner.
- Future removal rule: after an explicit lift, only one exact registered worktree may be removed after clean pushed commits, exact remote merge verification, and separate review.
- The only combined local gate approved under this directive is `/bin/sh
  tools/ci/check-local-readonly.sh`; it creates no scratch and runs no mutation,
  compiler, linker, image, container, emulator, or target operation. Its exact
  wrapper and sole executable policy dependency are digest-bound in CI.
- Run one writer at a time. Reviews are read-only. Do not create a persistent
  goal or heartbeat that retries after the task has already reported a blocker.
- After the rebaseline, keep the end-of-week implementation in one vertical
  branch, worktree, draft PR, and task. Checkpoint milestones there instead of
  creating a new task or PR for every subsystem.
- Batch related fixes. Retry an ordinary failure at most twice after diagnosis.
  On the third occurrence, record one concise blocker and stop without polling.
- Before implementation, verify the SSD mount/path, clean Git state, canonical
  remote, pushed checkpoint, usable internal-disk headroom, and a GitHub Actions
  run that actually starts. A check with zero steps is infrastructure-blocked.
- The repository-selected `rar-os-ssd` permission profile must resolve from the
  separately owner-installed, reviewed user-level definition. A missing profile
  is a blocker; never fall back to a legacy or broader sandbox mode.
- Never force-push, use force-with-lease, delete or move a published checkpoint
  tag, rebase a published sprint branch, or discard a pushed checkpoint. Each
  A–G checkpoint uses the immutable tag defined by
  `docs/tasks/sprint-alpha-vertical.md` and is remotely verified before the next
  milestone starts.

## Stop and escalate

Stop before changing a constitutional principle, trust boundary, persistent-data promise, tier meaning, dependency policy, public format, native application model, or release commitment. Propose an ADR with alternatives and consequences.

## Completion report

Report owned paths, behavior, tests, evidence commands, documentation, unsafe/security review, limitations, and remaining risks.
