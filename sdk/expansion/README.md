# Experimental Expansion SDK

First-party Rust and C **wire libraries**, plus an Expansion-only native
Terminal network tool. No independently packaged SDK application, general
app installer, signing authority or stable application ABI is provided yet.

`rust/lib.rs` is a standalone no_std crate without target dependencies.
`c/network.h` is a freestanding C11 header using only standard integer/size
types. Neither links kernel, NIC or channel internals. The C library itself
does not call libc, allocate memory, execute syscalls or obtain authority.
The host-only C tests use libc inside the existing pinned cloud test image.

Both implement [the same experimental network byte contract](../../docs/interfaces/expansion-service-v0.md).
C wire enums have the same values as Rust; local API error codes are not wire
status codes. A streamed 345-record cross-language test checks all send payload
lengths, all operations, three full-width IDs, exact lengths and zero tails.
Each language also tests response framing, status rules, closure and incarnation.

Client construction binds expected network-service principal 7 and its full
incarnation from trusted kernel bootstrap. Client state is not a capability:
the native transport must enforce a real caller-local SEND capability, stamp
received sender identity and protect payload buffers. Never use an identity
supplied inside peer message bytes as the envelope identity.

Call begin into a 128-byte array, send only its returned prefix, then accept the
exact reply and kernel-stamped sender metadata. There is one outstanding
request. Malformed/wrong-peer replies preserve it. IDs increase without wrap;
they correlate replies, not authenticate UDP or deduplicate service effects.
Neither client retries. Retire permanently after an uncertain transport result:
it cannot undo an enqueue or recall remote effects.

Send success means queued, not delivered. Empty successful Receive payload is
distinct from Empty status. Only successful Receive may carry payload bytes.
Closed/I/O status or successful Close permanently closes the client.

## C ownership requirements

The caller must provide valid live storage of the documented sizes. Client,
output, returned-length and response-structure storage must not overlap.
The encoder supports payload/output overlap by copying the bounded payload
before clearing output. NULL payload is valid only with zero length.
Response payload borrows the input reply buffer; its lifetime ends when that
buffer is modified or released. Never serialize `struct rnet_client` or
`struct rnet_response`: only the explicitly encoded bytes form a wire ABI.
Host pointer sizes/padding are not part of the contract.

## Native integration and remaining acceptance

The Terminal tool uses the Rust wire client through a bounded private native
adapter. It retains full sender incarnation, one-send/no-retry semantics,
deadline/iteration bounds and keyboard backpressure. It is not a separately
built third-party app. See [native tool details](../../docs/interfaces/expansion-network-app-v0.md).

Compilation, unit tests and cross-language byte checks run only in the existing
cloud Specifications sandbox. They are not guest execution evidence.
Native SDK packaging, independently signed Rust/C example apps, general
application ABI, private document ownership and causal launch/persistence proof
remain M5.2 work.
