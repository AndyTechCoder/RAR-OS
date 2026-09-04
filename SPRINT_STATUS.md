# Sprint Alpha 0.1 Status

## Active Fast-Track status — 2026-09-04

- Current milestone: Fast-Track Alpha Milestone 1 — Foundation.
- Authority: ADR 0032 and `docs/tasks/fast-track-alpha-milestone-1.md`.
- Runtime evidence: source `42be166b38790f18cf5cb7d2d1b4632e06f87e33` passed cloud run `33923002436`:
  two byte-identical builds, 11 tests per build, 26 negative controller tests,
  normal readiness, deterministic panic and invalid-instruction containment.
- Delivery gate: final documentation/review/checks, PR #149 merge and release
  publication. See `docs/evidence/foundation-milestone-1.md` for exact proof.
- This is a serial-only foundation, not a graphical or production OS.
- Local safety: GitHub-only mutations; no local target build or execution.
- Historic status follows unchanged.

- Date: 2026-09-03
- Current milestone: post-P0 pre-Milestone A controller, machine-evidence, and
  SSD-confinement gates; acceptance-v2 publication/recovery remains a separate
  pre-Milestone B gate
- Objective: close the remaining controller/helper, retained-machine-evidence,
  and SSD-confinement gates before Milestone A, and separately finish
  acceptance-v2 publication/controller closure before Milestone B
- Status content baseline: reviewed C3VA source candidate at
  `51f639cbd1f071f637c4b13eca3c2cb9fb8880c3`, plus this status-only reconciliation
- Status rebaseline evidence: docs-only PR #58 merged at
  `2765128406210040bc0f16de586e9ccc8d39a452`; exact-main Specifications
  run `33309132199`, job `99250884902`, passed
- PR #58 status worktree (historical, preserved): `/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha/worktrees/alpha-status-after-closure`
- PR #58 status branch (historical, preserved): `codex/alpha-status-after-closure`
- Latest merged P0 contract validation: PR #79 merged at
  `acfd70c9646209f71376d0d03c20d68d2c610746`; exact-main Specifications run
  `33436244747`, job `99633274359`, passed runner attestation,
  trusted-controller authority binding, full pinned read-only-container
  validation, and all external mutation tests
- Bootstrap pull request: [#13](https://github.com/AndyTechCoder/RAR-OS/pull/13),
  merged at `abd75bfccf4fcd2f197871225d1a78233c0d87dc`; exact post-merge trusted-
  controller run `33294557261` passed
- Durability proof: exact-head `1f409393995c8e558d71eed34e3641daf2f21f3c`
  passed run `33268129412`; the distinct exact-merge `main` run `33268347971`,
  job `99142287272`, passed complete validation and mutation testing
- Published runner-attestation checkpoint: `9e4da4ca0b23a4392fa4d3318c11808e6f8bd307`.
  Run `33266007613` at that exact head passed runner attestation, portable-stat
  enforcement, specifications, immutable fixtures, Release 0 conformance,
  generated SDK checks, and host/bootstrap tests. Its final cloud-only mutation
  phase exposed a sequence of bounded test-harness portability defects. The
  reviewed repairs preserved the production Darwin boundary, kept source
  read-only, moved fixture output to attested ephemeral scratch, and replaced
  the undeclared `xxd` assumption with already-pinned byte-output tools. Runs
  `33266499096`, `33267022811`, `33267302085`, and `33267767640` isolated those
  defects in sequence; exact-head run `33268129412` proved the complete repaired
  suite, and exact-merge `main` run `33268347971` independently proved the merged
  result. Earlier published source/safety commits fix branch whitespace/hash
  bindings, strengthen
  no-deletion auto-review and command rules, bind the authoritative directive
  in `AGENTS.md` and host safety, add one digest-bound read-only local gate, and
  record the bounded Actions recovery sequence
- Local source-only validation: full-branch whitespace, tracked-shell syntax,
  and the bound host-policy checker pass via
  `/bin/sh tools/ci/check-local-readonly.sh`. The Alpha preimplementation
  contract structure also passed before the read-only wrapper was narrowed and
  digest-bound. No Alpha completion evidence is claimed. The mutation-capable
  specification suite remains cloud-only and is not claimed locally
- Independent review: architecture, correctness, and security reviews are clean
  for the proposed Alpha platform/state boundary after bootstrap, parser,
  immutable-source, and ownership remediation. Consolidated correctness review
  is clean for the decision-integration and completion-evidence maps. The
  accepted-evidence codec and its post-approval publication/recovery task packet
  each passed architecture, correctness, and security review. The boot/platform
  integration packet also passed clean architecture, correctness, and security
  re-review after its lifecycle and identity gates were made non-circular.
  The inactive compiler-closure observer source passed clean architecture,
  correctness, and security re-review after exact output, grammar, topology,
  generator, resource-bound, and trust-boundary remediation.
  These are source-only preparation reviews, not target implementation or
  completion proof
- Target functionality: 0%; no RAR OS target implementation has started, built,
  booted, or run
- Preparation is tracked by the discrete authoritative gates below, not a
  percentage. Overall readiness remains blocked
- Development Lab state: the three-role v2 field schemas and transcript contract
  are source-ready and reviewed in their inactive forms. Exact per-role mounts, bounded
  outputs, empty allowlisted environments, and no-extra-authority rules are
  bound and their policy tests pass. The exact v2 profile instance and
  single-pass validator are complete, fast, independently reviewed, and reject
  every activating identity or ready state. Provisioning is absent,
  candidate images remain unbuilt, the legacy v1 profile is permanently
  non-activating. The blocked nine-phase v2 controller plan now fixes role
  order, non-overlap, freeze, reference verification, launch evidence, and
  retention without containing runnable cloud commands. Its focused review is
  clean after binding the conditional reference verdict into final evidence and
  enforcing every phase row exactly. The bounded binary reference-evidence and
  strict controller-verdict contracts now have accepted/not-required fixtures
  and adversarial host-only validation. The owner confirmed that cloud
  infrastructure exists, but wiring and provisioning are intentionally deferred;
  this checkpoint performs neither. Real immutable output identities plus a
  reviewed runnable v2 controller are still required, and any future activating
  controller/profile change requires fresh independent review. The controller's
  host-only stop/open/copy/recheck handoff and 256-byte durable manifest are now
  byte-bound with 49 positive and adversarial cases. The dependency-free host
  core now implements the manifest codec, streaming SHA-256,
  typed phase plans, and a descriptor-operation-parametric transaction policy
  with attempt-local cleanup. It now defines thirteen focused source tests,
  including the structural attempt-codec round trips, and includes an
  independent 256-byte golden vector. Local gates inspect but do not compile or
  execute changed Rust; isolated cloud test evidence remains blocked on a
  reviewed pinned host compiler identity. A source-only x86-64 Linux adapter
  now consumes sealed, already-open root descriptors and confines all `unsafe`
  FFI to one documented module. It has no executable entry point, root-path
  resolver, process, network, cloud, external/release publication, launch, or
  autonomous root-acquisition authority. The eventual controller still requires a bounded outer
  watchdog and persistent attempt-root recovery before activation. The exact
  persistent-attempt contract is now source-complete: a bounded
  active record, hash-chained state transitions, outer watchdog states, and a
  durable recovery inventory cover forced termination and controller restart.
  It grants no helper spawn, process-FD protocol, path lookup, cloud command,
  or activation authority; its 97-case adversarial table and policy validator
  are static-only. A dependency-free, side-effect-free structural codec source
  now encodes and decodes the three record families with byte bounds, reserved-
  zero checks, local ordering, and record hashes. It intentionally contains no
  contextual chain, session-takeover, inventory-origin, or cleanup authority;
  those policy APIs and behavioral tests remain blocked on the reviewed
  isolated compiler identity. It performs no journal I/O, process or watchdog
  operation, cloud action, or activation. A separate dependency-free 20-line
  accepted-evidence codec with distinct A/F language-neutral fixtures is now
  merged, positively byte-pinned, and exact-main validated. It defines no
  filename, publication, recovery, cleanup, controller, or activation behavior
- Probe dispatch state: GitHub rejected superseded run `33092166312` before job
  creation because the workflow referenced `runner.temp` at job-level `env`,
  before the runner context exists. The published correction binds the path only
  at consuming steps and adds a static regression check. The correction is
  merged and covered by the complete green exact-head and exact-merge
  Specifications runs. The top-level Development Probe remains deliberately
  activation-blocked: when dispatched with the blocked v2 plan it validates the
  plan and stops with status 73 before target build, launch, or cloud-role
  effects. The superseded v1 two-role runner also stops before reading cloud
  context or issuing any container command; no default dispatch path can reach
  it after ADR 0020
- Controller-helper build trust: accepted ADR 0024 Alternative A selects two
  builds from a fully pinned compiler closure on the approved Linux runner for
  Alpha. The source-only C1 runtime, exhaustive verifier, closure-acceptance,
  and v1 evidence contracts merged through PR #101 and bounded follow-up repairs
  #102-#108. Exact-main Specifications run `33467136133` at
  `10537d55003949797d7ca8ab7adcf0b1deab5331` passed runner attestation,
  pinned read-only validation, and all 26 isolated mutation-policy tests.
  C1 is complete. Real compiler/helper identities and runtime evidence remain
  absent; no helper compilation, cloud provisioning, target execution, or Lab
  activation is authorized. The inactive observer source from PR #57 remains
  unwired and unexecuted, the CI closure lock remains `none`, and the helper
  inventory remains blocked. The authoritative literal-path C2 packet merged
  through PRs #109-#110, and exact-main Specifications run `33469515632` at
  `4137f3397590958490dff261ee617df5f40e7cad` passed validation and all 26
  isolated mutation-policy tests. C2A then merged through PRs #111-#112;
  exact-main Specifications run `33473648494` at
  `c862e27771b999e593ae26ea12ef424613b67bac` passed validation and all 27
  isolated mutation-policy tests. C2 completed at exact main
  `70a683dfb6dbde03f0f884ddc16ac2a2680a4f4f`; Specifications run
  `33608694457` and Observer run `33608694456` passed. C3V D0 and bounded
  corrections then merged through PRs #132-#135; exact-main Specifications
  run `33674068492` passed at
  `730520de48897a9b41412e0662c0a30d57a59f0c`. C3VA is the active
  source-only candidate in PR #136. Its exact code/spec head
  `2e67eec1c6a77250c3d4fc0f9a55163150db36d7` passed architecture,
  correctness, and security source review; documentation-only head
  `51f639cbd1f071f637c4b13eca3c2cb9fb8880c3` passed run `33798059680`
  and delta review. The PR is ready but unmerged because the automated
  safety reviewer requires a fresh owner acknowledgement before placing
  source on `main`; therefore distinct exact-main full validation and
  mutation evidence do not yet exist. C3VB one-shot verification and C3VR
  retirement remain separate gates, followed by C3A evidence acceptance.
  No verifier runtime, compiler, helper, target, Lab activation, acceptance,
  or readiness authority is granted
- Controller-helper inventory: an option-neutral blocked instance now binds the
  required builder, compiler closure, trusted source, golden vector, twice-
  reproduced binary, and isolated test-evidence identities. All activating
  values remain `unavailable`; 40 declarative cases and mutation tests prevent
  a ready claim before real reviewed cloud evidence.
  Strict contextual parsers now validate the future aggregate build record,
  two distinct controller-owned build receipts; the v1 validator now requires all
  97 inherited attempt cases plus 30 runtime/authority cases and rejects the legacy 13-case receipt
  against their actual selected inputs. They reject aliases, hardlinks when the
  host filesystem supports exercising them, path escape, stale logs, reused
  job/root nonces, missing/duplicate cases, and self-declared results without
  controller context. The source-attestation check passes in cloud CI without compiling or running
  the helper. Real validation and mutation evidence remain pending until the
  reviewed merge receives an exact-main run. Mutation-based policy tests skip
  locally before writing and require a dedicated pinned validation container with
  an independent clean exact-revision checkout mounted read-only and a bounded
  `nosuid,nodev` tmpfs as their only approved scratch; they are explicitly
  non-activating and are not helper build/test or cloud execution evidence
- Boot implementation state: accepted ADR 0021 selects the Alpha boot-volume,
  payload-loader, and Root-to-Recovery boundary. P0 now supplies the reviewed,
  merged, exact-main-validated private Alpha boot/platform contract set.
  Accepted ADR 0023 Alternative C selects the deterministic private Alpha
  profile; its retained machine evidence and other cumulative start gates are
  still absent, so no target implementation began
- Milestone A preparation: a non-authoritative execution map now decomposes the
  existing packet into eight ordered work packets, maps all 50 mandatory boot
  cases, fixes one-writer paths and stop conditions, and keeps every target
  build/boot/evidence action cloud-only. It changes no contract or readiness
  state and does not authorize implementation. PR #55 adds a byte-pinned,
  non-authoritative integration packet for then-proposed ADRs 0027–0029. It separates
  owner decision, contract, cumulative source-start, source-only merge,
  pre-build, and post-build launch/acceptance gates; exact-main run `33306580918`
  passed. Canonical ADRs 0027–0029 record B/A/B, and their P0 contracts are
  merged and exact-main validated. Target source remains blocked on the
  separate controller/helper, retained-machine-evidence, SSD-confinement, and
  fresh-worktree gates
- Milestones B–G preparation: one non-authoritative dependency/evidence map now
  organizes the existing sequential ownership, contract-before-code boundaries,
  all 40 later acceptance rows, cloud-only evidence, and stop conditions. It
  defines no interface and does not start or reorder implementation
- Acceptance-sequence defect: the fixed Milestone C plan requires
  `component:gui-responsive`, but A–C own no GUI/presentation implementation and
  Milestone E is the first graphics owner. No synthetic marker or hidden pre-E
  component is permitted; accepted ADR 0025's combined protocol-v2 correction
  is required before Milestone B starts
- Historical acceptance-input defect: v1 injects keyboard shortcuts to trigger
  B, C, and D, but input authority and implementation first exist at E. No
  hidden pre-E keyboard handler or marker without consumed input is permitted
- Accepted resolution: ADR 0025 Alternative B selects a new reviewed protocol version that
  auto-chains B–D through strict ordered `none` inputs, preserves the GUI-
  continuity row's exact post-crash position, and changes only its minimum from
  C to E. C would prove restart plus peer continuity; cumulative E–G probes
  would prove real GUI continuity. The exact v2 protocol, transcript, verifier,
  fixture, and anti-replay contracts are merged on `main` and
  reject historical v1 for every new A–G probe. The exact protocol, controller
  preparation, gate-report v2, policy tests, reviews, merges, and exact-main
  validation are complete through PRs #43–#49. The cutover is still inactive:
  the phase-8 durable publication/recovery writer and controller/profile wiring
  remain closed gates. Proposed ADR 0030 now documents the required persistent
  transaction choice without selecting or authorizing it; PR #50 merged and its
  exact-main validation passed. Proposals 0027–0029 were normalized as explicit
  B/A/B recommendations in PR #42 and are retained as history after the owner
  accepted those choices; acceptance grants no implementation authority. The
  independent record codec merged in PR #52,
  and the byte-pinned non-authoritative post-approval work packet merged in PR
  #53; exact-main Specifications runs `33304849966` and `33305373626` passed.
  The separate boot/platform integration packet merged in PR #55 at
  `514d5e72a162dc99794a3f1eee864c0a22f00738`; exact-main run `33306580918`
  passed. PR #56 rebaselined the status records, and PR #57 then merged the
  inactive compiler-closure observer source; exact-main run `33308569835`
  passed without activating it
- Owner decisions: ADRs 0022 C, 0023 C, 0024 A, 0025 B, and 0026 C are recorded
  as accepted on 2026-08-29; ADRs 0027 B, 0028 A, and 0029 B are recorded as
  accepted on 2026-08-30. Acceptance alone leaves every dependent contract,
  identity, evidence, controller, and readiness gate closed
- Decision integration readiness: one non-authoritative plan now sequences the
  accepted decisions into pre-A boot/platform/controller, pre-B evidence-
  protocol, and pre-E peripheral-authority gates. It grants no readiness or
  execution authority
- Implementation-task boundary: this persistent preparation goal and rebaseline
  worktree cannot roll into Milestone A. After PR #7 and every precondition pass,
  implementation begins from verified `main` in the packet-required fresh SSD
  worktree, `codex/sprint-alpha-vertical` branch, one Medium writer task, and no
  persistent goal
- Completion proof readiness: a non-authoritative eight-item traceability map
  now separates guest observations from same-run build, identity, authority,
  recovery, update, capture, and documentation proof. It explicitly records
  that no implementation or completion evidence exists at this checkpoint
- User orientation: `docs/sprint-alpha-dashboard.md` now provides one concise
  explanatory view of honest progress, missing gates, the A–G path, when boot
  and GUI first appear, and the local no-target-execution boundary. It grants no
  authority and reports target implementation as 0 of 7 milestones
- Predecessor handoff: PR #5 is closed, unmerged, and superseded. Its historical
  branch and evidence remain preserved; its live head is not part of PR #7 or
  Sprint Alpha
- Static enforcement: required-file, authority-status, ADR-classification, and
  exact approval-sentence checks now cover both execution maps, the owner brief,
  canonical ADRs 0022–0029, all three decision-integration gates, and all eight
  completion-proof items. Proposal 0030 remains machine-classified as
  owner-decision-required. Accepted ADR 0031 records the compact-BDF decision
  and D0 exact-main run `33415110465` passed. The merged P0 contract set binds
  the formula, vectors, 136-byte preimage, unchanged closure digest, trusted
  reconstruction, and complete T0/P0-B contract/checker set. Immutable
  checkpoint `e450f323a6a35a138499a585aed575c0c62ad85b` is preserved in its
  ancestry. PR #79 merged at
  `acfd70c9646209f71376d0d03c20d68d2c610746`, and exact-main run
  `33436244747` passed both validation and mutation. P0 is complete; machine
  activation and Milestone A remain blocked on their separate cumulative
  start gates, while the complete accepted-evidence publication task
  packet plus boot/platform integration packet are byte-pinned so preparation
  cannot silently become authoritative, incomplete, or disappear
- Gate-report migration: schema v1 remains unchanged. The versioned v2 reporter
  is merged, reviewed, and exact-main validated. It exposes ADR 0025,
  acceptance-v2, proposed-decision, private-platform, and fail-closed Milestone
  A/B states while every unavailable identity remains explicitly missing or
  blocked. It is orientation only; no active readiness or public format changed
- Platform delivery/state gaps: the existing Alpha boot grants no runtime
  storage authority and delivers no Core/component/app/state bytes. Accepted
  ADR 0026 selects four immutable Root-staged sources—fixed Core bootstrap,
  component bundle, initial system state, and initial preserved data—in one
  bounded Alpha envelope. It creates no persistence claim, device authority,
  or implementation readiness
- E ownership gap: accepted ADR 0022 requires narrow temporary
  Recovery/Nucleus adapter ownership, separate trusted profile/controller work,
  and full cumulative A–D revalidation instead of allowing E to edit unowned
  boot paths implicitly
- GUI/input architecture: accepted ADR 0022 identifies the authority missing
  from R0-002. Its reviewed peripheral contract and Lab evidence remain required
  before Milestone E target graphics/input code
- External blockers: the stale internal-Mac capacity safeguard is removed
  locally under the owner's 2026-08-27 SSD-workspace clarification; the SSD
  reserve and exact-root budget remain unchanged. The repository is now public
  and Actions is enabled. Specifications runs `33238786347`/`99064390254` and
  `33238991555`/`99064932344` started real `ubuntu-24.04` runners and completed
  setup, runner attestation, and both exact checkouts. The first identified
  missing executable modes on the new CI scripts; the repair run then identified
  the separately located `spec/alpha/lab/fixtures/generate.sh` omitted from the
  first mode inventory. That final required mode was repaired before the second
  and last permitted retry. Run `33239093362`, job `99065200412`, for exact
  head `19a88839cadf4e5f0cf322b77717b4f268788f35` then reached the controller
  handoff validator and failed because BSD-first `stat -f %z` returned
  successful filesystem text on Linux instead of a numeric byte count. The
  same fallback order occurs in thirteen validators, so a bounded source audit
  repairs the complete class and adds a regression check. On 2026-08-29 the
  owner explicitly reopened one bounded repair cycle and directed the task to
  diagnose, fix, and move on rather than stop at the simple CI error. Diagnostic
  run `33265595655`, job
  `99134978201`, at exact head `aa4c797f5db0b6979c24e7a2939dae6d34490a9a`
  proved the portable-stat guard, specification suite, immutable fixtures,
  Release 0 conformance, and generated SDK checks, then failed closed because
  GitHub had rotated its externally attested runner from `20260714.240.1` to
  `20260823.283.1`. The reviewed runner refresh was published as `9e4da4c`;
  run `33266007613`, job `99136083575`, then passed the complete primary
  validation phase and failed only because the final Linux mutation fixture
  retained the production Mac-only `uname` result. The independently reviewed
  repair fixed that fixture seam without weakening the production boundary.
  Its first exact-head run `33266499096` then failed only because the
  executable mock was denied in ephemeral `/tmp`; the no-executable successor
  then exposed the separate invalid `sed` delimiter in the next negative test.
  The delimiter-only correction passed in exact-head run `33267302085`, which
  then exposed the distinct QMP scratch-root defect. Its successor passed
  through QMP in run `33267767640`; the merged reference-evidence successor
  removed the undeclared `xxd` assumption and preserved the immutable
  OCI/toolchain pins. Exact-head run `33268129412` and exact-merge `main` run
  `33268347971` then passed the complete validation and mutation suites.
  The former zero-step account/billing blocker is cleared. A pattern-limited
  read on
  2026-08-28 confirmed that the user-level Codex config exists but contains no
  `rar-os-ssd`, `permission_profile`, or permissions-profile declaration; no
  external file was changed. The reviewed `rar-os-ssd` profile therefore still
  needs one-time owner-approved installation and confinement evidence in a
  fresh SSD-root task. The profile is now installed without replacing existing
  settings; effective confinement evidence remains a fresh-task pre-A gate
- Next durable actions: make the v2 controller/helper compiler,
  twice-reproduced helper, isolated test, and real immutable-identity evidence
  genuinely ready; retain matching firmware/q35/AHCI machine evidence; and
  retain the SSD confinement proof before Milestone A can begin. Separately,
  obtain the ADR 0030 owner choice and integrate ADR 0030's
  accepted-evidence publication contract through its merged task packet before
  Milestone B. The
  pure accepted-evidence record codec is already merged; publication, recovery,
  semantic verification, and activation are not. Acceptance-v2 publication
  remains mandatory before Milestone B;
  ADR 0022's reviewed peripheral-grant contract remains mandatory before
  Milestone E. Canonical accepted approvals remain in `docs/approval-record.md`
- Deadline: 2026-08-30 23:59 America/Los_Angeles
- Local safety: target compilation, image creation, firmware loading, QEMU,
  emulator, VM, guest execution, macOS modification, and access outside the
  dedicated RAR OS SSD subtree remain forbidden
