# ADR0037: Expansion Alpha composition

Status: Proposed — 2026-09-13
Decision candidate: Alternative A. Runtime authority is not active.

## Context and alternatives

M4 is released. The owner directs completion of M5 under the fast-track process.
A) Add bounded connected-app and agent/profile verticals over the existing custom
OS with explicit experimental contracts and retained cloud proofs.
B) Implement the entire long-term consumer roadmap before the next release.
C) Present host-model simulations or browser mockups as completed OS features.

Choose A for implementation planning; reject C. B remains the long-term roadmap,
not a completion claim for this experimental release. No constitutional principle,
tier definition, security guarantee or production commitment is removed.

## Proposed architecture and review boundary

Network packet handling lives in a replaceable isolated service with a
kernel-mediated bounded NIC grant. Start with static-peer Ethernet/IPv4/UDP,
no routing, fragmentation, IP options, DNS, DHCP or public Internet transport.
The first pure codec requires no I/O or new syscall. Its output is untrusted
until a service checks its caller's actual kernel-provided identity and grant.
A peer MAC/IP is a destination restriction, not authenticated peer identity.
Sensitive data and production pairing must wait for a separately reviewed
authenticated protocol.

The first grant table is internal policy, never an unforgeable capability by
itself. A future service must derive the principal/incarnation from its received
kernel envelope, never caller bytes; only owner-policy authority may install
or revoke a grant. Budgets are consumed before transmission and not refunded
on transport failure, preventing retry-based accounting bypass.

Native SDK apps retain separate protected execution and use versioned messages.
Do not expose raw device or kernel bootstrap authority. Agent providers receive
only tool proposals; a broker authorizes each action independently. A test
provider proves the broker behavior, not AI intelligence.

Presentation/resource profiles share application identity and state. Tier0
portable execution requires its own bounded interpreter evidence, not just
removing the GUI. No ARM64 or physical support is inferred.

## Host safety

No local effects. The existing cloud networking prohibition stays active until
the concrete closed peer backend, NIC/device grant, lifecycle and controller
code pass independent review. No Internet, bridge, TAP, physical passthrough,
owner files, secrets, arbitrary launch arguments or external network listener.
Legacy profiles remain unchanged. Pure codec tests may run in the existing
trusted network-disabled Specifications sandbox; they cannot launch guests.

## Validation, migration and replacement

The M5 packet specifies actual acceptance journeys. Protocol parsing tests
include independent wire vectors, exact length, checksum, field, exhaustion,
revocation and stale-generation cases. Review packet/controller trust boundaries
before device activation; final integrated review covers apps, agents, profiles,
update/recovery regressions and release evidence.

All new interfaces are experimental and separately versioned. No change to M4
persisted Data/System formats is authorized by this ADR. Replacements must pass
the same wire/API and causal guest proofs. New stored schemas need explicit
migration/rollback contracts rather than shared internal layout.
