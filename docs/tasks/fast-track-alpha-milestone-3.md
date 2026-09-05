# Fast-Track Alpha Milestone 3: Usable Graphical Alpha

Status: active owner-directed milestone; implementation and release evidence required.
Owner direction: 2026-09-05 UTC, "Now let's continue, with the next milestone 3".

## Baseline and authority

Preserve v0.1.0-foundation-alpha and v0.2.0-platform-alpha. Start at Platform
61916ca3a17c2f7013205c91afb3117dbb03cbc0. ADR 0032 controls delivery process;
the constitution, from-scratch policy and host-safety constraints remain binding.
ADR 0033 specifies the bounded experimental graphical composition, not a stable
application ABI or a change to historic Alpha/R0 formats.

All repository mutations use GitHub APIs. No Mac/SSD file creation, changes,
deletion, builds, packaging, mounts or target execution. No local permission
installation, broader filesystem inspection or workspace migration is required.
Cloud-only execution uses the reviewed disposable profile. Guest networking,
credentials, owner data, passthrough, raw host disks and host sharing stay forbidden.

## Required actual behavior

1. Boot the RAR kernel into a readable 640x480 keyboard-operated graphical
   desktop, rendered by a ring3 compositor, not firmware UI or host HTML.
2. Separate protected shell, compositor, input, storage, Files, Settings and
   Terminal execution containers. Only compositor maps framebuffer; only input
   has the fixed PS/2 read capability. Apps receive neither raw device capability.
3. Compositor owns window decoration, clipping, z-order and focus presentation.
   Apps submit bounded surface content only for their kernel-stamped identity.
   Shell opens, switches, hides and reopens the three built-in app windows with
   state preserved while their prestarted processes remain alive. This is not
   dynamic loading, arbitrary installation or restart-after-crash support.
4. Keyboard launcher shortcuts and focus routing: F1 Files, F2 Settings,
   F3 Terminal, Escape hides the focused window; typed text goes only to the
   focused app. No host keyboard capture. Pointer/USB support is a later increment.
5. Files lists and reads synthetic files through the storage service. Terminal
   supports help, list, read and write commands with bounded input editing,
   backspace, clear errors and readback. Its writes must become visible in Files.
6. Files and Terminal share only one explicitly granted volatile demo workspace.
   Storage maps those two approved sender identities to that namespace; other
   apps cannot request storage. Preserve quotas, validation and failure atomicity.
   Data disappears when the guest ends. No persistent user-file claim.
7. Settings changes a real desktop appearance setting (light/dark), reflected
   across windows. Setting and app state survive hide/reopen within the session.
8. A deliberate Terminal process fault is contained. The shell explains its
   stopped state; Files and Settings still respond afterward and workspace data
   remains readable. No claim of process restart or recovery is made.
9. Malformed, oversized, wrong-sender and unauthorized UI requests fail without
   granting another surface, focus, raw input, framebuffer or storage authority.
   Bounded queues, wait/retry policies and rendering bounds prevent unbounded work.
10. Two clean builds reproduce target binaries/boot media. Actual cloud proofs
    inject keys, verify multiple guest-rendered scenes and text readback, and
    retain post-fault interaction evidence. Run retained Platform/Foundation
    regressions. Missing input, stale frames, timeout, unexpected kernel panic,
    fabricated readiness or model-only results cannot count as completion.

## Cloud evidence extension

Use the existing pinned Rust/UEFI/QEMU/OVMF/Python inputs and single-CPU q35 TCG
256-MiB guest, with the same isolated no-network/no-capabilities container policy.
A separately reviewed trusted-main Desktop controller may send only bounded
allowlisted keyboard sequences and capture fixed-path 640x480 PPM images over
a private container QMP Unix socket. No new hardware or listeners are needed.
The controller may generate one eight-letter a-p test value internally and type
it into Terminal; no source-selected host command, filename or arbitrary QMP
operation is accepted. Retain that synthetic value in evidence.

Bound the complete guest scenario to 90 seconds, launcher to 95 seconds and
outer process to 100 seconds. Bound to 16 retained captures, 256 injected keys,
64-KiB serial, 24-MiB encoded result and 256-MiB ephemeral container scratch.
Framebuffer expectations are checked by trusted host code, not guest hashes.
Only the merged reviewed controller can launch the target proposal.

## Delivery

One consolidated controller/contract PR is permitted for outer authority;
one coherent target implementation PR contains UI code, focused tests and docs.
Use one integrated independent correctness/architecture/security review near
completion, consolidated fixes and relevant passing gates before merge.
Publish v0.3.0-usable-alpha only after final-head and exact-main runtime proofs,
regressions, documentation and evidence all pass. Preserve branches/history.

## Owned paths and limits

nucleus/desktop/ and narrowly selected Foundation entry integration;
core/desktop/; services/desktop/; apps/desktop/; tools/rar-lab/desktop/ and workflow;
focused tests, contracts, handoff and evidence. Reuse released Platform
mechanisms where possible without silently changing their fixture contract.

Non-goals: final design system, mouse/touch, multiple displays, persistent disk,
networking/browser, installed third-party apps, stable SDK/RCI loader, full user
accounts, AI/agents, signed live updates, recovery or production security.
The later Modern Architecture and Expansion milestones remain unclaimed.
