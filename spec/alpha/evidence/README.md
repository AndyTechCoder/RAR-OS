# Experimental Alpha Acceptance Evidence

Status: experimental, Alpha-only, replaceable

`acceptance-v1.plan` is immutable historical evidence. Its SHA-256 is
`f7e66d58200272fc239283c42d16389584e5d647362e8623ac439b71d728ec1e`; every
new A–G probe rejects its schema after the v2 cutover.

`acceptance-v2.plan` is the reviewed ADR 0025 replacement selected by the
trusted Development Lab controller. Its SHA-256 is
`ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd`.
It retains all 45 ordered observations and changes only the version, the first
B/C/D inputs to `none`, and the GUI-continuity minimum from C to E. The B–D
rows therefore auto-chain from preceding results without inventing pre-GUI
input authority. The GUI marker remains physical row 23, directly after the
component-restart marker.

Each plan row selects the first milestone that
requires an observation, the input that triggers it, one exact complete serial
trace line, an evidence label, and whether a framebuffer capture is required.
Later milestones prove all earlier rows. `none` means that the preceding input
must emit the next ordered result; it is not permission to accept a pre-existing
line. Pointer coordinates and button state are fixed and transcripted.

The preserved-data fixture is exactly the three bytes `abc`; both its pre- and
post-recovery SHA-256 value are fixed in the plan. Milestone F states are
separate and ordered so activation, rejection-before-execution, replacement,
failed health, rollback, and unaffected-component continuity cannot collapse
into one generic success marker. A missing, duplicated, reordered, stale, or
extra observation fails evidence validation.

`acceptance-v2.fields`, `acceptance-v2-cases.v0`, and
`tools/ci/check-acceptance-v2.sh` bind the schema, exact identity, bucket
counts, negative cases, and historical rejection. Activation remains
fail-closed pending review and merge.

The v2 action transcript is a bounded, canonical, LF-terminated eight-field
record. Capture rows carry the capture flag and exact image digest after the
selected marker, so retained evidence can reject stale, substituted, or
pre-marker screenshots. Immutable cumulative-selection digests cover A–G.

`accepted-evidence-v0.fields` defines the separate controller-owned phase-8
anti-replay record. It binds one fresh attempt and probe to the exact controller,
source, artifact, protocol, machine profile, emulator/firmware/QMP identities,
handoff/reference/inventory identities, and recomputed output digests. A record
or output cannot be accepted alone; every mismatch or downgrade rejects.

The durable phase-8 writer remains blocked on the publication and recovery
choice documented in `docs/proposals/0030-alpha-accepted-evidence-publication-recovery.md`.
No final/temporary naming, commit, cleanup, retry, or uncertain-state behavior
is authoritative until that proposal is accepted and integrated through a
reviewed contract change.

This protocol is acceptance evidence, not a stable target ABI. Changing it
requires an ADR-governed trusted-controller review before implementation code
may rely on the replacement.
