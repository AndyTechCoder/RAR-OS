# Sprint Alpha Dashboard

Status: Explanatory only — not authority or completion evidence

This is the short orientation view. The approved Sprint Alpha contract, active
task packet, accepted ADRs, GitHub checks, and retained evidence remain
authoritative.

## Current position

- Phase: pre-Milestone A rebaseline and contract preparation.
- Preparation orientation: approximately 95% complete.
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
- Proposed ADRs 0022–0026 explain the remaining boot, controller, evidence,
  payload/state, and graphics/input decisions.
- A five-choice owner brief, conditional integration plan, and eight-item
  completion-proof map prevent later tasks from improvising or overclaiming.

## What is not ready

- No Alpha boot, Nucleus runtime, component system, recovery service, GUI, app,
  signed update path, target image, or retained target evidence exists.
- No proposed ADR is accepted.
- The exact Alpha boot/platform contracts are not ready.
- The Lab/controller/helper lacks real activating identities and reviewed cloud
  build/test evidence.
- PR #7 is still draft. Runs `33266007613` and `33266499096` passed the complete
  primary validation phase. The latter proved that its final cloud-only test
  could not execute a tiny mock from ephemeral `/tmp`. The independently clean
  successor substitutes only the copied fixture's computed system value; run
  `33267022811` proved that test now passes and exposed a conflicting `sed`
  delimiter in the next portable-stat negative test. The delimiter-only third
  correction awaits exact-head verification; no green or implementation claim
  is made yet.
- The SSD confinement/profile proof is not recorded as passing for the future
  implementation task. SSD reserve and exact-root capacity evidence remain
  required; internal-Mac free space is not an Alpha workspace precondition.

## Gates before code starts

Milestone A target files may be created only after all of these are true:

1. Exact owner decisions for ADRs 0023, 0024, and 0026 are recorded; their
   selected contracts are implemented, reviewed, and ready.
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

ADR 0025 and the reviewed evidence-protocol cutover are additionally required
before B. ADR 0022 and the reviewed peripheral contract are required before E.
Delegation of safe operational choices does not replace the exact owner ADR
approval required by the authoritative packet.

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
