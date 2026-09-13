# Experimental network service IPC v0

Status: candidate implementation, not an activated native service or stable SDK.

## Authority and composition

`services/expansion/service.rs` owns one channel and one bounded Link adapter.
The NE2000 Device implements this adapter without adding port instructions.
Policy is supplied only by trusted composition: full principal/incarnation,
fixed interface, local/peer endpoint, issue/expiry ticks and separate ingress/
egress wire budgets. There is no application install-grant or endpoint-selection
operation. Kernel envelope identity and trusted clock metadata must be used by
the future native dispatcher; caller message bytes cannot supply identity.

One poll attempts at most one receive and one transmit. Malformed ingress is
charged and discarded without starving that turn's queued transmit. Time is
checked before receive and again before transmit because driver polling yields.
Expiry, backward time, I/O failures, invalid driver lengths and ingress-budget
exhaustion revoke the channel, clear both queues and stop the link, permanently.
An idle service must still be polled by its native scheduler for timely expiry;
this source implementation does not establish a native scheduling guarantee.
A peer becoming silent is not detected as death by UDP. Cloud peer-death cleanup
and native evidence are still required.

The Link contract is trusted: bounded synchronous calls, no retained borrowed
slices, no ambient host resources. A generic adapter implementation is not itself
proof of confinement. No kernel Network capability, syscall, role/slot mapping,
port adapter, or paired VM profile is activated by this code.

## Message encoding

All messages are at most128 bytes. The16-byte header is:

| Bytes | Meaning |
| --- | --- |
| 0..4 | ASCII RNET |
| 4 | version0 |
| 5 | operation:1 enqueue,2 receive,3 close |
| 6 | request zero; reply status |
| 7 | reserved zero |
| 8..16 | nonzero little-endian u64 correlation ID |

Enqueue may carry0..112 payload bytes. Receive and close have exactly16 bytes.
No implicit truncation, pointers, interface, address, port or capability fields.
IDs correlate replies only; they are not replay prevention or exactly-once
tokens. Repeating enqueue can enqueue again and consumes the ordinary budget.

Replies carry the same operation/ID for well-formed authorized requests.
Malformed or unauthorized messages return operation0/ID0, header only.
Statuses:0 OK,1 invalid,2 denied,3 closed,4 full,5 empty,6 budget,7 I/O,
8 oversize. Unused reply storage is initialized to zero. Receive success may
carry0..112 bytes. A larger valid UDP datagram is consumed with oversize and no
partial payload. An empty success payload is distinct from empty-queue status.
Enqueue success means accepted into the bounded service queue, not transmitted,
delivered, authenticated or acknowledged by a remote peer. Close is permanent;
already delivered data cannot be recalled.

The Rust request encoder clears its complete128-byte output before validating
and returns the exact encoded length. Native SDK packaging, C bindings and apps
remain separate M5 acceptance requirements, not provided by this encoder.

## Verification and replacement

Source tests compose channel accounting, packet encode/decode, service dispatch
and a bounded in-memory Link. They cover every application payload length,
copied ownership, unauthorized identities, malformed envelopes, stale clock/
expiry between receive and transmit, driver errors and invalid lengths, ingress
budget exhaustion, non-starvation and sticky close. The adapter's public clock
failure has a focused NE2000 test.

These tests are not guest execution or network interoperability evidence.
A replacement must preserve exact IPC bytes, authority source, bounded work,
queue/budget behavior, failure closure and native causal guest proofs.
Persisted Data/System formats and all existing Modern cloud profiles are unchanged.
