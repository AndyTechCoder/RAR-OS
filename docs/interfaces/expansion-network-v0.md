# Expansion network-v0 candidate contract

Status: experimental candidate; pure implementation only, no NIC authority.
Scope: static unicast Ethernet II + IPv4 + UDP. Specifications:
[RFC791](https://www.rfc-editor.org/rfc/rfc791),
[RFC768](https://www.rfc-editor.org/rfc/rfc768). No third-party target code.

## Packet boundary

Frames are supplied without preamble or FCS. Ethernet frame bytes range from
60 to 554, including bounded zero padding when transmitting. IPv4 uses a
20-byte header, version4/IHL5, TTL nonzero, protocol17, valid Internet checksum,
no reserved/fragment flags or nonzero fragment offset (DF allowed).
No options, fragments, VLAN or jumbo frames. UDP length equals the entire IP
payload, 8..520; datagram data is 0..512 bytes. UDP checksum is mandatory in
this restricted profile although IPv4 UDP generally permits omission.
Both checksums cover network-order bytes; computed UDP zero is sent as 0xffff.
Ignore only Ethernet padding outside the IP total length, bounded by frame max.

Addresses/ports must equal the configured direction. MAC is nonzero unicast;
IPv4 is nonzero unicast in this profile (first octet 1..223 except127; not
169.254/16), ports are nonzero. Static configuration must additionally be
approved by the lab controller; this parser does not choose network routes.
IP/MAC filtering and checksums do not authenticate a peer or encrypt content.

Encode validates all inputs before writing output. Too-small buffers or payloads
over512 fail without partial mutation. Decode returns a borrowed payload only
after all checks; no heap, unsafe code, unbounded loops or I/O.

## Policy boundary

A single explicit grant binds nonzero principal and full incarnation, exact
remote endpoint, exclusive expiration tick, total packet and byte ceilings.
Authorization checks current monotonic time (including backward-time rejection),
revocation, identity, destination and payload bounds before decrementing budgets.
Zero-length datagrams still consume one packet. An accepted attempt consumes
budget even if later device delivery fails. There is no automatic reset/refund.
Counters use checked comparisons before subtraction; equality at expiry denies.

Grant creation is internal trusted broker behavior, not caller access. Numeric
principal/incarnation inputs must come from kernel-authenticated message
metadata, and grant installation/revocation must come from policy authority.
This module alone cannot prove runtime isolation; M5 requires native service
and denial demonstrations.

## Failure and compatibility

Errors disclose only Invalid, Denied, Expired or Exhausted; no payload is logged.
Transport timeouts, queues and NIC lifecycle belong to a future reviewed service
contract and are not supplied by this codec. No persisted state or migration.
A later version may add optional protocols without silently broadening a v0
grant. Do not claim a connected OS until actual guest packet exchange passes.

## Native integration prerequisites

The future service must atomically bind kernel-envelope principal/incarnation,
its trusted clock, local interface/endpoint, exact destination and immutable
service-owned payload to the frame actually transmitted. Caller-supplied Grant
objects or independent reserve/encode calls grant no device access. Full wire
bytes (including discarded padding on ingress), packets, queues and deadlines
need separate resource ceilings, not payload-byte accounting alone.

Define revocation for already queued sends: revoke and discard unsent frames.
Destroy grants and queues on service death/restart; stale incarnation requests
cannot reuse them. Apply separate ingress limits before expensive parsing, and
retain malformed/drop/flood/peer-death tests. These are runtime prerequisites,
not guarantees supplied by the current codec.

## Candidate conformance sources

The cloud harness tests a fixed manually summed frame and the arithmetic vector
in [RFC1071 section3](https://www.rfc-editor.org/rfc/rfc1071). A RAR host-only
Python struct-based oracle independently emits every payload length0..512 for
exact comparison with the Rust encoder and decoder. It uses only the pinned
host Python standard library and never enters a target image. This is
cross-language conformance, not a third-party audit or actual NIC evidence.

## Service-owned channel implementation

The candidate Channel owns one local interface, local endpoint, remote endpoint,
principal and full incarnation. The native adapter must supply identity from the
kernel envelope and ticks from the kernel clock. Neither construction nor
revocation is an application operation. This module has no syscall or device
authority by itself.

It copies and encodes the exact packet before reserving budget. Both send and
receive queues hold at most four entries. Transmit budgets count the complete
Ethernet frame, including headers and padding. Every admitted ingress attempt,
including malformed packets and full-queue drops, consumes its packet/wire-byte
budget before parsing. Ingress budget exhaustion returns Budget and permanently revokes the channel,
clearing queued data. Smaller later frames cannot revive it. A new channel
requires fresh explicit policy authorization.

Driver transmission is one synchronous bounded attempt inside an exclusive
Channel borrow. No frame slice escapes the call. Any driver error revokes the entire channel,
clears both queues and never refunds or silently retries. The native driver must enforce its own deadline; a callback
does not itself prove hardware termination. Revocation linearizes when the
single-owner service processes it; it clears both unsent and unread queues.
It cannot undo already transmitted frames or already delivered application data.
Expiry or backward trusted clock observation permanently closes the channel.

The channel is deliberately non-Clone. Service restart cannot restore an old
queue/counter snapshot; it requires freshly authorized construction and the
current process incarnation. Drop clears logical buffers but is not a physical
or cryptographic erasure claim. Kernel page retirement remains required.
Counters are bounded/saturating and contain no payload logs.

Host unit tests cover exact copied bytes/endpoints, FIFO/full queues, stale
incarnations, wrong interfaces, clock rollback, expiry/revocation, immutable
receive copies, uncertain device completion, malformed/full ingress charging,
empty-packet wire accounting and invalid construction. Native runtime, driver
and controller evidence remain separate required gates.
