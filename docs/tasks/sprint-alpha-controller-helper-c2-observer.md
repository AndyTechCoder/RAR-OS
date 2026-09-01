# Sprint Alpha ADR 0024 C2 Observer Discovery

Status: Authoritative source-only C2 child packet - implementation requires exact-main validation

## Purpose

Register only the contract-first activation and later trusted-main
compiler-closure observer workflow required by C2. The bounded controller,
complete O001-O021 harness, versioned outer evidence, and retained candidate
bytes remain separate from compiler/helper/target authority. This packet itself grants no workflow execution,
compiler use, helper or target execution, Lab readiness, VM launch, local
execution, or ADR 0030 authority.

C2 has two mandatory, separately merged subphases. C2A corrects the observer-test
phase contract and freezes all runtime evidence formats before any workflow
exists. C2B may wire and execute the observer only after C2A merges and a
distinct exact-main Specifications run completes validation and mutation.
Neither subphase may be combined with C3V, C3A, helper, target, or readiness
work.

## C2A exact owned paths - contract activation only

The C2A writer owns only these paths:

- `spec/alpha/lab/controller-helper-closure-observer-test-v0.fields`
- `spec/alpha/lab/controller-helper-closure-observer-run-evidence-v0.fields`
- `tools/ci/contracts/controller-helper-closure-observer-case-evidence-v0.fields`
- `tools/ci/check-controller-helper-closure-observer-test-source.sh`
- `tools/ci/check-controller-helper-closure-observer-run-evidence-source.sh`
- `tools/ci/verify-controller-helper-closure-observer-run-evidence.sh`
- `tools/ci/test-controller-helper-closure-observer-run-evidence-policy.sh`
- `tools/ci/fixtures/controller-helper-closure-observer/run-evidence-valid.v0`
- `tools/ci/fixtures/controller-helper-closure-observer/run-evidence-malformed.v0`
- `tools/ci/fixtures/controller-helper-closure-observer/run-evidence-cases.v0`
- `tools/ci/check-alpha-preimplementation-contracts.sh`
- `tools/ci/check-sprint-static.sh`
- `tools/ci/check-specs.sh`
- `spec/alpha/lab/README.md`
- `docs/sprint-alpha-dashboard.md`
- `SPRINT_STATUS.md`

C2A changes `execution_authority=none-until-C3V` only to the narrow,
main-only C2 observer/harness authority defined here. That authority permits
O001-O021 harness execution and one candidate observation; it still denies
compiler invocation, helper/target execution, lock/profile/inventory/gate
updates, readiness, and C3V/C3A acceptance. C2A adds no workflow, wrapper,
runtime harness, executable observation, candidate identity, or retained
evidence. Architecture, correctness, and security review its exact head; it
must merge and pass a distinct exact-main validation and mutation run before
C2B starts.

## C2B exact owned paths - observer workflow and evidence

After C2A closes, the C2B writer owns only these paths:

- `.github/workflows/controller-helper-closure-observer.yml`
- `tools/ci/run-controller-helper-closure-observer.sh`
- `tools/ci/observe-controller-helper-closure.sh`
- `tools/ci/check-controller-helper-closure-observer-source.sh`
- `tools/ci/check-controller-helper-closure-observer-policy.sh`
- `tools/ci/test-controller-helper-closure-observer-policy.sh`
- `tools/ci/controller-helper-closure-observer-harness.sh`
- `tools/ci/fixtures/controller-helper-closure-observer/base-closure.v0`
- `tools/ci/fixtures/controller-helper-closure-observer/tool-pins.v0`
- `tools/ci/fixtures/controller-helper-closure-observer/cases.v0`
- `tools/ci/fixtures/controller-helper-closure-observer/expected-observation.receipt.v0`
- `tools/toolchain/class-b-host-tools.v1`
- `tools/ci/check-alpha-preimplementation-contracts.sh`
- `tools/ci/check-sprint-static.sh`
- `tools/ci/check-specs.sh`
- `spec/alpha/lab/README.md`
- `tools/toolchain/README.md`
- `docs/sprint-alpha-dashboard.md`
- `SPRINT_STATUS.md`

No glob or directory ownership is granted. The controller integration packet,
C1 schemas not explicitly named by C2A, existing Specifications workflow,
helper source, inventory instance, tool locks, Lab profile, controller plan,
target source, and every other path are read-only dependencies. A need to touch
any unnamed path stops the active subphase and requires a separately reviewed
packet amendment.

## Versioned outer evidence boundary

C2A must freeze a canonical, bounded
`rar-alpha-controller-helper-closure-observer-run-evidence-v0` record before
C2B. It binds schema/status, repository, ref, event, exact controller/source
SHA, run ID/attempt, runner image/OS/architecture, OCI digest, wrapper/subject/
fixture/tool-pin/case-evidence/manifest/receipt hashes, artifact name,
retention days, observed exit, byte counts, and a normalized
`candidate-not-reviewed-not-ready` verdict.

The schema fixes ASCII/LF encoding, field order, exact line and byte ceilings,
nonzero lowercase digests, canonical decimals, exact-main equality, distinct
run attempt identity, producer/validator separation, output-set equality, and
anti-replay rules. The independent contextual validator and mutation corpus
reject missing, extra, reordered, malformed, stale, cross-revision,
self-attested, zero, aliased, oversized, wrong-output, wrong-artifact, and
ready/success-substitution records. Valid and malformed fixtures are
byte-pinned. Mechanical validation never grants readiness.

## Trusted-main workflow boundary

The C2B workflow must:

- trigger only on a canonical `main` push; it has no pull-request,
  workflow-dispatch, repository-dispatch, schedule, or reusable-workflow entry;
- use `permissions: contents: read`, pinned actions, credential-free checkouts,
  exact `github.sha` controller/source equality, and the attested Ubuntu 24.04
  Linux X64 runner identity;
- run the policy/runtime O001-O021 suite before the real observation;
- launch exactly one production observer in the pinned Rust 1.95.0 OCI image
  with a read-only root and source, `--network none`, non-root identity,
  `no-new-privileges`, all capabilities dropped, fixed CPU/memory/PID limits,
  bounded noexec scratch/evidence mounts, and an empty allowlisted environment;
- produce exactly the candidate manifest, 23-line receipt, canonical O001-O021
  case evidence, and versioned outer run-evidence record.

Repository code, checkout steps, wrapper, harness, observer, validators, and
containers receive no secret, token, network, Docker socket, host path outside
the exact checkout and controller-owned scratch/evidence roots, or mutable
source mount. The sole bounded exception is the already pinned outer
`actions/upload-artifact` step after independent validation succeeds. It may
read only those four exact regular non-symlink evidence files, upload once
under `controller-helper-closure-observer-<run-id>-<run-attempt>`, use a fixed
14-day retention and no-overwrite behavior, and receive only GitHub's
action-scoped artifact transport authority. It cannot write repository,
workflow, release, lock, inventory, profile, gate, or readiness state.
Artifact retention is candidate evidence storage, not publication or
acceptance.

The workflow and wrapper reject stale or non-main revisions, unexpected
environment variables, missing/additional mounts, preexisting outputs,
unbounded output, and any attempt to reach network, credentials, compiler,
helper, target, profile, inventory, lock, gate, or publication state.

## Observer and test requirements

Production observation preserves the C1 observation contract exactly: it
enumerates the complete pinned Linux Rust compiler closure, emits only the
manifest and `observed-not-reviewed-not-ready` receipt, and never invokes
`rustc`, Cargo, a linker, helper, target tool, container client, network
client, cloud client, or credential source.

The harness executes O001-O021 exactly once in canonical order and emits the C1
case-evidence grammar with distinct nonces and roots. Synthetic fault controls
are unreachable from the production entry point. Any generated test subject is
mechanically bound to reviewed production source and may vary only explicitly
listed fixture roots, tool outcomes, and fault operations. Tests cover context,
revision, tool identity, topology, unsafe path, alias, output collision, bounds,
command/read/write/close/hash/enumeration failure, phase mutation, resource
failure, confinement, network, credential, and unexpected inherited descriptor
rejection.

No negative case may mutate the actual compiler closure, checkout, workflow,
host, lock, inventory, profile, gate report, or readiness state. Temporary case
roots are exclusive children of bounded cloud scratch; cleanup may remove only
those verified children and never source or retained evidence.

## C2B evidence and completion

C2B completion requires:

1. static policy proves exact triggers, action pins, mounts, resources,
   environment, commands, output set, and absence of activating authority;
2. all O001-O021 cases pass in the isolated cloud job before observation;
3. exact-main observation emits one bounded manifest and one 23-line receipt;
4. the independent validator accepts the one outer run-evidence record and
   mutation tests reject every invalid class;
5. the pinned outer action retains exactly the four validated evidence files;
6. architecture, correctness, and security review the exact implementation head;
7. merge occurs only with no blocking findings; distinct exact-main runs then
   pass Specifications validation/mutation and the C2 observer workflow.

The result remains a candidate. C2 does not update the compiler-closure lock,
helper inventory, Lab profile, controller plan, gate report, or readiness.
C3V re-observes and independently verifies the exact set at its own merge
revision. Only a later reviewed C3A acceptance record may authorize compiler use.

## Local and target safety

The Mac and SSD receive source storage only. No C2 compiler, observer, harness,
container, helper, target, firmware, VM, or RAR OS artifact may be compiled,
executed, or launched locally. C2 compiles and executes no helper or target
anywhere and grants no RAR OS target execution authority.

## Stop conditions

Stop on non-main or proposal-controlled workflow authority; workflow wiring
before C2A exact-main closure; changed role topology; mutable/unpinned tools;
credential, network, owner data, Docker-socket, ambient path, source-write,
compiler, helper, target, firmware, VM, repository publication, or readiness
authority outside the single bounded artifact-retention exception; incomplete
O001-O021 or outer-record mutation coverage; unbounded/unretained evidence;
local execution; source deletion; or any ADR 0030 behavior.
