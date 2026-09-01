# Sprint Alpha ADR 0024 C2 Observer Discovery

Status: Authoritative source-only C2 child packet - implementation requires exact-main validation

## Purpose

Register only the trusted-main compiler-closure observer workflow, its bounded
controller wrapper, its complete O001-O021 policy/runtime test harness, and its
retained candidate evidence. This packet itself grants no workflow execution,
compiler use, helper or target execution, Lab readiness, VM launch, local
execution, or ADR 0030 authority.

The C2 implementation writer may start only after this packet is independently
reviewed, merged, and a distinct exact-main Specifications run completes both
validation and mutation successfully.

## Exact owned paths

The later C2 implementation writer owns only these paths:

- `.github/workflows/controller-helper-closure-observer.yml`
- `tools/ci/run-controller-helper-closure-observer.sh`
- `tools/ci/observe-controller-helper-closure.sh`
- `tools/ci/check-controller-helper-closure-observer-source.sh`
- `tools/ci/check-controller-helper-closure-observer-policy.sh`
- `tools/ci/test-controller-helper-closure-observer-policy.sh`
- `tools/ci/controller-helper-closure-observer-harness.sh`
- `tools/ci/contracts/controller-helper-closure-observer-case-evidence-v0.fields`
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

No glob or directory ownership is granted. The C1 schemas and cases, controller
integration packet, existing Specifications workflow, helper source, inventory
instance, tool locks, Lab profile, controller plan, target source, and every
other path are read-only dependencies. A need to touch any unnamed path stops
C2 and requires a separately reviewed packet amendment.

## Trusted-main workflow boundary

The workflow must:

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
- expose no secret, token, Docker socket, host path outside the exact checkout
  and controller-owned scratch/evidence roots, or mutable source mount;
- retain only the bounded candidate manifest, 23-line receipt, canonical
  O001-O021 case evidence, and outer identity/result record using the already
  pinned artifact-retention action. Retention is evidence, never readiness.

The workflow and wrapper must reject stale or non-main revisions, unexpected
environment variables, missing or additional mounts, preexisting outputs,
unbounded output, and any attempt to reach network, credentials, compiler,
helper, target, profile, inventory, lock, gate, or publication state.

## Observer and test requirements

Production observation preserves the C1 contract exactly: it enumerates the
complete pinned Linux Rust compiler closure, emits only the manifest and
`observed-not-reviewed-not-ready` receipt, and never invokes `rustc`,
Cargo, a linker, helper, target tool, container client, network client, cloud
client, or credential source.

The harness must execute O001-O021 exactly once in canonical order and emit the
C1 case-evidence grammar with distinct nonces and roots. Synthetic fault
controls must be unreachable from the production entry point. Any generated
test subject must be mechanically bound to the reviewed production source and
may vary only the explicitly listed fixture roots, tool outcomes, and fault
operations. Tests cover context, revision, tool identity, topology, unsafe path,
alias, output collision, bounds, command/read/write/close/hash/enumeration
failure, phase mutation, resource failure, confinement, network, credential,
and unexpected inherited descriptor rejection.

No negative case may mutate the actual compiler closure, checkout, workflow,
host, lock, inventory, profile, gate report, or readiness state. All temporary
case roots are exclusive children of the bounded cloud scratch mount; cleanup
may remove only those verified children and never source or retained evidence.

## Evidence and completion

C2 completion requires:

1. static policy proves exact triggers, action pins, mounts, resources,
   environment, commands, output set, and absence of activating authority;
2. all O001-O021 cases pass in the isolated cloud job before observation;
3. the exact-main observer emits one bounded manifest and one 23-line receipt;
4. retained outer evidence binds repository, main SHA, run ID/attempt, runner
   identity, OCI digest, subject, fixtures, tool pins, case evidence, manifest,
   and receipt hashes;
5. architecture, correctness, and security review the exact implementation head;
6. merge occurs only with no blocking findings; a distinct exact-main run then
   passes Specifications validation/mutation and the C2 observer workflow.

The result remains a candidate. C2 does not update the compiler-closure lock,
helper inventory, Lab profile, controller plan, gate report, or readiness.
C3V must re-observe and independently verify the exact set at its own merge
revision. Only a later reviewed C3A acceptance record may authorize compiler use.

## Local and target safety

The Mac and SSD receive source storage only. No C2 compiler, observer, harness,
container, helper, target, firmware, VM, or RAR OS artifact may be compiled,
executed, or launched locally. C2 compiles and executes no helper or target
anywhere and grants no RAR OS target execution authority.

## Stop conditions

Stop on non-main or proposal-controlled workflow authority; changed C1
contracts or role topology; mutable/unpinned tools; credential, network, owner
data, Docker-socket, ambient path, source-write, compiler, helper, target,
firmware, VM, publication, or readiness authority; incomplete O001-O021
coverage; unbounded or unretained evidence; local execution; source deletion;
or any ADR 0030 behavior.
