# ADR0040: independently built fixed compositor image

Status: Accepted — owner explicitly approved this separate-compositor decision
in this task on 2026-09-14. Scope remains M5 under ADR0032/ADR0039.
This document grants no new host access, runtime authority or milestone acceptance.

## Context and measured evidence

Cloud run34834004462 at source5f97512 measured the common service at135168 bytes,
beyond the unchanged131072 limit. Network was already separated. Run34836921130
at reviewed/CI-passed source8ff1049 measured the same mapped bound after a
rendering-only code-sharing attempt. Both stopped before any guest launch.

## Alternatives and accepted decision

A. Keep the oversized image: cannot satisfy the required memory geometry.
B. Merge compositor/network build identities: rejected/unapplied, because it
   couples independently replaceable components even with separate processes.
C. Raise128KiB: rejected; IMAGE0x100000 would overlap USER_STACK0x120000.
D. Give compositor its own fixed PE, separate from both common and network code.
E. Keep guessing at compiler annotations: rejected after the measured no-change.

Adopt D. It preserves separate component identities, role-specific authority,
and replaceability. The rejected grouping is absent from this implementation.

## Exact implementation boundary

- Common service, network and compositor independently link. Their private
  build files are modern-service.efi, modern-network.efi and modern-compositor.efi.
- Network-only entry still admits only logical7. Compositor-only admits only3.
  Common native-Alpha entry no longer dispatches either role.
- Kernel selects the immutable compositor PE only for the existing compositor
  role3/slot3; network remains its independent PE at logical7/physical10.
- Existing private construction independently validates each PE, maps private
  W^X image/stack/data, publishes its unchanged kernel-owned bootstrap, and
  retains retirement/incarnation rules. No shared writable pages or grants.
- Only the existing compositor role receives its existing framebuffer mapping.
  No syscall, principal, capability, storage or public app ABI changes.
- All existing128KiB service bounds and6MiB build-transfer bound remain intact.
  Both independent device PEs must occur exactly once in the measured kernel.
- The private controller transfer now has exactly four ordered fixed images,
  with independent existing PE inspection and byte-identical two-build checks.
  This is not a stable public package or disk format.
- Non-Alpha builds retain their previous source dispatch and build paths.
  No signing, rollback, Data/System, tier or dependency policy is changed.

## Review and activation

A source proposal commit does not boot, merge or publish the OS. Focused review
must verify unchanged role-specific authority, private construction and exact
four-file controller extraction before controller activation. Exact-source CI
and actual cloud image-bound/role/GUI/isolation/campaign evidence remain mandatory.
The existing main-only confined cloud controller is the sole execution path.
No owner files, Mac/SSD activity, artifact download, new network route or
privileged device access is introduced.

If review finds a changed trust boundary or persistent promise, stop for an
owner decision; this proposal does not approve it implicitly. Target PR204 stays
draft until all M5 evidence and final review pass.

## Consequences and replacement

One additional fixed artifact and independent compile are required. This costs
bounded cloud build time but avoids shared component build identity and removes
rendering code from the common PE. Replacing either device service remains a
separate code/build change under the same process and evidence contracts.
No data migration or permission expansion is required.
