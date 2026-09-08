# Modern private block backend candidate

Status: unactivated host-only source under M4.1. No Modern VM/controller or
device profile is activated by these files. The cloud Specifications self-test
exercises private regular-file I/O and an AF_UNIX socketpair, not RAR OS.

## Fixed authority and transport

Each connection is bound by the trusted controller to exactly one exclusively
owned synthetic regular-file descriptor: Data194 sectors or System16384 sectors,
512 bytes per sector. The backend has no path-opening API. It rejects nonregular,
multiply linked, wrong-size or replaced descriptor identities. Writable disks
require O_RDWR and declared read-only disks require O_RDONLY. O_APPEND is always
rejected. Flags/identity/capacity are checked before writes and after fsync before
success, so Linux append-mode pwrite behavior cannot produce a false flush ACK. These checks
are NOT path confinement: the future launcher must create exact private files
exclusively inside its bounded disposable container scratch, keep source/boot/
recovery/Data/System in separate inode/backing domains, and never supply a host,
shared, raw or unrelated descriptor. QEMU receives only the socket endpoint,
not the disk descriptor or any writable runner filesystem bind.

The transport accepts only an already-connected non-listening AF_UNIX stream.
It does not create sockets, bind/listen/connect, invoke subprocesses, interpret
paths or accept a device selector. The intended reviewed launcher uses one
socketpair per device and passes the peer FD only to its fixed QEMU process.
No TCP, external endpoint, TLS/credential role or network listener is proposed.
QEMU7.2 may attempt reconnection even with reconnect-delay=0; that flag is not a
no-attempt guarantee. The single preconnected peer is never rebound or accepted
again, and no stopped backend permits replay. Cut/fatal outcomes require whole
VM/backend termination before any next boot.

Primary QEMU7.2 source accepts numeric preconnected socket descriptors outside
a monitor through [socket_get_fd](https://github.com/qemu/qemu/blob/v7.2.0/util/qemu-sockets.c#L1144)
and [NBD address configuration](https://github.com/qemu/qemu/blob/v7.2.0/block/nbd.c#L1682).
This is design evidence, NOT certification of the pinned patched Debian binary,
inherited FD lifetime, composed VM port layout or actual target execution.

## Protocol subset and bounds

The original RAR-owned host code follows the standard
[NBD protocol](https://github.com/NetworkBlockDevice/nbd/blob/6725f91e6a33bc9f62d31d82798c04ec1cde1c73/doc/proto.md).
No upstream implementation is copied or linked into the RAR target.

Fixed-newstyle client flags1 or3 are accepted. The124 legacy zero padding bytes
belong only to EXPORT_NAME, which is rejected; GO has no padding regardless of
C_NO_ZEROES. Both flag combinations have real socketpair GO/write/flush/read tests.
Fixed-newstyle only, at most16 option requests, option bodies at most4096 bytes.
INFO/GO expose only the default already-bound device, never an export selector.
GO requires explicit block-size negotiation: min/preferred512, max payload65536.
Legacy EXPORT_NAME fails closed; unsupported extensions receive UNSUP without
activation. Negotiation is one-way into transmission. Only READ, WRITE, FLUSH
and DISC exist; no FUA/TRIM/zeroing/cache/resize/structured/multi-connection
feature is advertised. Full64-bit request cookies are echoed unchanged.

All requests are framed completely and bounded before backend effects. Sector
alignment, exact device capacity and zero reserved fields are enforced. A WRITE
body is read completely before any mutation. Permission errors return EPERM;
backend I/O failures return EIO, never success data. Malformed framing, an
unsupported command/flag, partial transport failure or budget/deadline breach
ends the connection without replay. DISC never flushes uncommitted state.

Socket I/O has an absolute maximum180-second deadline,8192 requests and
128MiB aggregate inbound/outbound wire budget. Synchronous file reads/writes and
fsync are NOT bounded by that timer. The future launcher must place the backend
in a separately killable process with an externally enforced whole-backend
deadline; it must terminate/join both VM and backend before freezing evidence. It sends no request-derived text,
path or command to a shell. The concrete launcher must use tighter scenario and
whole-process limits as needed and handle every terminal outcome explicitly.

## Durability, cuts and non-reconstruction

Disk writes initially affect only a bounded volatile copy and dirty-sector set.
Reads see that copy within the one live connection. A successful FLUSH writes
every dirty sector with exact-count pwrite and calls fsync before returning;
only then may the transport send success. Actual short writes/fsync/identity
failures stop the backend permanently; they are not silently retried or repaired.
The generic file API is deliberately not a DataVault encoder.

The controller selects at most one fault by write/flush ordinal, before launch:
error, before-cut, after-cut, torn-cut or short-error. No guest success marker
chooses the cut. Events record the observed operation, ordinal, exact offset/
length, write payload hash, injected plan and completed/failed/cut outcome.
Read operations do not advance write ordinals. An unhit fault is not a passed
fault scenario; the launcher must reject that evidence.

Torn/short injection persists exactly the selected bounded prefix and fsyncs it.
A prefix longer than the actual observed write/pending flush is an error, never
silently truncated. Before-cut does nothing; after-cut occurs after the defined
operation but before a reply. A cut propagates without a success reply; the
controller must destroy the ENTIRE QEMU process, not just close the channel.
Short-error returns failure with sticky refusal; it is distinct from a cut.
For reordering tests, dirty sectors flush in descending offset order, but a
successful FLUSH still commits ALL preceding completed writes before ACK.

Backend events are limited to8192 and traffic to64MiB per disk instance. Memory
is bounded by the fixed image plus its dirty-sector set and bounded records.
One instance can attach only once. Detachment stops its I/O and never flushes.
A new VM needs a new backend object reading the SAME retained file; reattaching
an old volatile copy is refused. Frozen reads reject an attached connection and
read actual descriptor bytes, never the volatile buffer or a reconstructed
snapshot. The launcher must additionally prove VM termination and backend-process
join; a byte helper cannot prove an external process lifecycle.

This proves only virtual-device crash behavior across the chosen process cut.
Disposable tmpfs/fsync does not claim survival of host power loss, a production
filesystem, confidential keys or an anti-rollback hardware anchor.

## Tests and activation boundary

The new15-test suite is a candidate until exact-head cloud CI passes. It uses
only exclusively created small synthetic files under the already guarded cloud
Specifications /tmp and a private socketpair. It exercises actual write/flush/
fresh-reader behavior; reverse ordering; retained torn prefixes; before/after
cuts; sticky real/modeled I/O failure; read-only refusal; geometry/identity/
budget limits; negotiation and packet negatives; and fragmented real-socket
WRITE -> FLUSH -> READ. It never runs a VM, target executable or reference adapter.
Its fixture files are retired only by normal disposable-container teardown.

Before Modern guest use, complete focused independent review of this backend
AND the concrete launcher/profile, inherited-descriptor ownership, socket and externally enforced process deadlines/
kill/join handling, exclusive image provisioning, fresh firmware and read-only
boot composition. Crypto interoperability and the actual causal Terminal/Files
fresh-VM proof remain M4.1 acceptance requirements. This candidate does not waive
them and does not authorize any Mac/SSD operation.
