# Platform-v0 internal native fixture contract

Status: implementation proposal for independent review. This is a private,
replaceable Milestone-2 fixture convention, not RCI/RID/RBC, a persistent format,
a general application loader, or a stable SDK. Existing public contracts remain
unchanged. All data in the fixture guest is synthetic and volatile.

## Source ownership and initialization

`nucleus/platform/` owns mechanisms, traps and narrowly bounded platform setup.
`core/platform/` owns the private syscall/boot ABI and adversarial client fixtures.
`services/platform/` owns RAM storage, keyboard decoding and framebuffer drawing.
One RAR-owned native service fixture is linked at 0x400000 and embedded verbatim
inside the kernel. The separate exported service artifact must match those bytes.

The PE32+ adapter accepts x86-64, fixed base 0x400000, 4 KiB section alignment,
at most 16 sections, at most 128 KiB loaded image, bounded headers/raw ranges,
nonoverlapping mapped sections, RX entry, no W+X and no imports/relocations/TLS/
delay imports. It is not a general PE/Windows compatibility claim.
Headers are RO+NX, sections use checked permissions; writable sections are private
per process. No allocator, external boot library or third-party target crate.

Each of 14 fixed processes receives its own read-only Boot structure at 0x700000.
All fields are little-endian u64 in the order declared by
`core/platform/abi.rs::Boot`: magic, role, generation, entry, kernel probe,
peer-private probe, framebuffer, width, height, pitch, format, ten handles.
Probe addresses exist solely for negative fixtures, not a production discovery API.
Only the display role receives an RW+NX framebuffer mapping at 0x800000.

## CPU and memory boundary

One CPU, x86-64 UEFI qemu64/TCG, SSE2/x87. User CS=0x1b, SS=0x23; IOPL=0.
CR0.WP/NX are active. FSGSBASE and OSXSAVE are disabled; FS/GS bases are zero,
there is no TLS interface. Data selectors are preserved within the admitted
null/user-code/user-data selector set. Arithmetic flags and DF are preserved;
IF is enabled on return; tracing, IOPL, NT, VM, AC and unsupported modes are
cleared. This is not a promise to preserve unsupported CPU extension state.

The 32 MiB kernel-owned boot arena reserves 4 MiB for Foundation and 14 private
2 MiB task regions. Each region has a 1 MiB page-table budget, 128 KiB private
image area, 64 KiB user stack, guarded 64 KiB kernel stack and 4 KiB bootstrap page.
All roots independently map supervisor kernel sections and kernel-owned arena
pages, excluding private-user physical aliases and guards. User code is exposed
only after image construction and retirement of writable bootstrap aliases.

Interrupt gates normalize a 208-byte Trap and aligned 512-byte FXSAVE64 area.
Assembly saves every GPR and data selector before Rust, saves supported FPU/SIMD,
clears DF and restores a kernel floating-point environment. Scheduling runs with
IF=0; CR3/TSS.RSP0 change while kernel code/data/stacks remain identically mapped.
Return validates executable RIP, user stack and selectors before IRETQ.
User faults revoke only that task; kernel faults/double faults/NMI are fatal.

## Private syscalls and capabilities

INT 0x80, RAX operation/result, RDI/RSI/RDX/R10 arguments; other GPRs preserved.
Operations: 0 yield; 1 send(handle,pointer,length); 2 receive(handle,pointer,144,
blocking 0/1); 3 read-port(handle,port); 4 fixture-report(code); 5 exit.
All unknown operations fail. Reports are role-bound test instrumentation, not
arbitrary logging or privileged service APIs.

Results are zero/success or received length; negative i64 errors are -1 invalid,
-2 denied, -3 stale, -4 full, -5 empty, -6 exhausted. A blocking empty receive
returns -5 after the process is awakened so the fixture retries explicitly.
Send accepts 1–128 bytes. Per-process receive queues hold four messages.
The receive envelope is exactly 144 bytes: sender u64, generation u32, length
u32, zero-padded payload128. Identity comes only from kernel process state.
Receive validates writable destination before dequeue; failed copies do not
consume messages. Single CPU with IF=0 prevents remapping races during copies.

Handles encode generation in high32 and slot+1 in low32, interpreted only in the
caller's ten-slot table. Rights are send1, receive2, port-read4, draw8, constrained
by object type. Revocation increments slot generation; exhaustion retires the slot.
Endpoint target generation is checked on use; no process restarts/delegation API.
On exit/fault the queue and capabilities are revoked; other callers see stale.

Read-port accepts only the input capability and ports0x60/0x64. Trusted initial
PS/2 setup uses fixed configuration/enable commands, never reset or arbitrary
command-port writes. Input policy/Set1 make-break decoding stays in ring3.

## Volatile storage wire

Request128: operation u8 (create1/write2/read3/list4), name length u8, data length
u8, reserved zero, name12, data up to64 and zero padding. Names are 1–12 ASCII
alphanumeric/dot/hyphen bytes, excluding dot/dot-dot; list has no name or data.
Only write carries data. Unknown/reserved/noncanonical fields are rejected.
The service takes caller task+generation from the kernel envelope, never payload.

There are 16 total slots, at most four files and 128 bytes per caller namespace,
64 bytes per file. Writes replace contents; failures leave old contents intact.
Reply128: status u8 (ok0/invalid1/missing2/exists3/quota4), list count u8, data
length u8, reserved zero. Read data starts16; list starts4 with repeated
length-prefixed names. There is no deletion, durable disk, owner data, hierarchy,
snapshot or persistence promise.

## Evidence and unsafe review requirements

Ten kernel-model/loader/display tests and five service-model tests accompany the
implementation. Runtime must prove actual user/kernel and peer isolation,
preemption/context, capability failures and peer death, bounded IPC, namespaces/
quotas/readback, post-fault communication, one synthetic A make/break and captured
640x480 pixels. The cloud controller validates its fixed ordered records and every
pixel independently; missing/failed proof never counts as completion.

Unsafe sites are confined to firmware adapters, exclusively owned image/table
initialization, assembly gates, validated user copies, fixed port operations and
the ring3 MMIO/fault fixtures. Required independent review must check each ownership,
alignment, range, alias, stack and privilege invariant. Compilation or model tests
alone do not establish those invariants on hardware.

Reference basis: UEFI 2.10 GOP/boot-services definitions and Intel SDM long-mode
interrupt/IRET, protection and FXSAVE/FXRSTOR semantics. No implementation code is
copied from a third-party kernel or runtime.
