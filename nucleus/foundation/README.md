# RAR Foundation: experimental x86_64 UEFI nucleus

Status: implementation under cloud validation; no working-OS or completion claim.

This implements Fast-Track Alpha Milestone 1 using RAR-owned Rust and limited
assembly. It is an early nucleus, not a hosted program or Linux distribution.
The UEFI executable contains both the boot adapter and nucleus. After obtaining
the memory map it exits firmware boot services, switches to RAR-owned page tables
and stack, and calls the kernel. No firmware runtime service is used afterward.

## Boundaries and replacement

- boot.rs owns UEFI table validation, Loaded Image discovery, firmware allocation,
  memory-map conversion and ExitBootServices retry. Its private BootInfo carries
  owned arena and normalized regions. This is experimental internal handoff data,
  not a replacement for any stable RAR ABI or on-disk format.
- paging.rs owns four-level, 4 KiB supervisor mappings. It maps only explicit image
  sections and the reserved bootstrap arena. Code is read-only/executable, data
  is non-executable, writable/executable requests fail, and null is not mapped.
- model.rs contains hardware-independent region checks, physical-frame allocation,
  mapping validation and heap accounting. It is tested directly on a cloud host
  inside the same constrained builder used for target compilation.
- interrupts.rs owns the one-CPU GDT/TSS/IDT, fatal exception entries and the q35
  legacy PIC/PIT timer adapter. These platform details do not define future
  portable scheduling or driver interfaces.
- image.rs is a host-only deterministic FAT16 boot-media generator. FAT is used
  solely for standard UEFI removable-media boot; it is not RAR's future filesystem.

No persistent user data exists in this milestone. Every VM disk is generated
from source and discarded with the cloud test. Recovery, signed layers and
production data storage remain later milestones.

## Memory and lifetime

Firmware allocates a 4 MiB EfiLoaderData arena below 4 GiB. The first MiB holds up
to 256 page-table pages. A 128 KiB kernel stack and 64 KiB emergency stack have
absent boundary pages; a separate aligned 64 KiB region backs the bootstrap heap.
Image sections retain their declared PE permissions. No whole-physical-memory
identity map or user mapping exists.

Physical frames come only from validated conventional-memory descriptors below
4 GiB, excluding the null page. The bootstrap manager supports last-allocation
release; other releases fail. This deliberate limitation will be replaced by a
page allocator before multiple processes. A runtime self-test maps, writes,
reads, unmaps and releases a fresh frame using a high virtual address.

The heap uses 16-byte units and up to 128 live allocations, with alignments up to
4096. Handles record offset, size and a monotonically increasing identity.
Invalid requests, exhaustion, stale handles, forged extents and double frees
fail. An allocation is an offset into the explicitly owned heap region; it grants
no general pointer authority. Allocator state requires exclusive mutable access.

## Diagnostics and execution profiles

Serial is COM1, 115200 8N1. Each ASCII record is one line, at most 96 graphic
characters; total RAR output is limited to 4096 bytes. UART polling is bounded.
The ordered normal transcript is the exact contract in the milestone task.
The panic profile emits BEGIN / CODE=SELFTEST / HALT after allocator initialization.
The exception profile executes UD2 after installing the IDT and must report
VECTOR=6 / BEGIN / CODE=EXCEPTION-06 / HALT. Other unexpected vectors terminate
with a stable panic code. Panic disables maskable interrupts and halts forever;
it cannot reboot or continue initialization.

The timer has one writer on the sole virtual CPU, saturates at u64::MAX, and is
read atomically. Three actual IRQ0 ticks are required before TIMER:READY. The
harness bounds runtime if a timer is missing; a hardcoded ready string alone is
not acceptance evidence. The final ready state intentionally halts.

## Unsafe and assembly invariants

1. Firmware entry supplies readable table/Loaded Image pointers. Bounds, revision,
   signature and CRC are checked before using service pointers. This cannot
   authenticate malicious firmware; pinned OVMF is part of the development TCB.
2. Firmware owns memory until ExitBootServices succeeds. RAR writes only its own
   image statics and explicitly allocated arena beforehand. No allocation occurs
   between the final map and exit, and stale map keys receive bounded retries.
3. Page-table raw accesses address exclusively owned aligned arena pages. Existing
   intermediate entries must point inside that arena. Failed bootstrap mapping
   is fatal; a partially constructed address space is never reused.
4. CR3 activation occurs in a naked thunk that sets the new aligned stack before
   calling Rust. Code, data, handoff and stack are all mapped in the new tables.
   EFER.NXE and CR0.WP enforce the requested permissions. Clearing CR4.PGE
   flushes inherited global translations before CR3 activation; the kernel
   verifies the active root before its memory self-test.
5. Descriptor tables are initialized with IF clear. Exception paths use a guarded
   IST stack and never return; the timer stub preserves RAX, touches no SIMD state
   and returns with IRETQ. This profile has one CPU and no userspace context.
6. UART/PIC/PIT I/O uses only fixed ports for the certified emulated hardware.
   There is no physical-device passthrough or external network.
7. Memory intrinsics have ordinary C valid-range preconditions. Their bytewise
   volatile loops prevent accidental recursive intrinsic lowering; memmove
   chooses direction to support overlap.

These invariants require independent review plus cloud tests before acceptance.
Nested NMI/fatal exceptions and arbitrary hostile firmware are not claimed
recoverable. Interrupts remain disabled during most bootstrap work. This is not
a production kernel, general scheduler, security certification or device port.

## Build, verification and inventory

Only the trusted Foundation GitHub workflow may compile or boot this code.
It invokes pinned rustc directly: no Cargo dependencies, build scripts or registry
fetches. The compiler-provided core and compiler_builtins support accompany the
pinned Rust 1.95 bootstrap toolchain under ADR 0003; they are declared external
compiler support, not RAR-authored libraries. There is no std or alloc target
runtime, third-party allocator, kernel, filesystem or boot library. RAR owns the
memory intrinsics. LLVM/linker and QEMU are host tools; OVMF is platform firmware.
The cloud bundle records exact executable/image digests and both build outputs.

The model tests cover malformed maps and headers, forbidden mappings, physical
allocation, heap misuse and thousands of mixed allocations. VM tests prove
normal startup and deterministic panic/invalid-instruction containment. The
FAT generator checks the executable type and emits fixed geometry/timestamps.
Two clean builds must produce identical EFI and FAT bytes. Retain failure logs;
never reinterpret a failed boot as success based only on CI process completion.

Normative implementation references: UEFI 2.10 sections 2, 4, 7 and 9; Intel 64
SDM Volume 3A (paging, protected-mode descriptors and interrupt handling);
Microsoft FAT specification and PE/COFF format. RAR code is written for this
project from these public interfaces, not copied from another kernel.
