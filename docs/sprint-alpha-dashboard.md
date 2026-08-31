# Sprint Alpha Dashboard

Status: Explanatory only — not authority or completion evidence

This is the short orientation view. The approved Sprint Alpha contract, active
task packet, accepted ADRs, GitHub checks, and retained evidence remain
authoritative.

## Current position

- Phase: pre-Milestone A boot/platform/controller/SSD gates; acceptance-v2
  publication/recovery is a separate pre-Milestone B gate.
- Preparation is tracked by the discrete authoritative gates below, not a
  percentage. Overall readiness remains blocked.
- Latest recorded implementation checkpoint:
  `65ae7aedd11298c8f15ed96cd94166e2afa03e2a`; Specifications run
  `33308569835` passed full validation and mutation tests.
- Status rebaseline evidence: docs-only PR #58 merged at
  `2765128406210040bc0f16de586e9ccc8d39a452`; exact-main Specifications
  run `33309132199` passed. Later documentation commits do not invalidate
  the recorded implementation checkpoint.
- Target implementation: 0 of 7 milestones (A–G).
- Working bootable GUI: does not exist yet.
- Sprint Alpha completion evidence: none.
- V1 alpha (Releases 0–6): not complete and much larger than Sprint Alpha.

Preparation documents and merged scaffolding are not earned OS functionality
and cannot increase the implementation count.

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
- Acceptance-v2 controller preparation, gate-report-v2, their adversarial policy
  tests, independent reviews, merges, and exact-main validation are complete.
- Proposed ADR 0030 precisely records the remaining accepted-evidence
  publication/recovery choice while granting no authority until owner approval.
- Accepted ADR 0031 selects the experimental compact PCI BDF encoding for P0.
  D0 merged at `ce206f7`; exact-main run `33415110465` passed. The integrated
  P0 candidate binds the explicit formula, vectors, 136-byte preimage,
  unchanged closure digest, trusted reconstruction, and complete P0-B checker
  set. Immutable checkpoint `e450f323a6a35a138499a585aed575c0c62ad85b`
  remains in its ancestry, and exact-head run `33429540579` passed validation
  and mutation. Final reviews, merge, and exact-main validation remain.
- Accepted ADRs 0027–0029 select the reviewed B/A/B boot-retirement, identity,
  and state-authority directions; dependent contracts remain blocked until the
  decision integration merges and exact-main validates.
- Their byte-pinned integration packet is merged through PR #55 and separates
  decision, contract, source, pre-build, and post-build launch gates. It grants
  no implementation or execution authority before those gates pass.
- PR #57 adds a byte-pinned, inactive source observer for a future candidate
  Linux compiler-closure manifest. It is not wired to automation, cannot
  compile or execute the helper or target, and leaves every lock, inventory,
  controller, and readiness field blocked.
- The dependency-free accepted-evidence record codec, distinct A/F fixtures,
  static byte boundary, and non-authoritative post-approval work packet are
  merged, independently reviewed, and exact-main validated. They contain no
  publication/recovery or activation authority.
- A recorded five-choice owner brief, gated integration plan, and eight-item
  completion-proof map prevent later tasks from improvising or overclaiming.

## What is not ready

- No Alpha boot, Nucleus runtime, component system, recovery service, GUI, app,
  signed update path, target image, or retained target evidence exists.
- ADRs 0022–0026 are accepted, but none of their dependent contracts or
  controller changes is ready merely from acceptance.
- The exact Alpha boot/platform contracts are not yet authoritative. Their
  integrated P0 candidate has passed its immutable-checkpoint, T0/P0-B,
  exact-head validation, and mutation gates; final reviews, merge, and
  exact-main validation remain.
- The P0 compact PCI BDF encoding is decided by accepted ADR 0031, and its
  candidate fixture/contract bytes and opaque checker binding are integrated,
  but remain non-authoritative until P0 merges and exact-main validates.
- The phase-8 accepted-evidence writer is not ready. Only its pure record codec
  is merged; the preserved publication scaffold remains unmerged, and naming,
  journal, cleanup, recovery, retry, semantic-verification, and activation
  behavior remain blocked on owner selection and reviewed contract integration
  of proposed ADR 0030.
- Gate-report v2 therefore reports the platform envelope, Core bootstrap,
  component bundle, initial system source, initial preserved source, and
  fixture manifest as missing; Milestones A and B remain blocked.
- The Lab/controller/helper lacks real activating identities and reviewed cloud
  build/test evidence. The inactive observer has not run; no candidate compiler
  closure exists, and its tools, complete set, verifier, and retained output
  still require separate review before any helper build or test.
- PR #13 is merged at `abd75bfccf4fcd2f197871225d1a78233c0d87dc`;
  exact post-merge trusted-controller run `33294557261` passed. This closes only
  the controller bootstrap, not target implementation or Sprint completion.
- The SSD profile is installed without replacing existing user settings. A
  fresh task still must retain effective confinement evidence before A.

## Gates before code starts

Milestone A target files may be created only after all of these are true:

1. Accepted ADR 0023/0026/0027–0029 contracts are bound by the exact P0
   contract-set manifest, reviewed, merged, and exact-main validated. The
   machine profile remains separately blocked until retained cloud evidence
   exactly matches its firmware, q35, PCI, and AHCI inventory.
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
