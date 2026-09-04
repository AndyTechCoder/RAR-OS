# Release Roadmap

## Active Fast-Track Alpha overlay

ADR 0032 replaces the former authorization-chain process with five evidence
milestones while preserving the technical roadmap below:

1. Foundation: boot, memory, allocator, diagnostics, exceptions and timer.
2. Platform: processes, isolation, storage, input, framebuffer and drivers.
3. Usable Alpha: compositor, shell, launcher, settings, files and terminal.
4. Modern architecture: signed layers, atomic updates, rollback, recovery and
   separated user data.
5. Expansion: networking, SDK, applications, AI/agent interfaces and additional
   hardware profiles.

The active contract is
[`Fast-Track Alpha Milestone 1`](tasks/fast-track-alpha-milestone-1.md).
Later milestones do not become current merely because interfaces are prepared.

Status: Gate 0 approved direction — 2026-07-16
Strategy: plan the complete architecture, deliver verified releases in sequence

## Sprint Alpha 0.1 overlay

The owner-approved Sprint Alpha 0.1 vertical slice is defined in
`sprint-alpha.md` and ADRs 0017–0018. Its current end-of-week completion contract
prioritizes authentic working functionality across the roadmap without deleting,
renaming, or claiming completion of the long-term releases below. Cross-release
Alpha prototypes remain experimental until their normal release gates,
migrations, and evidence are complete.

## Release 0 — Architecture Laboratory

Purpose: prove boot, portability, isolation, and observability.

Delivers:

- Reproducible host toolchain and RAR Lab launcher
- x86-64, ARM64, and Tier 0 simulated targets
- RAR Root/Recovery stubs and Nucleus boot
- Memory, threads, scheduling, interrupts, IPC, capabilities, and tracing
- Constitution, glossary, architecture, threat model, and porting documentation

Exit: all targets boot repeatably; isolated test components communicate and failures remain contained.

## Release 1 — Component OS Developer Preview

Purpose: establish RAR's replaceable system model.

Delivers:

- RID interfaces and Rust/C bindings
- Component images, loader, service registry, lifecycle, budgets, and health checks
- Declarative system graph
- Tier 0 bytecode runtime
- Signed packages from a local repository
- Stateless warm/live component replacement
- CLI system inspector and developer tools

Exit: an independent example service can be replaced without changing clients or rebooting the device.

## Release 2 — Durable and Recoverable OS

Purpose: protect data and prove repairability.

Delivers:

- RAR Vault simulator and signing policy
- Custom RAR filesystem, System Store, Data Vault, snapshots, and schemas
- Signed layers, delta chunks, state migration, rollback, and quarantine
- Recovery Seed A/B and system reconstruction
- Single-owner identity UI/CLI sufficient for recovery

Exit: deliberate code and metadata corruption is repaired while verified intact user data remains unchanged.

## Release 3 — Connected Device Platform

Purpose: support Tier 0 and Tier 1 ecosystems.

Delivers:

- Driver service model and virtual hardware contracts
- Wired networking and update transport
- Wi-Fi personal client baseline, device discovery, pairing, and encrypted mesh
- Sensors, actuators, power, battery, and realtime safety-controller interfaces
- Tier 0 sensor and Tier 1 robot demonstrations

Exit: connected simulated devices update, communicate, fail, isolate, and recover through real RAR protocols.

Enterprise Wi-Fi, hotspot mode, broader Bluetooth/USB classes, and advanced roaming follow as Release 3 increments rather than blocking its first usable build.

## Release 4 — Graphical Personal Developer Preview

Purpose: establish native applications and adaptive presentation.

Delivers:

- RAR virtual GPU and software fallback
- Compositor, surfaces, adaptive layout, text, input, accessibility foundations, and external displays
- Rust/C application SDKs
- Provisional UI primitives
- Shell, launcher, terminal, files, settings, networking, updates, diagnostics, recovery, and editor
- Full multi-user account model and UI

Exit: users can onboard, switch accounts, run native apps, install layers, update, recover, and develop a sample app.

## Release 5 — Continuity and Dynamic Tiers

Purpose: prove one OS across device shapes and resources.

Delivers:

- Runtime tier/layer installation and removal
- Dormant state preservation
- Phone-to-desktop presentation transformation
- Remote surfaces and peripheral sharing
- App handoff, encrypted state transfer, and architecture-neutral workload relocation
- Agent platform, consent, audit, tools, and mock provider

Exit: the three ecosystem demonstrations work together across simulated devices without replacing the OS or losing state.

## Release 6 — Self-Hosted VM Consumer Alpha

Purpose: make RAR OS independently developable and suitable for broad controlled testing.

Delivers:

- Tier 3 compiler, linker, build, package, debug, signing, and documentation tools
- Stage 1/Stage 2 reproducible self-build
- Enterprise Wi-Fi and remaining promised VM peripheral support
- Polished onboarding, privacy, accessibility, updates, repair, and recovery
- Long-duration reliability and fault campaigns
- Frozen alpha interfaces with migration policy

Exit: RAR OS builds, packages, updates, repairs, and reproduces itself inside its Tier 3 VM profile.

## Release 7 — Physical Hardware Previews

Purpose: validate that simulation contracts map to real hardware.

Delivers one named x86-64 reference, one ARM64 board, and one Tier 0 board, followed by additional ports.

Each port requires boot, storage, display or console, input where applicable, networking, power, recovery, update, and hardware-provenance validation. Physical targets receive performance budgets only after baseline measurements.

## Later production gates

- Independent security and cryptography audit
- Physical reliability and destructive recovery testing
- Accessibility evaluation
- Update infrastructure and incident-response readiness
- Safety certification for any vehicle, robotics, medical, industrial, or flight claims
- Final RAR design system, broader apps, Pal, and future RAR language as separate ecosystem programs

## Release rule

Later-release architecture must not be implemented by bypassing earlier contracts. Features may be prototyped early, but they do not become stable until their release gate, documentation, migration, and tests are complete.
