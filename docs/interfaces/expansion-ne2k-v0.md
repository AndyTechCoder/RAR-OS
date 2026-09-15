# Candidate Expansion NE2000 driver and virtual binding

Status: candidate code; not activated, not certified runtime evidence.

## Sources and ownership

RAR-owned implementation from the National Semiconductor DP8390D/NS32490D
July1995 register/ring specification (manufacturer document, mirrored at
https://gramlich.net/projects/datasheets/national/dp8390d.pdf), sections6,7,10.
No existing driver source was copied or linked. QEMU remains a pinned host tool,
not RAR target runtime. Only a reviewed virtual NE2000 ISA profile is proposed;
no physical hardware compatibility claim is made.

## Driver boundary

Device<Io> contains no port instructions, host calls, assembly or unsafe Rust.
Its future adapter must expose only a distinct kernel Network capability:
register offsets0..15, a16-bit data port at+0x10, and the dedicated reset
operation at+0x1f. Proposed fixed virtual aperture is0x300..0x31f.
No storage Device capability, caller-chosen absolute port, pointer or physical
device selector may be reused. PIO transfers address NIC SRAM, not guest
physical memory; call this no guest-memory/bus-master DMA, not no DMA.

Initialization polls reset, masks NIC interrupts, selects little-endian word
PIO, verifies duplicated PROM MAC against the certified profile, programs and
reads back unicast address, disables multicast and configures the bounded ring.
No NIC IRQ handler is required; proposed IRQ5 must stay PIC-masked and be
verified not to collide. IRQ9 is not selected because q35 ACPI commonly uses it.

The implementation reserves SRAM0x4000..0x45ff for transmit and
0x4600..0x7fff for receive, a deliberate subset of the virtual NIC SRAM.
It never follows a ring header outside that region. Remote transfers are
word-rounded, at most MAX_FRAME+1 bytes; receive wrap is split explicitly.
RX status, count (including CRC), exact expected next page, bounds and overflow
are checked before payload copying. CRC bytes are discarded; full frame bounds
remain60..554. Malformed ring state closes the device instead of guessing a
recovery pointer. No automatic reset/retry.

TX copies the frame into NIC SRAM, waits for remote transfer completion, then
starts transmission and requires TX completion without TX error. Each wait
has both a512-poll ceiling and20 trusted ticks; clock rollback/overflow also
fails closed. Every poll yields. Actual driver/clock adapter and scheduling
termination remain unproven until native evidence. Any I/O, metadata, overflow
or timeout anomaly permanently stops the device; caller Channel must also revoke
and drop queues. Invalid caller frame length is rejected before device writes.

## Proposed composition (review prerequisite)

Physical task slot7 remains reserved for Settings trial/rollback. Networking
uses physical slot10 with logical principal7 only through an explicit reviewed
mapping. Current Boot peer validation, bootstrap slot==role assumption, capability
graph and signed desktop activation must change together under a versioned
Expansion composition. A piecemeal role7 insertion is prohibited.

The future cloud backend uses one private AF_UNIX SOCK_DGRAM socketpair wholly
inside the existing network-none disposable container, one connected endpoint
per QEMU. No TAP, bridge, slirp, forwarding, Internet, network listener or owner
resources. Existing private QMP control sockets are not network listeners.
Bound/verify family, type, buffer sizes, connected endpoints, descriptor sets,
MAC, QOM model/iobase/IRQ/netdev and flat I/O-map ownership/no overlaps.
Preserve existing Modern profiles unchanged. On partial launch or either peer
death, terminate/reap both guests and their private block backends and close
all endpoint copies. No proposal-selected arguments or descriptor inheritance.

## Evidence and remaining work

Unit tests exercise register sequencing/PROM refusal, exact odd/even TX bytes,
receive CRC removal and ring wrap, corrupt headers/overflow, timer timeout and
stalled-clock poll bound with sticky failure. Fake Io is a deterministic model,
not QEMU evidence. Actual NIC binding, port adapter, service loop, signed
composition, paired controller and causal guest packet exchange are still
required before any connected-OS or M5.1 completion claim.
