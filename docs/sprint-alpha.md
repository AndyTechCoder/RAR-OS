# RAR OS Sprint Alpha 0.1

Status: Owner-approved rebaseline — 2026-08-20
Time box: 14 days, ending 2026-09-03 in Europe/Sofia

## Objective

Build the closest authentic implementation of the RAR OS vision that can be
demonstrated reproducibly in the cloud. Every milestone adds observable
functionality. Experimental Alpha behavior does not become a stable long-term
contract merely because it crosses a later roadmap release.

## Mandatory vertical path

The Alpha demonstration must show:

1. custom Root → Recovery → Nucleus boot on an x86-64 cloud VM;
2. R0-002 hardware and handoff validation;
3. physical/virtual memory, exceptions, interrupts, timer, address spaces,
   threads, and scheduling;
4. rights-enforced capability handles and bounded IPC with timeout/cancellation;
5. isolated components with crash containment and restart;
6. separate system and preserved-data regions;
7. signed component/layer loading and tamper rejection;
8. component replacement without a whole-OS reboot and safe layer rollback;
9. recovery reconstruction that preserves verified intact data;
10. an interactive framebuffer GUI with keyboard, launcher, terminal, and at
    least two native demonstrations or applications;
11. reproducible cloud builds and accurate build/debug/extension documentation.

## Milestones

### A — Hosted x86-64 boot

A clean checkout reproducibly builds in the approved cloud lab and repeatedly
boots Root → Recovery → Nucleus with verified structured traces.

### B — Nucleus memory and execution

Memory allocation, protected address spaces, exceptions, timer, threads, and
scheduling operate under deterministic tests.

### C — Capabilities, IPC, and component isolation

Isolated components use bounded capability-controlled IPC; forged or excessive
authority is rejected; one component can crash and restart independently.

### D — System/data separation and recovery

System and preserved-data regions are distinct; deliberate corruption or
component failure invokes recovery while verified intact data remains unchanged.

### E — Interactive experience

The OS displays a framebuffer shell, accepts keyboard input, launches a terminal,
and runs at least two native demonstrations or applications.

### F — Signed layers, update, and rollback

The OS installs or replaces a signed component/layer, rejects tampering,
demonstrates replacement without a full reboot where supported, and rolls back.

### G — Maximum breadth and closure

The mandatory Alpha acceptance demonstration passes from a clean checkout.
Remaining time follows this order: simple persistence, A/B system state,
adaptive layouts, browser-accessible RAR Lab, virtual networking/minimal native
service, principals, Rust SDK/sample, agent/mock-provider contract, ARM64,
Tier 0, delta updates, more apps, C SDK, tiny local-model interface.

## Development rules

- One active implementation or diagnostic task, one writer, and one owned path
  set at a time.
- Worktrees live only under the SSD `worktrees/` directory and use the SSD
  `repository/` Git metadata.
- Each milestone uses its named `codex/sprint-*-...` branch, SSD worktree, and
  draft pull request.
- Batch coherent fixes before pushes. Development Probes are manual and
  non-gating; required milestone CI is strict.
- Correctness/security review occurs near milestone completion. Architecture
  review is added for public-contract or trust-boundary changes.
- Never execute target code, QEMU, firmware, an emulator, or a VM locally.
- Never compile/link the RAR target or create boot images locally.
- Keep `SPRINT_STATUS.md` current at every durable transition.

## Out of sprint

Physical devices, production cloud services or credentials, production Secure
Enclave work, complete Wi-Fi, self-hosting compiler infrastructure, a future RAR
language, a broad application ecosystem, formal certification, final visual
polish, and Linux compatibility do not block Alpha 0.1.
