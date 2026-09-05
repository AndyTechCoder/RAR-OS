# Fast-Track Alpha Milestone 2: Platform

Status: Owner-directed implementation; completion requires runtime evidence.
Owner direction: 2026-09-05 UTC, "Next, let's reach the second milestone.
Start working with the same core principles."

## Baseline and outcome

Start from Foundation release v0.1.0-foundation-alpha at
27ce297f4ed67117bb9176b98fd3817d095dd29a. Preserve that tag, history, source and
evidence. The outcome is a real protected execution/service platform that the
graphical Usable Alpha can use, not a GUI or production OS.

ADR 0032 governs delivery. Existing constitutional, dependency, privacy,
capability, recovery, rollback and user-data commitments are unchanged.

## Required working behavior

1. Native ring3 execution in private address spaces, with private writable
   memory, guarded user and kernel stacks, supervisor-only kernel mappings,
   W^X/NX, IOPL0 and no ambient port/MMIO access. At least two independently
   isolated services and a separate client must actually execute.
2. Bounded single-CPU timer-preemptive scheduling. A non-yielding task must not
   prevent another task from making progress. Preserve all general registers,
   architectural return state and the supported floating-point/SIMD context.
   A task can yield/exit; exit or fault revokes its authority.
3. Kernel-mediated bounded IPC using process-local object handles with rights
   and generations. Sender identity is kernel-assigned. Reject forged, stale,
   wrong-type and unauthorized handles, oversized messages, queue exhaustion,
   invalid user pointers, overflow and attempts to cross another process's
   memory. No public delegation or stable application ABI is introduced here.
4. A ring3 RAM-backed storage service supports bounded create/write/read/list
   for a separate client through IPC, with caller namespaces, quotas and errors.
   Data is VOLATILE and disappears on VM termination. This proves service and
   storage foundations, not disk-driver support, persistence, encryption,
   snapshots or user-file safety across reboot. No owner data enters a test.
5. A ring3 input service decodes actual emulated PS/2 keyboard input delivered
   by the trusted cloud controller. The nucleus mediates only fixed bounded
   device reads/notifications; it must not grant arbitrary command-port writes
   (the controller can reset a machine). No host keyboard capture.
6. A ring3 framebuffer service draws through a validated, narrowly granted
   UEFI GOP framebuffer mapping. Drawing and pixel-format policy stay outside
   the nucleus. The cloud controller verifies actual captured pixels, not just
   a serial claim that drawing happened.
7. A deliberately faulting task is contained and loses its capabilities while
   unrelated services continue to communicate and retain their own RAM state.
   Kernel faults remain fatal and must never be reclassified as user faults.
8. Kernel mechanisms, RAM storage policy, input decoding and drawing have
   explicit separate source ownership/boundaries and focused conformance tests.
   Fixed fixture programs are experimental native test services, not a new RCI,
   RBC, RID, persistent format or replacement for the approved native app model.
9. Two clean cloud builds yield byte-identical target programs and boot media.
   Retain all Foundation normal/panic/exception regression evidence, plus the
   integrated Platform runtime proof and negative-test outputs.

## Prototype limits and replacement

Keep this milestone bounded to one CPU and the existing x86_64 UEFI cloud
machine. Fixed boot-granted service identities and capability tables are
acceptable; general component loading and dynamic device discovery are not
claimed. Each privileged operation must have a documented narrow purpose and
failure contract. Driver policy may not be silently placed in the nucleus.

Experimental platform calling conventions and fixture-image layouts must be
documented with version/bounds/error behavior before their implementation is
accepted. They are internal test-service contracts, cannot be consumed as stable
app APIs, and must not reinterpret existing public Alpha formats or state slots.
Future native loading/RID integration requires an explicit compatibility map.

Durable storage is deliberately deferred: later work must provide separately
reviewed formats, transactional updates, recovery and Data Vault separation.
RAM storage must never be marketed as preservation of real user files.

## Certified cloud profile extension

All Mac/SSD restrictions remain absolute: no local creation, edit, move,
deletion, build, packaging, mount or target execution. All repository mutations
use GitHub. No new local tool/runtime/permission installation.

Reuse the pinned compiler, firmware, emulator and unprivileged, capability-free,
read-only-root, networkless, bounded disposable containers from Foundation.
Only reviewed trusted-main controller code can add:
- one fixed emulated VGA/GOP display, with bounded validated geometry;
- synthetic PS/2 events injected through a private cloud-container QMP Unix socket;
- bounded capture of the emulated framebuffer into disposable cloud scratch.

QMP is not exposed to the guest, network or owner device. Controller commands,
socket path and capture paths are fixed and allowlisted. No TCP listeners, VNC,
SPICE, clipboard, host display/input, camera/microphone, passthrough, raw disk,
shared filesystem, credentials, Docker socket or persistent writable mount.
The original boot image remains a read-only input with a disposable overlay.

Review the extension and negative tests before it becomes trusted on main.
This is project-reviewed development certification, not production assurance.
No AHCI/virtio DMA or additional block hardware is needed for volatile storage.

## Tests and observable acceptance

The runtime transcript must distinguish initialized mechanisms from completed
behavioral proofs. Define its exact bounded grammar in the trusted controller,
including process execution, timer preemption, capability rejection, IPC,
storage readback, fault containment, input decoding and framebuffer capture.
A final RAR-PLATFORM-READY marker is valid only after all required proofs.

Required tests include:
- real user/kernel and cross-process memory violations;
- user privileged-instruction and NX/guard-page faults;
- continued progress of a survivor after a peer fault and a non-yielding peer;
- register/SIMD preservation over preemption and syscalls;
- forged/stale/wrong-right capability and peer-death handling;
- cross-page/overflow/null/kernel user-buffer rejection and bounded IPC queues;
- storage namespace separation, readback, quotas, missing objects and invalid ranges;
- malformed/oversized framebuffer metadata, pitch/format/range validation;
- synthetic key press/release decoding with timeouts and invalid input handling;
- framebuffer evidence independent of guest assertions;
- rejection of networking, passthrough, host mounts, arbitrary QMP commands,
  external socket/capture paths, unpinned inputs and unbounded resources.

No missing input event, failed screenshot, lost task, unexpected panic,
timeout or static-only result may count as completion.

## Delivery and review

Use one coherent implementation branch/PR. A separate consolidated cloud-
controller/contract PR is allowed because proposed source must not authorize its
own outer launcher. Do not create authorization packets per service or fix.
Use focused continuous tests; one integrated independent architecture,
correctness and security review near completion; consolidate remediation.

Merge only with passing relevant checks and no blocking findings. Verify the
merged source, preserve branches/history, publish hash-bound durable evidence,
and document tested limitations. Completion requires actual Platform runtime
proof plus retained Foundation regressions, not simply specifications or compilation.

## Owned paths and non-goals

Expected ownership: nucleus/platform/ and narrow integration in nucleus/foundation/;
core/platform/ for fixture/runtime boundaries; services/platform/ for volatile
storage, input and framebuffer services; tools/rar-lab/platform/ and its workflow;
focused tests and documentation. No unrelated cleanup or file removal.

Excluded: compositor, GUI shell/apps, networking, production services, AI,
physical hardware, SMP, general USB/GPU stacks, persistent filesystems, signed-
layer activation and recovery demonstrations. Those remain subsequent milestones.
