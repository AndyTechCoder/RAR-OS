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
Cutover/rebootstrap and actual trial-entry handling are not yet implemented;
a real runtime must supply active grants only after the authorized cutover.
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
8 trial ready. The native dispatcher and userspace assembly wrappers remain
unimplemented. Error returns are negative; tick success must fit nonnegative
i64 and fail on exhaustion rather than wrap. Tick units must be fixed and
documented with the actual timer implementation before runtime acceptance.
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

These are source tests, not running OS evidence. Real trap/assembly integration,
kernel pointer validation, fixed native adapters, timer, bootstrap publication,
trial handover, service/UI wiring and reviewed cloud execution are still gates.
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
