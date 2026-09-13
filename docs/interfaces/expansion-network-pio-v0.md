# Candidate fixed Network PIO leaf

Status: source/compile candidate only. No syscall dispatch, initialization call,
network-service scheduling or VM activation is added.

nucleus/modern/native_net.rs is the isolated unsafe I/O leaf for the proposed
NE2000 ISA device. Actual instructions exist only for x86_64 UEFI. All native
operations return Denied on other targets, including the Linux source test
configuration. Fake Ports tests execute no actual port instructions.

The unsafe constructor requires the separately certified cloud profile,
firmware exit, CPL0, IF=0, sole CPU and exclusive ownership. It verifies IRQ5
is PIC-masked by reading0x21 and neither changes PIC masks nor touches NIC
registers during construction. This does not certify the VM or device topology.

## Typed request boundary

Only Runtime.network(actual trapped caller, caller-local handle) authorizes an
operation. The expected active principal7 is physically slot10. Every framing
check precedes I/O. No absolute port, device ID, memory address or pointer is
accepted. The candidate operation fields are:

| Operation | Value | Extra | Effect |
| --- | --- | --- | --- |
| 0 | register0..15 | 0 | byte read at0x300+register |
| 1 | register0..15 | byte0..255 | byte write at0x300+register |
| 2 | 0 | 0 | word read at0x310 |
| 3 | word0..65535 | 0 | word write at0x310 |
| 4 | 0 | 0 | reset-port byte read at0x31f |

CR writes permit only21,22,61,62,0a,12,26 hexadecimal: the driver-used stopped,
started, page1, remote read/write and transmit states. Register15 writes require
zero on every page, preventing NIC interrupt enable (and constraining the last
multicast byte to zero). The register grant controls NIC SRAM through PIO,
not bus-master access to guest or host memory. Service code still owns frame
bounds, MAC/ring setup, grant accounting and bounded polling.

A successful authorized operation binds full owner endpoint and handle.
I/O failure closes the leaf and makes one best-effort STOP write; there is no
retry or alternative port. Later operations are refused. Reconciliation stops
a previously owned device if its full capability/binding is revoked or its
process dies. An owner-incarnation mismatch also closes rather than adopting a
replacement. No reopening exists.

The future native trap path MUST invoke reconciliation immediately after
relevant policy revocation and before scheduling another user. This candidate
method alone is not a device-death guarantee. Kernel I/O ownership, per-call
CPL0/IF=0 requirements, and controller peer/process teardown remain mandatory.

Tests check exact port transcripts, every operation's I/O-error closure,
wrong/stale/cross-role handles, all CR bytes, truncation/extra-field refusal,
no I/O on rejected input, sticky owner-death reconciliation and inert native
behavior off UEFI. Full native UEFI compilation, profile certification and
causal guest I/O tests remain required. Existing Data/System ports and profiles
are unchanged.
