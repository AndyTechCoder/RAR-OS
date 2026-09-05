# Desktop-v0 usable graphical alpha implementation

Status: implementation under review; no runtime or release claim yet.

The source follows ADR 0033 and docs/interfaces/desktop-runtime-v0.md.
Eight prestarted protected execution roles reuse the Platform paging, traps,
capabilities and bounded IPC mechanisms. Seven roles supply shell, storage,
keyboard, compositor, Files, Settings and Terminal; one idle role yields.
Slots 8–15 remain dead. Foundation/Platform releases and tests remain intact.

The Desktop storage adapter explicitly shares one temporary namespace between
Files and Terminal only. No persistent medium is mounted and no user data enters
the guest. Settings changes session appearance. F1/F2/F3 open or raise windows;
Escape hides without destroying the process. Keyboard input follows shell focus.
Only the compositor owns framebuffer access and only input owns PS/2 reads.

Surface transactions contain up to six 48-byte lines. The compositor stages rows
by sender identity and version, then commits atomically. Apps cannot name another
surface, change z-order, draw chrome, or supply framebuffer addresses. Window
geometry and the provisional RAR-authored 5x7 font are compositor policy.

Terminal supports help, list, read NAME, write NAME TEXT, and crash. Its editor is
64 printable ASCII bytes with backspace. File contents are at most 64 bytes;
visual text lines show the first 48 bytes. Four files and 128 total content bytes
are shared; names are at most 12 ASCII characters. Files selection is keyboard
Up/Down and F1 refresh. All content disappears at guest shutdown.

The crash command executes a user invalid instruction. The kernel revokes only
that process. The shell marks Terminal stopped only after a later endpoint send
returns kernel Stale; Files and Settings continue. No restart or recovery claim.

## Unsafe invariants

- The int80 veneer follows the existing complete kernel-owned trap-frame ABI.
- Bootstrap is copied from the fixed read-only mapping after magic/generation
  validation; no firmware pointers are used in ring3.
- Framebuffer writes belong only to compositor role 3, with kernel-validated
  mapping/pitch/format and clipping to 640x480 for every primitive.
- Kernel image copies and mapping setup use exclusively zeroed cloud guest arena
  allocations, preserving W^X, private pages, guarded stacks and alias retirement.
- Reused trap assembly preserves integer, SIMD and architectural context and
  validates kernel-owned return frame bounds before switching address spaces.
- Terminal UD2 is a deliberate isolated userspace fault, not host execution.
- Runtime memory intrinsics use bounded caller-provided spans as required by
  Rust-generated code; they do not grant new device or process authority.

## Verification required before completion

Cloud model tests cover inherited kernel/storage properties, the exact capability
matrix, editor/parser bounds, sender authorization, atomic surfaces and PS/2
decoding. The trusted cloud Desktop controller must reproduce both builds and
validate 12 complete guest-rendered scenes including a newly typed synthetic
value and post-fault readback. Retained Foundation and Platform regressions and
independent correctness/security review are required before merge.

Storage replies are operation-specifically validated, including status, reserved
bytes and padding. An app waits at most 256 nonblocking receive/yield rounds.
After timeout or malformed reply it disables its own storage channel for the
session and displays an error; its GUI remains responsive. It does not retry
uncorrelated requests or mistake a late reply for a later operation. Short IPC
envelopes are discarded by receivers rather than terminating shared services.

No Mac/SSD writes, local compilation or target execution are part of this work.
Production security, persistence, mouse/touch, dynamic loading, networking,
SDK stability, AI, updates and recovery remain later milestones.

Focused unsafe evidence includes the inherited Platform copy bounds/readonly/
overflow/guard tests, framebuffer metadata rejection and PE mapping tests; the
retained Foundation mapping suite; Desktop bootstrap size/alignment/identity
tests; and cloud memory-intrinsic canary, overlap, zero-length and comparison
tests. Invalid host pointers are never dereferenced as a test technique. These
checks complement, not replace, the kernel/unsafe source review and actual
isolated cloud process execution.
