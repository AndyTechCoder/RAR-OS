# R0-002 hardware and boot contracts

Status: Prompt 5 implementation complete; draft PR and independent Prompt 6 review required

## Delivered boundary

R0-002 freezes compiler-independent Release 0 v1 byte contracts for x86-64 and AArch64 hardware description and boot handoff. It includes stable validation codes, ownership/lifetime rules, generated unsafe-free `no_std` Rust semantic types, a malformed fixture corpus, and host-only conformance checks.

Out of scope and absent: target boot code, Nucleus implementation, Tier 0 layout, firmware callbacks, executable pointers, storage, networking, GUI, agents, packages, applications, VM/emulator launch, device access, trace-record framing, signatures, and entropy authenticity claims.

## Contract index

- `spec/hardware/rhd-v1.fields` and `rhd-v1.md`: normalized RHD source and reference.
- `spec/boot/handoff-v1.fields` and `handoff-v1.md`: fixed handoff, failure codes, pointer validation, and ownership.
- `sdk/generated/release-0/lib.rs`: generated owned Rust representation; Rust layout is not wire ABI.
- `spec/fixtures/release-0/cases.v1`: valid, boundary, and malformed decoded-wire fixtures.

The authoritative boot memory map describes dynamic range ownership. RHD memory records describe the same normalized topology and must compare equal. Entropy is explicitly untrusted seed input. The trace channel is only a bounded versioned byte sink; its record format belongs to R0-008.

## Acceptance and evidence

| Requirement | Evidence |
| --- | --- |
| CPU, memory, interrupts, timers, serial, boot source, reserved ownership | RHD records plus authoritative memory-map kinds/owners |
| Magic, version, architecture, memory map, RHD location, entropy, trace | 128-byte boot handoff offset table |
| Bounds, alignment, ownership, failure codes | Boot validation order and codes 0–29 |
| x86-64/AArch64 semantic equivalence | valid fixtures share semantic ID `baseline` |
| Required malformed cases | truncated, oversized, misaligned, overlap, unknown-critical, invalid-pointer, and architecture mismatch rows |
| No unverified pointer execution | fixtures contain decoded integers; oracle performs range checks only; contract requires validate/copy before decode |

Host-only checks:

```sh
/bin/sh spec/fixtures/release-0/run.sh
/bin/sh sdk/generated/release-0/check.sh
/bin/sh -n spec/fixtures/release-0/run.sh
/bin/sh -n sdk/generated/release-0/check.sh
tools/ci/check-specs.sh
```

No generated Rust or RAR target code is compiled or executed on the physical Mac. Compile validation of `lib.rs` is deferred to the pinned Linux host route and is not claimed by local evidence.

## Security, unsafe, dependencies, and recovery

The contracts expose no host virtual address, raw Rust pointer, function pointer, firmware callback, or unchecked MMIO operation. Candidate ranges use checked integer arithmetic, declared access windows, pairwise disjointness, bounded copies, and deterministic first-error codes. Structural validation does not authenticate the producer.

Generated Rust denies unsafe code. This task adds no assembly, third-party crate, target-linked code, firmware, binary asset, external dependency, persistent user data, migration, or signing behavior. Invalid input enters the later R0 platform's defined invalid-handoff recovery halt without granting authority.

## Versioning, limitations, and next action

Minor versions may add ignorable non-critical records; changed meaning or required layout needs a new major version and parallel decoder. Platform evidence may require a versioned correction, never silent reinterpretation.

The compact corpus is a decoded-field transport, not yet a raw byte corpus or authenticity test. It covers every malformed class mandated by the R0-002 packet; broader one-case-per-code and raw encode/decode corpora remain appropriate Prompt 6 findings or R0-009 conformance work if reviewers require them. The next action is Prompt 6 independent architecture, correctness, and security review. It must not authorize VM execution.
