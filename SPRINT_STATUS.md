# Sprint Alpha 0.1 Status

- Date: 2026-08-26
- Current milestone: End-of-week rebaseline, before Milestone A target implementation
- Objective: merge the trusted cloud Development Lab controller, then implement
  and retain the authentic A–G bootable GUI vertical slice
- Worktree: `/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha/worktrees/sprint-alpha-rebaseline`
- Branch: `codex/sprint-alpha-rebaseline`
- Pull request: [#7](https://github.com/AndyTechCoder/RAR-OS/pull/7), draft
- Current outcome: the controller transition, decision-complete A–G packet,
  experimental signing contract, cumulative 45-observation acceptance protocol,
  two-build reproducibility check, isolated build/launch boundaries, frozen
  artifact, bounded SSD/static safeguards, RAR-owned QMP client source and fault
  tests, immutable Development Lab input pins, and role-separated draft image
  recipes are complete in the working diff
- Validation: `tools/ci/check-sprint-static.sh` passes locally without compiling
  or executing target or QMP code; Linux QMP compilation/fake-server tests and
  any future candidate provisioning remain deliberately unexecuted
- Independent review: final architecture, correctness, and security reviews are
  clean after remediation; Proposed ADR 0020 remains an intentional owner
  decision before any reference-oracle provisioning or Milestone F work
- Target functionality: 0%; no RAR OS target implementation has started, built,
  booted, or run
- Development Lab state: upstream image/tool inputs are `decision-blocked` and the
  QMP client is `source-ready`; provisioning remains absent, candidate images
  remain unbuilt, and the active
  Lab profile remains `blocked` until candidate evidence is reviewed, immutable
  output identities replace every unavailable pin, and Proposed ADR 0020
  resolves reference-oracle isolation
- External blockers: GitHub Actions billing/spending still produces zero-step
  failures; the Mac internal disk remains below the 10 GiB unattended-work
  threshold; and the reviewed `rar-os-ssd` user profile still needs one-time
  owner installation/evidence in a fresh SSD-root task
- Next durable action: retain this reviewed checkpoint in PR #7, then resolve
  ADR 0020 and the external Lab preconditions; do not retry zero-step Actions,
  merge, tag, activate the lab, or start Milestone A until every precondition is
  genuinely green
- Deadline: 2026-08-30 23:59 America/Los_Angeles
- Local safety: target compilation, image creation, firmware loading, QEMU,
  emulator, VM, guest execution, macOS modification, and access outside the
  dedicated RAR OS SSD subtree remain forbidden
