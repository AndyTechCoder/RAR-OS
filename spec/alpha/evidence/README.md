# Experimental Alpha Acceptance Evidence v1

Status: experimental, Alpha-only, replaceable

`acceptance-v1.plan` is the exact ordered guest-observation protocol used by the
trusted Development Lab controller. Each row selects the first milestone that
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

This protocol is acceptance evidence, not a stable target ABI. Changing it
requires an ADR-governed trusted-controller review before implementation code
may rely on the replacement.
