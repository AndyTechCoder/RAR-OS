# ADR 0025: Alpha GUI Continuity Evidence Sequencing

Status: Proposed — owner decision required
Decision: Undecided

## Context

The fixed Alpha acceptance plan requires `component:gui-responsive` during
Milestone C immediately after a deliberately crashed component restarts. That
is impossible to prove honestly at C: Milestones A–C own boot, Nucleus runtime,
capability/IPC, and registry paths, while framebuffer graphics, input, shell,
and apps first exist and are owned by Milestone E.

Implementing a hidden pre-E presentation component would violate milestone
ownership and order. Emitting the marker without a real GUI would replace guest
behavior with a fabricated success. Removing GUI continuity entirely would
weaken the owner-approved crash-containment demonstration.

The trusted controller already interprets the first plan field as the minimum
milestone for each row. It filters rows by that value while preserving file
order. Therefore one row may remain immediately after the C crash/restart
sequence but become required only in cumulative E–G probes, when the GUI exists.

## Decision drivers

- Keep C independently testable before any GUI is implemented.
- Prove real GUI continuity after the same crash/restart sequence once E exists.
- Preserve exact row order, marker, evidence label, and total observation count.
- Avoid adding a hidden presentation component or moving GUI paths into C.
- Keep the trusted controller fail-closed and cumulative.
- Version the corrected protocol rather than silently reinterpreting v1.

## Considered options

### A. Implement a minimal presentation component before C

Add an early component under A or B so C can observe it. This expands ownership,
creates presentation behavior before the graphics contract, and risks a second
throwaway GUI path. It is not recommended.

### B. Change only the GUI-continuity row's minimum from C to E

Create `acceptance-v2.plan` with the same 45 rows and exact ordering as v1,
except:

`C|none|component:gui-responsive|gui-continuity|0`

becomes:

`E|none|component:gui-responsive|gui-continuity|0`

At a C probe, the controller skips that E-minimum row and still requires the
C-owned restart and peer-responsive observations. At an E, F, or G probe, the
full E-capable guest first executes the same C crash/restart sequence, then must
emit the real GUI-continuity marker before the peer-responsive capture. This
keeps temporal correlation to the failure without requiring GUI code at C.

Version 1 remains immutable historical evidence and is not accepted for any new
A–G probe once v2 is activated. The controller, verifier, tests, documentation,
and profile bind the exact v2 digest before activation. This option is proposed.

### C. Add a second crash sequence during E

Keep the C plan unchanged except to remove its impossible GUI row, then add a
new E-specific crash, restart, GUI-continuity, and peer-continuity sequence.
This is explicit but increases inputs, observations, runtime, controller tests,
and failure surface without improving the proof over Alternative B.

### D. Drop GUI continuity from Alpha

Remove the row and prove only another non-GUI peer remains responsive. This
weakens the approved demonstration and is rejected.

## Proposed direction

Select Alternative B. It is the smallest honest correction and uses the
controller's existing minimum-milestone semantics. The correction changes only
when one existing observation becomes mandatory, not its meaning or its
post-crash position.

Acceptance of this ADR would authorize a separately reviewed protocol-v2
change and corresponding trusted-controller policy/tests. It would not
authorize target implementation, cloud provisioning, credentials, builds, VM
launch, Mac execution, merge, or a readiness claim.

## Consequences if accepted

- Milestone C can finish with only C-owned isolation behavior.
- Milestones E–G prove GUI continuity after the cumulative crash/restart path.
- The observation count remains 45 and all existing labels/markers remain.
- Minimum-milestone buckets become A:5, B:7, C:11, D:7, E:7, F:7, G:1;
  cumulative selections become A:5, B:12, C:23, D:30, E:37, F:44, G:45.
- Evidence consumers must bind v2 exactly and reject v1 for every new A–G probe
  after cutover.
- A later protocol correction remains a new version, never silent mutation.

## Security and correctness impact

Alternative B prevents a fabricated GUI marker and preserves evidence ordering.
The trusted controller still requires every selected marker after the preceding
input and serial offset, rejects missing/duplicate/reordered/extra descendants,
and captures only after both post-crash continuity observations. No new guest
authority, data, device access, or host capability is introduced.

## Validation if accepted

- Immutable fixtures prove A–D selections exclude the E-minimum GUI row.
- E–G selections include the row in its original post-restart position.
- Exact minimum buckets are A:5/B:7/C:11/D:7/E:7/F:7/G:1 and exact cumulative
  selections are A:5/B:12/C:23/D:30/E:37/F:44/G:45; the controller, verifier,
  fixtures, B–G execution map, and status assertions must update together.
- C still requires restart complete and peer responsive.
- E–G reject missing, early, stale, duplicated, or reordered GUI continuity.
- The v2 plan has exactly 45 unique rows and differs from v1 only in the one
  minimum-milestone byte plus schema/version documentation.
- Controller, verifier, profile, retained evidence, and documentation bind the
  reviewed v2 digest.
- After activation, every new A–G probe rejects v1; v1 is retained only as
  immutable historical evidence from runs that predate the cutover.

## Replacement path

The Alpha evidence protocol is experimental. Production fault-containment and
UI-availability evidence will use later reviewed scenario versions appropriate
to the production component graph. No v1 or v2 marker becomes a stable target
ABI.
