# Alpha Decision Integration Plan

Status: Non-authoritative integration plan — decisions accepted, activation gated

## Purpose

This plan records how the current Sprint Alpha packet is amended after the
owner accepted the selected alternatives in ADRs 0022–0026. It prevents a
future task from treating an owner choice as a ready contract, editing a trusted
controller from an implementation branch, or inventing path ownership.

This document is not the approval record and changes no active contract,
evidence protocol, task ownership, execution authority, or readiness state. The
decision files, approval record, reviewed specifications, and authoritative
vertical packet always win.

## Decision and activation rule

Each selected ADR is accepted only because the approval record contains its
exact owner decision through the repository's ADR process. Acceptance
authorizes only the work stated by that ADR. It does not make a format ready,
satisfy a preflight, merge a PR, authorize a cloud run, or authorize any Mac
target build or execution.

An accepted alternative activates only after its specification/controller
change is complete, independently reviewed at the required risk level, merged
to the correct branch, and bound by exact immutable identities. Until then the
existing version remains authoritative and the dependent milestone remains
closed.

## Gate 1 — before Milestone A implementation

Accepted decisions: ADR 0023 Alternative C, ADR 0024 Alternative A, and ADR
0026 Alternative C. Accepted ADR 0022 Alternative C permits the common private
envelope to be reviewed once, but its peripheral record remains absent and
grants no authority until Gate 3.

After recorded owner acceptance, one architecture-owned contract change must:

1. Complete the private Alpha Boot Contract and Machine Profile v2 selected by
   ADR 0023, including byte-exact image construction, fixed source slots, total
   firmware-memory conversion, timer identity, and enforced x86 W^X state.
2. Add the ADR 0026 private outer source-set and `AlphaPlatformEntryV0`
   contracts, the fixed `AlphaCoreBootstrapV0` mechanism, four immutable input
   formats, parser ownership, rights transfer/revocation, bounds, identities,
   negative fixtures, and replacement notes. Stable R0-002 remains unchanged.
3. Amend the authoritative vertical packet so Milestone C narrowly owns
   `spec/alpha/component/` and `core/loader/`. Milestone A owns only the already
   assigned boot/Recovery/Nucleus adapter and deterministic image-tool paths.
   Milestone D keeps its existing state/recovery/storage paths and receives no
   raw device authority.
4. Update both non-authoritative execution maps in the same change. The A map
   must bind ADR 0026 at its start gate and cover Root staging, Recovery outer-
   source validation, the fixed Core bootstrap, and all four immutable sources.
   The B–G map must add C loader ownership and D inner-state parsing without
   granting either milestone A boot-path ownership.
5. Keep ADR 0022's optional peripheral record reserved but absent. Reserving a
   separately framed record does not select an input transport or create a
   capability.

A separate default-branch controller change must implement ADR 0024 with the
complete pinned Linux compiler closure, two fresh network-disabled identical
helper builds, independent helper conformance evidence, ready v2 Lab bindings,
and no target or owner-data access. The vertical implementation branch must not
edit this controller authority.

A separately reviewed gate-report schema v2 must also land before Milestone A.
It classifies ADR 0026 and binds exact ready identities for the private platform
envelope, Core bootstrap, component bundle, initial system state, and initial
preserved-data source. It may report acceptance protocol v2 as
`reviewed-implementation-required` until the separate pre-B cutover.

Milestone A remains closed until the amended packet and contracts are reviewed
and ready; the helper/controller evidence is real and merged; PR #7 and every
required workflow are green, reviewed, merged, and verified; and every local,
remote, SSD-profile, Lab, identity, and immutable-checkpoint precondition in the
authoritative packet passes. A zero-step workflow is not a passing check.

## Gate 2 — before Milestone B implementation

Accepted decision: ADR 0025 Alternative B.

After recorded owner acceptance, one separately reviewed default-branch evidence
change must:

1. Add immutable `acceptance-v2.plan` with exactly the four field changes,
   45-row order, bucket counts, and cumulative counts specified by ADR 0025.
2. Bind the exact v2 digest in the controller, verifier, profile, fixtures, and
   documentation; reject v1 for every new A–G probe after cutover.
3. Update the authoritative vertical packet and B–G execution map together so
   B–D auto-chain without pre-E keyboard authority and GUI continuity first
   becomes mandatory in cumulative E–G probes.

Gate-report v1 and acceptance v1 remain immutable historical evidence and are
never reinterpreted or used for active readiness after these decisions.

This change is trusted controller/protocol work, not Milestone B target work.
The B writer must not edit `.github/workflows/`, trusted `tools/ci/`, or the
historical v1 files. Milestone B remains closed until the v2 cutover is merged,
reviewed, exactly bound, and a successful real-step workflow proves it.

## Gate 3 — before Milestone E graphics/input implementation

Accepted decision: ADR 0022 Alternative C.

The approved cloud Lab must first produce reviewed capability evidence for the
pinned QEMU/firmware profile. The evidence selects either a proven non-DMA input
transport or the bounded trusted xHCI/USB fallback; implementation must not
guess. An architecture-owned contract change then fixes the exact optional
peripheral record, profile digest, framebuffer/input transport, address spaces,
ranges, rights, interrupts, DMA bounds if unavoidable, validation order, error
outcomes, and attenuation rules.

The corresponding default-branch profile/controller change is reviewed and
merged separately. Only then may the authoritative vertical packet and B–G
execution map name the exact Recovery/Nucleus Alpha-adapter files receiving
temporary E ownership. They must not grant entire boot or architecture
directories. The E gate reruns complete cumulative A–D boot, R0, memory,
isolation, and recovery evidence plus architecture, correctness, and security
review before accepting graphics/input behavior.

## Milestone payload ownership after integration

- A stages and authenticates the fixed Core bootstrap, component bundle, and
  immutable state sources; it does not implement ordinary component policy.
- C implements the Core loader and component lifecycle against the reviewed
  bundle contract without reopening A boot paths.
- D validates state-source internals, creates non-aliased mutable regions, and
  reconstructs only the system destination without preserved-data write rights.
- E adds apps and services as reviewed bundle entries and receives only
  attenuated framebuffer/event capabilities.
- F consumes signed candidate-layer data through the reviewed loader/update
  contracts; it does not make the boot volume or host filesystem visible.

Any later need to cross these path boundaries requires explicit temporary
ownership, cumulative revalidation through the active milestone, and the
reviews required by the changed trust boundary or format.

## Implementation task transition

This preparation branch, worktree, and persistent goal do not become the Alpha
implementation task. After PR #7 is green, reviewed, merged, and verified—and
after every accepted-decision, contract, controller, Lab, SSD-profile, capacity,
and immutable-checkpoint precondition passes—the release driver creates the
packet-required fresh SSD worktree and `codex/sprint-alpha-vertical` branch from
the verified `main` merge. Exactly one Medium-effort writer task owns the active
milestone paths, and that task has no persistent goal.

No preparation task may carry unmerged state, inherited target authority, or a
stale base into Milestone A. If any precondition fails during transition, the
implementation task is not created and this document grants no fallback path.

PR #5 is a non-ancestor historical branch for the deferred production-style
one-shot approach. PR #7's exact merge and distinct green `main` workflow were
verified on 2026-08-29, so PR #5 is now closed, unmerged, and superseded with
its history and evidence preserved. PR #5 is never merged, rebased, or
wholesale cherry-picked into the Sprint Alpha line; any useful idea is
reimplemented only through the active reviewed packet and owned paths.

## Final activation checklist

- Exact owner choices are recorded; no explanatory document self-approves.
- Accepted ADR status, approval record, indexes, and task packet agree.
- Every new experimental format has deterministic bytes, ceilings, validation
  precedence, negative fixtures, parser ownership, migration/replacement notes,
  and independent review.
- Stable R0-002 and historical evidence/report versions remain unchanged.
- Controller changes are on the reviewed default-branch path, never smuggled
  into a milestone implementation branch.
- The authoritative packet names every added owned path and exact temporary
  cross-milestone file before a writer starts.
- The preparation task has ended and the fresh one-writer implementation task
  starts at the exact verified `main` merge with no inherited working diff.
- Historical PR #5 is closed as superseded only after PR #7 durability proof;
  it is never merged into the Alpha implementation line.
- Required GitHub workflows execute real steps and pass; zero-step failures,
  missing identities, or draft/failed checks cannot become readiness.
- No local target compilation, image creation, firmware loading, VM launch, or
  target execution occurs on the Mac or SSD.
