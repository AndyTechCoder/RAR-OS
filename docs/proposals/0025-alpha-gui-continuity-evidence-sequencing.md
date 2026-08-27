# ADR 0025: Alpha Pre-GUI Evidence Input and Continuity Sequencing

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

The same plan triggers Milestones B, C, and D with `key:ctrl-alt-b`,
`key:ctrl-alt-c`, and `key:ctrl-alt-d`. Keyboard and pointer authority, the
input service, and their owned paths first exist at Milestone E. A pre-E guest
therefore has no approved route to consume those key events. Adding a hidden
keyboard path would create device authority before ADR 0022 and E's reviewed
contract; emitting results without consuming the input would fabricate cause.

The trusted controller already interprets the first plan field as the minimum
milestone for each row. It filters rows by that value while preserving file
order. Therefore one row may remain immediately after the C crash/restart
sequence but become required only in cumulative E–G probes, when the GUI exists.

## Decision drivers

- Keep C independently testable before any GUI is implemented.
- Keep B–D independently triggerable without pre-E input/device authority.
- Prove real GUI continuity after the same crash/restart sequence once E exists.
- Preserve exact row order, marker, evidence label, and total observation count.
- Avoid adding a hidden presentation component or moving GUI paths into C.
- Keep the trusted controller fail-closed and cumulative.
- Version the corrected protocol rather than silently reinterpreting v1.

## Considered options

### A. Implement minimal keyboard and presentation support before C

Add an early input path under B and an early presentation component so B–D can
consume key triggers and C can observe a GUI. This expands ownership, grants
device authority before ADR 0022/E, and risks second throwaway input and GUI
paths. It is not recommended.

### B. Auto-chain pre-E stages and defer GUI continuity to E

Create `acceptance-v2.plan` with the same 45 rows and exact ordering as v1,
with exactly four field changes:

`B|key:ctrl-alt-b|nucleus:page-allocator-pass|page-allocator|0`

becomes:

`B|none|nucleus:page-allocator-pass|page-allocator|0`

`C|key:ctrl-alt-c|capability:forged-rejected|forged-handle|0`

becomes:

`C|none|capability:forged-rejected|forged-handle|0`

`D|key:ctrl-alt-d|state:regions-distinct|region-separation|0`

becomes:

`D|none|state:regions-distinct|region-separation|0`

Finally:

`C|none|component:gui-responsive|gui-continuity|0`

becomes:

`E|none|component:gui-responsive|gui-continuity|0`

`none` retains its existing strict meaning: after the preceding selected marker,
the guest must emit the next marker in order without another controller input;
it cannot reuse a pre-existing line. B–D therefore run their deterministic
test sequences automatically after the prior milestone's terminal marker,
without any keyboard or new control transport.

At a C probe, the controller skips the E-minimum GUI row and still requires the
C-owned restart and peer-responsive observations. At an E, F, or G probe, the
full E-capable guest first executes the same C crash/restart sequence, then must
emit the real GUI-continuity marker before the peer-responsive capture. This
keeps temporal correlation to the failure without requiring GUI code at C.

Version 1 remains immutable historical evidence and is not accepted for any new
A–G probe once v2 is activated. The controller, verifier, tests, documentation,
and profile bind the exact v2 digest before activation. This option is proposed.

### C. Add a private pre-E test-control transport and a second E crash

Define a new serial, debug-port, or memory-mailbox control interface for B–D,
then remove the impossible C GUI row and add a new E-specific crash, restart,
GUI-continuity, and peer-continuity sequence. This is explicit but creates a new
guest control contract and increases inputs, observations, runtime, controller
tests, and failure surface without improving the proof over Alternative B.

### D. Drop GUI continuity from Alpha

Remove the row and prove only another non-GUI peer remains responsive. This
weakens the approved demonstration and is rejected.

## Proposed direction

Select Alternative B. It is the smallest honest correction and uses the
controller's existing minimum-milestone semantics. The correction changes only
three impossible pre-E input fields plus when one existing observation becomes
mandatory. It changes no marker meaning or post-crash position.

Acceptance of this ADR would authorize a separately reviewed protocol-v2
change and corresponding trusted-controller policy/tests. It would not
authorize target implementation, cloud provisioning, credentials, builds, VM
launch, Mac execution, merge, or a readiness claim.

## Consequences if accepted

- Milestone C can finish with only C-owned isolation behavior.
- Milestones B–D require no keyboard, pointer, or private test-control authority.
- Real keyboard/pointer inputs begin at E, where their owned implementation and
  reviewed authority contract exist.
- Milestones E–G prove GUI continuity after the cumulative crash/restart path.
- The observation count remains 45 and all existing labels/markers remain.
- Minimum-milestone buckets become A:5, B:7, C:11, D:7, E:7, F:7, G:1;
  cumulative selections become A:5, B:12, C:23, D:30, E:37, F:44, G:45.
- Evidence consumers must bind v2 exactly and reject v1 for every new A–G probe
  after cutover.
- A later protocol correction remains a new version, never silent mutation.

## Security and correctness impact

Alternative B prevents a fabricated GUI marker and preserves evidence ordering.
It also prevents pre-E key injection from being mistaken for consumed guest
input. The existing `none` rule still requires each new marker after the prior
selected serial offset, so automatic progression cannot satisfy evidence with
stale output.
The trusted controller still requires every selected marker after the preceding
input and serial offset, rejects missing/duplicate/reordered/extra descendants,
and captures only after both post-crash continuity observations. No new guest
authority, data, device access, or host capability is introduced.

## Validation if accepted

- Immutable fixtures prove A–D selections exclude the E-minimum GUI row.
- B, C, and D selections begin their new milestone rows with `none`; A retains
  the sole pre-E `continue` controller action, and key/pointer inputs begin at E.
- E–G selections include the row in its original post-restart position.
- Exact minimum buckets are A:5/B:7/C:11/D:7/E:7/F:7/G:1 and exact cumulative
  selections are A:5/B:12/C:23/D:30/E:37/F:44/G:45; the controller, verifier,
  fixtures, B–G execution map, and status assertions must update together.
- C still requires restart complete and peer responsive.
- E–G reject missing, early, stale, duplicated, or reordered GUI continuity.
- The v2 plan has exactly 45 unique rows and differs from v1 only in the three
  pre-E input fields, one minimum-milestone field, and schema/version
  documentation; byte-diff fixtures reject every other change.
- Controller, verifier, profile, retained evidence, and documentation bind the
  reviewed v2 digest.
- After activation, every new A–G probe rejects v1; v1 is retained only as
  immutable historical evidence from runs that predate the cutover.

## Gate-report compatibility and migration

The current read-only Sprint gate report remains schema v1 and must not gain
fields silently. If this ADR is accepted, its implementation creates gate-
report schema v2 with explicit `adr_0025`, `acceptance_protocol_v2`, and
`milestone_b_readiness` fields. V2 reports the protocol as
`reviewed-implementation-required` after decision acceptance and cannot report
Milestone B ready until the exact reviewed protocol/controller/verifier cutover
is complete and prior Milestone A evidence exists.

Gate-report v1 remains immutable historical orientation output. After the v2
cutover, new coordinating tools bind v2 exactly; they do not reinterpret v1 or
accept missing fields as ready. Strict local/remote preflight evidence remains
separate and unchanged.

## Replacement path

The Alpha evidence protocol is experimental. Production fault-containment and
UI-availability evidence will use later reviewed scenario versions appropriate
to the production component graph. No v1 or v2 marker becomes a stable target
ABI.
