# Sprint Alpha ADR 0024 C1 Contract Closure

Status: Authoritative source-only child packet - implementation requires exact-main validation

## Purpose

Close only the missing experimental host contracts required by C1 of
`sprint-alpha-controller-helper-integration.md`. This child packet grants no
workflow wiring, observer/verifier run, compiler use, helper build or execution,
Lab readiness, target build, VM, boot, local execution, or ADR 0030 authority.

The C1 writer may start only after this packet is reviewed, merged, and a
distinct exact-main Specifications run completes validation and mutation.

## Exact owned paths

The later C1 implementation writer owns only these paths:

- `spec/alpha/lab/README.md`
- `tools/toolchain/README.md`
- `docs/sprint-alpha-dashboard.md`
- `SPRINT_STATUS.md`
- `spec/alpha/lab/controller-helper-closure-verifier-test-plan-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-validation-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-errors-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-precedence-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-case-dispositions-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-scalar-semantics-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-basic-filesystem-semantics-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-scalar-repair-semantics-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-synchronized-link-semantics-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-observation-repair-semantics-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-rebuild-observation-canonical-semantics-v0`
- `spec/alpha/lab/controller-helper-runtime-v0.fields`
- `spec/alpha/lab/controller-helper-runtime-cases.v0`
- `spec/alpha/lab/controller-helper-closure-observer-test-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-faults-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-cases-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields`
- `spec/alpha/lab/controller-helper-closure-acceptance-v0.fields`
- `spec/alpha/lab/controller-helper-test-evidence-v1.fields`
- `spec/alpha/lab/controller-helper-build-evidence-v1.fields`
- `tools/ci/check-controller-helper-runtime-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-validation-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-input-domain-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-case-dispositions-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-case-templates-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-operator-inventory-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-scalar-semantics-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-basic-filesystem-semantics-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-scalar-repair-semantics-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-synchronized-link-semantics-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-observation-repair-semantics-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-rebuild-observation-canonical-semantics-source.sh`
- `tools/ci/check-controller-helper-closure-observer-test-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-faults-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-cases-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-evidence-source.sh`
- `tools/ci/check-controller-helper-closure-acceptance-source.sh`
- `tools/ci/check-controller-helper-test-evidence-v1.sh`
- `tools/ci/check-controller-helper-build-evidence-v1.sh`
- `tools/ci/test-controller-helper-evidence-v1-policy.sh`
- `tools/ci/check-alpha-preimplementation-contracts.sh`
- `tools/ci/check-specs.sh`

No glob, directory, workflow, existing v0 build/test evidence schema, helper
Rust source, profile, inventory instance, or tool lock is owned. Existing staged
verifier contracts, their bound checkers, and the three named status documents
are owned only to replace now-stale absence statements and rebind exact bytes. A need to
touch any other path stops C1 and requires a separately reviewed packet change.

## Contract requirements

### Runtime and recovery

The runtime contract must fix:

- the exact inherited descriptor table and purpose of every descriptor;
- canonical basename and byte-length rules;
- `O_NOFOLLOW|O_EXCL|O_CLOEXEC` and no ambient path/root lookup;
- UID, GID, mode, link-count, device, inode, size, and pre/post identity checks;
- exact-binary descriptor execution without shell or PATH interpretation;
- closing every unintended inherited descriptor and clearing environment data;
- non-reusable controller-owned process handles, never bare PID authority;
- parent stop/ack protocol and bounded monotonic watchdog deadlines;
- cancel, forced termination, wait/reap, and ambiguous-exit behavior;
- durable hash-chained transition order, synchronization, and restart takeover;
- source-never-deleted recovery, exclusive attempt roots, bounded cleanup;
- fail-closed uncertain commit/cleanup and the attempt/recovery ceiling.

Every existing declarative attempt row and every new process/watchdog/recovery
row must have one deterministic disposition and forbidden-next-effect rule.

### Observer runtime testing

The observer-test contract must cover success plus context, revision, tool,
topology, path, alias, output-collision, bound, command, read, write, close,
hash, enumeration, mutation, resource, confinement, network, credential, and
unexpected-FD failures. It fixes canonical case evidence and a normalized
not-ready verdict. No case can update a lock, inventory, profile, gate, or
readiness.

### Verifier cases, faults, and evidence

The verifier contracts must:

- instantiate every constructible class-and-occurrence pair exactly once;
- define executable dual-invalid precedence oracles;
- bind phase-synchronized mutation and independent repair;
- define injected command/read/write/close/tool-output/resource failures;
- bind an immutable base fixture, fixture image, tools, subject, and controller;
- keep scratch unreachable by controller mutation after launch;
- define exact result ordering, byte ceilings, exit mapping, evidence grammar,
  normalized verdict, revision/nonces, anti-replay, and no-success effect;
- preserve unrepresentable cases as explicit source-proof or residual rows
  rather than silently omitting them;
- update every owned staged verifier contract, checker digest, toolchain note,
  dashboard, and sprint status so no authoritative text still says the new C1
  contracts are absent, while continuing to say execution and wiring are absent.

### Closure acceptance

The closure-acceptance schema is not an acceptance instance. It must bind the
future exact C3V observation receipt, verification receipt, manifest, topology,
tool pins, compiler identity, licenses, provenance, authenticated acquisition,
retained bytes, source/controller revisions, runtime case evidence, and
normalized fault verdict. It must reject missing, stale, cross-revision,
self-attested, mutable, incomplete, replayed, aliased, or zero identities.

Mechanical exact-set equality remains not-ready. Only a separately reviewed
C3A instance may later authorize compiler use.

### Versioned helper evidence

The v1 test schema and validator must cover the 97 attempt rows plus all new
descriptor, process, watchdog, journal, restart, cleanup, attempt-ceiling, and
authority-denial cases. They bind exact controller/source/helper/closure-
acceptance identities, distinct nonces and roots, controller-observed exits,
canonical case results, and complete bounded logs.

The v1 aggregate build schema and contextual validator must consume exactly the
v1 test evidence and C3A closure-acceptance identity. It must require two
distinct fresh build receipts and byte-identical outputs. Legacy v0 aggregate
or 13-case evidence cannot satisfy v1 or readiness.

## Validation

C1 changes remain source-only. Static checkers validate exact schemas,
cardinality, ordering, bounds, cross-file identities, and non-wiring.
Cloud mutation tests must show each critical field, missing case, stale
identity, wrong version, replay, and v0 substitution fails closed.

No checker may invoke a compiler, helper, container, emulator, firmware, target
tool, network client, credential source, or workflow. The Mac runs only the
approved no-scratch read-only gate.

## Reviews and completion

Architecture, correctness, and security review the exact C1 implementation
head. Merge is allowed only with no blocking finding and the trusted-controller
policy satisfied. Because `check-specs.sh` changes, the PR job may defer real
validation; C1 completes only after the merge commit receives a distinct
exact-main run where validation and mutation both execute and pass.

C1 completion does not start C2 automatically. C2 requires its own literal-path
child packet, reviews, merge, and exact-main validation.

## Stop conditions

Stop on any new role topology, executable authority, workflow, runtime
activation, compiler/helper/target execution, stable target format, target
dependency, local execution, network/credential/owner-data exposure, root path
acquisition, source deletion, weaker evidence version, incomplete runtime/fault
coverage, premature readiness, or ADR 0030 behavior.
