# Release 0 Task Packets — Architecture Laboratory

Status: Ready — Gate 0 owner approval recorded 2026-07-16
Rule: Release 0 proves architecture; it does not implement storage, networking, GUI, live updates, or consumer applications.

## Shared task-packet contract

Every R0 task below is defined by this shared contract, its row in the task field matrix, and its detailed owner, dependencies, deliverables, and acceptance criteria.

- **Approved specifications:** `docs/constitution.md`, `docs/from-scratch-policy.md`, `docs/release-roadmap.md`, `docs/architecture.md`, `docs/security-and-recovery.md`, `docs/interfaces-and-formats.md`, `docs/handoff.md`, `docs/host-safety.md`, ADRs 0001–0022, and this packet. A task uses only the subset relevant to its row and dependencies.
- **Global in-scope rule:** implement only the mechanisms, host tools, contracts, tests, and documentation named by the active task.
- **Global out-of-scope rule:** no Release 1+ component model, filesystems, networking, GUI, agents, package system, applications, physical-device enablement, or unapproved stable contract. No task may weaken a prior gate.
- **Ownership rule:** the paths in the matrix are exclusive write ownership while that task is active. Dependency paths are read-only unless the coordinator records an ownership handoff. Root and shared governance files require coordinator ownership.
- **Required tests:** the detailed acceptance criteria are mandatory and include positive, boundary, malformed, failure, and fault cases applicable to the task. A skipped criterion is an explicit limitation, not a pass.
- **Documentation and evidence:** update the task's owned `docs/release-0/` path, record exact host commands and tool versions, map every acceptance statement to output or a limitation, and include dependency, unsafe-code, security, and target-execution attestations.
- **Unresolved-risk rule:** the matrix lists known risks. Newly discovered ambiguity in a public contract, trust boundary, target dependency, or release promise stops implementation for ADR/owner review.

## Task field matrix

| Task | Objective and owned paths | Public contracts | Required failure cases and unresolved risks |
| --- | --- | --- | --- |
| R0-000 | Establish the non-executing certified-VM boundary. Owns `tools/rar-lab/safety/`, `spec/lab/vm-profile/`, `tests/host-safety/`, and `docs/release-0/host-safety/`. | Host-only VM profile schema, allowlisted command model, authorization record input, and certification evidence format. None is a target OS interface. | Reject every forbidden host integration, unsafe path, unbounded resource, missing pin, malformed profile, absent certification, and absent owner authorization. Risk: certification must remain impossible until R0-001 supplies exact tool/firmware pins. |
| R0-001 | Establish reproducible host bootstrap tools. Owns `tools/rarbuild/`, `tools/toolchain/`, `tests/bootstrap/`, `docs/release-0/build/`, `Cargo.toml`, `rust-toolchain.toml`, and `rustfmt.toml`. | Host CLI commands `check`, `build`, `image`, `run`, `test`, and `evidence`; tool/dependency lock and build-evidence schema. No command may imply execution authorization. | Missing/mismatched tools, hashes, target dependencies, non-reproducible plans or artifacts, unsafe output paths, and every unauthorized execution-capable command. Risk: supported host discovery differs between ARM64 macOS and Linux. ADR 0011 proves planning now and retains identical target artifacts as a blocking Release 0 closure gate after artifacts exist. |
| R0-002 | Freeze Release 0 hardware and boot handoff contracts. Owns `spec/hardware/`, `spec/boot/`, `spec/fixtures/release-0/`, `sdk/generated/release-0/`, and `docs/release-0/contracts/`. | Release 0 RAR Hardware Description, boot handoff, validation/failure codes, and generated Rust representation. | Truncated, oversized, misaligned, overlapping, unknown-critical, invalid-pointer, and architecture-inconsistent fixtures. Risk: fields frozen before platform evidence require versioned correction rather than silent reinterpretation. |
| R0-003 | Bring up the x86-64 Release 0 platform after authorization. Owns `nucleus/arch/x86_64/`, `tests/platform/x86_64/`, and `docs/release-0/x86_64/`. | x86-64 adapter implementation of R0-002 plus structured platform trace records; no new portable contract. | Invalid handoff, mapping/exception/timer failure, unexplained reset, and unsafe VM assumption. Risk: pinned UEFI/QEMU behavior must not leak into portable code. |
| R0-004 | Bring up the ARM64 Release 0 platform after authorization. Owns `nucleus/arch/aarch64/`, `tests/platform/aarch64/`, and `docs/release-0/aarch64/`. | ARM64 adapter implementation of R0-002 plus semantically equivalent trace records; no new portable contract. | Invalid handoff, exception-level, translation, GIC/timer, shutdown, and reset failures. Risk: machine-specific behavior must not alter the shared semantic description. |
| R0-005 | Prove the bounded Tier 0 experiment after authorization. Owns `nucleus/arch/armv8m/`, `tests/platform/tier0/`, and `docs/release-0/tier0/`. | Experimental static task/capability-index contract and reduced-assurance trace semantics; not a claim of full Nucleus isolation. | Invalid capability index, task overrun, watchdog, timer, reset, and nondeterministic-seed failure. Risk: simulator evidence must not overstate MMU-less security or physical support. |
| R0-006 | Implement portable Release 0 memory and execution mechanisms. Owns `nucleus/portable/`, `nucleus/runtime/`, `tests/nucleus/`, and `docs/release-0/nucleus/`; platform adapter paths are read-only dependencies. | Architecture-neutral page, address-space, thread, scheduler, timer, and exception traits internal to Release 0. | Cross-address-space access, invalid mapping, guard fault, allocation exhaustion, leaked page/thread, and architecture divergence. Risk: every unsafe/assembly boundary requires explicit invariants. |
| R0-007 | Prove capability and IPC isolation. Owns `nucleus/capability/`, `nucleus/ipc/`, `core/registry/`, `spec/capability-ipc/`, `tests/capability-ipc/`, and `docs/release-0/capability-ipc/`. | Release 0 handle rights, endpoint/message/wait semantics, structured errors, trace correlation, and logical registry behavior. | Forged, stale, cross-process, over-privileged, rights-increasing, oversized, backpressured, timed-out, cancelled, closed, and peer-crash cases. Risk: deadlock, stale authority, and replacement races require independent security review. |
| R0-008 | Turn Release 0 behavior into deterministic laboratory evidence. Owns `spec/trace/`, `tools/rar-lab/scenarios/`, `tests/lab/`, and `docs/release-0/lab/`; R0-000 safety paths change only by recorded handoff. | Structured trace, scenario, evidence-directory, timeout, and first-divergence formats for Release 0. | Port collision, timeout, malformed event, missing evidence, nondeterministic replay, crash containment, starvation, and power-loss cases. Risk: evidence must remain bounded and privacy-scrubbed. |
| R0-009 | Close Release 0 documentation and conformance. Owns `docs/release-0/`, `spec/conformance/release-0/`, `sdk/examples/release-0/`, and coordinator-approved updates to `README.md`, `BACKLOG.md`, and ADRs. | Approved Release 0 guides, conformance corpus, examples, limitations, evidence map, and Release 1 migration notes. | Missing promise mapping, non-executable example, stale limitation, architecture mismatch, and unreproducible clean-checkout evidence. Risk: incomplete evidence must remain a limitation and cannot be converted into a claim. |

## Shared target matrix

- `x86_64-unknown-none` on a pinned QEMU `q35`-class machine, serial console, UEFI bootstrap followed by RAR-owned boot path.
- `aarch64-unknown-none` on pinned QEMU `virt`, serial console, UEFI bootstrap followed by RAR-owned boot path.
- `thumbv8m.main-none-eabi` microcontroller simulation for Tier 0 runtime experiments.

Exact QEMU and firmware versions are recorded by the bootstrap lock before implementation begins. All output uses deterministic machine-readable event records plus human serial logs.

## R0-000 — Host safety and certified VM boundary

Owner: RAR Lab/security workstream
Dependencies: Gate 0 owner approval

Deliver:

- Enforced host-safety rules from `docs/host-safety.md`.
- VM profile schema and allowlisted command generator.
- A fail-closed pre-authorization gate consumed by every launcher entry point.
- Refusal of raw disks, host devices, passthrough, shared folders, networking, elevated execution, unsafe paths, and direct arbitrary emulator arguments.
- Static profile inspection and negative test corpus.
- A certification evidence format for later owner-authorized guest boot.

Acceptance:

- No RAR target artifact is executed during this task.
- Negative tests reject every forbidden configuration class.
- Generated safe command references only pinned tools, firmware, workspace artifacts, disposable workspace images, and bounded resources.
- Security review confirms that first guest execution remains separately owner-gated.
- Host-only negative evidence proves every run path refuses before resolving or spawning an emulator unless both immutable profile certification and a separate owner-authorization record are present.

## R0-001 — Bootstrap and reproducible host tools

Owner: Build/bootstrap workstream
Dependencies: R0-000 policy and schemas; implementation may proceed in parallel where it cannot execute target code

Deliver:

- Pinned Rust 1.95.0 host toolchain initially, `rust-src`, LLVM/Clang/LLD discovery, QEMU discovery, and firmware hashes.
- `rarbuild` Release 0 command surface: `check`, `build`, `image`, `run`, `test`, and `evidence`; `run` remains a refusal-only path until the separate owner authorization gate is satisfied.
- Reproducible output directories separated from source.
- Dependency inventory proving no target-linked third-party crates.
- ARM64 macOS and Linux host instructions.

Acceptance:

- One command reports missing tools without mutating the host.
- Before the separate owner authorization is recorded, `rarbuild run`, execution-capable `rarbuild test` modes, aliases, and every other path that could launch or delegate to an emulator refuse before executable resolution or process spawn; host-only negative tests prove each path.
- While no target artifact exists, two clean planning runs from the same checkout and validated lock produce byte-identical canonical build plans without target compilation, linking, loading, or execution.
- After Release 0 target artifacts exist and before R0-009 closes the release, two clean builds from the same checkout and locked inputs produce byte-identical unsigned target artifacts for every required Release 0 target. Missing or unequal artifacts block release closure under ADR 0011.
- Build evidence records tool versions, hashes, target, configuration, and source revision.

## R0-002 — Hardware description and boot contract

Owner: Architecture/specification workstream
Dependencies: R0-001

Deliver:

- Release 0 RAR Hardware Description schema for CPU, memory, interrupts, timers, serial, boot source, and reserved regions.
- Boot handoff structure with magic, version, architecture, memory map, hardware-description location, entropy input, and trace channel.
- Bounds, alignment, ownership, validation, and failure codes.
- Valid/malformed conformance fixtures and generated Rust types.

Acceptance:

- x86-64 and ARM64 boot paths produce identical semantic descriptions.
- Nucleus rejects every malformed fixture without executing unverified pointers.

## R0-003 — x86-64 platform and boot

Owner: x86-64 workstream
Dependencies: R0-001, R0-002

Deliver:

- Minimal RAR boot image, page-table setup, long-mode entry, serial output, interrupt controller, timer, exception entry, and shutdown.
- Architecture adapter implementing common Release 0 traits only.
- Exception register dump through structured trace records.

Acceptance:

- Boots 100 consecutive times in the pinned VM.
- Detects invalid boot handoff and enters defined recovery halt.
- Timer and deliberate exception tests pass without unexplained reset.

## R0-004 — ARM64 platform and boot

Owner: ARM64 workstream
Dependencies: R0-001, R0-002

Deliver:

- Minimal RAR boot image, exception-level transition, translation tables, serial output, GIC/timer integration, exception vectors, and shutdown.
- Architecture adapter matching the x86-64 semantic contract.

Acceptance:

- Same evidence and repetition requirements as R0-003.
- Common Nucleus code contains no QEMU machine-name branches.

## R0-005 — Tier 0 runtime experiment

Owner: Tier 0/ARM workstream
Dependencies: R0-001, R0-002

Deliver:

- Microcontroller reset, memory layout, timer, serial trace, watchdog, static task table, and capability-index experiment.
- Document which full Nucleus guarantees cannot exist without an MMU.
- Minimal deterministic task scheduler and fault containment available on the target.

Acceptance:

- Repeated deterministic task sequence under a pinned simulation seed.
- Invalid capability index and task overrun are detected and reported.
- No claim of full Tier 0 security is made from the experiment.

## R0-006 — Nucleus memory and execution

Owner: Nucleus workstream
Dependencies: R0-003 and R0-004 boot contracts

Deliver:

- Physical page allocator, address-space abstraction, guarded kernel stacks, thread objects, context switching, timers, fair scheduler baseline, and architecture-neutral exceptions.
- No heap dependency before allocator initialization.
- Unsafe-code inventory and invariants.

Acceptance:

- Isolation tests prove one test address space cannot read/write another.
- Stress creates, schedules, and destroys threads without leaked pages.
- Guard-page and invalid-mapping faults identify the responsible test component.
- Common suite passes on x86-64 and ARM64.

## R0-007 — Capability and IPC proof

Owner: Nucleus/component-fabric workstream
Dependencies: R0-006

Deliver:

- Process-local handle tables, endpoint objects, send/receive, bounded messages, wait sets, timeout, cancellation, handle transfer with rights reduction, and endpoint closure.
- Logical test-service registry outside the Nucleus.
- Structured error taxonomy and trace correlation.

Acceptance:

- Forged, stale, over-privileged, and cross-process handles fail safely.
- Delegation cannot increase rights.
- Endpoint replacement redirects new calls without changing clients.
- Backpressure, timeout, peer crash, and cancellation tests pass on both architectures.

## R0-008 — Trace, crash, and architecture laboratory

Owner: RAR Lab/testing workstream
Dependencies: R0-003 through R0-007

Deliver:

- Structured event schema for boot, memory, thread, capability, IPC, exception, and shutdown.
- RAR Lab CLI wrappers for the three target profiles.
- Deterministic scenario runner, evidence collector, timeout enforcement, and initial record/replay boundary.
- Fault scenarios for component crash, invalid message, starvation attempt, and VM power loss.

Acceptance:

- CI can boot both full architectures concurrently without port collisions.
- Failed scenario produces one evidence directory and first-divergence summary.
- Expected component crash does not prevent unrelated test service from finishing.

## R0-009 — Documentation and conformance release

Owner: Architecture/documentation coordinator
Dependencies: all Release 0 tasks

Deliver:

- Approved Release 0 architecture, boot, hardware, memory, scheduling, capability, IPC, trace, porting, and debugging guides.
- Executable examples and conformance corpus.
- Known limitations, security claims, measured baselines, and Release 1 migration notes.
- Updated glossary, ADRs, and dependency report.

Acceptance:

- A fresh implementation agent follows only repository documentation to build, run, debug, and add a trivial isolated test service.
- All examples run in CI.
- Release evidence maps every promise to a passing test or an explicit limitation.

## Integration order

1. R0-000 establishes the non-execution and certified-VM boundary.
2. R0-001 and R0-002 establish tools/contracts without guest execution.
3. Prompt 7A may construct and certify one exact candidate without execution under ADRs 0017–0021.
4. Prompt 7 is rerun with fresh architecture and security review; a separate exact owner authorization unlocks one certified VM boot.
5. R0-003, R0-004, and R0-005 run in parallel.
6. R0-006 uses both full-architecture ports.
7. R0-007 proves the component boundary.
8. R0-008 turns tests into repeatable laboratory evidence.
9. R0-009 closes documentation and acceptance.

No later release subsystem is merged into Release 0 merely to demonstrate progress.

ADR 0011 re-phases only the timing of the R0-001 reproducibility evidence. It does not waive or weaken the two-clean-build target-artifact requirement, which remains mandatory before Release 0 closes.
