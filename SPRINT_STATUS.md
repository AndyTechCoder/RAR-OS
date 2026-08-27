# Sprint Alpha 0.1 Status

- Date: 2026-08-27
- Current milestone: End-of-week rebaseline, before Milestone A target implementation
- Objective: merge the trusted cloud Development Lab controller, then implement
  and retain the authentic A–G bootable GUI vertical slice
- Worktree: `/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha/worktrees/sprint-alpha-rebaseline`
- Branch: `codex/sprint-alpha-rebaseline`
- Pull request: [#7](https://github.com/AndyTechCoder/RAR-OS/pull/7), draft
- Remote PR head: `a39a805abfdbd673085a9ba50f565aee65566464`.
  Local reviewed source/safety checkpoint:
  `233bdd85f0d875cd4e7efb1e933798ebcb1c1111`. Subsequent local commits are
  documentation-only status and recovery-runbook corrections. The branch is
  intentionally unpushed while every
  private-repository Actions job is account-blocked before runner assignment.
  Its four source/safety commits fix branch whitespace/hash bindings, strengthen
  no-deletion auto-review and command rules, bind the authoritative directive
  in `AGENTS.md` and host safety, and add one digest-bound read-only local gate
- Validation of the local successor: the worktree is clean; full-branch
  whitespace, tracked-shell syntax, and the bound host-policy checker pass via
  `/bin/sh tools/ci/check-local-readonly.sh`. The Alpha preimplementation
  contract structure also passed before the read-only wrapper was narrowed and
  digest-bound. No Alpha completion evidence is claimed. The mutation-capable
  specification suite remains cloud-only and is not claimed locally
- Independent review: architecture, correctness, and security reviews are clean
  for the proposed Alpha platform/state boundary after bootstrap, parser,
  immutable-source, and ownership remediation. Consolidated correctness review
  is clean for the decision-integration and completion-evidence maps. These are
  source-only preparation reviews, not target implementation or completion proof
- Target functionality: 0%; no RAR OS target implementation has started, built,
  booted, or run
- Development Lab state: the three-role v2 field schemas and transcript contract
  are source-ready pending final re-review. Exact per-role mounts, bounded
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
  reviewed runnable v2 controller are still required. The controller's
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
  operation, cloud action, or activation
- Probe dispatch state: the top-level Development Probe now validates the v2
  blocked plan and stops with status 73. The superseded v1 two-role runner also
  stops before reading cloud context or issuing any container command; no
  default dispatch path can reach it after ADR 0020
- Controller-helper build trust: proposed ADR 0024 records three bounded cloud
  build choices and recommends a twice-reproduced, fully pinned compiler closure
  on the approved Linux runner for Alpha. It remains owner-decision-required;
  no helper compilation, cloud provisioning, or execution is authorized
- Controller-helper inventory: an option-neutral blocked instance now binds the
  required builder, compiler closure, trusted source, golden vector, twice-
  reproduced binary, and isolated test-evidence identities. All activating
  values remain `unavailable`; 40 declarative cases and mutation tests prevent
  a ready claim before ADR 0024 acceptance and real reviewed cloud evidence.
  Strict contextual parsers now validate the future aggregate build record,
  two distinct controller-owned build receipts, and a thirteen-case test receipt
  against their actual selected inputs. They reject aliases, hardlinks when the
  host filesystem supports exercising them, path escape, stale logs, reused
  job/root nonces, missing/duplicate cases, and self-declared results without
  controller context. Immutable and static checks pass locally without
  compiling or running the helper. Mutation-based policy tests skip locally
  before writing and require a dedicated pinned validation container with
  an independent clean exact-revision checkout mounted read-only and a bounded
  `nosuid,nodev` tmpfs as their only approved scratch; they are explicitly
  non-activating and are not helper build/test or cloud execution evidence
- Boot implementation state: accepted ADR 0021 selects the Alpha boot-volume,
  payload-loader, and Root-to-Recovery boundary. The candidate specification is
  explicitly `draft-incomplete` and cannot authorize implementation until its
  five review findings are decided and fixed. Proposed ADR 0023 groups them
  into one owner choice with Alternative C recommended; no target implementation began
- Milestone A preparation: a non-authoritative execution map now decomposes the
  existing packet into eight ordered work packets, maps all 41 mandatory boot
  cases, fixes one-writer paths and stop conditions, and keeps every target
  build/boot/evidence action cloud-only. It changes no contract or readiness
  state and does not authorize implementation
- Milestones B–G preparation: one non-authoritative dependency/evidence map now
  organizes the existing sequential ownership, contract-before-code boundaries,
  all 40 later acceptance rows, cloud-only evidence, and stop conditions. It
  defines no interface and does not start or reorder implementation
- Acceptance-sequence defect: the fixed Milestone C plan requires
  `component:gui-responsive`, but A–C own no GUI/presentation implementation and
  Milestone E is the first graphics owner. No synthetic marker or hidden pre-E
  component is permitted; an architecture-governed continuity witness or plan
  correction is required before C starts
- Acceptance-input defect: the fixed plan injects keyboard shortcuts to trigger
  B, C, and D, but input authority and implementation first exist at E. No
  hidden pre-E keyboard handler or marker without consumed input is permitted
- Proposed resolution: ADR 0025 recommends a new reviewed protocol version that
  auto-chains B–D through strict ordered `none` inputs, preserves the GUI-
  continuity row's exact post-crash position, and changes only its minimum from
  C to E. C would prove restart plus peer continuity; cumulative E–G probes
  would prove real GUI continuity. The proposal records no decision and changes
  no active protocol
- Owner decision readiness: the plain-language choice brief now explains ADR
  0025 and ADR 0026 alongside ADRs 0022–0024 and provides one exact five-decision approval
  sentence. The brief remains non-authoritative and records no approval
- Decision integration readiness: one non-authoritative plan now sequences the
  recommended decisions into pre-A boot/platform/controller, pre-B evidence-
  protocol, and pre-E peripheral-authority gates. It records no approval and
  changes no active packet, protocol, ownership, or readiness state
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
- Predecessor handoff: draft PR #5 remains open and unmerged as the historical
  deferred one-shot branch; its live head is not part of PR #7. It is never
  merged into Sprint Alpha and is closed as superseded only after PR #7's exact
  merge plus distinct real-step green `main` workflow are verified durable
- Static enforcement: required-file, authority-status, ADR-classification, and
  exact approval-sentence checks now cover both execution maps, the owner brief,
  proposed ADRs 0025/0026, all three decision-integration gates, and all eight
  completion-proof items so preparation cannot silently become authoritative,
  incomplete, or disappear
- Gate-report migration: schema v1 remains unchanged. Proposed ADR 0025 now
  requires a versioned v2 compatibility cutover exposing ADR 0025, protocol-v2,
  and fail-closed Milestone B states if the proposal is accepted; no active
  readiness or public format changed
- Platform delivery/state gaps: the existing Alpha boot grants no runtime
  storage authority and delivers no Core/component/app/state bytes. Proposed
  ADR 0026 recommends four immutable Root-staged sources—fixed Core bootstrap,
  component bundle, initial system state, and initial preserved data—in one
  bounded Alpha envelope. It records no decision, persistence claim, device
  authority, or implementation readiness
- E ownership gap: proposed ADR 0022 now explicitly requires narrow temporary
  Recovery/Nucleus adapter ownership, separate trusted profile/controller work,
  and full cumulative A–D revalidation instead of allowing E to edit unowned
  boot paths implicitly
- GUI/input architecture: proposed ADR 0022 identifies the authority missing
  from R0-002. It remains owner-decision-required, so it does not block the
  Milestone A boot foundation but must be accepted before Milestone E target
  graphics/input code
- External blockers: the 2026-08-26 local preflight still reports the existing
  internal-space safeguard; GitHub PR #7 is open/draft and the exact latest job
  has no runner and no steps. GitHub's annotation says: `The job was not started
  because recent account payments have failed or your spending limit needs to
  be increased.` The private repository cannot obtain the required real-step
  gate until that account condition is fixed. The reviewed `rar-os-ssd` user
  profile still needs one-time owner installation/evidence in a fresh SSD-root
  task
- Next durable action: record exact owner choices for ADRs 0023, 0024, and 0026,
  make the boot contract and v2 controller/helper evidence genuinely ready, and
  pass the external Lab/PR/checkpoint gates before Milestone A. ADR 0025 remains
  mandatory before Milestone B; ADR 0022 and its reviewed peripheral-grant
  contract remain mandatory before Milestone E. The non-authoritative
  plain-language choice brief is indexed at
  `docs/proposals/alpha-owner-choice-brief.md`; it records no approval
- Deadline: 2026-08-30 23:59 America/Los_Angeles
- Local safety: target compilation, image creation, firmware loading, QEMU,
  emulator, VM, guest execution, macOS modification, and access outside the
  dedicated RAR OS SSD subtree remain forbidden
