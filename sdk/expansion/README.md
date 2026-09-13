# Experimental Expansion Rust SDK

Candidate source library only. No native application, syscall shim, package,
C binding, installer, signing authority or stable ABI is provided yet.

`rust/lib.rs` is a standalone no_std crate with no target dependencies.
It exposes the network protocol in `wire`; applications need not import
service, kernel, NIC or channel implementation modules.
See docs/interfaces/expansion-service-v0.md for the experimental byte contract.

Client construction binds expected network-service principal7 and its full
incarnation from trusted kernel bootstrap. Client state is not a capability:
the native transport must independently enforce a real caller-local SEND
capability, stamp received sender identity and protect payload buffers.
Never copy an identity supplied by a peer message into accept() arguments.

Call begin() into a128-byte array, send only its returned prefix through the
native transport, then call accept() on the exact reply and kernel-stamped
sender metadata. There is one outstanding request. Malformed/wrong-peer replies
do not consume it. IDs increase without wrap and correlate replies only.
They do not authenticate UDP packets or deduplicate work at the service.
The client never retries. If a transport result is uncertain, retire() closes
the client permanently; this cannot undo an enqueue or recall remote effects.
The owner decides any fresh-session policy rather than silently replaying.

A successful Send reply means queued, not delivered. Receive success may have
an empty payload, distinct from Empty status. Only successful Receive replies
may contain payload bytes. Closed/I/O status or successful Close permanently
closes the client. Payload borrowing is valid only while the reply buffer lives.

Standalone no_std compilation and source tests run only in the existing cloud
Specifications sandbox. Service/SDK interoperability tests exchange exact bytes
through the candidate dispatch and packet codecs; they are not guest evidence.
Native SDK packaging, independently signed apps, Rust/C examples and causal
launch/persistence proofs remain M5.2 requirements.
