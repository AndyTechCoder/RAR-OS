# RAR OS Backlog

Status: Gate 0 approved; Release 0 implementation pending
Repository state: no implementation yet
Primary objective: define and build a fully custom, adaptable, privacy-first operating system and ecosystem foundation.

## How to use this backlog

- Work from top to bottom unless a task is explicitly marked parallel.
- A decision is not final until its plain-language behavior has been reviewed.
- Low-level engineering choices may be proposed by the implementation team, but must include rationale, alternatives, risks, and consequences.
- Public behavior, interfaces, persistent formats, and security boundaries require specifications before implementation.
- Documentation, tests, migration behavior, and failure handling are part of completion—not follow-up work.
- RAR OS must not be built on Linux, BSD, Android, or another existing OS.
- External standards may be implemented for compatibility. Shipped RAR runtime code should be RAR-owned wherever realistically possible.
- Every unavoidable external dependency or firmware payload must be documented, isolated behind a replaceable interface, signed, and auditable.

### Status labels

- `[ ]` Not started
- `[~]` In progress
- `[x]` Complete and reviewed
- `[?]` User/product decision required
- `[!]` Blocked or requires external validation

Backlog Gates 1–7 map to roadmap Releases 0–6. Roadmap Release 7 covers later physical-hardware previews.

## Gate 0 — Finish the handoff specification

Gate 0 was approved on 2026-07-16. Every P0 item in this gate is complete as an approved direction or process contract; release-specific details remain governed by their task packets and ADRs.

### Product constitution and vocabulary

- `[x]` **P0 — Write the RAR OS constitution.** Approved in `docs/constitution.md`. Defines one adaptable OS, privacy-first behavior, user authority, replaceability, state preservation, minimal dependencies, hardware versatility, agent readiness, and transparent recovery.
- `[x]` **P0 — Create a shared glossary.** Approved in `docs/glossary.md`; it remains extensible through normal documentation review.
- `[x]` **P0 — Define what “from scratch” means.** Approved in `docs/from-scratch-policy.md`, including external-input classes and the exception process.
- `[x]` **P0 — Define replaceability as an architectural invariant.** Approved in `docs/replaceability.md`.
- `[x]` **P0 — Define simplicity principles.** Approved in `docs/simplicity-principles.md`.

### Release structure

- `[x]` **P0 — Confirm staged release structure.** Approved in `docs/release-roadmap.md` and ADR 0001.
- `[x]` **P0 — Define milestone names and promises.** Approved as Releases 0–7 in `docs/release-roadmap.md`.
- `[x]` **P0 — Assign features to milestones.** Approved in `docs/release-roadmap.md`.
- `[x]` **P0 — Define the VM consumer-alpha exit criteria.** Approved under Release 6 and handoff gates.
- `[~]` **P1 — Define later production gates.** Drafted in `docs/release-roadmap.md`.

### Tier and profile model

- `[x]` **P0 — Specify four cumulative tiers in plain language.** Approved in `docs/tiers-and-profiles.md` and ADR 0009.
- `[x]` **P0 — Define the contract of each tier for Gate 0.** Approved in `docs/tiers-and-profiles.md`; numeric physical budgets remain intentionally deferred.
- `[x]` **P0 — Define tier installation and removal.** Approved in `docs/tiers-and-profiles.md`.
- `[x]` **P0 — Separate tiers from profiles.** Approved in `docs/tiers-and-profiles.md` and ADR 0009.
- `[~]` **P1 — Define tier discovery.** Capability-based rule drafted in `docs/tiers-and-profiles.md`.

### Master architecture decisions

- `[x]` **P0 — Review the hybrid microkernel proposal.** Approved in `docs/architecture.md` and ADR 0002.
- `[x]` **P0 — Review Rust plus assembly as the core implementation choice.** Approved in `docs/architecture.md` and ADR 0003.
- `[x]` **P0 — Specify the component model.** Approved across `docs/architecture.md`, `docs/replaceability.md`, and ADR 0004.
- `[x]` **P0 — Specify typed inter-component communication.** Approved direction in `docs/architecture.md` and `docs/interfaces-and-formats.md`; exact Release 0 capability and IPC contracts remain R0-007 work.
- `[x]` **P0 — Specify the capability model.** Approved direction in `docs/architecture.md`, `docs/security-and-recovery.md`, and ADR 0004.
- `[x]` **P0 — Specify system composition.** Approved direction in `docs/architecture.md` and `docs/interfaces-and-formats.md`.
- `[~]` **P1 — Define compatibility policy.** Native-first isolated-later policy drafted in `docs/architecture.md`.

### Security, privacy, data, and recovery

- `[x]` **P0 — Write the threat model.** Initial Gate 0 model is approved in `docs/security-and-recovery.md`; detailed subsystem models remain implementation tasks.
- `[x]` **P0 — Specify the boot trust chain.** Approved direction in `docs/security-and-recovery.md`.
- `[x]` **P0 — Specify RAR Vault.** Approved direction in `docs/security-and-recovery.md`.
- `[x]` **P0 — Specify executable signing.** Approved direction in `docs/security-and-recovery.md`.
- `[x]` **P0 — Specify mandatory layer metadata.** Approved direction in `docs/interfaces-and-formats.md`.
- `[x]` **P0 — Specify system/data isolation.** Approved in `docs/security-and-recovery.md` and ADR 0006.
- `[x]` **P0 — Specify automatic isolation and repair.** Approved direction in `docs/security-and-recovery.md`.
- `[x]` **P0 — Specify data preservation guarantees and limits.** Approved in the constitution, `docs/security-and-recovery.md`, and ADR 0006.
- `[x]` **P0 — Select established cryptographic baseline and approval rules.** Approved in `docs/security-and-recovery.md` and ADR 0007; final protocols require follow-up ADRs.
- `[~]` **P1 — Define privacy UX.** Behavioral principles drafted; detailed screen design intentionally deferred.

### Persistent formats and public interfaces

- `[x]` **P0 — Define the RAR Interface Definition direction.** Approved in `docs/interfaces-and-formats.md` and ADRs 0004–0005; exact fields remain release-specific.
- `[x]` **P0 — Define the RAR component image direction.** Approved in `docs/interfaces-and-formats.md` and ADR 0005.
- `[x]` **P0 — Define the layer/package direction.** Approved in `docs/interfaces-and-formats.md` and ADR 0005.
- `[x]` **P0 — Define the installed-system manifest direction.** Approved in `docs/interfaces-and-formats.md` and ADR 0005.
- `[x]` **P0 — Define persistent state schema requirements.** Approved in `docs/interfaces-and-formats.md` and `docs/replaceability.md`.
- `[~]` **P1 — Define compatibility policy for public interfaces.** Drafted in `docs/interfaces-and-formats.md`.

### Development laboratory

- `[x]` **P0 — Specify RAR Lab as a separate host-side testing product.** Approved in `docs/rar-lab.md` and ADR 0008.
- `[x]` **P0 — Define initial virtual machines.** Approved direction in `docs/rar-lab.md`; certified Release 0 profiles remain R0-000 work.
- `[x]` **P0 — Define hot-pluggable virtual devices.** Approved direction in `docs/rar-lab.md`.
- `[x]` **P0 — Define device transformation scenarios.** Approved direction in `docs/rar-lab.md`.
- `[x]` **P0 — Define fault injection.** Approved direction in `docs/rar-lab.md`.
- `[x]` **P0 — Define deterministic record/replay.** Approved direction in `docs/rar-lab.md`.
- `[~]` **P1 — Define simulation-to-hardware boundaries.** Drafted in `docs/rar-lab.md` and `docs/architecture.md`.

### Documentation system

- `[x]` **P0 — Create the documentation tree.** The approved indexed specification set exists in `docs/README.md`; subsystem guides expand during implementation.
- `[x]` **P0 — Define documentation requirements for every task.** Approved in `docs/documentation-policy.md`.
- `[x]` **P0 — Adopt Architecture Decision Records.** Policy and ten initial ADRs are approved for Gate 0.
- `[x]` **P0 — Make examples executable.** CI policy is approved; executable examples begin with implementation.
- `[x]` **P0 — Make public interface documentation generated.** The RID source-of-truth rule is approved.
- `[~]` **P1 — Add documentation-change checks.** CI rule drafted; implementation awaits repository scaffolding.

### Handoff mechanics

- `[x]` **P0 — Define the monorepo layout and ownership boundaries.** Approved in `docs/handoff.md`.
- `[x]` **P0 — Divide implementation into agent-owned workstreams.** Approved in `docs/handoff.md`.
- `[x]` **P0 — Define shared-interface change control.** Approved in `docs/handoff.md`.
- `[x]` **P0 — Define integration gates.** Approved in `docs/release-roadmap.md` and `docs/handoff.md`.
- `[x]` **P0 — Define completion evidence.** Approved in `docs/handoff.md`.
- `[x]` **P0 — Produce the final decision-complete master specification.** The indexed specifications, ten ADRs, Release 0 packet, repository rules, CI, and explicit owner approval are assembled.

## Gate 1 — Bootstrap and architecture proof

- `[ ]` Create the reproducible host build environment for ARM64 macOS and supported Linux hosts.
- `[ ]` Establish zero target-linked third-party dependencies and dependency-audit tooling.
- `[ ]` Implement custom boot paths for x86-64 and ARM64 VM targets.
- `[ ]` Implement the Tier 0 simulated runtime target.
- `[ ]` Boot the minimal Nucleus on all initial architectures.
- `[ ]` Demonstrate memory protection, threads, scheduling, interrupts, timers, IPC, capabilities, and crash isolation.
- `[ ]` Create tracing and inspection tools before adding complex services.
- `[ ]` Publish complete boot, architecture-porting, and debugging documentation.

## Gate 2 — Component OS foundation

- `[ ]` Implement RID and generated Rust/C bindings.
- `[ ]` Implement component loading, lifecycle, dependencies, budgets, and health checks.
- `[ ]` Implement process isolation and safe component co-location policy.
- `[ ]` Implement the service registry and replaceable endpoint routing.
- `[ ]` Implement the declarative system graph.
- `[ ]` Implement RAR bytecode and its verifier for Tier 0 portable components.
- `[ ]` Demonstrate replacement of a running stateless service without rebooting.
- `[ ]` Document how to create, test, inspect, replace, and remove a component.

## Gate 3 — Trust, storage, updates, and recovery

- `[ ]` Implement RAR Vault simulation and trust-chain verification.
- `[ ]` Implement signing, owner trust roots, developer mode, revocation, and anti-rollback.
- `[ ]` Implement the custom copy-on-write RAR filesystem.
- `[ ]` Implement immutable System Store and encrypted Data Vault domains.
- `[ ]` Implement snapshots, quotas, checksums, state schemas, and transactional migrations.
- `[ ]` Implement content-addressed packages and delta downloads.
- `[ ]` Implement live component updates, health validation, and automatic rollback.
- `[ ]` Implement component quarantine and clean reconstruction.
- `[ ]` Implement Recovery Seed A/B and whole-system reconstruction without rewriting intact user data.
- `[ ]` Pass power-loss and corruption testing at every persistent-write boundary.

## Gate 4 — Hardware and connectivity services

- `[ ]` Implement the driver-service model and virtual hardware protocol.
- `[ ]` Implement storage, display, input, timer, entropy, sensor, motor, battery, and power-management drivers.
- `[ ]` Implement Ethernet, IPv4, IPv6, TCP, UDP, DNS, TLS, HTTP, and local discovery.
- `[ ]` Implement Wi-Fi client, hotspot, roaming foundations, WPA2/WPA3 Personal, and Enterprise authentication.
- `[ ]` Implement USB and Bluetooth architecture; schedule exact protocol/device support by release.
- `[ ]` Implement network capability enforcement, namespaces, firewalling, and access visibility.
- `[ ]` Implement secure device discovery, pairing, authentication, and encrypted communication.
- `[ ]` Publish driver-authoring and hardware-porting documentation.

## Gate 5 — Graphical personal OS

- `[ ]` Implement the RAR virtual GPU and software-rendering fallback.
- `[ ]` Implement compositor, surfaces, adaptive layout, text, input, windows, external displays, and application presentation.
- `[ ]` Implement provisional accessible UI primitives without locking a final branded design system.
- `[ ]` Implement application lifecycle: launch, suspend, snapshot, restore, crash recovery, and removal.
- `[ ]` Implement complete multi-user identity, login, switching, shared resources, and per-user data protection.
- `[ ]` Implement system shell, launcher, terminal, files, editor, settings, networking, permissions, updates, diagnostics, and recovery applications.
- `[ ]` Implement notifications, clipboard, sharing, background work, localization foundations, and accessibility services.
- `[ ]` Demonstrate phone-to-desktop transformation after virtual display/input attachment.

## Gate 6 — Apps, agents, tiers, and continuity

- `[ ]` Ship stable Rust and C SDKs with templates, emulator integration, debugging, signing, packaging, and documentation.
- `[ ]` Implement the agent principal type, tool schemas, consent, auditing, budgets, and lifecycle.
- `[ ]` Implement a mock agent provider; Pal intelligence remains outside this project until ready.
- `[ ]` Implement live tier and layer installation/removal with preserved dormant state.
- `[ ]` Implement remote capability invocation, encrypted object transfer, shared peripherals, and remote surfaces.
- `[ ]` Implement application handoff and architecture-neutral stateful workload relocation.
- `[ ]` Demonstrate Tier 0 sensor, Tier 1 robot, Tier 2 personal device, and Tier 3 compute scenarios together.
- `[ ]` Document the future RAR language requirements learned from real SDK and application development.

## Gate 7 — Self-hosting and consumer alpha

- `[ ]` Port required compiler, linker, assembler, build, package, debugger, and documentation tools to Tier 3.
- `[ ]` Implement the RAR-native build orchestrator and development environment.
- `[ ]` Rebuild all tiers, SDKs, system applications, recovery images, and packages from inside RAR OS.
- `[ ]` Verify reproducible Stage 1 and Stage 2 builds.
- `[ ]` Complete onboarding, accessibility, privacy, update, repair, and recovery user experiences.
- `[ ]` Run long-duration VM, multi-user, networking, continuity, corruption, and update tests.
- `[ ]` Record performance baselines without treating VM measurements as physical-device targets.
- `[ ]` Freeze and document the consumer-alpha interfaces and migration policy.
- `[ ]` Produce physical-device porting kits and select later reference hardware.

## Deferred ecosystem work

- `[ ]` Final branded RAR design system.
- `[ ]` RAR application language and self-hosted compiler.
- `[ ]` Pal intelligence and model delivery.
- `[ ]` Production package publishing, accounts, CDN, and global update infrastructure.
- `[ ]` Optional isolated Linux/POSIX application compatibility.
- `[ ]` Browser and broad first-party application ecosystem.
- `[ ]` Physical device drivers and validation beyond selected references.
- `[ ]` Independent security audits and safety/industry certifications.

## Immediate next items

1. Publish and inspect the reviewed reference-evidence portability correction.
   Run `33267767640` proved every preceding mutation suite, including QMP,
   passes and exposed only an undeclared `/usr/bin/xxd` test dependency.
2. Make PR #7's required workflow pass; complete review,
   merge the exact rebaseline head, verify the merge on GitHub, and require the
   distinct `main` workflow to pass before calling the replacement durable.
3. Only after that durability proof, close draft PR #5 as superseded while
   preserving its history and evidence. Never merge PR #5 into the Alpha line.
4. Record the owner's ADR 0023, ADR 0024, and ADR 0026 choices; complete and
   re-review the resulting boot/platform contracts and cloud-only
   controller/helper evidence before Milestone A.
5. Make the v2 Lab profile/controller ready only with real reviewed identities,
   then pass the SSD-profile, capacity, local, remote, PR, and immutable-
   checkpoint gates.
6. Start Milestone A only in the packet-required fresh Medium writer task and
   execute target work only in the approved cloud Development Lab.
7. Record ADR 0025 and complete the reviewed protocol/controller v2 cutover
   before Milestone B.
8. Record ADR 0022 and its reviewed peripheral-grant contract before Milestone E.
9. Preserve strict local Mac/SSD non-execution, non-target-build, and current
   no-deletion rules while continuing Milestones B–G through their durable
   checkpoints.
