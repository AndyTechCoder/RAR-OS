# Modern storage service transport candidate

Unactivated RAR-owned bounded ATA PIO sequencer. No native port instructions,
syscalls, block attachment, filesystem or OS execution entrypoint exists here.
The Io trait is not a security boundary: a future kernel bridge must enforce
one fixed nondelegable adapter capability, register/width/command whitelists,
and no caller-supplied device selector.

read512/write512 transfer exactly256 little-endian words at one bounded LBA28
sector; flush is explicit. Polling uses at most65536 status reads per phase,
yielding every16. Missing/faulted/timeout/transport errors permanently poison
this instance. No ambiguous operation retries. Bounds errors touch no I/O.
A completed write is not a durable write until flush succeeds.

Proposed topology: System role9 owns 0x1f0..0x1f7 plus0x3f6; Data role1 owns
0x170..0x177 plus0x376. Master only; the kernel bridge must mask IRQ14/15 and set nIEN before constructing Device;
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
support with valid feature words, exact LBA28 capacity and consistent LBA48
capacity if advertised, and 512-byte logical/physical geometry. Serial and model
must match exact nonempty printable space-padded profile values after ATA word
byte-order decoding. DMA advertisement is not DMA authority. The caller cannot
learn authority by supplying a matching identity: fixed kernel adapter ownership,
IRQ masking/nIEN and separate reviewed System/Data profile identities are still
mandatory and not yet implemented.

Source fixtures cover exact word-order identity, each required feature/capacity/
geometry/text mismatch, invalid expected profiles, all 256 transfer interruption
points, task-file/command failures and phase failures. These are cloud model
tests only, not actual device compatibility or disk persistence evidence.
The field interpretation follows the pinned QEMU ide_identify/ide_identify_size
implementation linked above. No target third-party code was copied or linked.
