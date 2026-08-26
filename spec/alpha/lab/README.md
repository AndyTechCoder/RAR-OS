# Experimental Alpha Development Lab Isolation v0

Status: accepted architecture, inactive until real reviewed identities exist

These contracts implement ADR 0020's three-role cloud boundary without making
the cloud service part of RAR OS:

1. The **build role** receives the read-only source checkout plus only the
   pinned compiler and linker. It emits one bounded unsigned image and one
   bounded comparison request transcript. It cannot see or execute reference
   tools, QEMU, firmware, the launch profile, credentials, or a network.
2. The **reference role** receives only the canonical transcript. It has two
   independently pinned cryptographic references, recomputes every request,
   compares both references with the target result, and emits bounded evidence.
   It receives no source checkout, target image, compiler, linker, QEMU,
   firmware, launch authority, credentials, or network.
3. The **launch role** receives only the frozen image and trusted controller.
   It has QEMU, firmware, the fixed machine profile, and the QMP client. It has
   no source, compiler, linker, or cryptographic reference.

The trusted controller validates the three distinct image identities, role
inventories, frozen artifact, transcript, comparison evidence, and launch
evidence. Target-build output cannot declare itself verified. Missing,
duplicated, reordered, malformed, oversized, unknown-critical, or mismatched
comparison records fail before signing evidence can pass.

`controller-state-machine-v0.fields` fixes the controller order before any
runnable v2 controller exists. Two fresh build roles run first; the trusted
controller freezes only identical bounded outputs; the isolated reference role
runs only for Milestones F/G; the launch role receives only the frozen artifact;
and controller-owned verification precedes bounded retention. At most one role
container may run, and the controller tree never enters build or reference.

`reference-evidence-v0.fields` fixes the bounded binary evidence emitted by the
isolated reference role and the 13-line controller-owned verdict. F/G verdicts
require both references and the target output to match for every transcript
record. A–E verdicts explicitly say `not-required`, carry zero reference/evidence
digests, and prove that no reference role ran. The final evidence set retains the
verdict and its digest.

The v1 Lab, image, and crypto inventory files remain permanently blocked. The
v2 field schemas define the replacement shape but do not contain runnable image
digests or authorize provisioning. A candidate instance becomes `ready` only in
a separately reviewed change with real immutable identities and two-build image
reproduction evidence.

This is Development Lab evidence, not production trust, certification, or a
target dependency. No file in this directory links into or ships with RAR OS.
