# Sprint Alpha ADR 0024 Controller/Helper Integration

Status: Authoritative preparation packet - execution remains phase-gated

## Objective

Close accepted ADR 0024 Alternative A before Milestone A: independently
observe and verify the complete Linux compiler closure, implement the
descriptor-only controller helper and recovery controller, reproduce the
helper twice in isolated cloud jobs, retain adversarial evidence, and bind only
reviewed identities into the trusted default-branch controller.

This packet authorizes no RAR target build, image, VM, boot, launch, or Mac
execution. It does not make the Development Lab ready and grants no ADR 0030
publication/recovery authority.

## Fixed authority

- Alternative A means a runner closure, two byte-identical builds, isolated
  Linux tests, zero dependencies, and no target linkage.
- ADRs 0017 and 0020 keep controller, build, reference, and launch authority
  separate.
- The helper receives only sealed controller-selected filesystem descriptors.
  It cannot acquire roots by path or gain process, network, container, cloud,
  credential, GitHub, target-launch, or publication authority.
- Observation and verification are exact-main controller operations. Candidate
  evidence is never readiness evidence.
- Local work is documentation, specification, static policy, hashes, and source
  inspection only. Runtime tests and compiler use are cloud-only.

Every phase starts from its predecessor's exact reviewed green merge. One writer
owns only named paths. Architecture, correctness, and security reviews are
required for each contract, workflow, executable, unsafe boundary, or trust
state change. Red checks, stale revisions, missing identities, mutable inputs,
or unresolved findings stop progression.

## D0 - Packet registration

Owned paths:

- `docs/tasks/sprint-alpha-controller-helper-integration.md`
- `docs/README.md`
- packet-only assertions in `tools/ci/check-specs.sh`

D0 adds no executable path, workflow, identity, format authority, or readiness.
The checker byte-pins this packet.

## C1 - Contract closure before wiring

Owned paths are limited to these new experimental host contracts, matching
source-only checkers, `spec/alpha/lab/README.md`, and minimum checker
registration:

- `spec/alpha/lab/controller-helper-runtime-v0.fields`
- `spec/alpha/lab/controller-helper-runtime-cases.v0`
- `spec/alpha/lab/controller-helper-closure-observer-test-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-faults-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-cases-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields`
- `spec/alpha/lab/controller-helper-closure-acceptance-v0.fields`
- `spec/alpha/lab/controller-helper-test-evidence-v1.fields`
- exact matching `tools/ci/check-*-source.sh` files

The runtime contract fixes a sealed inherited-FD table, canonical basenames,
`O_NOFOLLOW|O_EXCL|O_CLOEXEC`, UID/GID/mode/link/device/inode attestation,
exact-binary descriptor execution without shell parsing, non-reusable process
handles, parent stop tokens, bounded monotonic watchdog, forced termination and
reap, durable hash-chained journal transitions, restart/takeover, fail-closed
cleanup uncertainty, and attempt ceilings. Source roots are never deleted.

The observer gains runtime negative, mutation, fault, and confinement coverage.
Verifier contracts complete every constructible occurrence, dual-invalid
precedence oracle, phase mutation, injected command/read/write/close/tool/
resource failure, immutable fixture identity, canonical evidence, normalized
verdict, replay binding, and no-success-effect rules.

Versioned helper evidence covers the 97 attempt cases plus process/FD failures
and every durable-boundary fault. The legacy 13 cases cannot prove readiness
after runtime behavior exists.

A closure-acceptance record binds observation and verification receipts,
complete manifest, tool pins, compiler identity, licenses, provenance,
authenticated acquisition, retained bytes, source revision, and controller
revision. Mechanical exact-set equality grants no compiler-use authority.

C1 is source-only: no workflow, harness, compiler use, or ready identity.

## C2 - Observer discovery

Owned paths are one trusted-main observer workflow, one bounded controller
wrapper, its policy/runtime harness under `tools/ci/`, and controller-owned
fixtures under `spec/alpha/lab/fixtures/`.

The wrapper enforces canonical main push, exact controller/source revision,
pinned read-only OCI, no network or credentials, non-root, no capabilities,
bounded resources/output, exact mounts, and one observer process. The full
negative, mutation, fault, and confinement suite passes before observation.

The manifest and 23-line receipt remain
`observed-not-reviewed-not-ready`. C2 compiles or executes no helper or target
and changes no lock, inventory, profile, gate report, or readiness.

## C3 - Exact-set verification and closure acceptance

Owned paths are one trusted-main verifier workflow, bounded controller/harness
and tests under `tools/ci/`, reviewed pins/fixtures under
`spec/alpha/lab/fixtures/`, and one closure-acceptance instance under
`tools/toolchain/`.

Observation receipts bind current exact main. C3 therefore re-runs the observer
at the exact C3 merge revision immediately before verification; an earlier C2
receipt cannot be reused.

The verifier performs two stable topology/manifest passes and the full success,
mutation, fault, precedence, resource, and confinement matrix. It retains the
canonical 31-line receipt and case evidence. Separate architecture,
correctness, security, license, provenance, and acquisition review binds the
closure-acceptance record. Only that record authorizes later compiler use. C3
compiles or executes no helper or target.

## H1 - Descriptor-only helper source

Owned paths are exact entrypoint/contextual runtime modules under
`tools/rar-lab/controller-handoff/`, existing files explicitly named by the
H1 child packet, and focused source checks/documentation.

The dependency-free host helper accepts only the sealed descriptor protocol and
implements bounded stop/open/copy/recheck plus watchdog/journal recovery. It has
no ambient path, shell, network, cloud, credential, GitHub, publication, or
target authority. H1 remains source-only until C3 acceptance merges. No local
compilation or execution is allowed.

## H2 - Reproducible cloud evidence

The exact-main controller verifies C3 closure acceptance, then makes two
distinct fresh bounded network-disabled builds from identical controller SHA,
source, plan, compiler, and closure. Producers stop before descriptor-safe copy.
Size, hash, and byte equality are mandatory.

A separate isolated Linux job runs the versioned helper suite: durable-boundary
kills, process/FD failures, watchdog timeout/cancel/kill/reap, restart/takeover,
stale/replay/cross-attempt rejection, cleanup uncertainty, attempt ceiling,
root/identity/alias rejection, authority denial, and output bounds.

Evidence binds distinct job/root nonces, exact source and closure acceptance,
controller-observed exits, canonical results, logs, both build receipts, test
evidence, and aggregate evidence. Failure emits no ready or success record.

## I1 - Bind helper without readying the Lab

Owned paths are `tools/sprint-alpha/controller-helper-v0.env`, exact runnable
v2 controller files named by an I1 child packet, and focused validators, tests,
and documentation.

Bind only real reviewed identities. `development-lab-v2.env` stays blocked
and `development-controller-v2.plan` stays activation-forbidden until
separate role-image, compiler/linker, reference, QEMU, firmware, machine, QMP,
retained-machine-evidence, and SSD-confinement gates pass.

## I2 - No-target integration proof

On exact main, a fixed synthetic host-only fixture proves stop/open/copy/
recheck, termination at every durable transition, recovery/discard/block,
replay rejection, descriptor confinement, bounded failure, and no readiness or
publication after failure.

I2 performs no target build, link, image creation, VM launch, firmware load, or
RAR execution. Architecture, correctness, security, and exact-main validation
are mandatory.

## Stop conditions

Stop on changed contracts or role topology; helper authority expansion; mutable
or unauthenticated inputs; license/provenance/acquisition gaps; network,
credential, owner-data, target-source, container/cloud API exposure; local
helper or target execution; reused, aliased, or path-acquired roots; unequal
builds; incomplete fault/recovery evidence; premature ready state; source-branch
workflow edits; target build/VM/launch; or ADR 0030 scope.

Any expansion beyond ADR 0024 Alternative A requires a new ADR and owner
decision.
