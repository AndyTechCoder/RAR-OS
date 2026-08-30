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

`controller-helper-closure-verification-v0.fields` defines the separate
source-only exact-set verifier required after observation. It has no candidate
manifest, reviewed verifier-tool pin file, runtime evidence, or execution
authority and is not wired to automation. Its staged source requires exact
read-only candidate inputs, independently regenerates the complete closure
twice, rejects topology aliases and mutation, and can emit only
`candidate-exact-set-verified-not-reviewed-not-ready`. Even that future
receipt cannot update a lock, inventory, controller, profile, gate, build, or
readiness state; provenance, licenses, acquisition, tool pins, retained bytes,
and runtime evidence still require separate review.

`controller-helper-closure-verifier-test-plan-v0.fields` records an initial,
explicitly incomplete controller-owned validation, mutation, and confinement
design. It contains no harness or execution authority. Exact error/precedence
coverage, injected runtime faults, canonical evidence/verdict schemas, fixtures,
controller source, and wiring remain required separate reviews. Static checks
bind only these draft bytes and confirm that no directly named future harness or
plan reference is wired; they do not claim transitive workflow-call proof.

`controller-helper-closure-verifier-validation-v0.fields` and its error and
precedence catalogs bind the inactive verifier's exact source hash, 32 ordered
control-flow stages, 127 byte-exact representable error templates, and 50
class-located source-order relationships. Shared verifier source sites name each
possible occurrence stage, and the adjacent-stage prefix is checked against the
stage list rather than merely counted. Dynamic diagnostic bytes use one-pass
opaque hex-decoded substitution; inputs outside the stated diagnostic domain
remain an explicit coverage gap. The source-order table does not claim that
every defensive guard is fixture-reachable or define executable dual-invalid
oracles; that separate contract remains required. Digest syntax and receipt
shape guards are source-proven, while a freshly hashed all-zero SHA-256 value is
recorded as residual cryptographic risk rather than a practical fixture. The
catalogs deliberately exclude injected
command, read, write, close, tool-output, and resource-exhaustion failures;
the input domain described below is now present, while those failures still
require a separate reviewed fault contract. No catalog is a test result,
controller implementation, workflow permission, or readiness evidence.

`controller-helper-closure-verifier-input-domain-v0.fields` fixes the future
controller-owned environment, mount, file, mutation-target, diagnostic-token,
and case-isolation domains against the exact inactive verifier and validation
catalog bytes. It requires opaque one-pass token substitution, a fresh bounded
private fixture per case, explicit treatment of unrepresentable inputs, and no
ambient host state. It creates no fixtures, cases, fault injection, controller,
workflow, or execution authority; explicit constructible cases and fault
behavior remain separate blocking contracts.

`controller-helper-closure-verifier-case-dispositions-v0` classifies all 147
class-and-occurrence-stage pairs exactly once. It separates ordinary pre-start
fixtures, phase-synchronized controller mutations, separately reviewed injected
faults, validation-declared source proofs, additional source-dominated guards,
the fresh SHA-256 zero-digest residual, and four occurrences needing reviewed
domain extensions: input aliasing, unrelated mount-info mutation, and a
cross-device mount during either closure pass. Lossy NUL-bearing
tool-pin and observation behavior are assigned to the future fault contract
rather than silently excluded. The catalog is not an executable case instance,
runtime oracle, harness, result, or authority.

For every future case, verifier scratch becomes controller-inaccessible before
launch: no retained path, descriptor, backing handle, mount-namespace entry,
`proc`/ptrace write, primary mutation, or repair may reach `/tmp` or the
scratch subtree. Scratch syscall, tool, and output failures belong only to the
separately reviewed fault contract.

`controller-helper-closure-verifier-case-templates-v0` enumerates one structural
base-relative primary token, phase window, repair token, and error oracle for
each of the 125 currently constructible class-and-stage occurrences. Its base
fixture and operator/repair semantics are intentionally absent, so the tokens
are opaque identifiers and these are inactive templates rather than executable
instances, fixtures, results, or acceptance evidence. A reviewed versioned
operator/repair semantics contract, immutable base descriptor, fixture-image
identity, controller, phase instrumentation, fault contract, evidence/verdict
contracts, and exact-main validation remain blocking prerequisites.

`controller-helper-closure-verifier-operator-inventory-v0` closes the lexical vocabulary used by those templates to 34 primary families and eight repair tokens.
It checks parameter shapes and exact coverage mechanically, but does
not decode or apply a mutation. Exact targets, preconditions, postconditions,
deterministic derivations, resource feasibility, repair independence, and the
base fixture remain separate blocking semantics work.

`controller-helper-closure-verifier-scalar-semantics-v0` defines raw-byte,
line, UID, offset, and deterministic alternate-digest behavior for nine
complete families plus the existing-target `hex` and decimal-literal UID
subdomains, covering 72 templates. Link count and the two absent-file creation
cases are explicitly deferred because they require filesystem-structural
semantics. UID mutation is narrowed to the exact 1000-to-1001 fixture change;
manifest byte and entry increments use explicit mathematical ranges and
checked unbounded arithmetic. The slice fails closed on decoding, bounds, target type, and
postcondition errors. It defines no filesystem-structural operation, repair,
fixture, controller, runtime result, or execution authority.

`controller-helper-closure-verifier-basic-filesystem-semantics-v0` adds exact
create, remove, same-device fresh-inode replacement, and owner-execute mode
semantics for ten more templates. It pins ownership and metadata capture while
deferring symlink, hardlink, raw-name, mount, tree, manifest-specific, and
repair behavior. It remains source-only and cannot create or mutate a fixture.

`controller-helper-closure-verifier-scalar-repair-semantics-v0` defines the
reviewed-tool environment-digest repair used by five templates. Four UID
primaries correctly require no repair: the byte-pinned base already supplies
their comparison context. The contract remains inactive and grants no mutation,
repair, or execution path.

`controller-helper-closure-verifier-synchronized-link-semantics-v0` defines
post-trigger hardlink insertion and atomic regular-file-to-self-symlink
replacement for six no-repair closure templates. It requires exact trigger and
ack ordering, nonscratch private backing, complete parent/entry metadata, and
no visible staging residue. Pre-start and repair-coupled link cases remain
blocked.

`controller-helper-closure-verifier-observation-repair-semantics-v0` defines
the three observation-receipt repair tokens used by eight structural templates.
It derives exact manifest digest, LF-record-count, and raw-byte-count fields
from the recorded post-primary manifest bytes while preserving every
untargeted receipt field and the primary manifest. The contract is inactive,
source-only, and grants no repair, controller, workflow, or execution authority.

`controller-helper-closure-verifier-rebuild-observation-canonical-semantics-v0`
defines the canonical whole-receipt rebuild used by three scalar field-primary
templates. It preserves exactly one deliberately mismatched manifest field,
proves the two untargeted manifest fields against the unchanged manifest, and
inherits the pinned fail-closed receipt stability boundary. It remains inactive
and grants no repair, controller, workflow, or execution authority.

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
