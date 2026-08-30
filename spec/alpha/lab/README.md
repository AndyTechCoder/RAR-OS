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

`controller-handoff-v0.fields` fixes the host-only stop/open/copy/recheck
primitive used between isolated roles. The controller opens an exact basename
relative to its own directory descriptor without following links, copies only
from and to already-open descriptors, hashes the exact bytes copied, and
rechecks the same source descriptor before publishing a manifest. Its negative
case table makes races, aliases, wrong ownership, extra outputs, and partial
copies fail before the next role can start. The manifest is a fixed 256-byte
experimental host record, validated and durably synchronized before progression.
Destination files use one read/write descriptor so the controller can seek and
rehash the exact copied bytes without reopening a path. Output ordinals are
fixed by output kind or the ordered launch allowlist. Failure cleanup may remove
only identity-matched destination and manifest files created by that attempt;
any cleanup uncertainty permanently blocks the next role.

`controller-handoff-attempt-v0.fields` fixes the persistent outer-controller
journal needed when the helper is terminated or the controller restarts. A
durable exclusive active marker binds the task, controller, helper, roots,
expected outputs, and watchdog. Fixed hash-chained transition records prevent
missing, reordered, duplicated, or forked state. Recovery first persists a
bounded descriptor-derived inventory, never deletes source roots, and removes
only identity-matched entries from exclusive attempt-local destination and
manifest roots. Durable `discarded` permits only a fresh attempt; `blocked`
permanently prevents progression. Journal bytes contain no paths, commands,
credentials, URLs, or cloud authority and cannot locate roots by themselves.

`controller-helper-inventory-v0.fields` and its build-evidence contract define
the identities required to turn that primitive into a trusted Linux helper.
ADR 0024 Alternative A is accepted, but the checked instance remains
non-activating: it contains no compiler, builder, source, binary, or evidence
identity and cannot become ready. The selected path must reproduce the same
bounded helper twice and bind isolated test evidence before controller activation.

`controller-helper-closure-observation-v0.fields` defines a source-only,
inactive observer for a future separately authorized cloud run. The observer is
not wired to automation, cannot compile or execute the helper or target, and
cannot update any lock, inventory, profile, gate report, or readiness state. Its
only permitted future output is a candidate closure manifest plus an
`observed-not-reviewed-not-ready` receipt; both still require exact-set review
and pinning before any helper build or test can be considered.

`controller-helper-build-receipt-v0.fields` and
`controller-helper-test-evidence-v0.fields` fix controller-owned receipts for
two separately terminated build jobs and one thirteen-case test job. Contextual
validators require distinct job/root nonces, non-aliased single-link output
copies, exact runner/source/compiler/log identities, controller-observed exits,
and canonical per-case results. They accept only files confined beneath a
reviewed controller-owned root after the producer has stopped; they are not an
untrusted live-path or concurrent-mutation boundary. The checked fixtures are
deliberately tiny synthetic text and prove only parser and policy behavior;
they are not compiler, helper, cloud, or activation evidence.

The v1 Lab, image, and crypto inventory files remain permanently blocked. The
v2 field schemas define the replacement shape but do not contain runnable image
digests or authorize provisioning. A candidate instance becomes `ready` only in
a separately reviewed change with real immutable identities and two-build image
reproduction evidence.

This is Development Lab evidence, not production trust, certification, or a
target dependency. No file in this directory links into or ships with RAR OS.
