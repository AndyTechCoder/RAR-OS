# RAR OS From-Scratch and Dependency Policy

Status: Gate 0 approved direction — 2026-07-16
Applies to: all code, firmware, assets, tools, packages, images, SDKs, tests, simulations, and documentation used to build or distribute RAR OS

## Purpose

RAR OS should own and understand as much of its implementation as realistically possible.

“From scratch” does not mean ignoring processor manuals, hardware protocols, internet standards, established cryptography, or development tools. It means that the shipped operating system is designed and implemented as RAR OS rather than assembled from another operating system or an uncontrolled collection of external runtime libraries.

This policy protects four goals:

1. RAR can understand and modify its system deeply.
2. Critical behavior is not permanently controlled by external projects.
3. Necessary compatibility with hardware and networks remains possible.
4. External code and firmware never enter the system invisibly.

## Core rule

RAR should create its own implementation whenever all of the following are true:

- The behavior belongs in the shipped OS or SDK runtime.
- A public specification or sufficient technical understanding exists.
- Reimplementation is legally and technically possible.
- A custom implementation provides meaningful control, clarity, integration, safety, efficiency, or future replaceability.
- The implementation can be validated to the level required by its risk.

Existing code is not adopted merely because it is convenient or popular.

## Classification model

Every non-RAR input is assigned to exactly one primary class.

### Class A — External standard or specification

Examples:

- CPU architecture manuals
- UEFI
- USB
- PCI Express
- Ethernet
- Wi-Fi
- Bluetooth
- IPv4 and IPv6
- TCP, UDP, DNS, TLS, and HTTP
- Unicode and supported font formats
- Image, audio, and media formats
- Established cryptographic algorithms

Policy:

- RAR may and usually should implement the standard itself.
- The standard defines compatibility, not RAR’s internal architecture.
- RAR extensions must not silently break interoperability.
- Deviations and unsupported portions must be documented.
- Conformance and interoperability tests are mandatory before claiming support.

### Class B — Host development tool

Examples:

- Rust compiler and related compiler infrastructure
- C/C++ compiler
- Assembler and linker
- QEMU or another emulator
- Debuggers and binary inspection tools
- Documentation generators
- Source control and CI runners
- Static analysis and fuzzing tools

Policy:

- Host tools may be used to bootstrap development.
- They are not considered part of the shipped RAR runtime unless installed as an explicit development layer.
- Versions, checksums, licenses, and setup instructions must be pinned and documented.
- Build behavior that affects target output must be reproducible.
- RAR should eventually be able to build itself from within its development tier.
- Replacing every compiler and emulator is not required before initial OS development.

### Class C — Host-only test or reference implementation

Examples:

- Protocol reference implementations
- Official cryptographic test-vector runners
- Differential-testing tools
- Filesystem image inspectors
- Network packet generators
- Model checkers
- Simulator backends

Policy:

- Host-only reference code may be used to test RAR behavior.
- It must not be linked into target images or silently become production logic.
- Differential testing does not replace RAR-owned specifications and tests.
- Test provenance and licenses must be recorded.

### Class D — Target data or asset

Examples:

- Fonts
- Unicode databases
- Time-zone databases
- Locale information
- Hardware-description tables
- Certificate trust lists
- Public suffix or protocol registries
- Calibration data

Policy:

- Data and creative assets are not automatically treated as runtime-code dependencies.
- Their source, version, license, update process, and integrity hash must be documented.
- Parsers and runtime behavior should be RAR-owned.
- RAR-specific generated forms must be reproducible from the documented source.
- User-replaceable or jurisdiction-specific datasets should not be hard-coded into the Nucleus.

### Class E — Device firmware or microcode

Examples:

- Wi-Fi adapter firmware
- GPU firmware
- Storage-controller firmware
- CPU microcode
- Secure-element firmware
- Camera or media-processor firmware

Policy:

- Signed vendor firmware is allowed when the hardware cannot function without it and no practical RAR-owned replacement exists.
- Firmware must be isolated from normal OS authority as far as hardware permits.
- Every payload requires provenance, exact supported hardware, license, version, hash, signing status, update path, known privileges, and security history.
- Firmware must be delivered and updated independently from RAR executable components where possible.
- The corresponding RAR driver must treat firmware as potentially fallible and validate all shared data.
- Hardware requiring undocumented firmware may receive a lower assurance rating.
- A future open or RAR-owned replacement must remain architecturally possible.

### Class F — External target-linked runtime code

Examples:

- Third-party kernel libraries
- External allocators or collections
- External filesystem implementations
- External network stacks
- External TLS or cryptography libraries
- External GUI frameworks
- External application runtimes
- External compression or serialization libraries
- External driver frameworks

Default policy: prohibited.

An exception requires the formal process defined below. Approved code must be isolated behind a stable RAR interface and must not spread its types, assumptions, global state, or build system across unrelated subsystems.

### Class G — Imported application or compatibility code

Examples:

- Future Linux/POSIX compatibility environment
- Ported language runtime
- Third-party application
- Game engine
- Browser engine
- Developer tool installed in Tier 3

Policy:

- This code may exist in an optional application, package, or compatibility layer.
- It is not part of the native RAR architecture or trusted foundation.
- It receives normal capabilities and isolation.
- Its license, origin, updates, and security boundary must be visible.
- Compatibility code must be removable without preventing native RAR operation.

## What RAR must implement itself

Unless an approved exception exists, RAR-owned target code includes:

- Boot manager and recovery coordination
- Nucleus and architecture adapters
- Memory management, scheduling, IPC, and capabilities
- Component runtime, loader, lifecycle, and service routing
- Native executable, interface, manifest, package, and state formats
- Allocators, collections, target runtime support, and native standard libraries
- Filesystem, System Store, Data Vault, snapshots, and migration engine
- Cryptographic implementations used by the OS
- Signing, verification, trust, rollback, and recovery logic
- Network, transport, discovery, and update protocols
- Driver framework and supported hardware drivers
- Graphics, compositor, software renderer, UI runtime, and accessibility interfaces
- User, application, service, device, and agent identity systems
- Native Rust and C SDK runtime components
- Diagnostics, logging, tracing, update, repair, and recovery applications

Writing an implementation internally does not remove the requirement to follow compatible standards or validate security.

## What RAR must not do

RAR OS must not:

- Use Linux, BSD, Android, or another kernel as its normal foundation.
- Present a customized existing distribution as RAR OS.
- Copy external code without compatible licensing and recorded provenance.
- Copy code from leaked, confidential, illegally obtained, or reverse-engineered proprietary source.
- Automatically paste generated code into trusted components without review and tests.
- Import a library because a feature seems too inconvenient to implement.
- Modify a cryptographic primitive casually to make it “RAR-specific.”
- Claim an external implementation is RAR-owned merely because it was forked or renamed.
- Hide external firmware, generated data, or build tools from dependency records.
- Allow one approved exception to become an informal precedent for unrelated dependencies.

## Learning from existing systems

RAR may study publicly available:

- Specifications
- Research papers
- Architecture descriptions
- Public documentation
- Benchmarks
- Open-source behavior and code where licenses permit examination
- Published failure analyses and security advisories

The preferred process is:

1. Understand the problem and external approaches.
2. Write an RAR requirement and interface specification.
3. Record useful ideas and known failure modes.
4. Create an implementation suited to RAR’s architecture.
5. Validate independently with tests and interoperability checks.

When license compatibility, originality, or clean-room development matters, the specification and implementation roles must be separated and documented.

## Forking or modifying existing code

Forking target runtime code is an exception, not the default.

A fork may be considered when:

- No complete public specification exists.
- Hardware enablement would otherwise be impossible.
- The code is highly specialized and rewriting offers little architectural value.
- The code can remain isolated behind a RAR-owned interface.
- Its license is compatible with the proprietary RAR source model.
- The maintenance and replacement costs are understood.

Before approval, compare:

- Cost and risk of a RAR implementation
- Security history and code quality
- Size and transitive dependencies
- License and distribution obligations
- Portability assumptions
- Ability to test and audit it
- Difficulty of future replacement

Minor edits to an external library do not create architectural ownership. If RAR needs deep control over a small or central subsystem, reimplementation is normally preferred over a permanent fork.

## Cryptography policy

RAR may implement established cryptographic standards internally, but cryptography is treated differently from ordinary utility code.

Requirements:

- Use established, publicly reviewed primitives and protocols.
- Implement from normative specifications and official test vectors.
- Maintain constant-time behavior where secret-dependent timing is relevant.
- Test against multiple independent implementations.
- Fuzz parsers, state machines, and invalid-input handling.
- Separate keys, algorithms, protocols, and policy behind versioned interfaces.
- Support algorithm agility without silently downgrading security.
- Require specialist review before enabling a new implementation for sensitive data.
- Require independent audit before production security claims.

RAR-specific protocols may compose established primitives only through a documented design and threat analysis. “Improving” a primitive by changing its mathematics is outside normal OS implementation and requires a separate cryptographic research and review program.

## Generated and AI-assisted code

AI may assist with design, implementation, tests, documentation, and review, but generated code receives no automatic trust.

For target code, the accepting agent or reviewer must establish:

- The intended behavior is specified.
- The code’s provenance is acceptable.
- No incompatible copied material is apparent.
- Unsafe operations and security assumptions are documented.
- Tests cover normal, boundary, malformed, and failure cases.
- The code follows dependency and architecture rules.
- A human-readable explanation exists for critical algorithms.

Trusted code that cannot be adequately explained, tested, or reviewed must not be accepted merely because it appears to work.

## Dependency manifests

Every build output must be traceable to:

- RAR source revision
- Toolchain versions and hashes
- External specification versions where behavior depends on them
- Host-only dependency lockfile
- Target assets and data
- Firmware payloads
- Approved target-code exceptions
- Build configuration and target hardware profile
- Generated-code inputs

Release manifests must distinguish RAR-owned code, external firmware, external data/assets, and optional imported applications.

## Exception process

An external target-linked dependency requires a Dependency Exception Record containing:

1. The exact repository, revision, version, hash, and license.
2. The required capability and why RAR cannot reasonably implement it now.
3. Alternatives considered.
4. Full transitive dependency inventory.
5. Security and privacy analysis.
6. Runtime privilege and data access.
7. Isolation boundary.
8. Update and vulnerability-response owner.
9. Replacement interface and migration plan.
10. Review date or removal milestone.

Approval requires architecture, security, licensing, and project-owner review.

Emergency exceptions may be time-limited but must not enter a stable release without normal approval.

## Enforcement

The build and CI systems must:

- Fail when undeclared target dependencies are linked.
- Generate dependency and firmware inventories for every image.
- Check pinned hashes and licenses.
- Detect duplicate vendored code and unexpected binary blobs.
- Separate host-tool dependencies from target-runtime dependencies.
- Require an approved exception identifier for allowed external target code.
- Verify that exception review dates have not expired.
- Produce a human-readable dependency report with every release.

## Relationship to self-hosting

Initial RAR OS builds may use external host compilers, assemblers, linkers, and emulators.

Self-hosting later means RAR OS can run the required development tools and rebuild its own source. It does not retroactively make compiler source RAR-owned, nor does it require compiler code to become part of lower tiers.

The long-term toolchain may evolve toward more RAR-owned infrastructure, including a future RAR language, but that is separate from eliminating external code in the shipped OS runtime.

## Review checklist

Before approving this policy, confirm:

- RAR may implement external standards without importing external runtime code.
- Host compilers, emulators, and test tools are permitted and documented.
- External target-linked runtime code is prohibited by default.
- Signed vendor firmware is permitted but isolated and visible.
- Fonts and maintained public datasets are treated as assets/data, not hidden code.
- Cryptographic implementations follow established standards and receive stronger review.
- Optional third-party applications do not redefine native RAR architecture.
- Every exception has a replacement path and expiry/review point.
