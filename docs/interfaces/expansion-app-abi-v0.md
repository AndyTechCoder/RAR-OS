# Experimental native app boundary v0 (candidate)

Status: source/SDK conformance candidate, not kernel-activated or stable.
This is separate from the private 368-byte Modern bootstrap. M5.2 still requires
independent executable examples and actual GUI launch. Related: ADR0038.

## Bootstrap

Exactly 256 bytes from a kernel-owned read-only mapping, provisionally 0x700000.
Bytes received from an app or network are never bootstrap authority.
Unsigned integers are little-endian; C/Rust structures are not wire layouts.

| Offset | Bytes | Value |
| --- | --- | --- |
| 0 | 8 | ASCII RARAPP00 |
| 8 | 4 | ABI 0 |
| 12 | 4 | Length 256 |
| 16 | 16 | Nonzero signed application ID |
| 32 | 8 | Nonzero full kernel process incarnation |
| 40 | 4 | Logical app principal 10 or 11 |
| 44 | 4 | Requested-and-granted rights subset, UI1 mandatory, Doc2, Net4, Agent8 |
| 48 | 8 | Validated PE entry within 0x400000..0x420000 |
| 56 | 40 | Five local 64-bit capability handles |
| 96 | 16 | Four 32-bit expected peer principals |
| 112 | 32 | Four nonzero 64-bit expected peer incarnations, if enabled |
| 144 | 112 | All zero |

Handle0 is self receive. Handles1..4 correspond to UI, document, network, agent.
Enabled handles have nonzero high32 generation and low32 equal to slot+1.
Disabled handles are zero. Peer principals are respectively compositor3,
Storage1, network7, broker12; disabled peer identity fields are zero.
No framebuffer, raw disk, NIC, Manager, signing, System or recovery grant is
conveyed. The kernel derives handles; manifest flags never manufacture them.

The codecs validate framing, not actual page ownership or live revocation.
Native integration must provide the exact mapping, derive current identities,
revoke kernel endpoints and authenticate every received kernel envelope.

## Message frame

Exactly 128 bytes, no trailing bytes. Header: ASCII RAPP at0..4; version byte4=0;
operation byte5; status byte6 (0..7); reserved7=0; nonzero LEu32 sequence8..12;
LEu16 payload length12..14 (0..112); reserved14..16=0; data16..16+length;
all remaining bytes zero.

Operations: Paint1, ReadDocument2, WriteDocument3, Input4, Tool5, Datagram6.
These are proposed routing identifiers, not activated service operations.
Before dispatch, each service must validate its operation-specific payload,
status direction, actual stamped caller identity and grant. This generic codec
does not authorize a tool, paint, write or network action.

Replies must match operation, nonzero sequence and full expected kernel peer
identity. Correlation does not itself prevent duplicate actions: service and
client lifecycle/deadline rules must do that. No automatic retry is provided.

## SDK and tests

sdk/alpha/app_contract.json is the constant source; Rust constants.rs and C
constants.h are deterministically derived and checked by the read-only
tools/rar-lab/expansion/app_bindings.py --check path. Its fixed semantic guard
requires explicit review/conformance changes for contract evolution.

Rust is no_std safe code with owned bounded messages. C is freestanding, uses
no allocator, libc runtime, or syscalls; callers provide valid memory spans.
The C codecs stage output locally, so invalid input never partially overwrites
the output. Neither structure's memory layout is public.

Cloud Specifications compiles both bindings independently and compares 678
canonical message frames plus 513 valid/malformed/short bootstrap decisions.
Rust tests cover exact lengths, zero padding, disabled grants, unknown IDs,
incarnation width, correlation and budgets. No target entry is executed.

Pending: documented semantic payload contracts, native syscall adapter and
kernel bootstrap publication, independent Rust/C executable examples, installer,
GUI app launch, runtime malformed/revocation tests, and integrated M5 acceptance.
A wire codec is not a finished native application SDK.
