# Public Interfaces and Persistent Formats

Status: Gate 0 approved direction — 2026-07-16

## Rule

Public contracts and persistent formats are specified before production implementation. Generated Rust and C bindings are derived from the same source. Compiler-specific memory layouts are never public contracts.

## RID — RAR Interface Definition

RID describes:

- Interface identity and semantic version
- Operations, messages, events, streams, and shared buffers
- Field types, bounds, optionality, ownership, and validation
- Structured errors, deadlines, cancellation, and retry safety
- Required capabilities and audit category
- Lifecycle and health operations
- Compatibility and deprecation metadata

RID compiler outputs target bindings, validators, documentation, test fixtures, and conformance stubs.

## RME — RAR Metadata Encoding

A deterministic bounded binary encoding used for manifests and signed metadata.

- Canonical ordering and integer representation
- Explicit lengths and total-size ceilings
- No duplicate fields
- Optional and critical extension fields
- Streaming-safe parsing
- Stable hashing independent of in-memory representation
- Generated parsers with malformed-input tests

## RCI — RAR Component Image

Contains:

- Component manifest and RME version
- Architecture slices or RBC portable bytecode
- Entry points and RID interface table
- Read-only assets
- Relocation/import information through the stable RAR ABI
- Content hashes and publisher signature references

Native slices initially support x86-64 and ARM64. RCI does not expose Rust ABI.

## RBC — RAR Bytecode

Deterministic portable execution format primarily for Tier 0 and portable policies.

- Statically typed and verified before execution
- Bounded memory and stack declarations
- No ambient syscalls; capabilities are explicit imports
- Deterministic mode for control and replay
- Instruction, time, memory, and energy budgets
- Versioned instruction set and verifier

RBC is not intended to replace optimized native code universally.

## RLM — RAR Layer Manifest

Declares layer identity, publisher, version, tier requirements, architecture support, components, dependencies, conflicts, capabilities, privacy labels, budgets, state schemas, migrations, health checks, rollback, firmware, SBOM, documentation, hashes, signatures, transparency proof, and revocation generation.

## RPK — RAR Package

Transport envelope containing an RLM/RSM plus content-addressed chunks. Packages support resumable acquisition, independent chunk verification, deduplication, local repository and peer transfer, and offline bundles.

Installation is a system-graph transaction; unpacking files is not installation.

## RSM — RAR System Manifest

The complete reproducible declaration of an installed system:

- Hardware profile and assurance level
- Root, Recovery, Nucleus, and Core identities
- Active tiers, profiles, layers, and component versions
- Interface and state-schema versions
- Capability/policy bindings
- Resource budgets and device bindings
- User-independent configuration
- Trusted roots and rollback candidates

User secrets and private content are referenced through protected state, never embedded in RSM.

## State schemas

Each stateful component declares schema identity, version, ownership, encryption domain, constraints, export/import format, migrations, downgrade policy, retention, and deletion behavior.

Migrations are separately signed components with narrowly scoped access to old and new state copies.

## Stable RAR ABI

The ABI defines integer sizes, calling convention, handle representation, message framing, buffer ownership, executable mapping, startup data, and component exit behavior for each architecture. It remains deliberately small; most functionality lives in RID services.

## Versioning

- Patch: compatible correction with unchanged contract.
- Minor: additive optional behavior.
- Major: breaking contract requiring adapter or coordinated migration.
- Experimental interfaces carry no long-term stability promise and cannot become silent dependencies of stable layers.
- Persistent formats require readers, writers, migration, recovery, and rollback policy before change.

## Initial core interfaces

The first RID set covers component lifecycle, capability brokerage, endpoint registry, clocks/timers, memory buffers, logs/traces, hardware discovery, block devices, network devices, surfaces/input, state, package/update, identity, audit, and recovery escalation.

Exact fields and wire numbers are generated only after subsystem specifications and threat review.

## Conformance

Every format ships valid, boundary, forward-compatible, malformed, truncated, duplicated, oversized, signature-invalid, and rollback test corpora. Independent parsers must produce identical canonical hashes and validation outcomes.
