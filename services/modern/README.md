# Modern storage service transport candidate

Unactivated RAR-owned bounded ATA PIO sequencer and Modern service composition.
runtime.rs now connects Io to the Modern device syscall and mounts DataVault.
It has no native port instructions or device attachment authority.
The Io trait is not a security boundary: the kernel bridge must enforce
one fixed nondelegable adapter capability, register/width/command whitelists,
and no caller-supplied device selector.

read512/write512 transfer exactly256 little-endian words at one bounded LBA28
sector; flush is explicit. Polling uses at most65536 status reads per phase,
yielding every16. Missing/faulted/timeout/transport errors permanently poison
this instance. No ambiguous operation retries. Bounds errors touch no I/O.
A completed write is not a durable write until flush succeeds.

Candidate topology (matching native_pio.rs): Data role1 owns 0x1f0..0x1f7 plus0x3f6;
System role9 owns 0x170..0x177 plus0x376. Master only; the kernel bridge must mask IRQ14/15 and set nIEN before constructing Device;
Io deliberately exposes no control-port write. No DMA.
The immutable boot disk stays explicitly on Q35 AHCI, separate from both.
This proposal does not activate or extend the existing Desktop profile.

Prerequisites before production construction: IDENTIFY verifies ATA/LBA/FLUSH,
512-byte sectors, exact fixed capacity and distinct System/Data serial/model;
kernel and cloud profile independently enforce ownership. Exact pinned QEMU
ISA IDE availability, bus names, simultaneous topology and port-collision proof
remain pending. No geometry or identity supplied by untrusted IPC is accepted.

Source basis:
- https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/isa.c
- https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/ioport.c
- https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/core.c

Tests use a deterministic in-memory fake only, in the cloud test sandbox.
They are not physical disk, runtime isolation or persistence evidence.

Successful polling requires DRDY as well as clear BSY/ERR/DF and the exact DRQ
phase. Tests include absent DRDY, busy transitions, every partial data transfer,
status/register/command/yield transport failures, and post-transfer/flush errors.
Read data is a private temporary array and is returned only after completion.

## DataVault candidate

vault.rs adds a bounded append-only encrypted full-snapshot store over an abstract
Block trait. Its exact proposed schema, nonce lifecycle and crash rules are in
docs/interfaces/modern-data-v0.md. Mount never writes or seals; publish reserves,
flushes/readbacks, seals once, writes/flushes/verifies payload, then publishes the
fixed commit marker. Consumed slots are not reused. Errors lock current-boot
writes; exhaustion is read-only. No format/erase path exists.
This remains unactivated model code, not actual guest persistence, independent
crypto interoperability or M4 completion. Device/capability/profile integration
and real cloud crash/persistence evidence remain required.

## Verified IDENTIFY constructor (source candidate, not activated)

Production construction now goes through Device::identify with an Identity from
the fixed trusted profile, not an IPC request. The old unverified capacity-only
constructor is test-only. Initialization selects master, zeros the IDENTIFY task
file, reads exactly 256 words under existing bounded status/transport rules and
returns no Device on failure. It issues no sector write or flush while identifying.

The verifier requires the pinned QEMU ATA disk type, LBA and enabled FLUSH
support with valid feature words, exact LBA28 capacity and mandatory supported/enabled LBA48 with matching
capacity, and 512-byte logical/physical geometry. Serial and model
must match exact nonempty printable space-padded profile values after ATA word
byte-order decoding. DMA advertisement is not DMA authority. The caller cannot
learn authority by supplying a matching identity: fixed kernel adapter ownership,
IRQ masking/nIEN and separate reviewed System/Data profile identities are still
mandatory; the kernel source candidate enforces the fixed grant/port split,
but actual profile and runtime proof remain pending.

Source fixtures cover exact word-order identity, each required feature/capacity/
geometry/text mismatch, invalid expected profiles, all 256 transfer interruption
points, task-file/command failures and phase failures. These are cloud model
tests only, not actual device compatibility or disk persistence evidence.
The field interpretation follows the pinned QEMU ide_identify/ide_identify_size
implementation linked above. No target third-party code was copied or linked.

Focused review narrowed admission to actual pinned QEMU output: word106 is
zero or 0x6000 only; LBA48 support and enabled bits are mandatory and the full
extended capacity must match. Tests explicitly reject each cleared LBA48 bit,
zeroed extended capacity and unsupported 0x4000 geometry. Additional transport/
yield tests cover every initialization status boundary and all three wait phases,
and retain command/write logs proving IDENTIFY-only, with no Write or Flush.

## PIO-to-vault connection

The verified Device implements the existing vault Block trait directly. Sector
reads and writes preserve their exact 512-byte framing; flush remains a separate
explicit call. Bounds remain Bounds and all device/transport failures become Io.
Transport poisoning remains in force, so a failed or partial operation cannot
silently retry through the vault interface. No cache or hidden flush is added.

This is source-level transport composition, not a new capability boundary.
Production construction still requires IDENTIFY, and eventual kernel ownership
must supply only the Data role's fixed adapter to the vault. No user-selected
device or shared unrestricted port authority is admitted by this connection.
The kernel/source syscall bridge now exists; guest persistence proof remains pending.

Cloud-only source tests cover verified-constructor read/write/flush sequencing,
word order, out-of-bounds refusal without I/O, every partial transfer boundary,
failed flush and no subsequent I/O after poisoning. Vault crash tests also
publish distinct content after recovery and remount after a second crash to
verify the new chain beyond any burned slots.


## Real service and GUI wiring candidate

runtime.rs constructs Ports only from the trusted Data/System bootstrap grant,
maps each Io operation to the bounded DEVICE syscall, and checks result widths.
Data identifies once, mounts Store once using full Files/Terminal incarnations,
then serves the correlated Server. There is no RAM store, welcome seed, format,
retry or automatic remount. Publication errors stay locked inside Store/Device.

If IDENTIFY or mount fails, the service remains alive solely to return canonical
correlated Unavailable responses to authenticated current Files/Terminal
envelopes. The failure-only helper executes no request, creates no successful
ACK and performs no I/O; it need not preserve execution sequence state because
there is no mounted operation executor. Malformed/unauthorized frames are dropped.
Replies get one send attempt; a lost ACK is handled as uncertainty by the app.

FileRuntime connects the tested Session to actual checked kernel ticks, one
storage send, one own-queue receive and yield. GUI input is separately bounded
and shape checked. gui.rs uses full u64 peer incarnations for shell and surfaces;
it contains no DesktopStore or legacy storage backend. Fixed bitmap rendering
still belongs only to compositor3, and prominently labels public lab data.

These are implementation candidates, not VM proof. Actual pinned UEFI compile,
private Data provisioning, crypto references, profile certification and fresh-VM
write/read/oracle evidence are still required. M4.2 manager/System operations,
surface rebind and refreshed peer incarnations after replacement remain future
integration; the initial GUI uses its kernel-supplied bootstrap epoch.
