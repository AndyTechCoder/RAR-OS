# Alpha Boot Follow-up Choice Brief

Status: Proposed — 2026-08-29

The first boot/platform contract draft correctly remained uncommitted after
independent reviews found three owner-level trust-boundary gaps. The recommended
decision set for only those gaps is:

1. ADR 0027 Alternative B — keep firmware placement for Root, deterministically
   retire/unmap all Root ranges, and enforce q35 AHCI shutdown plus PCI
   bus-master disablement before re-hashing sources.
2. ADR 0028 Alternative A — derive non-circular, domain-separated identities
   from exact immutable Root, Recovery, contract, and service executable bytes.
3. ADR 0029 Alternative B — hold state authority in revocable Nucleus slots and
   give Core only opaque slot selectors; Nucleus injects a separate
   identity-bound redeem token directly into each verified state service.

These are experimental Alpha specification choices. They do not authorize
target compilation, image creation, provisioning, VM launch, Mac-native target
activity, production claims, or activation. Every affected contract stays
inactive until fresh architecture, correctness, security, fixture, controller,
and cloud-profile review passes.

This B/A/B set does not close the draft's ordinary correctness work. Outer-entry
length semantics, component/state wire grammar, Core memory ceilings, the total
UEFI attribute table, complete semantic fixtures, validation precedence, W^X
transitions, static-check integration, and readiness documentation all remain
blocking and must be repaired and freshly reviewed after these ADRs are
accepted.

The owner's exact conditional statement was: `If it's safe, I approve.`

The statement is mapped only to the independently recommended B/A/B set above.
It grants no broader authority and becomes effective only if the exact decision
record is merged after all required independent reviews and checks report no
blocking finding.
