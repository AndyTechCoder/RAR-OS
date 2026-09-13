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
