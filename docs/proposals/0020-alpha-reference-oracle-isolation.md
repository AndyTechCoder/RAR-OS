# ADR 0020: Alpha Reference-Oracle Isolation

Status: Proposed — owner decision required
Decision: Undecided

No candidate provisioner or workflow is included while this decision remains
open. The build-image recipe preserves ADR 0017's compiler/linker-only boundary;
adding any oracle execution topology requires an accepted decision and a new
independent review.

## Context

ADR 0017 permits untrusted target build code to receive only the reviewed
compiler/linker identities. ADR 0019 later requires two host-only cryptographic
references to exist in the network-disabled build image. The current controller
passes their executable paths into the same container that executes the
untrusted target build driver. Even after headers and static libraries are
removed, that makes the comparison oracles readable and executable by source
code and conflicts with ADR 0017's stronger separation promise.

This cannot be resolved as a routine implementation detail because moving or
restricting the oracles changes a Development Lab trust boundary and determines
which party owns cryptographic comparison evidence.

## Decision drivers

- Untrusted target source must not read, execute, replace, or link a reference
  oracle.
- The trusted controller must still compare bounded target results with both
  pinned independent references before Milestone F can pass.
- Reference binaries, inputs, outputs, and versions must remain reproducible and
  reviewable without entering a target image.
- The build and launch roles must remain independently replaceable.

## Alternatives

### A. Keep both oracles in the untrusted build container

The target build driver can invoke comparisons directly. This is the smallest
change, but it weakens ADR 0017 and lets untrusted source inspect or misuse the
oracles. Selecting it requires explicitly amending that accepted boundary.

### B. Keep root-only oracles in the build image and run a trusted preflight

The untrusted UID cannot read or execute the binaries, while a separate
capability-dropped root container can verify versions and hashes. This preserves
basic tool isolation but does not by itself compare target-produced vectors; a
new controller-owned result handoff is still required.

### C. Add a distinct reference image and controller-owned comparison phase

The untrusted build image contains compiler/linker tools only and emits a
bounded, canonical vector/result transcript beside the target artifact. A
separate digest-pinned reference image receives no source checkout or target
execution authority, recomputes the transcript with both oracles, and emits
comparison evidence. The launch image remains reference- and compiler-free.
This most closely preserves ADR 0017, at the cost of a fourth image, a new
experimental transcript contract, and additional retained evidence.

## Proposed direction

Alternative C is proposed. No implementation, image provisioning, profile
activation, or Milestone F work may assume that choice until the owner accepts
this ADR or selects another alternative.

## Consequences if accepted

- The image-input and Development Lab profiles gain a separately pinned
  reference-image identity.
- The Alpha build driver emits a bounded experimental comparison transcript but
  cannot invoke reference tools.
- Trusted controller code owns reference execution and transcript comparison.
- Candidate provisioning proves build/reference/launch role absence rules and
  retains all three inventories, licenses, hashes, and exact publishable bytes.
- No reference code or transcript parser links into RAR OS.

## Validation

- A malicious build driver cannot discover, read, execute, or link either
  reference binary or library.
- Reference execution receives no source mount, target launch authority,
  network, credentials, or writable controller files.
- Missing, reordered, duplicated, oversized, malformed, or mismatched transcript
  entries fail Milestone F before signing evidence is accepted.
- Image inventories prove compiler/linker-only build, oracle-only reference, and
  QEMU/firmware/QMP-only launch roles.
- Two independent candidate builds reproduce every image and named binary.

## Replacement path

The experimental transcript and reference image can be replaced by a later
certified cryptographic validation service through another accepted ADR. Target
formats and algorithms are unaffected.
