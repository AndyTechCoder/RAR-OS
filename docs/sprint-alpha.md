# RAR OS Sprint Alpha 0.1

## Active process supersession

The product requirements in this document remain direction. ADR 0032 replaces
its expired time box and historic A–G authorization mechanics with five
Fast-Track Alpha milestones. Milestone 1 proves only the authentic bootable
Foundation; GUI and the remaining requirements follow in later milestones.

Status: Owner-approved end-of-week rebaseline — 2026-08-25
Time box: ends 2026-08-30 at 23:59 in America/Los_Angeles

## Objective

Produce one authentic, bootable, graphical RAR OS vertical slice from a clean
GitHub checkout in the cloud. The Alpha must visibly work while representing
the architecture's isolation, recovery, data separation, signed-layer update,
and rollback principles in minimal tested form. These prototypes do not claim
production completeness or silently close their later roadmap releases.

## End-of-week completion contract

Alpha 0.1 is complete only when one retained cloud demonstration proves that a
clean checkout:

1. reproducibly builds RAR-owned x86-64 target code with no unapproved linked
   target dependency;
2. boots RAR Root → Recovery → Nucleus in the approved Development Lab;
3. displays an interactive framebuffer GUI and accepts keyboard and pointer
   input;
4. opens a launcher, terminal, settings, and at least two native demo apps;
5. runs isolated components through capability-controlled IPC and restarts one
   deliberately crashed component without losing the rest of the experience;
6. keeps system and preserved-data regions separate and demonstrates recovery
   while a verified test file remains unchanged;
7. installs or replaces a signed layer, rejects a tampered layer, and rolls back
   a failed activation without rebuilding the whole OS; and
8. publishes the exact source, tool, firmware, artifact and evidence identities,
   plus build, boot, debugging, recovery, update and extension documentation.

Passing only a mock, host application, Linux process, screenshot, prerecorded
animation, or unbooted image does not satisfy this contract.

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

The OS displays a framebuffer shell, accepts keyboard and pointer input, opens
the launcher, terminal, and settings, and runs at least two native
demonstrations or applications.

### F — Signed layers, update, and rollback

The OS installs or replaces a signed component/layer, rejects tampering,
demonstrates replacement without a full reboot on the selected x86-64 Alpha
profile while an unaffected component continues, and rolls back.

### G — End-of-week closure and maximum breadth

The end-of-week completion contract passes from a clean checkout.
Remaining time follows this order: simple persistence, A/B system state,
adaptive layouts, browser-accessible RAR Lab, virtual networking/minimal native
service, principals, Rust SDK/sample, agent/mock-provider contract, ARM64,
Tier 0, delta updates, more apps, C SDK, tiny local-model interface.

## Development rules

- One active implementation or diagnostic task, one writer, and one owned path
  set at a time.
- Worktrees live only under the SSD `worktrees/` directory and use the SSD
  `repository/` Git metadata.
- After this rebaseline merges, Milestones A–G use one
  `codex/sprint-alpha-vertical` branch, one SSD worktree, and one draft pull
  request. Each milestone is a pushed, tested checkpoint on that branch; do not
  create seven scattered implementation tasks or PRs.
- Batch coherent fixes before pushes. A–G Development Probes are automatically
  requested repository-dispatch acceptance gates whose trusted controller comes
  only from `main`; branch protection is separate, and neither can substitute
  for the other.
- Correctness/security review occurs near milestone completion. Architecture
  review is added for public-contract or trust-boundary changes.
- Never execute target code, QEMU, firmware, an emulator, or a VM locally.
- Never compile/link the RAR target or create boot images locally.
- Keep `SPRINT_STATUS.md` current at every durable transition.
- Every durable transition is pushed before the next implementation task begins.
- Published milestone commits and annotated checkpoint tags are append-only: no
  force-push, tag movement/deletion, rebase, or history rewrite.
- Diagnose once and batch a coherent repair. Retry an ordinary failure at most
  twice; on a third identical failure, record one blocker and stop without a
  polling loop or persistent goal.
- Never start implementation while GitHub Actions cannot start a job or while
  the local Codex state store lacks safe disk headroom.

`SPRINT_STATUS.md` records the exact completed implementation or published
checkpoint being described. Because a Git commit cannot contain its own SHA,
the authoritative current branch head is resolved through the recorded PR and
`git rev-parse HEAD`; status-only commits label their parent checkpoint rather
than claiming to embed their own identity.

The Release Driver requests each Development Probe through GitHub's repository
dispatch API with event type `development-probe`, the corresponding probe
`milestone-a` through `milestone-g`, and an exact 40-character source commit
SHA. It never asks the owner to run routine probes manually. The workflow
controller always comes from `main`; only the separate source checkout comes
from the requested SHA.

## Out of sprint

Physical devices, production cloud services or credentials, production Secure
Enclave work, complete Wi-Fi, self-hosting compiler infrastructure, a future RAR
language, a broad application ecosystem, formal certification, final visual
polish, and Linux compatibility do not block Alpha 0.1.
