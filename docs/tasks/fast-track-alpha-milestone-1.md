# Fast-Track Alpha Milestone 1: Foundation

Status: Owner-approved for implementation — 2026-09-04

## Outcome

Produce an authentic, reproducible, bootable RAR OS foundation for one
experimental x86_64 UEFI cloud-VM profile. The milestone proves a custom boot
path, kernel entry, memory foundations, deterministic diagnostics and bounded
hardware-event foundations. It does not claim a usable graphical OS.

## Certified disposable cloud profile

Target build and boot evidence may run only through a trusted default-branch
GitHub workflow on `ubuntu-24.04`, using identity-pinned host tools and the RAR
Lab wrapper. The boot guest is disposable and must have:

- no guest or bridged networking;
- no repository, host filesystem, raw disk or physical-device passthrough;
- no credentials, tokens, clipboard, camera, microphone or shared folders;
- no elevation, extra capabilities, Docker socket or persistent runner state;
- bounded CPU, memory, processes, disk, log size and wall time;
- identity-pinned UEFI firmware and emulator recorded in the evidence;
- a fresh generated VM disk containing only the reviewed artifact;
- serial-only machine evidence captured outside the guest.

Nothing in this profile authorizes target build, image creation or execution on
the owner's Mac or SSD.

## Required implementation

1. A RAR-owned x86_64 UEFI boot path validates its handoff and transfers control
   to a RAR-owned kernel entry.
2. The kernel constructs and owns page-table state, establishes documented
   physical and virtual memory regions, rejects invalid mappings, and exposes no
   implicit ambient mapping authority.
3. A bounded kernel allocator supports aligned allocate/deallocate behavior,
   detects exhaustion and invalid requests, and has focused tests.
4. Serial logging has a documented stable record grammar and bounded output.
5. Panic handling emits a deterministic terminal record, disables further
   unsafe progress and halts without reboot loops.
6. Exception foundations install a validated descriptor table and deterministic
   fatal exception path.
7. Interrupt foundations establish explicit enable/disable state and contain
   unexpected vectors.
8. A timer source produces monotonic bounded ticks sufficient for later
   scheduling work.
9. The build emits a bootable UEFI artifact and identity manifest from a clean
   checkout without an unapproved target-linked dependency.
10. Two independent clean cloud builds produce byte-identical target artifacts
    and matching recorded digests.

## Required ready transcript

A successful certified boot must emit these ordered records exactly once:

```text
RAR-BOOT:UEFI
RAR-KERNEL:ENTRY
RAR-MEMORY:READY
RAR-ALLOCATOR:READY
RAR-INTERRUPTS:READY
RAR-TIMER:READY
RAR-FOUNDATION-READY
```

A deliberate panic profile must instead terminate with one ordered bounded
sequence containing `RAR-PANIC:BEGIN`, a stable panic code, and
`RAR-PANIC:HALT`. It must not reach `RAR-FOUNDATION-READY`.

## Negative safety tests

Automated policy tests must reject guest networking, writable host/repository
mounts, raw or physical disks, passthrough, credentials, elevation, unpinned
firmware/emulator identities, unbounded resources, direct emulator launch
outside RAR Lab, malformed UEFI handoff, invalid memory maps, invalid page
mappings, allocator misuse, unexpected exceptions and ready-marker spoofing.

## Evidence and documentation

Retain an immutable evidence bundle that binds:

- source commit and workflow/run/job identities;
- compiler, linker, firmware, emulator and container identities;
- dependency inventory and approved exceptions, if any;
- both clean-build manifests and artifact digests;
- boot configuration, bounded serial transcript and exit classification;
- negative-test results;
- focused unsafe Rust and assembly invariants;
- build, boot, debugging, failure and extension documentation.

Cloud workflow success alone is insufficient. Evidence must be checked against
the exact reviewed source and artifact identity.

## Delivery shape

Use at most three implementation pull requests unless a concrete security or
correctness boundary requires separation:

1. boot path, reproducible artifact, logging and deterministic panic;
2. page tables, physical/virtual memory and allocator;
3. exceptions, interrupts, timer and integrated certified-cloud evidence.

Use quick checks throughout. Before milestone closure, perform one independent
architecture/correctness/security review of the integrated exact head and one
consolidated remediation pass where practical.

## Completion gate

Milestone 1 is complete only when all required behavior and negative tests pass,
the two-build reproducibility proof matches, the certified cloud VM reaches
`RAR-FOUNDATION-READY`, the panic profile proves deterministic containment,
the evidence bundle is retained and reviewed, documentation matches behavior,
and no blocking review finding remains. Static specifications or an unbooted
image do not satisfy this gate.

## Non-goals

GUI, applications, networking, persistent user storage, signed-layer activation,
rollback demonstrations, AI/agents, ARM64, Tier 0 and physical hardware support
belong to later milestones.
