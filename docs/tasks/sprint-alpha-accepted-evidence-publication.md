# Sprint Alpha Accepted-Evidence Publication and Recovery Task Packet

Status: Non-authoritative preparation — ADR 0030 owner decision required

This packet turns proposed ADR 0030 Alternative B into a bounded implementation
sequence so a future writer does not invent persistent-data or recovery policy.
It is not an ADR, contract, approval, implementation authorization, readiness
claim, or permission to execute RAR OS. Until the owner approves ADR 0030 and
the proposal is converted through the normal accepted-ADR process, the only
implementation source mergeable in this area is the already-reviewed,
side-effect-free 20-line record codec. This documentation packet may merge
while remaining non-authoritative.

## Objective

After the start gate passes, implement one dependency-free, host-only,
descriptor-relative publication transaction that:

- journals ownership before acquiring cleanup authority;
- verifies the accepted-evidence record before it becomes visible;
- never overwrites a prior accepted record;
- distinguishes definitely uncommitted, committed-uncertain, and durable;
- recovers deterministically before any retry; and
- grants no target, cloud, launch, path-resolution, or autonomous root authority.

## Decision integration prerequisite

Before P0, a separate architecture/governance task must:

1. receive the exact ADR 0030 Alternative B owner sentence;
2. verify that it is informed approval for this exact proposal;
3. convert the proposal to the canonical accepted ADR and add only its exact
   approval-record row; and
4. pass architecture, correctness, security, required checks, merge, and
   exact-main validation.

No contract or implementation writer owns the ADR or approval record. Generic
approval, recommendation text, a task packet, or passing CI cannot substitute
for the exact owner decision.

## Contract-writer start gate

P0 may start only after the decision-integration prerequisite passes and the
exact branch is based on verified current `main`, with no overlapping writer,
conflict, red check, or unresolved review finding. P0 creates the contract that
later implementation depends upon; that contract is not a prerequisite to P0.

## Implementation-writer start gate

P1–P6 may start only after P0 is reviewed, merged, and exact-main validated.
The merged contract must fix every byte, name, state, transition, validation
precedence, error class, recovery result, retention and replacement rule,
separate root-attestation type/non-aliasing comparison, and the pinned Linux
syscall/errno set in which only enumerated results prove `NotCommitted` and all
others are `CommittedUncertain`.

Approval closes only the architecture choice. It does not activate the writer,
controller, profile, cloud commands, or a Sprint milestone.

## Owned paths

The P0 contract writer owns only:

- `spec/alpha/evidence/evidence-publication-journal-v0.fields`;
- `spec/alpha/evidence/evidence-publication-journal-v0-cases.v0`;
- the exact evidence README and specification check registrations.

After that contract is merged and exact-main validated, the implementation
writer owns only:

- `tools/rar-lab/controller-handoff/evidence_journal.rs`;
- `tools/rar-lab/controller-handoff/publication.rs`;
- `tools/rar-lab/controller-handoff/publication_recovery.rs`;
- the narrowly required additions to `accepted_evidence.rs`, `linux.rs`,
  `lib.rs`, `build-plan.v0`, and the controller-handoff README;
- immutable publication/journal fixtures; and
- the dedicated static, policy, and isolated-Linux test registrations.

The writer does not own workflows, Development Lab profiles, controller
orchestration, target source, acceptance plans, bound output files, or an
unrelated attempt journal. Wiring is a later, separately reviewed task.

## Ordered packets

### P0 — Contract and fixtures

- Define fixed binary or textual layouts for `Intent`, `Created`, `Prepared`,
  `Durable`, `Aborted`, and `ManualStop`.
- Bind schema/version, attempt ordinal 1–3, fresh nonce, probe, full expected
  record bindings, 19-line preimage digest, both root identities, fixed names,
  stable file identity, size, record digest, previous-transition digest, and
  terminal state exactly where the ADR requires them.
- Define canonical filenames, field order, byte order, reserved bytes, length
  ceilings, transition ordering, validation precedence, and exact errors.
- Provide independently reproducible golden preimages and every declarative
  valid/invalid transition and recovery case.

Evidence: contract checker, immutable fixture hashes, mutation policy tests,
and clean architecture/correctness/security review. No Rust implementation in
this packet.

### P1 — Side-effect-free journal codec and chain policy

- Encode/decode every fixed transition with checked arithmetic and no I/O.
- Validate exact previous-record chaining, attempt/root/name bindings, ordinal,
  legal predecessor, terminal behavior, and reserved fields.
- Reject duplicate, missing, reordered, unknown, stale, cross-attempt,
  cross-root, nonce-reuse, ordinal-reuse, and fourth-attempt state.
- Keep parsing separate from authority: a valid record does not prove durable
  publication or authorize cleanup.

Evidence: golden round trips and complete pure negative matrix in the reviewed
isolated host compiler environment.

### P2 — Sealed descriptor operations

- Add distinct crate-private record-root and journal-root types; reject equal
  `(device,inode)` identities and any wrong purpose, owner, mode, locality, or
  exclusivity attestation.
- Append journal transitions with descriptor-relative new/no-follow creation,
  mode 0600, same-descriptor re-read, `fdatasync`, identity recheck, and parent
  `fsync`.
- Observe final/temporary entries without following links.
- Expose identity-bound removal only after durable `Created` authority.
- Keep all Linux `unsafe` and syscall numbers in `linux.rs`, with documented
  invariants and bounded `EINTR` behavior.

Evidence: static authority checks and isolated Linux descriptor tests. No path,
process, network, container, cloud, raw-descriptor, or root-acquisition API.

### P3 — Publication state machine

Implement the exact durable ordering:

`Intent → create → record-parent fsync → Created → write/fdatasync/re-read → Prepared → RENAME_NOREPLACE → revalidate → record-parent fsync → Durable`

- No temporary creation before durable `Intent`.
- No cleanup authority before durable `Created`.
- No rename before durable `Prepared`.
- A successful no-replace rename is visibility, not durable success.
- Only post-rename validation, record-parent sync, and durable `Durable` return
  success.
- Return only the contract's three outcomes; never collapse uncertainty into a
  Boolean or retryable error.

Evidence: fake-backend fault injection before and after every operation and
sync boundary, with exact effect logs.

### P4 — Recovery-before-retry

- Enumerate and validate the complete immutable journal chain before observing
  or changing record entries.
- Implement every ADR 0030 recovery row exactly, including valid
  `Prepared`-to-final adoption and terminal revalidation.
- Remove only a journal-authorized temporary inode and sync its parent before
  appending `Aborted`.
- Preserve all state and append `ManualStop` only when the journal root remains
  trustworthy.
- Make repeated recovery idempotent; an existing identical transition must be
  same-descriptor validated and parent-synced before use.
- Permit retry only after terminal `Aborted`, with a fresh nonce and unused
  ordinal. `ManualStop` never auto-progresses.

Evidence: complete recovery matrix plus crash/failure injection at every
durability boundary.

### P5 — Linux conformance

In the approved isolated Linux host environment only:

- prove real `RENAME_NOREPLACE` collision and competing-writer behavior;
- exercise the pinned definitely-not-committed errno set and unknown/ambiguous
  outcome classification;
- reject symlink, hardlink, FIFO, wrong owner/mode/link count, root alias, and
  rejected network/FUSE attestation;
- terminate the host helper after each durable boundary and prove recovery;
- prove a valid interrupted `Prepared` commit can be adopted without replacing
  or deleting evidence.

These are host-helper tests. They do not compile, build, image, boot, or execute
RAR OS target code.

### P6 — Review and source-only merge gate

- Architecture review checks the accepted ADR/contract/state-machine match.
- Correctness review checks total transitions, precedence, retry ceiling,
  fixtures, fault matrix, and idempotence.
- Security review traces every cleanup/adoption authority and unsafe syscall
  boundary.
- Fix all accepted findings in one bounded remediation and obtain clean
  re-review.
- Merge only when exact-head required checks are green and conflict-free.
- Require the exact resulting `main` Specifications run to pass before any
  later controller/profile integration begins.

## Activation remains blocked until

The source-only merge is not activation. A separate owner of the authoritative
vertical packet, trusted controller, Development Lab profile, semantic
verifier, and readiness/gate-report paths must prove all of the following in a
later reviewed change:

1. The writer, semantic verifier, immutable fixtures, controller, and exact
   profile are all reviewed, merged, and exact-main validated.
2. The controller recomputes every accepted-evidence binding and set/member
   digest from the post-handoff, controller-owned immutable output descriptors;
   no caller-supplied digest or target output grants authority.
3. The controller binds two exact, distinct sealed record/journal roots with
   reviewed mode-0700, owner, local non-network/non-FUSE, exclusive-mutation,
   and non-alias attestations for the full transaction.
4. Journal recovery completes before every retry or record use, and every
   terminal state is revalidated as required by the accepted contract.
5. Publication success alone cannot progress the phase. Progression requires
   the semantic verifier to validate the durable final record against the exact
   attempt, controller/source/artifact/protocol/profile/tool/output/handoff/
   reference/inventory expectations and retained bound outputs.
6. The separate activation PR updates the authoritative vertical packet,
   controller/profile instances, readiness fields, and gate report without
   allowing the source branch to choose trusted launcher or verdict behavior.
7. Complete isolated host evidence, architecture/correctness/security review,
   required checks, merge, and exact-main validation all pass.

Until then the module remains unreachable, inactive, and incapable of making a
probe ready even if its source-only tests pass.

## Mandatory matrix

The contract and tests must cover at least:

- every legal and illegal state pair, including every terminal state;
- malformed, oversized, wrong-schema, wrong-chain, wrong-attempt, wrong-root,
  wrong-name, wrong-ordinal, nonce-reuse, and fourth-attempt records;
- failure before and after every transition write, file sync, parent sync,
  record create/write/re-read, rename, post-rename validation, and cleanup;
- no journal; each nonterminal state with neither/temp/final/both entries;
- identity/type/owner/mode/link-count/size/digest mismatch;
- cleanup uncertainty, ambiguous rename, repeated recovery, and conflicting
  already-present recovery transition; and
- `Durable` revalidation, `Aborted` absence proof, and permanent automatic stop
  for `ManualStop`.

Every rejection must prove no unjournaled removal, overwrite, adoption, retry,
phase progression, bound-output mutation, or target/cloud effect occurred.

## Stop conditions

Stop without broadening scope if work would:

- start P0 before exact ADR approval and accepted decision integration, or
  start P1–P6 before the reviewed journal contract is merged and exact-main
  validated;
- invent or change a persistent field, filename, transition, error class,
  retention promise, cleanup/adoption rule, retry ceiling, or trust boundary;
- reuse or reinterpret `controller-handoff-attempt-v0`;
- make publication operations public/generic or accept arbitrary caller paths,
  labels, roots, flags, raw descriptors, or precomputed output authority;
- delete or replace a final record, bound output, journal transition, or
  unrelated temporary entry;
- classify an unknown rename outcome as definitely uncommitted;
- activate a profile/controller, run cloud commands, or compile/execute target
  code; or
- weaken tests, isolation, signing, validation, or evidence to pass a gate.

No deadline or generic approval overrides these conditions.
