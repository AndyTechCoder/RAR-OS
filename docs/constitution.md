# RAR OS Constitution

Status: Gate 0 approved direction — 2026-07-16
Applies to: RAR OS, its tiers, system components, SDKs, applications, recovery environments, and supported device profiles

## Purpose

RAR OS exists to create a modern, adaptable operating system for people, applications, agents, and machines.

It is intended to grow from tiny connected devices into personal computers, robots, vehicles, servers, and future device categories without becoming a collection of unrelated operating systems. It should remain fast, understandable, repairable, and open to major internal improvement throughout its lifetime.

This constitution defines the principles that implementation decisions must preserve. It does not permanently lock a particular kernel design, programming language, interface style, or visual design.

## 1. One operating system

RAR OS is one coherent operating system with shared foundations, interfaces, security concepts, package rules, and development tools.

Hardware and product differences are expressed through cumulative tiers, capabilities, layers, and device profiles—not incompatible editions.

A component written for a lower tier should work on a higher tier when its required capabilities are present. A device may add or remove supported layers without being treated as a different operating system.

## 2. Adaptation instead of editions

RAR OS adapts its behavior to available hardware, displays, input methods, sensors, resources, and user intent.

A phone connected to a display may provide a desktop workspace without changing operating systems. A headless device may gain a graphical layer. A larger system may run software created for a small ecosystem node.

Applications should describe their needs and presentation intent instead of assuming one fixed device shape.

## 3. People remain the highest authority

The device owner and authorized users control their devices, data, permissions, trusted software sources, agents, and recovery choices.

Applications, services, administrators, remote systems, and AI agents receive explicit authority. None receives invisible universal access merely because it is important to the system.

Sensitive actions must be attributable, reviewable, and revocable wherever technically possible.

## 4. Privacy is structural

Privacy must be enforced by architecture rather than promises alone.

Components receive only the data, devices, communication paths, and actions they require. Local processing is preferred when it provides an acceptable result. Network access is explicit and inspectable.

RAR OS must not require unnecessary cloud accounts or remote services to perform essential local functions. User data must not silently become analytics, advertising, or model-training data.

## 5. Software and user data are separate

Replaceable software must not own irreplaceable user data.

The trusted recovery foundation, installed system components, application code, user data, application state, and disposable caches must occupy clearly separated security and storage domains.

Reinstalling, repairing, upgrading, removing, or rewriting an OS component must not erase unrelated intact user data. Data migrations must be explicit, transactional, validated, and reversible when feasible.

## 6. Recovery exists below the everyday OS

Every supported device class must have a minimal trusted recovery path appropriate to its hardware.

The recovery foundation verifies executable system state, isolates damage, reconstructs replaceable components, and protects intact data. Normal applications and services cannot modify it freely.

RAR OS must prefer repairing the smallest damaged unit. Rebuilding the entire installed system is a fallback, not the normal response to one failed component.

No software-only design can guarantee survival after physical destruction or total storage failure. RAR OS must state such limits honestly and support backups or replication where appropriate.

## 7. Replaceability is a permanent requirement

Every substantial subsystem must be designed with the expectation that it may eventually be replaced or rewritten completely.

A replaceable subsystem requires:

- A documented responsibility and boundary
- Versioned inputs, outputs, and failure behavior
- No undocumented dependence on another subsystem's internals
- Separate persistent state with a documented schema
- Migration and rollback procedures
- Conformance tests that a replacement can run
- A way to operate provisionally before becoming trusted

Some foundational interfaces and formats require greater stability, but none is exempt from documented evolution and migration planning.

## 8. Components fail independently

A failure in one application, driver, service, agent, or optional layer should not collapse unrelated parts of the system.

RAR OS must isolate failures, revoke damaged components' authority, preserve useful diagnostic evidence, and restore a known-good implementation when possible.

Components must declare their dependencies, requested capabilities, resource expectations, health checks, persistent state, and recovery behavior.

## 9. Updates are narrow, transactional, and reversible

Routine updates replace only the components and data transformations that changed.

RAR OS verifies updates before activation, preserves the previous working state, validates replacements, and rolls back automatically when activation fails. Routine component updates should not require restarting the whole device.

Deep changes to the trusted foundation may require a controlled restart. The system must not compromise integrity merely to claim that every update is restart-free.

Update size should be determined primarily by changed content, not by the total size of the installed OS.

## 10. RAR owns its operating system

RAR OS is not built on Linux, Android, BSD, or another existing operating system.

Where interoperability requires an external standard, RAR should normally create and maintain its own implementation from the specification. Internal protocols and formats may be RAR-native when external compatibility provides no meaningful benefit.

External ideas may be studied and improved upon. Existing code should be adopted only when rewriting has no proportionate benefit or when hardware makes an external payload unavoidable.

Every external runtime dependency or firmware payload must have documented provenance, licensing, update policy, security boundary, and replacement path.

Established cryptographic algorithms must not be casually modified. RAR implementations require official test vectors, interoperability testing, specialist review, and independent audit before production security claims.

## 11. Standards serve interoperability

RAR follows external standards when doing so is necessary to communicate with processors, hardware, networks, media, storage, security systems, or other software.

Standards are not automatically adopted when they impose obsolete architecture on internal RAR behavior. Compatibility belongs behind explicit interfaces so it cannot silently define the whole OS.

RAR-native standards must be documented sufficiently for independent implementations and future migration.

## 12. Hardware versatility is designed in

Virtual machines are the first safe and reproducible testing environment, not the final target of RAR OS.

The architecture must distinguish universal behavior from processor, board, and device-specific behavior. Physical and simulated devices should use the same driver-facing contracts where practical.

Supporting a new device should primarily require a hardware description, boot adapter, drivers, firmware declarations, and a profile—not a fork of the operating system.

Hardware support claims require physical testing. Simulation alone must never be presented as proof that a physical device is supported.

## 13. Performance and efficiency are features

RAR OS should remain responsive under load and economical when idle.

Components must have observable resource behavior. The system should measure processor time, memory, storage, energy, network use, latency, and background activity without requiring invasive debugging.

Architecture abstractions must justify their cost. Performance problems should be measured and fixed without discarding isolation or safety by default.

Lower tiers must not inherit unnecessary higher-tier code, services, or resource requirements.

## 14. Simplicity outside, clarity inside

Normal users should encounter understandable actions such as install, remove, allow, connect, update, undo, and repair.

The operating system may be internally sophisticated, but it must not expose complexity without benefit. At the same time, simplicity must not become secrecy: advanced users and developers should be able to inspect components, permissions, updates, resource use, failures, and recovery decisions.

Important system behavior must be explainable without requiring source-code archaeology.

## 15. Applications are adaptive and native

RAR applications use documented RAR interfaces rather than depending on another operating system's assumptions.

Applications should preserve identity and state while adapting to different displays, inputs, tiers, and device profiles. Application authority remains explicit and independent from visual presentation.

Future compatibility environments may be added as isolated optional layers. They must not redefine the native RAR application model.

## 16. Agents are first-class but not all-powerful

Agents use the same identity, capability, resource, audit, and recovery foundations as other software.

An agent may coordinate applications and devices only through authority granted by a person or trusted policy. Agent actions must be attributable. Sensitive or irreversible actions require suitable confirmation or pre-authorized policy.

RAR OS must remain bootable, recoverable, and meaningfully usable without Pal or another AI model.

## 17. The SDK is part of the operating system

Interfaces, build tools, debugging, packaging, signing, simulation, examples, and documentation are part of the product—not optional developer conveniences.

RAR should provide stable language-neutral contracts with first-party SDKs. A future RAR programming language should emerge from demonstrated application and systems needs rather than delaying the initial foundation.

Developers must be able to test a component independently, inspect its authority, simulate failures, replace it provisionally, and understand why it failed.

## 18. Documentation is implementation

A feature is incomplete until its behavior can be understood and independently verified.

Every subsystem must document:

- Purpose and non-goals
- Architecture and boundaries
- Public interfaces and persistent formats
- Security and privacy assumptions
- Resource behavior
- Failure, update, migration, and recovery behavior
- Examples and debugging guidance
- Tests and acceptance criteria
- Known limitations and future replacement path

Public behavior changes require updated documentation, tests, and migration notes in the same change.

## 19. Tests define promises

Every public interface, persistent format, security boundary, recovery path, and compatibility claim requires executable tests.

RAR OS should use deterministic simulation, fault injection, fuzzing, conformance tests, interoperability tests, record/replay, and physical validation as appropriate.

A subsystem is not considered replaceable until an independent implementation could use its documentation and conformance tests to reproduce the required behavior.

## 20. Security and safety claims are evidence-based

RAR must distinguish design goals, tested guarantees, audited guarantees, and certified guarantees.

Virtual testing does not prove physical safety. Internal review does not replace independent security audit. General reliability does not equal suitability for flight, medical, automotive, industrial, or other safety-critical control.

Limitations must be visible in releases and documentation.

## Decision hierarchy

When principles conflict, decisions should generally prioritize:

1. Protection of people and prevention of unsafe physical action
2. Preservation of user authority, privacy, and intact data
3. Integrity, isolation, recovery, and truthful behavior
4. Compatibility of persistent state and documented interfaces
5. Responsiveness, efficiency, and availability
6. Simplicity and ease of use
7. Compatibility with external software
8. Implementation convenience

This ordering does not eliminate judgment. Significant tradeoffs must be recorded in an Architecture Decision Record.

## Exceptions

An exception to this constitution requires:

- A documented reason
- The affected principles and risks
- Alternatives considered
- The narrowest possible scope
- A security and data-impact analysis
- An owner and expiry or review date
- A plan to remove or replace the exception where feasible

Exceptions must not be hidden inside implementation details.

## Amendment process

This constitution is intended to remain stable but is not immutable.

An amendment must explain why the existing principle is insufficient, how the new wording affects current systems and user promises, which migrations are required, and whether the change weakens privacy, ownership, security, recovery, replaceability, or compatibility.

Technical implementation changes that preserve these principles do not require a constitutional amendment.

## Review checklist

Before approving this draft, confirm that it accurately captures:

- The intended meaning of one adaptable RAR OS
- The desired balance between custom implementation and external standards
- User ownership and privacy expectations
- System/data/recovery separation
- Replaceability and rewriteability
- Live updates, rollback, and repair
- Hardware versatility beyond virtual machines
- Simple user experience with deep inspectability
- Applications, agents, SDKs, and the future language
- Documentation and testing as completion requirements
