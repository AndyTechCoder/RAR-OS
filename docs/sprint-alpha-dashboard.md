# Sprint Alpha Dashboard

Status: Explanatory only — not authority or completion evidence

This is the short orientation view. The approved Sprint Alpha contract, active
task packet, accepted ADRs, GitHub checks, and retained evidence remain
authoritative.

## Current position

- Phase: rebaseline durable; remaining pre-Milestone A contracts and authority.
- Rebaseline preparation orientation: 100% complete.
- Target implementation: 0 of 7 milestones (A–G).
- Working bootable GUI: does not exist yet.
- Sprint Alpha completion evidence: none.
- V1 alpha (Releases 0–6): not complete and much larger than Sprint Alpha.

The percentage is only an orienteer for preparation work. It is not earned OS
functionality and cannot increase the implementation count.

## What is ready

- The A–G vertical objective, ownership, order, failure behavior, and evidence
  are mapped.
- The cloud-only host-safety boundary and source-only SSD boundary are defined.
- Controller, Lab, reproducibility, frozen-artifact, and evidence scaffolding
  exists in source form.
- Accepted ADRs 0022–0026 select the remaining boot, controller, evidence,
  payload/state, and graphics/input directions while activation stays gated.
- Gate-report schema v2 now classifies canonical ADR 0026 and the inactive
  acceptance-v2 preparation while failing closed on every unavailable private
  platform identity. It is orientation only, not readiness evidence.
- A recorded five-choice owner brief, gated integration plan, and eight-item
  completion-proof map prevent later tasks from improvising or overclaiming.

## What is not ready

- No Alpha boot, Nucleus runtime, component system, recovery service, GUI, app,
  signed update path, target image, or retained target evidence exists.
- ADRs 0022–0026 are accepted, but none of their dependent contracts or
  controller changes is ready merely from acceptance.
- The exact Alpha boot/platform contracts are not ready.
- Gate-report v2 therefore reports the platform envelope, Core bootstrap,
  component bundle, initial system source, initial preserved source, and
  fixture manifest as missing; Milestones A and B remain blocked.
- The Lab/controller/helper lacks real activating identities and reviewed cloud
  build/test evidence.
- PR #13 is merged at `abd75bfccf4fcd2f197871225d1a78233c0d87dc`;
  exact post-merge trusted-controller run `33294557261` passed. This closes only
  the controller bootstrap, not target implementation or Sprint completion.
- The SSD profile is installed without replacing existing user settings. A
  fresh task still must retain effective confinement evidence before A.

## Gates before code starts

Milestone A target files may be created only after all of these are true:

1. The selected ADR 0023/0026 contracts are implemented, reviewed, and ready.
2. The reviewed Lab profile, controller, compiler/helper identities, twice-
   reproduced helper evidence, and immutable cloud inputs are genuinely ready.
3. PR #7 and every required real-step workflow are green, reviewed, merged to
   `main`, and remotely verified.
4. The exact SSD permission profile has retained one-time confinement evidence,
   the worktree is clean/pushed, and all local/remote capacity and identity
   preflights pass.
5. A fresh SSD worktree and `codex/sprint-alpha-vertical` branch are created
   from verified `main` for one Medium-effort writer task with no persistent
   goal or inherited diff.

The reviewed ADR 0025 evidence-protocol cutover is additionally required before
B. The reviewed ADR 0022 peripheral contract is required before E.

## Execution path

`A boot → B Nucleus → C components/IPC → D recovery → E GUI/apps → F signed updates → G retained proof`

- The first authentic boot appears at A.
- The first visible interactive GUI and native apps appear at E.
- The requested working Sprint Alpha is complete only at G, after one retained
  clean exact-head run proves all eight completion items.

No milestone can be skipped, replaced by a host mock, or declared complete from
markers/screenshots alone.

## Safety boundary

The Mac and SSD hold source only. RAR target compilation, linking, image
creation, firmware loading, QEMU/emulator/VM launch, and guest execution remain
forbidden locally. Nothing in this dashboard grants cloud credentials, target
execution, broader SSD access, deletion, merge, or owner approval.

## Where to verify details

- Current detailed state: `../SPRINT_STATUS.md`
- Completion contract: `sprint-alpha.md`
- Authoritative task packet: `tasks/sprint-alpha-vertical.md`
- Decision integration: `proposals/alpha-decision-integration-plan.md`
- Completion proof map: `tasks/sprint-alpha-completion-evidence-map.md`
- Current gate orientation: run
  `/bin/sh tools/ci/report-sprint-alpha-gates-v2.sh`; validate its fail-closed
  policy with `/bin/sh tools/ci/check-sprint-alpha-gate-report-v2-policy.sh`.
