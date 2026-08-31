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

Every phase starts from its predecessor's exact reviewed green merge and a
distinct successful exact-main Specifications validation and mutation run. Red
checks, stale revisions, missing identities, mutable inputs, or unresolved
findings stop progression.

D0 is the only phase that grants paths directly. Every later description below
is scope guidance, not ownership. Before C1, C2, C3V, C3A, H1, H1C, H2, I1, or I2
starts, a separately reviewed child packet must list every exact path, with no
glob or directory ownership, and must itself merge and pass distinct exact-main
validation. Architecture, correctness, and security reviews are required for
each contract, workflow, executable, unsafe boundary, or trust-state change.

## D0 - Packet registration

Owned paths:

- `docs/tasks/sprint-alpha-controller-helper-integration.md`
- `docs/README.md`
- packet-only assertions in `tools/ci/check-specs.sh`

D0 adds no executable path, workflow, identity, format authority, or readiness.
The checker byte-pins this packet. Because D0 changes the trusted checker, its
pull-request validation is intentionally deferred. C1 is forbidden until D0 is
merged and a distinct exact-main run completes both validation and mutation
successfully.

## C1 - Contract closure before wiring

The C1 child packet may select these new experimental host contracts, matching
source-only checkers, `spec/alpha/lab/README.md`, and minimum checker
registration, but it must name every selected path literally:

- `spec/alpha/lab/controller-helper-runtime-v0.fields`
- `spec/alpha/lab/controller-helper-runtime-cases.v0`
- `spec/alpha/lab/controller-helper-closure-observer-test-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-faults-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-cases-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields`
- `spec/alpha/lab/controller-helper-closure-acceptance-v0.fields`
- `spec/alpha/lab/controller-helper-test-evidence-v1.fields`
- `spec/alpha/lab/controller-helper-build-evidence-v1.fields`
- individually named matching source-only and contextual validator files under `tools/ci/`

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

Versioned helper test evidence covers the 97 attempt cases plus process/FD
failures and every durable-boundary fault. A versioned aggregate build-evidence
contract and contextual validator must consume that exact v1 test evidence and
closure-acceptance identity. The legacy v0 aggregate and 13-case evidence cannot
prove readiness after runtime behavior exists.

A closure-acceptance record binds observation and verification receipts,
complete manifest, tool pins, compiler identity, licenses, provenance,
authenticated acquisition, retained bytes, source revision, controller
revision, and the complete observer/verifier runtime fault verdict and evidence.
Mechanical exact-set equality grants no compiler-use authority.

C1 is source-only: no workflow, harness, compiler use, or ready identity.

## C2 - Observer discovery

The C2 child packet must literally name its single trusted-main observer
workflow, bounded controller wrapper, policy/runtime harness files, and every
controller-owned fixture. This paragraph grants no directory ownership.

The wrapper enforces canonical main push, exact controller/source revision,
pinned read-only OCI, no network or credentials, non-root, no capabilities,
bounded resources/output, exact mounts, and one observer process. The full
negative, mutation, fault, and confinement suite passes before observation.

The manifest and 23-line receipt remain
`observed-not-reviewed-not-ready`. C2 compiles or executes no helper or target
and changes no lock, inventory, profile, gate report, or readiness.

## C3V - Exact-set verification execution

The C3V child packet must literally name its single trusted-main verifier
workflow, bounded controller/harness and test files, and every reviewed pin and
fixture. It must not contain or modify a closure-acceptance instance.

Observation receipts bind current exact main. C3V therefore re-runs the
observer at the exact C3V merge revision immediately before verification; an
earlier C2 receipt cannot be reused.

The verifier performs two stable topology/manifest passes and the full success,
mutation, fault, precedence, resource, and confinement matrix. It retains the
canonical 31-line receipt, normalized verdict, and complete case evidence. C3V
compiles or executes no helper or target and grants no compiler-use authority.

## C3A - Closure-acceptance integration

Only after the C3V exact-main evidence exists may a separate C3A child packet
literally name the closure-acceptance instance, every retained C3V input, its
validator, and documentation. Architecture, correctness, security, license,
provenance, and acquisition review must bind the exact C3V revision, observer
and verifier receipts, manifest, tool/compiler pins, retained bytes, and runtime
fault verdict/evidence.

C3A merges separately and must pass distinct exact-main validation and mutation.
Only that merged record authorizes later compiler use. H1 and H1C are forbidden
before C3A closure.

## H1 - Descriptor-only helper source

The H1 child packet must literally name every entrypoint, contextual runtime
module, existing touched file, source checker, and documentation file. This
paragraph grants no directory ownership.

The dependency-free host helper accepts only the sealed descriptor protocol and
implements bounded stop/open/copy/recheck plus watchdog/journal recovery. It has
no ambient path, shell, network, cloud, credential, GitHub, publication, or
target authority. H1 remains source-only until C3A acceptance merges. No local
compilation or execution is allowed.

## H1C - Trusted build/test controller source

Before H2, a separate child packet must literally name and implement the
trusted-main build/test controller, its one workflow, harness, static policy,
fixtures, and refusal tests. H1C is source-only: it cannot invoke a compiler,
execute the helper, run a container, or produce build/test evidence.

H1C requires architecture, correctness, and security review, merge, and a
distinct exact-main validation and mutation run. Only the resulting trusted
exact-main controller may execute H2.

## H2 - Reproducible cloud evidence

The H2 child packet must literally name every immutable input, evidence
destination, validator, and documentation path it adds or consumes. It cannot
change any H1C executable-authority byte, including its workflow, wrapper,
controller, harness, static policy, or refusal tests. The exact-main H1C
controller verifies C3A closure acceptance, then makes two
distinct fresh bounded network-disabled builds from identical controller SHA,
source, plan, compiler, and closure. Producers stop before descriptor-safe copy.
Size, hash, and byte equality are mandatory.

A separate isolated Linux job runs the versioned helper suite: durable-boundary
kills, process/FD failures, watchdog timeout/cancel/kill/reap, restart/takeover,
stale/replay/cross-attempt rejection, cleanup uncertainty, attempt ceiling,
root/identity/alias rejection, authority denial, and output bounds.

Evidence binds distinct job/root nonces, exact source, C3A closure acceptance, and versioned aggregate/test evidence,
controller-observed exits, canonical results, logs, both build receipts, test
evidence, and aggregate evidence. Failure emits no ready or success record.

## I1 - Bind helper without readying the Lab

The I1 child packet must literally name `tools/sprint-alpha/controller-helper-v0.env`,
every runnable v2 controller file, and each validator, test, and documentation
file. This paragraph grants no unnamed ownership.

Bind only real reviewed identities. `development-lab-v2.env` stays blocked
and `development-controller-v2.plan` stays activation-forbidden until
separate role-image, compiler/linker, reference, QEMU, firmware, machine, QMP,
retained-machine-evidence, and SSD-confinement gates pass.

## I2 - No-target integration proof

The I2 child packet must literally name every fixed synthetic fixture, harness,
validator, evidence, and documentation path. On exact main, that fixture proves
stop/open/copy/
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
