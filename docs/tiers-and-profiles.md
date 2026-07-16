# Tiers and Profiles

Status: Gate 0 approved direction — 2026-07-16

## Model

A tier is a cumulative compatibility contract. A profile is a role-oriented composition. Tiers do not represent separate editions, visual styles, or product prices.

Every component declares its minimum tier plus specific capabilities. Capability checks, not tier numbers alone, decide whether it can run.

## Tier 0 — Micro

Purpose: connect tiny sensors, trackers, actuators, and controllers to the RAR ecosystem.

Required contract:

- Verified boot or declared reduced-assurance boot
- Device identity and signed component/script verification
- Deterministic RAR bytecode or statically approved native tasks
- Cooperative or preemptive scheduling appropriate to hardware
- Capability-scoped access to memory, timers, communication, sensors, and actuators
- Local state with transactional update appropriate to available storage
- Secure pairing and RAR mesh subset
- Signed update, rollback, watchdog, and recovery mode
- Resource budgets and observable failures

Tier 0 does not require an MMU, GUI, filesystem hierarchy, dynamic native loading, or large network stack.

## Tier 1 — Device

Purpose: capable embedded devices, appliances, robots, drones, routers, and controllers.

Adds:

- Full Nucleus isolation when hardware protection exists
- Native replaceable components and user-space drivers
- Filesystem/System Store/Data Vault subset
- Full component lifecycle and service registry
- Wired/Wi-Fi networking appropriate to profile
- Realtime/deadline scheduling classes
- Rich sensors, actuators, power, safety-controller, and diagnostics interfaces
- Tier 0 component and mesh compatibility
- Optional GUI and media layers

Tier 1 is not synonymous with headless; a robot or appliance may have displays.

## Tier 2 — Personal

Purpose: phones, tablets, laptops, consoles, personal assistants, and shared personal devices.

Adds:

- Complete encrypted multi-user identity and state domains
- Adaptive graphical experience and accessibility services
- Native application lifecycle, notifications, clipboard, sharing, and background policy
- Audio, camera, media, location, and personal-device permission models
- Local agent-hosting interfaces
- Device continuity, external displays, and personal mesh behavior
- Tier 0 and Tier 1 compatibility

## Tier 3 — Compute

Purpose: workstations, servers, development systems, simulations, and intensive workloads.

Adds:

- Development and self-hosting toolchain layer
- Large-memory, multi-core, multi-display, accelerator, server, and virtualization policies
- Workload relocation and distributed service hosting
- Extended diagnostics, profiling, build, package, and simulation tools
- All lower-tier contracts

Tier 3 does not require a GUI; a server profile may remain headless.

## Profiles

Initial profiles:

- **Sensor:** Tier 0, periodic sensing, minimal local state, mesh communication.
- **Actuator:** Tier 0, tightly scoped commands, safety limits, watchdog.
- **Robot:** Tier 1+, realtime safety domain, sensors, motors, networking, optional vision/GUI.
- **Drone:** Tier 1+, hard resource and safety policies, intermittent connectivity.
- **Phone:** Tier 2+, touch-first experience, battery policy, radios, cameras, external-display transformation.
- **Desktop:** Tier 2+, multiwindow, keyboard/pointer, broad peripherals.
- **Server:** Tier 1 or 3, headless services, remote management, storage/network policy.
- **Developer:** Tier 3, SDKs, build tools, RAR Lab integration, inspection, provisional components.
- **Vehicle:** Tier 1+, separated safety and experience domains; no safety claim without certification.

## Tier change

Upward transition:

1. Verify hardware/resource prerequisites.
2. Resolve and download signed missing layers.
3. Install without replacing current working graph.
4. Activate services and migrate only required state.
5. Change experience policy when new capabilities appear.
6. Commit after health validation.

Downward transition:

1. Identify dependents and present consequences.
2. Quiesce affected components.
3. Preserve state as dormant unless explicitly deleted.
4. Remove executable layers and release resources.
5. Retain rollback metadata according to policy.

A device cannot activate a tier whose security or resource requirements it cannot meet. It may still install compatible assets for future use.

## Compatibility

- Higher tiers run lower-tier RID interfaces directly or through specified adapters.
- Portable bytecode is preferred for architecture-neutral Tier 0 behavior.
- Native cross-architecture handoff serializes state and restarts a compatible component; it does not copy raw memory.
- Components must query capabilities and handle their disappearance.
- Profile-specific behavior cannot become an undocumented requirement of a tier.

## Open measurements

Image-size, memory, boot, energy, and latency budgets will be measured in simulation but set as release requirements only after representative physical hardware is selected.
