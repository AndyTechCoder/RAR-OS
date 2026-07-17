# R0-002 hardware and boot contracts

Status: Prompt 6 bounded remediation proposed; owner ADR decisions required before contract freeze

## Delivered boundary

R0-002 drafts compiler-independent Release 0 v1 byte contracts for x86-64 and AArch64 hardware description and boot handoff. The current branch includes machine-readable wire rows, deterministically generated unsafe-free `no_std` Rust semantic types, committed raw binary structural fixtures, a checked host-only reference oracle, and exact-head CI coverage.

Independent Prompt 6 review found three unresolved public decisions. They are isolated in proposed ADRs and are not silently implemented:

- [Proposed ADR 0013](../../adr/proposed/0013-pre-copy-trust-and-mmio-authority.md): trusted entry/snapshot rules and separate MMIO authority.
- [Proposed ADR 0014](../../adr/proposed/0014-hardware-binding-and-record-identity.md): typed hardware windows and canonical record identity.
- [Proposed ADR 0015](../../adr/proposed/0015-deterministic-validation-precedence.md): total predicate order and access budgets.

Out of scope and absent: target boot code, Nucleus implementation, Tier 0 layout, firmware callbacks, executable pointers, storage, networking, GUI, agents, packages, applications, VM/emulator launch, device access, trace-record framing, signatures, and entropy authenticity claims.

## Contract index

- `spec/hardware/rhd-v1.fields` and `rhd-v1.md`: normalized RHD source and reference.
- `spec/boot/handoff-v1.fields` and `handoff-v1.md`: fixed handoff, failure codes, pointer validation, and ownership.
- `sdk/generated/release-0/lib.rs`: byte-for-byte regenerated owned Rust representation; Rust layout is not wire ABI.
- `spec/fixtures/release-0/bin/*.bin`: valid, boundary, and malformed raw binary fixture bundles.

The authoritative boot memory map describes dynamic range ownership. RHD memory records describe the same normalized topology and must compare equal. Entropy is explicitly untrusted seed input. The trace channel is only a bounded versioned byte sink; its record format belongs to R0-008.

## Acceptance and evidence

| Requirement | Evidence |
| --- | --- |
| CPU, memory, interrupts, timers, serial, boot source, reserved ownership | RHD records plus authoritative memory-map kinds/owners |
| Magic, version, architecture, memory map, RHD location, entropy, trace | 128-byte boot handoff offset table |
| Bounds, alignment, ownership, failure codes | Boot validation order and codes 0–29 |
| x86-64/AArch64 structural semantic equivalence | pinned Linux reference oracle computes equality from decoded record kinds and normalized memory records |
| Required malformed classes | committed raw bytes for truncated, oversized, misaligned, overlap, unknown-critical, invalid-pointer, and architecture mismatch cases |
| No unverified pointer execution | host oracle uses injected test-only windows, checked `u64` arithmetic, and byte slices; it does not dereference fixture addresses |
| Generated Rust source consistency | complete deterministic rendering from `rust-*` schema rows compared byte-for-byte and metadata-compiled in pinned Linux CI |

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

The contracts expose no host virtual address, raw Rust pointer, function pointer, firmware callback, or unchecked MMIO operation. Candidate ranges use checked integer arithmetic, declared access windows, pairwise disjointness, bounded copies, and deterministic first-error codes. Structural validation does not authenticate the producer.

Generated Rust and the host reference oracle deny unsafe code. The committed `.bin` files are deterministic host-only conformance inputs, not target assets or executable payloads. This task adds no assembly, third-party crate, target-linked code, firmware, external dependency, persistent user data, migration, or signing behavior. Invalid input enters the later R0 platform's defined invalid-handoff recovery halt without granting authority.

## Versioning, limitations, and next action

Minor versions may add ignorable non-critical records only after the owner approves the outstanding identity, hardware-binding, trust-boundary, and precedence decisions. Changed meaning or required layout needs a new major version and parallel decoder after freeze. Platform evidence may require a versioned correction, never silent reinterpretation.

The binary corpus covers the task packet's named malformed classes with one intended structural fault per case. It deliberately does not encode unresolved MMIO authorization, record-identity, or compound-fault precedence choices. If proposed ADRs 0013–0015 are approved, their focused and dual-fault cases, schemas, generated types, and documentation must be added before R0-002 freezes. Prompt 7 and VM authorization remain out of scope.
