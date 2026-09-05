# ADR 0033: Bounded Usable Alpha Composition

Status: Proposed for independent review under owner-directed Milestone 3
Decision proposed: A

## Context

Platform is released with actual protected processes, capability IPC, volatile
storage and PS/2/GOP services. The owner now requests Milestone 3. A working GUI
must build on those mechanisms without putting application policy in ring0 or
quietly presenting private fixtures as the finished native application model.

## Options

A. Extend the protected Platform with separately isolated built-in shell,
compositor and apps, using a documented private Desktop-v0 protocol. Keep the
fixed hardware profile and volatile workspace. Recommended.
B. Implement stable RID/RCI loading, persistent filesystem, USB pointer,
production font/text and SDK infrastructure first. Valuable later, but expands
this milestone beyond the requested usable graphical alpha.
C. Draw an all-in-kernel mock desktop or host website. Rejected: neither proves
native apps, isolation or the required service architecture.

## Proposed decision

Use A. Seven protected roles are shell0, storage1, input2, compositor3, Files4,
Settings5 and Terminal6. Reserve an idle role so blocking every UI participant
does not become a kernel panic. Reuse bounded Platform trap, page, PE fixture
and capability mechanisms. The private Desktop bootstrap retains checked
fixed images and boot-granted roles; it does not interpret RCI/RID/RBC or alter
the existing Platform-v0 wire contract. General app loading is deferred.

The compositor alone owns framebuffer MMIO, app content clipping and display
composition. Shell owns focus/window visibility; input sends decoded keys only
to shell. Each app can submit content only for itself; neither app payload IDs
nor visual position grant authority. Kernel-stamped sender/generation is checked.

Files and Terminal explicitly share one volatile demonstration namespace through
a storage service policy mapping of those two boot-approved principals. No other
role gets its endpoint. This is a narrowly declared shared object, not a global
filesystem, user account model, implicit cross-app access or persistent schema.
Settings changes only session-local appearance; shell/compositor accept that
message only from Settings.

Closing a window hides it; reopening preserves the still-running app's state.
A deliberate terminal fault stops only that process. Other apps remain usable.
Restart/replace/update/recovery promises remain later gates, not fake successes.

## Security and compatibility

Preserve W^X, private memory, guarded stacks, full supported context, validated
bounded IPC, capability revocation and fatal kernel fault attribution.
UI/driver policy remains outside the nucleus. No new target-linked dependency.
ASCII bitmap glyphs will be RAR-authored and explicitly provisional.
No real user data, credentials, host devices or guest network enter the test.

The experiment follows ADR 0022's least-authority/non-DMA intent using the
already reviewed Platform PS/2/GOP path; it does not reinterpret its historical
AlphaPlatformEntryV0 envelope, R0 bytes or acceptance-v2 protocol. It follows
ADR 0025's real post-crash GUI-continuity intent, without claiming that historic
A-G or restart/recovery gates have been completed. Keyboard-first scope is
explicit; pointer support is not silently claimed.

A separate trusted-main controller/contract review is needed only for the
multi-scene input/capture scenario and its bounded duration/output extension.
It retains networkless unprivileged ephemeral execution and fixed QMP operations.
No local authority or physical-device support is added.

## Validation and replacement

Document Desktop-v0 operations, sender checks, bounds, errors, backpressure,
surface/window rules and keyboard semantics before accepting implementation.
Test malformed UI requests, clipping/overflow, focus isolation, workspace grants,
quotas, input editing and dead peers. Runtime must show distinct interactive
scenes, unpredictable synthetic file readback across apps, and post-fault UI
progress, with independent capture validation and preserved earlier regressions.

Future RID/RCI-backed apps replace the private fixture launcher and messages via
an explicit adapter/contract change. Persistent storage needs separately reviewed
transactional formats and data/recovery separation. No migration of owner data
is needed because none is accepted and all experiment state is volatile.
