# RAR OS Architecture

Status: Gate 0 approved direction — 2026-07-16

## Architectural choice

RAR OS uses a capability-based hybrid microkernel architecture implemented primarily in `no_std` Rust with small architecture-specific assembly modules.

This is a recommended engineering choice, not an irreversible constitutional rule. It is selected because RAR needs failure isolation, live component replacement, strong permissions, broad portability, and the ability to rewrite services independently.

## System stack

1. Hardware, firmware, or virtual hardware
2. RAR Root and Recovery Seed
3. RAR Nucleus
4. RAR Core services and component fabric
5. Replaceable hardware and system services
6. Installed layers and profiles
7. Applications, system experiences, Pal, and other agents
8. People and external paired devices interacting through policy

Vertical position does not grant authority. Capabilities do.

## Nucleus boundary

The Nucleus contains only mechanisms requiring highest privilege:

- Early architecture initialization
- Physical and virtual memory protection
- Address spaces and mapping
- Threads, scheduling classes, timers, and interrupts
- Capability object enforcement
- IPC primitives and waiting
- Shared-memory and DMA mediation
- Minimal entropy plumbing
- Crash capture, tracing hooks, and recovery entry

The Nucleus does not contain filesystems, network stacks, application policy, GUI, package management, AI, user databases, or general drivers unless hardware makes a narrowly documented exception unavoidable.

## RAR Core

RAR Core consists of isolated services that establish the universal component environment:

- Component loader and lifecycle manager
- Service registry and logical endpoint routing
- Capability broker and policy evaluator
- Principal and device identity coordination
- Declarative system-graph manager
- Resource budget and power-policy coordinator
- Update, state-migration, health, quarantine, and rollback coordinator
- Driver discovery and binding manager
- Audit, logging, tracing, and recovery escalation

Core services remain separately replaceable but receive more conservative update policy because other components depend on them.

## Execution containers

A component is the unit of behavior and replacement. An execution container is the isolation mechanism hosting one or more components.

- Components run in separate protected containers by default.
- Co-location requires identical trust domain, compatible lifecycle, explicit manifest permission, and measured benefit.
- Co-located components still use RID-defined boundaries in source code.
- A component cannot assume a stable process ID, address, or neighboring component.
- Tier 0 maps the same model to verified bytecode or static compartments where processes are unavailable.

## IPC model

RID defines typed interfaces. The transport supports:

- Request/response operations
- One-way asynchronous messages
- Event subscriptions
- Bounded streams with backpressure
- Shared immutable or explicitly owned buffers
- Cancellation, deadlines, and structured errors
- Logical endpoint rebinding during replacement

Messages are validated at trust boundaries. Small messages use kernel-mediated transfer; large data uses capability-scoped shared memory or zero-copy device buffers.

## Capability model

Capabilities are unforgeable handles to objects or actions. A process-local handle table prevents components from manufacturing authority from numeric values.

Capabilities may carry:

- Allowed operations
- Resource or data scope
- Destination restrictions
- Time or usage limits
- Delegation limits
- Confirmation requirements
- Audit policy

Remote capabilities use separately authenticated tokens and never expose raw local handles.

## Scheduling

The Nucleus provides:

- Fair general-purpose scheduling
- Interactive latency hints
- Deadline and realtime classes
- CPU-affinity and isolation controls
- Per-principal and per-component budgets
- Priority-inversion handling for IPC

Safety-critical controllers receive reserved resources and cannot depend on an agent, GUI, cloud service, or best-effort scheduler. Exact algorithms remain replaceable behind scheduler policy interfaces and conformance tests.

## Memory

- Per-container virtual address spaces where an MMU exists
- W^X executable-memory policy
- Guarded stacks and validated executable mappings
- Capability-scoped shared memory
- Explicit pinned/DMA buffers controlled by a DMA broker
- Memory quotas and pressure notifications
- Recoverable out-of-memory policy rather than uncontrolled global termination
- Tier 0 static regions and verified bytecode bounds where hardware isolation is absent

## Drivers

- Drivers are services outside the Nucleus by default.
- Bus managers enumerate hardware and bind signed driver components by declared identifiers.
- Drivers receive only their device registers, interrupts, DMA windows, firmware, and required services.
- Consumers use device-class RID interfaces, not chipset-specific calls.
- Virtual and physical drivers implement the same class contracts.
- Driver failure triggers device isolation, restart, rebinding, or fallback without crashing unrelated services.

## Hardware description

Architecture ports consume a normalized RAR Hardware Description derived from UEFI/ACPI, device tree, board manifests, or static Tier 0 configuration. It describes processors, memory, interrupts, buses, devices, firmware, security facilities, and reserved regions.

Platform-specific discovery does not leak into portable services.

## System composition

The signed system graph declares:

- Active components and implementations
- Interface and state-schema versions
- Dependencies and activation order
- Capabilities and policy sources
- Resource budgets
- Hardware bindings
- Installed tiers, profiles, and layers
- Rollback and recovery candidates

RAR Core computes an activation plan, rejects unresolved or unsafe graphs, and commits graph changes transactionally.

## Language and ABI

- Nucleus and first-party target services: `no_std` Rust plus reviewed assembly.
- Stable native boundary: RAR ABI and RID-generated wire/data contracts, not Rust ABI.
- C applications and components use generated headers and `librar`.
- Rust applications use generated crates and a RAR-owned core/alloc/runtime layer.
- Unsafe Rust is isolated, justified, reviewed, and tested.
- The future RAR language targets the same ABI and interfaces.

## Boot flow

1. Firmware or platform reset enters RAR Root.
2. Root validates anti-rollback state and a Recovery Seed slot.
3. Recovery Seed validates the selected Nucleus and base system graph.
4. Nucleus initializes isolation and starts the minimal Core bootstrap component.
5. Core validates and activates drivers and services according to the graph.
6. Identity and protected data become available only after integrity and authorization checks.
7. The selected experience/profile starts.

## Compatibility

RAR native interfaces do not reproduce POSIX internally. A future compatibility profile may provide translated processes, files, signals, and APIs in an isolated environment. It cannot receive implicit authority or become required for RAR boot, recovery, SDKs, or native applications.

## Performance strategy

- Measure every abstraction before bypassing it.
- Use bounded messages, zero-copy buffers, batching, and asynchronous work.
- Co-locate components only through declared policy.
- Keep idle services stopped or event-driven.
- Preserve separate implementations for correctness and optimized fast paths behind the same tests.
- Track performance continuously in VMs; set physical budgets after hardware validation.

## Architecture acceptance

The architecture is proven when x86-64, ARM64, and Tier 0 simulations run the same component contracts; failures remain contained; a service implementation can be replaced without client changes; and system reconstruction preserves intact Data Vault state.
