# ADR 0030: Alpha Accepted-Evidence Publication and Recovery

Status: Proposed — owner decision required
Decision: Undecided

The proposal identifier does not imply that every lower number exists or is
accepted. This proposal grants no implementation, publication, recovery,
controller, launch, or target-execution authority.

## Context

The accepted Alpha evidence contract fixes the exact 20-line record and requires
a future phase-8 writer to use new/no-follow mode-0600 creation, same-descriptor
parse and hash verification, identity recheck, `fdatasync`, and parent `fsync`.
It does not yet define the persistent transaction state machine.

Implementation cannot safely invent final or temporary names, the visibility
commit point, rename error meaning, crash recovery, stale entry handling,
retry/adoption rules, or cleanup authority. These are persistent-data and trust-
boundary promises and therefore require an owner-approved ADR before the writer
can become contract-complete or activation-ready.

## Invariants shared by every option

- The Evidence-record and Evidence-publication-journal roots are distinct,
  non-aliased, already-open, controller-owned mode-0700 directories on reviewed
  local non-network, non-FUSE task storage.
- Independent sealed attestations prove the controller is the sole mutator of
  both roots for the entire transaction, including against other same-UID
  processes and descriptor holders.
- Records are regular, mode 0600, link-count one, bounded to 4096 bytes, and are
  opened descriptor-relative with no link following.
- The controller supplies the expected attempt and all trusted identities;
  target output, a record, or a filename never grants authority.
- Bound outputs are never removed or replaced by evidence-writer rollback.
- Any unclassifiable state, identity mismatch, or cleanup uncertainty blocks
  phase progression and preserves evidence for reviewed recovery.
- RAR OS target code is never executed by this host-only transaction.

## Decision drivers

- Never overwrite a prior accepted record.
- Never report success before the record and directory entry are durable.
- Recover deterministically after process death or power loss at every boundary.
- Distinguish definitely uncommitted, committed-but-uncertain, and durable states.
- Bind cleanup to the exact controller-created inode and durable attempt journal.
- Keep the Alpha-only format replaceable and dependency-free.

## Alternative A — Create the final record directly with `O_EXCL`

Create `accepted-evidence.v0` directly using descriptor-relative
`O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC`, write and verify it through the same
descriptor, call `fdatasync`, then `fsync` the parent.

This is small and provides no-replace behavior without rename. However, an
incomplete final name is visible before verification and durability. Recovery
must distinguish a partial writer-created final inode from a complete uncertain
commit using the durable attempt journal and same-descriptor validation. Until
recovery completes, the final name grants no acceptance.

## Alternative B — Verified temporary record plus no-replace rename

Use final basename `accepted-evidence.v0`. Derive the private temporary basename
as `.accepted-evidence.<64-lower-hex-attempt-nonce>.tmp`.

Alternative B selects a new, separate `EvidencePublicationJournalV0`; it does
not extend or reinterpret `controller-handoff-attempt-v0`. The journal and the
record use distinct, non-aliased, sealed controller roots. Journal transitions
are immutable, hash-chained, attempt-nonce-named records created with `O_EXCL`,
then individually `fdatasync`ed and parent-`fsync`ed. They are never edited or
deleted in v0. Exact fields and byte layouts follow in a reviewed contract, but
the ownership, separation, states, and ordering below are selected by this ADR.

1. Append durable `Intent`. It binds the active controller attempt ordinal
   (1..3), fresh nonce, probe, complete trusted expected bindings, exact 19-line
   preimage digest, final name, temporary name, protocol, record schema, and
   both root identities. It has no inode because no record exists yet.
2. Create the temporary entry with descriptor-relative
   `O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC`, mode 0600.
3. `fsync` the record parent, then append durable `Created`. It binds only the
   stable cleanup identity `(device,inode)`, plus required type, owner, mode,
   link-count one, and initial size zero. Creation grants cleanup authority only
   after `Created` is durable.
4. Write exactly, call `fdatasync`, seek to zero, parse and hash through that
   same descriptor, and recheck the root plus stable `(device,inode)`. Append
   durable `Prepared`, binding size, record digest, and stable identity.
   Timestamps are audit observations, never stable identity keys.
5. Atomically rename temporary to final with Linux
   `renameat2(..., RENAME_NOREPLACE)`. Success is the visibility commit point.
6. Re-stat the retained descriptor and require the stable identity, exact size,
   type, owner, mode, link count, and record digest. Then `fsync` the record
   parent and append durable `Durable`. Only completion of all three is success.
   The receipt contains the post-rename observation and stable identity.

`Aborted` is appended only after an authorized temporary removal and record-
parent `fsync`. `ManualStop` is appended when state is preserved for review.
`Durable`, `Aborted`, and `ManualStop` are terminal. The complete journal chain
and bound outputs are retained for the Alpha evidence-retention period; v0 has
no journal deletion, compaction, nonce reuse, or in-place transition.

The adapter exposes three outcomes, not a Boolean result:

- `NotCommitted`: the reviewed local syscall contract proves no rename occurred.
- `CommittedUncertain`: rename succeeded, or its result/durability cannot be
  proven; the final entry is left untouched and phase progression stops.
- `Durable`: post-rename identity, record-parent synchronization, and the
  terminal `Durable` journal transition are all durable.

Recovery runs before any retry:

- No journal: no entry is owned. If either name exists, append no authority,
  preserve it, and stop for review. If both are absent, no recovery is needed.
- Latest `Intent`, both entries absent: append `Aborted`; this covers a crash
  before create or loss before the record-parent sync. Repeated recovery is a
  no-op after the terminal transition.
- Latest `Intent`, temporary present: it was created before durable `Created`,
  so no cleanup identity is trusted. Preserve it and append `ManualStop`.
- Latest `Created`, final absent, temporary present with the exact journaled
  stable `(device,inode)`, type, owner, mode, and link count: remove only that
  inode, `fsync` the record parent, then append `Aborted`. Its size may be any
  value from zero through 4096 after an interrupted write; size and timestamps
  are not cleanup identity keys.
- Latest `Prepared`, final absent, temporary present with the exact journaled
  stable identity, final size, and digest: remove only that inode, `fsync` the
  record parent, then append `Aborted`.
- Latest `Created` or `Prepared`, both entries absent: synchronize the observed
  record parent, append `Aborted`, and never reuse the nonce.
- Latest `Prepared` with final present and temporary absent: open final no-
  follow, require the journaled stable `(device,inode)`, validate the complete
  expected attempt/record/bindings and digest through one descriptor, re-stat,
  `fsync` the record parent, then append `Durable`. This adopts a valid commit
  whose rename or later durability acknowledgement was interrupted.
- Latest `Intent` or `Created` with final present is a protocol contradiction:
  neither state grants rename authority. Preserve all state and append
  `ManualStop` only if the journal root remains trustworthy. Record contents or
  a matching filename cannot substitute for the missing durable `Prepared`.
- Terminal `Durable` requires the same final validation on every use. Terminal
  `Aborted` requires both entries absent. Terminal `ManualStop` never progresses
  automatically. Any contradiction preserves all state and stops.
- Both entries present, final for another attempt, unjournaled temporary,
  identity/metadata/digest mismatch, malformed record, broken journal chain, or
  ambiguous backend behavior: preserve everything and append `ManualStop` only
  if the journal root remains trustworthy.

Every recovery transition is itself durable before return. A crash during
recovery repeats the same idempotent observation/transition; an already-present
identical transition is validated through one descriptor and its journal parent
is `fsync`ed before it is treated as durable; a conflicting transition stops. A
new attempt may begin only after `Aborted`, and always uses a fresh nonce.
`Durable` completes the probe with no retry. `ManualStop` blocks every retry
until a separately reviewed future recovery authority resolves it; terminal
does not mean retry-eligible. Publication shares the existing attempt-ordinal
ceiling of three per phase: every chain consumes its bound ordinal, ordinal
reuse rejects, and a fourth attempt is durably blocked. Recovery never
reinterprets an older schema. The exact contract
enumerates the pinned local-Linux rename errors that prove `NotCommitted`; every
other rename result is `CommittedUncertain`.

## Alternative C — Per-attempt immutable records plus a durable selected index

Publish a no-replace final record for every attempt, then separately publish an
immutable selected-attempt index. This avoids a single final-name collision and
keeps complete history, but introduces a second persistent format, two commit
boundaries, selection recovery, retention ceilings, and index rollback. It is
the most extensible and the largest Alpha-only mechanism.

## Proposed recommendation

Alternative B most directly preserves “verify before visible,” no replacement,
and deterministic recovery while adding one separate Alpha-private publication
journal state machine. This recommendation is not a decision.

## Consequences if Alternative B is selected

- The accepted-evidence contract and a new separate publication-journal contract
  gain exact names, fields, transitions, recovery, retry, retention, rejection,
  migration, and replacement rows in a separately reviewed change. The existing
  handoff-attempt journal remains v0 and unchanged.
- Publication operations remain crate-private/sealed and return the three-state
  commit outcome. A generic public trait cannot weaken these guarantees.
- The trusted controller must durably journal attempt ownership and implement
  recovery before writer activation.
- Focused fake-backend tests cover every write/sync/read/stat/rename/parent-sync
  fault and cleanup outcome. Isolated Linux tests cover actual
  `RENAME_NOREPLACE`, existing-final collision, symlink/hardlink/FIFO rejection,
  competing-writer denial, process death, and power-loss recovery.
- The Development Lab profile, controller, helper, and all readiness fields stay
  blocked until writer, verifier, fixtures, recovery, controller/profile wiring,
  independent reviews, and exact-main validation all pass.

## Compatibility and replacement

This is a private, experimental Alpha host format. A replacement requires a new
version, contract, migration/recovery policy, and review. Old state is rejected,
never silently reinterpreted. It creates no target ABI or production persistence
promise.

## Exact approval sentence

`I approve ADR 0030 Alternative B for experimental Alpha accepted-evidence publication and recovery under the documented safety limits.`
