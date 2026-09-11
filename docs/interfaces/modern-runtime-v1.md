# Modern runtime-v1 private boundary

Status: proposed experimental source contract; not an activated kernel ABI,
stable SDK or VM profile. Desktop-v0 and its 176-byte bootstrap/144-byte receive
envelope are unchanged. This is the concrete candidate under ADR0034.

## Ownership

The kernel alone constructs a read-only bootstrap page at user VA0x700000,
derives the caller from its saved CPU context, owns capability tables and stamps
message identity. Neither bootstrap shape validation nor the Rust device decoder
authenticates a caller or grants authority. The runtime must enforce the source
mechanisms in modern-lifecycle-v0.md. No user-provided principal, generation,
port, disk identifier or pointer to another process selects authority.

## Bootstrap

core/modern/abi.rs defines an explicit 368-byte repr(C), 8-byte aligned layout,
with no implicit padding. All fields are initialized, including four reserved
zero bytes. Its byte magic is RARMOD01; version is1; the size field must be368.
The full layout in declaration order is:

- Fifteen u64 fields: magic, version, bytes, role, phase, generation, entry,
  kernel_probe, peer_probe, framebuffer, width, height, pitch, format, health_token.
- Twelve u64 caller-local capabilities at byte120.
- Ten u64 current logical-principal incarnations at byte216; absent bindings are
  zero and unused logical principal7 remains zero.
- u64 device_sectors at byte296, then20-byte exact ATA serial and40-byte model.
- Four reserved zero bytes at byte364.

Phase0 is active. It accepts roles0..6,8,9 and the dedicated no-authority idle
role15; active logical roles must match their full current peer incarnation.
Role15 is not a logical principal and must never acquire a capability or IPC
identity. Its eventual physical CPU context is separate from the lifecycle
model's vacant logical slot bookkeeping.

Phase1 is a Settings trial (role5) with only cap9 and a nonzero health token.
It has no self receive, production sends, framebuffer, input or device fields.
The userspace trial entry now performs bounded local Settings health, reports
its one-shot token, and takes a fresh validated bootstrap after resumption;
see [Modern trial entry](modern-trial-entry.md). Kernel cutover, bootstrap
republication and rescheduling remain unimplemented. Active grants must be
published only after the authorized cutover.
An old immutable bootstrap or cached trial descriptor cannot imply those grants.

Active cap indices exactly match the lifecycle contract. Present handles must
have nonzero upper32-bit capability generation and their own one-based index
in the lower32 bits; all other slots are zero. This checks encoding only.
The active health token is zero. kernel_probe and peer_probe are reserved
placeholders and must both be zero; unlike the historical Desktop fixture,
Modern bootstrap construction must not insert kernel or another process's
private addresses. Any future diagnostic use needs an explicit reviewed contract.
Entry lies inside the fixed 1MiB service-image
VA window0x400000..0x500000; real PE/executable-page validation is separate.

Only compositor3 receives framebuffer VA0x800000, 640x480 dimensions,
640..4096 pixel pitch and format0 or1. Other roles receive zero display fields.
Only Data1/System9 receive nonzero device metadata: fewer than2^28 sectors and
exact printable, nonblank, space-padded ATA identity strings. The trusted
profile must independently supply and certify actual geometry/identity; these
shape checks do not certify a disk. Other roles receive zero device metadata.

## IPC and syscalls

The Modern receive envelope is152 bytes: u64 logical sender, u64 incarnation,
u64 length,128 payload bytes. There is no implicit padding. The kernel zeros
unused payload bytes and copies only after validating the full writable user
span. Do not reuse Desktop's 144-byte pointer bound or truncate an incarnation.
SEND remains at most128 payload bytes. Kernel queue operations must not consume
a message before validating the receive destination.

Private int80 numbers retain0 yield,1 send,2 receive,3 keyboard port read,
4 bounded evidence report,5 exit;6 is monotonic ticks,7 fixed device operation,
8 trial ready;9 System-only staging copy (specified below). The initial kernel dispatcher is now a source candidate;
userspace syscall/service composition is implemented in source, including the
trial receiver handoff above. Complete UEFI runtime acceptance remains pending.
Error returns are negative; tick success must fit nonnegative
i64 and fail on exhaustion rather than wrap. Tick semantics are specified below.
Trial ready must redeem the model's exact one-shot handle/token; the number
alone neither implements health nor grants active authority.

Device arguments are: caller-local handle, operation, value, zero. Decoder:

| Operation | Meaning | Accepted value |
| --- | --- | --- |
| 0 | Alternate status byte | 0 |
| 1 | Sector-count register | 0 or1 |
| 2,3,4 | LBA low/mid/high register | 0..255 |
| 5 | Master head register | 0xa0 or0xe0..0xef |
| 6,7,8,9 | IDENTIFY, READ, WRITE, FLUSH command | 0 |
| 10 | Read one data word | 0 |
| 11 | Write one data word | 0..65535 |

Other operations, slave selection, high-bit truncation and nonzero extra
arguments are rejected. There is no arbitrary native ATA opcode, raw port,
DMA control or reset. The real dispatcher must resolve the trapped caller's
device capability first and map its derived Data/System kind to separate fixed,
reviewed PIO ports. It must not expose this decoder directly as native authority.
ATA command sequencing, geometry, IRQ masking and transport poisoning remain
separate driver/kernel/profile responsibilities.

## Evidence and remaining work

Source tests cover exact offsets/sizes, every active cap mask, extra/missing or
malformed handles, full-width identities, wrong versions/resources, trial-only
authority and device argument bounds. A cross-module test constructs active
bootstrap descriptors from the real mechanism's handles and bindings.

These are source tests, not running OS evidence. The initial kernel entry below
connects trap, pointer checks, native adapters, ticks and bootstrap publication
in source. Its actual UEFI build/runtime validation, trial handover, service/UI
wiring and reviewed cloud execution are still gates.
No native execution, disk/profile activation or production ABI is authorized by
this candidate.

## Native fixed PIO candidate

nucleus/modern/native_pio.rs is the privileged leaf for the DEVICE operation.
It resolves the trapped caller's live capability, decodes the bounded operation,
then derives one fixed register set: Data1 uses0x1f0..0x1f7/control0x3f6;
System9 uses0x170..0x177/control0x376. No userspace port/device selector exists.
Initialization checks PIC slave mask bits6/7 (IRQ14/15), writes nIEN=1/SRST=0
to both device controls and rejects absent-status0/255. It issues no reset,
data transfer or disk command, and returns at the first error without retry.

The pinned Desktop cloud tool recipe uses Debian QEMU
1:7.2+dfsg-7+deb12u18+b3. Upstream QEMU7.2 ISA IDE exposes iobase, iobase2 and irq
properties ([primary source](https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/isa.c)).
These facts support a candidate layout, not certification of the patched Debian
binary or a composed q35 VM. Exact runtime property/port collision, IRQ,
master-only attachment, identity, geometry and confinement evidence remain
mandatory before this adapter is used. No upstream implementation is copied or
linked into RAR target code.

Only x86-64 UEFI builds contain the four in/out primitives. Other builds return
Denied without I/O. The public Adapter initialize/execute functions are unsafe:
the real kernel must ensure firmware exit, CPL0, IF=0, one CPU, exclusive
ownership, the exact reviewed cloud devices and current caller derivation on
every call. Adapter is neither Send nor Sync and has no public port fields,
ordinary constructor, Clone or reset. Native assembly intentionally omits nomem
so compiler memory operations are not allowed to move across the I/O boundary.

Every execution performs at most one native I/O and checks authority/arguments
first. It does not implement driver IDENTIFY/geometry, whole-command sequencing,
durability or transport poisoning; services/modern/pio.rs owns those mechanisms.
Unsafe code is permitted only in this explicit native leaf; model.rs and abi.rs
retain their own forbid(unsafe_code) guards. Tests use a private fake backend to
pin all operation/port mappings, denied/expired/type/value paths with zero I/O,
initialization boundaries and no retry. Linux tests/no_std compilation do NOT
compile or execute the UEFI-only asm branch; actual UEFI build, focused unsafe
review and VM tests are required before runtime acceptance.


## Initial kernel integration candidate — M4.1

nucleus/modern/main.rs is a distinct Foundation platform module selected by
rar_platform + rar_modern. Selecting Modern together with rar_desktop, or without
rar_platform, is a compile error. Existing Foundation/Platform/Desktop selections
are unchanged. The shared GOP adapter selects Modern's equivalent checked
geometry function only under rar_modern. No workflow or controller is activated
by these source changes.

The kernel reuses RAR's protected PE parser, guarded private CPU arenas, W^X
user mappings, bootstrap alias retirement, saved-register/SIMD trap boundary and
round-robin mechanism. It constructs initial CPU contexts0..6,8,9 and15 from the
fixed Modern service image. Logical authority lives only in model::Runtime,
not a duplicate legacy capability table. Idle15 has no logical endpoint or
capabilities; it can only yield, read ticks or terminate. Slot7 is not scheduled.

support::bootstrap derives grants and full u64 peer incarnations from live
kernel policy, initializes every bootstrap byte, inserts no private addresses
and checks the complete ABI shape before publication. The immutable one-page
handoff is mapped read-only. Compositor3 alone receives framebuffer pages.
The candidate fixed synthetic expectations are Data194 sectors (64 vault slots)
and System16384 sectors, both512-byte sectors. Exact space-padded ATA identity:

- Data serial: RAR-M4-DATA-00000001; model: RAR M4 DATA PIO.
- System serial: RAR-M4-SYS-000000001; model: RAR M4 SYSTEM PIO.

These are source expectations, not certification, secret keys, authentication,
a complete System slot layout or actual image provisioning. The controller must
independently enforce the matching collision-free separate PIO devices and
private synthetic attachments before entry execution. Distinct Data image
keys/nonces and no writable clones remain mandatory. No real user disk is used.

The trap checks saved CPU frame ownership/alignment/bounds before dereferencing
its pointer. SEND validates a readable1..128-byte current-process span; the
model stamps its logical identity and full incarnation. RECEIVE calls the same
tested support function that requires the exact152-byte writable span and valid
mode before popping the queue. Invalid destinations or handles cannot consume
a queued message. The copied envelope is fully initialized, with no padding or
incarnation truncation. IF=0 and the sole CPU serialize checks/copies with policy
changes. Successful sends may wake any logically active blocked context;
spurious wakeups confer no authority and remain bounded to16 contexts.

DEVICE delegates the trapped caller and its live policy directly to the fixed
native adapter. Keyboard reads independently require current caller-local input
authority and only ports0x60/0x64. Storage initialization occurs after firmware
exit and PIC installation but before ring3 entry. It is one attempt, no reset or
fallback. A missing/misconfigured mandatory controller is a boot failure, not
permission to probe other ports. Userspace still must IDENTIFY exact metadata
and apply bounded sequencing/poisoning before vault access.

TICKS counts delivered Modern IRQ0 events from zero. The retained Foundation
PIT divisor is11932 at nominal1193182Hz (approximately100Hz); this is scheduling
time, not wall-clock or a precise40.96-second guarantee. At i64::MAX it becomes
permanently exhausted and syscall6 returns the negative Exhausted error rather
than wrapping. App-session message and iteration bounds remain necessary when
ticks do not advance. Error encodings are Invalid=-1, Denied=-2, Stale=-3,
Full=-4, Empty=-5, Exhausted=-6, Busy=-7.

Fault/exit invalidates the exact saved incarnation in live policy, purges its
messages and grants, and synchronizes destroyed logical slots out of CPU
scheduling. No killed process is resurrected. Initial image mappings are not
reused by this entry. TRIAL_READY resolves the exact model handle/token and
parks a successful healthy candidate; there is currently no production path
to construct/start a trial. Trial preemption budgets, sealed candidate loading,
cutover/rebootstrap and post-cutover fallback remain M4.2 implementation work.
The GUI-READY serial marker means only the initial keyboard/compositor reports,
never durable files, successful update, recovery or M4 acceptance.

The service image input /tmp/modern-service.efi must be produced by the eventual
pinned cloud build, not copied from Desktop as a compatible substitute.
The source tests exercise the support functions used by this entry. The existing
guarded Specifications cloud check additionally compiles the real Foundation +
Modern entry to a Linux relocatable object, with no linking or execution.
rar_modern_compile_only substitutes an empty service input and is rejected for
UEFI or without rar_modern. No fake target_os cfg is used. This catches module,
type, borrow and common-assembly errors but does NOT compile the UEFI-only native
PIO branch or prove the real target ABI. It reuses the existing pinned compiler,
read-only source and bounded ephemeral scratch, with no new workflow or network.
Its actual pinned UEFI build, complete
service/UI composition, focused unsafe review, certified cloud profile and
causal fresh-VM persistence evidence remain mandatory before activation/release.

## M4.2 bounded staging-copy addition

Active System9 alone now receives slot10 StageCopy in addition to its receive
and System-device grants (exact mask0xc01). It is a separate object/right from
Manager8's caller-local slot10. Data/app/trial grants are unchanged. StageCopy
cannot select a disk or map/execute bytes.

Syscall9 uses a48-byte canonical request and16-byte reply, full-u64 seals and
a separate bounded512-byte readable-input helper. The original152-byte IPC
bound is unchanged. The kernel alone chooses a logically Vacant, physically
Clean, dead/root-zero Settings slot; copying is sequential and output pointers
are checked before state mutation. FINISH/ABORT and the separate manager view
are specified in the following immutable-verifier section; actual trial loading
is still pending. The earlier copy-only checkpoint in modern-update-transaction.md
is superseded for these operations by that section. This is experimental source
integration, not runtime acceptance.


## M4.2 immutable verifier window (source integration candidate)

STAGE_COPY syscall9 retains its exact48-byte request and16-byte reply.
Operation2 FINISH has nonzero seal, zero offset/pointer/length, and a checked
reply pointer. It requires complete sequential copying and all-zero allocation
padding. Before successful reply the kernel preflights every bootstrap/live
root, removes supervisor write permission from all513 identity-mapped staging
pages, reloads current CR3 under IF=0 with PGE/PCIDE off, then publishes only
manager8's fixed RO/NX user window at0x1400000 (513 pages). Both adjacent pages
remain absent. No byte becomes executable. Every future constructed root must
inherit this read-only state while the sealed allocation exists.

Operation3 ABORT has the same seal/zero-field framing, and is System-only before
FINISH. It clears only a matching COPYING reservation; it cannot remove a sealed
manager view or abort an accepted/running trial. Full non-elidable scrub and
readback precede Empty; reply returns the old seal and accepted count0.
Invalid reply spans are rejected before state mutation.

STAGE_VIEW syscall10 uses manager8's existing caller-local Manager capability.
Operation0: RDI cap, RSI0, RDX reply pointer, R10 exact32. Reply is four LEu64:
seal, logical length, fixed window address0x1400000, kernel-reserved physical
process slot5/7. Unknown op/extra bytes, stale or nonsealed state fail; System,
apps and trial Settings cannot obtain this authority. The view's page padding
is zero, but only the returned logical length is a signed-package input.

Operation1: RDI cap, RSI1, RDX nonzero exact seal, R10 zero. Reject is allowed only
before any staged executable/trial owns the reservation. Remove manager user
PTEs and tracked readable range, flush current translations, restore supervisor
writes across every root, flush again, and scrub/readback the full owned buffer.
Only then publish Empty; seals are never reused. Raw scrub pointers derive from
the Buffer owner's mutable slice, not an unrelated physical pointer.

Native safety invariants: sole certified guest CPU; validated ring3 trap with
IF=0; PGE and PCIDE off; exclusively kernel-owned table pools; all-root
preflight before first alias change; no writable/executable/user staging alias
outside the fixed manager view; no scheduling between mutation and TLB flush;
no root reuse or candidate construction while stale aliases exist. Impossible
internal map/context failures halt before exposure/reuse, rather than returning
a misleading successful seal.

Actual page-table-byte cloud tests cover guard absence, all513 exact aliases,
A/D-bit tolerance, rejection of U/W/X/global/cache/wrong-physical entries,
preflight nonmutation, RO/NX view publication/removal and no table allocation
during transitions. Pure tests do not execute CR3/INVLPG or establish native
guest evidence. System package IPC, manager verification, trial construction,
cutover/fallback and cloud acceptance remain required before M4.2 completion.


## M4.2 manager-only trial bridge (source candidate)

All additions use syscall10 STAGE_VIEW with the existing manager capability in
RDI. No new capability is granted to System, Data or applications. The manager
must first authenticate the exact immutable view using the intact selected
generation policy. Accepting a seal is its verification decision; the kernel
does not implement signature policy.

- Operation2: RSI2, RDX exact seal, R10 writable32-byte response. The kernel
  derives metadata and PE layout from the sealed reservation, independently
  checks resource/W^X geometry, removes the manager view and constructs a
  fresh health-only trial. Reply is four LEu64: seal, token, physical slot,
  full incarnation. Validate the whole output before mutation. Failure returns
  no reply, retains no runnable partial process and never activates the old
  candidate. Any dirty partial stride is retired before reuse.
- Operation3: RSI3, RDX exact seal, R10=0. Release after the trial/staged policy
  state is gone and physical retirement is complete. The verifier view must
  already be absent. Restore supervisor writes only after all-root checks and
  TLB invalidation, then erase/read back the full allocation.
- Operation4: RSI4, RDX exact trial token, R10=0. Abort exact current trial and
  revoke it; physical destruction is deferred to a surviving trap.
- Operation5: RSI5, RDX exact seal, R10 writable32-byte response. Return four
  LEu64: seal, trial token, full incarnation, kernel phase1 Trial or2 Healthy.
  Missing/stale trial returns an error, never an uncorrelated success flag.

Trial construction uses a16KiB user stack instead of the initial64KiB service
stack. Native user-return checks use the actual per-process upper bound.
Guard pages are absent. A Healthy trial remains blocked; only the later
durable-ACK cutover may give it production authority and resume it.

These are source mechanisms, not full update integration or a VM demonstration.
Boot barrier, System/manager protocol, signature-verifier loop, durable cutover,
peer GUI rebinding and fallback cloud evidence remain required.
