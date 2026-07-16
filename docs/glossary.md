# RAR OS Glossary

Status: Gate 0 approved direction — 2026-07-16
Purpose: provide one shared vocabulary for product planning, specifications, code, documentation, tests, and implementation handoffs

These definitions describe intended architectural meaning. They do not lock implementation details that have not yet been reviewed.

## System foundations

### RAR OS

The complete operating system: trusted foundations, executable components, services, applications, SDKs, recovery facilities, tiers, profiles, and the contracts connecting them.

RAR OS is not a Linux distribution or a shared brand for unrelated operating systems.

### RAR Root

The smallest hardware-anchored trust foundation available on a device. It verifies that an approved Recovery Seed may start and protects the earliest trust and anti-rollback decisions.

RAR Root is not the everyday kernel, recovery interface, or full operating system.

### Recovery Seed

The minimal isolated recovery environment verified by RAR Root. It can inspect the installed system, verify components, isolate damage, reconstruct replaceable system code, and protect intact user data.

Where supported, two independently bootable Recovery Seed slots allow safe recovery updates.

### Nucleus

The smallest privileged runtime of the everyday operating system. It provides fundamental enforcement such as memory isolation, execution, scheduling, interrupts, capabilities, and component communication.

The term avoids assuming that RAR must permanently follow the structure of an existing monolithic or microkernel.

### RAR Core

The universal system services immediately above the Nucleus. RAR Core manages components, capabilities, identity, system composition, updates, state coordination, resource policy, and recovery escalation.

RAR Core is distinct from optional graphical, robotics, server, or personal-computing layers.

### Trusted computing base

All hardware and software whose compromise could bypass a stated security guarantee. RAR aims to keep this set small, explicit, measurable, and documented.

Being important to the user experience does not automatically make a component part of the trusted computing base.

## Composition

### Component

The primary replaceable unit of executable behavior in RAR OS. A component has an identity, declared interfaces, dependencies, capabilities, resource expectations, lifecycle, health checks, and update behavior.

Drivers, services, applications, and agent integrations may all contain one or more components.

### Service

A component that provides a documented interface to other components. Services may manage storage, networking, graphics, identity, devices, state, or other shared behavior.

A service is defined by its contract, not by a permanently fixed implementation or process.

### Application

User-facing or user-installed software built on native RAR interfaces. An application may contain visual components, background components, data schemas, commands, and agent-accessible actions.

An application does not gain authority merely because the user can see or launch it.

### System application

An application delivered as part of the RAR OS experience, such as Settings, Files, Terminal, Updates, Recovery, or the graphical shell.

System applications remain capability-controlled and replaceable unless their specification explicitly says otherwise.

### Layer

A signed, installable collection of components, interfaces, assets, configuration, and metadata that adds or replaces system functionality.

Examples may include graphical interaction, robotics, development tools, media support, or a device profile. A layer is not a separate OS edition.

### Tier

A cumulative compatibility and capability contract. A tier states the minimum operating-system facilities a device provides and which lower-tier components it must support.

A tier is not determined only by whether a device has a display, nor is it a marketing edition.

### Profile

A documented composition of layers, defaults, resource policies, and device behavior for a role such as phone, desktop, robot, vehicle, server, sensor, or development system.

Profiles may overlap and change without changing the identity of RAR OS.

### System graph

The declarative description of the components, layers, interfaces, dependencies, capabilities, versions, policies, and state schemas that form a running RAR OS installation.

The system graph must be inspectable, signed where required, reproducible, and sufficient for repair.

### Manifest

Canonical machine-readable metadata describing an executable component, layer, package, system graph, firmware payload, or update.

Security-relevant manifests are signed and strictly validated before their content is trusted.

### Package

A transport and installation unit containing signed manifests and content-addressed executable or data chunks. Installing a package changes the declared system graph transactionally.

A package is not automatically a standalone application; it may contain any approved layer or component.

### Content-addressed chunk

An immutable block identified by a cryptographic digest of its content. Identical chunks can be reused, verified independently, downloaded once, and retained for rollback.

## Identity and authority

### Principal

An identity that may own data, hold capabilities, request actions, or appear in audit history. People, groups, applications, components, services, agents, and devices may all be principals.

### Device owner

The principal with authority to establish device trust, manage users, approve owner-controlled signing roots, initiate recovery, and change fundamental policy.

Ownership does not imply that every application launched by the owner inherits owner authority.

### Capability

An unforgeable, restrictable grant of authority to perform a specific action or access a specific resource.

A capability can be delegated only within its rules and may support limits such as scope, duration, usage count, destination, resource budget, or required confirmation.

### Ambient authority

Access granted implicitly because of process identity, installation location, administrator status, or global environment rather than an explicit capability.

RAR OS aims to minimize ambient authority.

### Delegation

Giving another principal some or all of an existing capability. Delegation must not create more authority than the delegating principal possesses.

### Revocation

Removing or invalidating previously granted authority. Revocation may be immediate or may require a documented transition when an operation is already being committed safely.

### Consent

A person’s informed approval for a specific access or action. Consent is not valid when the request hides its consequence or combines unrelated authority unnecessarily.

### Audit event

A structured record of a security-, privacy-, recovery-, update-, or agent-relevant action and the principals involved.

Audit records must avoid collecting unrelated sensitive content.

## Agents

### Agent

Software that interprets goals, context, or events and may propose or perform multi-step actions through RAR interfaces.

Agents remain principals governed by capabilities, resource limits, consent, auditing, isolation, and recovery.

### Pal

RAR’s first-party AI and agent experience. Pal is expected to integrate deeply with RAR OS through documented agent interfaces but is not part of the Nucleus and does not receive automatic universal authority.

Pal intelligence and model development are separate from the initial RAR OS implementation.

### Tool

A documented action exposed to an agent. A tool defines input, output, required capability, confirmation policy, side effects, errors, and audit behavior.

### Agent provider

A replaceable component that supplies reasoning or decision-making for an agent identity. It may be local, remote, deterministic, model-based, or a test implementation.

The agent’s identity, permissions, and persistent state must not be inseparably owned by one provider.

## Communication

### Interface

A versioned contract describing messages, operations, results, errors, lifecycle behavior, security requirements, and compatibility expectations between components.

### Endpoint

A routable instance of an interface. Logical endpoints can be rebound from an old component to a replacement without requiring every client to know the replacement’s process identity.

### IPC

Inter-component communication within one device. It may use calls, asynchronous messages, streams, events, or shared memory while preserving capability enforcement.

### Device mesh

A secure relationship among paired RAR devices that supports authenticated communication, remote capabilities, data transfer, peripheral sharing, surfaces, and continuity.

The device mesh does not imply that every paired device can access everything on another device.

### Remote capability

A capability intentionally exposed across an authenticated device-mesh connection. It remains scoped, revocable, auditable, and subject to both devices’ policies.

## Data and lifecycle

### State

Persistent or transferable information required to preserve a component’s meaningful behavior across restarts, replacement, updates, suspension, or device handoff.

State is distinct from executable code, temporary memory, disposable caches, and diagnostic logs.

### State schema

A versioned definition of the structure, meaning, constraints, ownership, and compatibility of persistent component state.

### Migration

A transactional transformation from one state schema, component version, storage format, tier, or device context to another.

A migration must define validation, interruption behavior, rollback, and treatment of the original state.

### Snapshot

A point-in-time reference to system or user state that can be inspected or restored without requiring a full duplicate when the storage design supports sharing unchanged data.

### Rollback

Returning executable components, configuration, or state to a previously verified version after a failed or rejected change.

Rollback must not silently discard newer user work.

### Dormant state

Preserved application or component state whose required layer or capability is not currently installed or available.

Dormant state can become active again when compatible functionality returns.

### Data Vault

The protected storage domain containing user data, application state, credentials, and other information that must remain separate from replaceable OS code.

Different users and applications may have separately encrypted and authorized areas within the Data Vault.

### System Store

The verified storage domain containing installed executable components, layers, manifests, and content-addressed chunks.

The System Store is reconstructible without treating intact user data as disposable.

### Scratch storage

Disposable caches, temporary files, build output, and other data that recovery may remove without violating user-data preservation promises.

## Updates and recovery

### Live update

Replacement of a running component without restarting the entire device. A safe live update verifies the replacement, handles state, redirects endpoints, validates health, and preserves rollback.

Live update does not mean arbitrary modification of trusted executable memory.

### Transaction

A change that becomes fully committed or has no externally visible partial result. Transactions are used for installation, state migration, filesystem changes, and system-graph updates.

### Health check

A bounded test used to determine whether a component is ready, responsive, compatible, and safe enough to receive normal traffic or authority.

Passing a health check is evidence for activation, not proof that a component contains no defects.

### Isolation

Preventing a component from affecting resources or principals outside its granted authority. Isolation may use memory protection, process boundaries, capabilities, scheduling, bytecode verification, hardware support, or a combination.

### Quarantine

A restricted state entered after suspected compromise, corruption, or repeated failure. A quarantined component loses relevant capabilities and cannot resume normal service until policy permits it.

### Repair

Restoring correct behavior by replacing or reconstructing the smallest damaged component, validating its state, and safely returning service.

### Reconstruction

Recreating the installed system from its verified manifests and trusted content sources. Reconstruction must not rewrite intact user data merely because system code was damaged.

### Recovery escalation

Moving repair responsibility to a more trusted layer when the current layer cannot establish integrity—for example, from a service manager to RAR Core, then to Recovery Seed.

## Hardware and execution

### Driver

A replaceable component that translates a RAR device interface into the protocol of physical or virtual hardware.

Drivers should run outside the Nucleus unless a documented enforcement or performance requirement proves otherwise.

### Firmware

Executable code running within or before hardware rather than as a normal RAR OS component. Firmware may be vendor-provided, open, or RAR-owned and must have documented provenance and trust policy.

### RAR Vault

The security service protecting device identity, keys, credentials, rollback state, and other high-value secrets. It uses hardware isolation where available and declares its assurance level when only a software fallback exists.

### Attestation

Evidence about a device’s verified hardware or software state. Attestation must be narrowly scoped and privacy-preserving rather than a universal tracking identifier.

### Hardware abstraction

A documented contract separating portable RAR behavior from processor-, board-, and device-specific implementation.

It does not mean reducing every device to the lowest common feature set.

### Hardware profile

A machine-readable description of a supported processor, memory layout, interrupt system, boot path, buses, devices, firmware requirements, and available security facilities.

### Virtual device

A simulated hardware implementation exposing the same RAR-facing contract intended for physical hardware. Virtual devices support reproducible development, fault injection, and automated testing.

### RAR Lab

The host-side simulation and VM-control environment used to run, connect, resize, transform, inspect, record, and damage virtual RAR devices.

RAR Lab controls the environment; RAR OS inside the virtual device performs its real installation, update, security, and recovery behavior.

### Host

The existing computer and operating system used to build or simulate RAR OS during bootstrapping.

### Target

The processor, device, VM, or tier for which RAR software is being built or executed.

### Physical support

A claim that a documented RAR configuration has been tested on the named physical hardware. Architectural compatibility or successful simulation alone is not physical support.

## User experience

### Surface

A graphical or otherwise presentable application area managed by the active experience. A surface may appear as a phone view, window, dashboard panel, remote display, or another presentation chosen by policy and available capabilities.

### Experience

The coordinated user-facing behavior of installed shell, presentation, input, accessibility, notification, and system-interaction components.

An experience is replaceable and does not define a separate RAR OS edition.

### Adaptive application

An application that preserves identity and meaningful state while adjusting presentation and interaction to available displays, input methods, resources, tiers, and profiles.

### Continuity

Preservation or transfer of useful activity across presentation changes, tier changes, devices, or temporary disconnection.

Continuity may use state transfer, remote surfaces, shared peripherals, or component restart rather than raw process-memory migration.

## Development

### SDK

The supported tools, libraries, interface definitions, templates, examples, documentation, simulator integration, debugger support, packaging, signing, and tests required to build RAR software.

### Native RAR application

An application designed for RAR interfaces, capabilities, lifecycle, state, presentation, and packaging rather than another operating system’s ABI.

### Compatibility layer

An optional isolated environment that translates another platform’s software expectations into RAR behavior. It is not part of the native architecture and must remain removable.

### Self-hosting

The ability to build RAR OS, its SDKs, tools, packages, and system images from within RAR OS itself.

Self-hosting does not require compiler or development tools to be installed on every lower-tier device.

### Conformance test

An executable test of a documented interface, format, protocol, or subsystem contract that can be run against independent implementations.

### Architecture Decision Record

A short permanent document recording the context, considered options, selected decision, consequences, and replacement path for an important architectural choice.

### Definition of done

The complete evidence required before work is accepted. For RAR OS, this includes implementation, tests, documentation, failure behavior, migration or rollback where relevant, security analysis, reproducible commands, and known limitations.

## Terms requiring later decisions

The following names or boundaries remain provisional until their dedicated specifications are approved:

- Measured tier budgets and the detailed tier-discovery protocol
- Final boundary between Nucleus and RAR Core
- Component image and portable bytecode formats
- RAR filesystem and state API names
- RAR Lab product name and distribution model
- Final application presentation terminology
- Future RAR programming language terminology
