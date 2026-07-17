# R0-002 hardware and boot contracts

Status: Owner-approved contracts implemented; exact-head review and CI required before merge

## Delivered boundary

R0-002 defines compiler-independent Release 0 v1 byte contracts for x86-64 and AArch64 hardware description and boot handoff. The branch includes machine-readable wire rows, deterministically generated unsafe-free `no_std` Rust semantic types, committed raw binary structural fixtures, a checked host-only reference oracle, and exact-head CI coverage.

The public decisions are recorded in [ADR 0013](../../adr/0013-pre-copy-trust-and-mmio-authority.md), [ADR 0014](../../adr/0014-hardware-binding-and-record-identity.md), [ADR 0015](../../adr/0015-deterministic-validation-precedence.md), and [ADR 0016](../../adr/0016-release-0-entry-validation-and-authority-closure.md). They establish the immutable pre-copy entry boundary, separate device authority, sole record-header identity, typed register windows, staged deterministic validation, exact alias/memory rules, and architecture entry preconditions.

Out of scope and absent: target boot code, Nucleus implementation, Tier 0 layout, firmware callbacks, executable pointers, storage, networking, GUI, agents, packages, applications, VM/emulator launch, device access, trace-record framing, signatures, and entropy authenticity claims.

## Contract index

- `spec/boot/handoff-v1.fields` and `handoff-v1.md`: Boot Entry, fixed handoff, authority descriptors, failure codes, ownership, and total predicate order.
- `spec/hardware/rhd-v1.fields` and `rhd-v1.md`: normalized RHD, sole record identity, typed register windows, and model rules.
- `sdk/generated/release-0/lib.rs`: byte-for-byte regenerated owned Rust representation; Rust layout is not wire ABI.
- `spec/fixtures/release-0/bin/*.bin`: valid and malformed raw binary fixture bundles.
- `spec/fixtures/release-0/validation-precedence.v1`: executable declarations for all 36 focused predicates, all 35 adjacent edges, and eight security-sensitive non-adjacent pairs.
- `spec/fixtures/release-0/conformance-scenarios.v1`: both-architecture adapter, provider, compatibility, access-log, and effect-sink cases.

The authoritative boot memory map describes dynamic range ownership. RHD memory records describe the same normalized topology and must compare equal. An RHD register window is descriptive only: access also requires its exact Boot Entry authority descriptor, and system-memory windows require device-owned MMIO map containment. Entropy is explicitly untrusted seed input. The trace channel is only a bounded versioned byte sink; its record format belongs to R0-008.

## Acceptance and evidence

| Requirement | Evidence |
| --- | --- |
| CPU, memory, interrupts, timers, serial, boot source, reserved ownership | sole-ID RHD records plus authoritative memory-map kinds/owners |
| Trusted entry and immutable snapshot | exact architecture-adapter tuple, inline `BootEntryV1` descriptors, descriptor-keyed provider, and enforced one-copy rule |
| Device description without authority escalation | typed RHD windows cross-checked against owner-bound MMIO/I/O descriptors |
| Bounds, alignment, ownership, deterministic failure | staged 36-row predicate table executed as 36 single, 35 adjacent, and eight non-adjacent compound-invalid cases |
| x86-64/AArch64 normalized categories | valid APIC/16550-I/O and GICv3/PL011 raw bundles decoded by one oracle |
| Required malformed classes | both-architecture framing, alias, reordered-descriptor, provider-fault, minor-version, cardinality, model, authority, consistency, and architecture cases |
| No unverified pointer execution | host oracle uses byte slices and checked `u64` arithmetic; fixture addresses are never dereferenced |
| Generated Rust source consistency | complete deterministic rendering from `rust-*` schema rows, byte comparison, pinned Linux metadata compilation |

Host-only checks:

```sh
/bin/sh spec/fixtures/release-0/run.sh
/bin/sh sdk/generated/release-0/check.sh
/bin/sh -n spec/fixtures/release-0/generate.sh
/bin/sh -n spec/fixtures/release-0/run.sh
/bin/sh -n sdk/generated/release-0/generate.sh
/bin/sh -n sdk/generated/release-0/check.sh
tools/ci/check-specs.sh
```

The required `Specifications` workflow additionally runs `spec/fixtures/release-0/run.sh --ci` and `sdk/generated/release-0/check.sh --compile` in the pinned Rust 1.95.0 read-only Linux container. The workflow binds checkout and evidence to the PR head through `RAR_EXPECTED_SOURCE_REVISION`. Generated Rust receives metadata-only host compilation; the host-only reference oracle runs against committed binary fixtures. No RAR target code is compiled or executed on the physical Mac or in CI.

## Security, unsafe, dependencies, and recovery

The contracts expose no host virtual address, raw Rust pointer, function pointer, firmware callback, or unchecked register operation. Candidate ranges use checked integer arithmetic, immutable snapshots, explicit access windows, pairwise disjointness, bounded single copies, and deterministic first-error codes. The instrumented effect sink proves rejection cannot clear entropy, activate trace, or construct device authority. Structural validation does not authenticate the producer.

Generated Rust and the host reference oracle deny unsafe code. The committed `.bin` files are deterministic host-only conformance inputs, not target assets or executable payloads. This task adds no assembly, third-party crate, target-linked code, firmware, external dependency, persistent user data, migration, or signing behavior. Invalid input enters the later R0 platform's defined invalid-handoff recovery halt without granting authority.

## Versioning and limitations

Changes to identity, descriptor authority, required register roles, address-space meaning, or predicate precedence require a new major version and parallel decoder. Safely ignorable non-critical additions require explicit minor-version compatibility analysis. Platform evidence may require a versioned correction, never silent reinterpretation.

R0-002 provides structural host-oracle evidence only. Agreement by the real architecture decoders belongs to the later authorized R0-003/R0-004 implementation tasks. Prompt 7 and all target/VM execution remain out of scope.
