# Sprint Alpha Boot and Platform Contract Integration Task Packet

Status: Non-authoritative preparation — ADRs 0027–0029 owner decisions required

This packet prepares the bounded path from proposed ADR 0027 Alternative B,
ADR 0028 Alternative A, and ADR 0029 Alternative B to reviewed experimental
Alpha contracts and, later, Milestone A source. It is not an ADR, approval,
contract, readiness claim, build authorization, or permission to execute RAR
OS. It may merge while remaining non-authoritative.

The preserved `codex/alpha-boot-platform-contracts` worktree is historical
draft material only. No commit, file, or fragment may be merged, rebased,
cherry-picked, or copied from it. It predates the proposed choices, retains the
rejected Core-held ticket model, and does not contain complete reviewed
grammars or fixtures. If it is consulted read-only, every useful idea is
untrusted reference material that must be independently re-derived from the
accepted ADRs in a fresh current-main diff and receive complete review.

## Objective

After every applicable start gate passes, produce exact experimental Alpha
boot/platform contracts that:

- retire Root-owned bootstrap memory deterministically;
- close the fixed q35/AHCI DMA path inside the guest before parsing payloads;
- bind executable authority to domain-separated immutable-byte identities;
- keep initial state bytes unreadable to Core through Nucleus-held,
  identity-bound state slots; and
- preserve the accepted R0-002 boundary and all host/cloud safety controls.

## D0 — decision integration prerequisite

Before contract work, a separate architecture/governance task must:

1. receive the exact informed owner sentence selecting 0027 B, 0028 A, and
   0029 B;
2. convert each proposal through the normal canonical accepted-ADR process;
3. add only the matching approval-record entries and update indexes/status; and
4. pass architecture, correctness, security, required checks, merge, and
   exact-main validation.

No contract or implementation writer owns an ADR, proposal-history file, or
`docs/approval-record.md`. Generic approval, recommendation text, this packet,
or passing CI cannot substitute for the exact owner decision. D0 authorizes
contract work only; it does not authorize target source, build, image, launch,
or execution.

## P0 — contract-writer start gate and ownership

P0 may start only after D0 is merged and exact-main validated, with no
overlapping writer, conflict, failing required check, or unresolved review
finding. The writer starts from that verified `main` and owns only:

- `spec/alpha/boot/**`;
- `spec/alpha/platform/**`;
- `tools/ci/check-alpha-boot-platform-contracts.sh`;
- `tools/ci/test-alpha-boot-platform-contract-policy.sh`;
- registration-only edits to `tools/ci/check-alpha-preimplementation-contracts.sh`,
  `tools/ci/test-alpha-preimplementation-contract-policy.sh`,
  `tools/ci/check-specs.sh`, and `tools/ci/check-sprint-static.sh`; and
- readiness references in `docs/tasks/sprint-alpha-vertical.md`,
  `docs/tasks/sprint-alpha-milestone-a-execution-map.md`, and
  `docs/sprint-alpha-dashboard.md`.

P0 does not own target source, workflows, Development Lab profiles,
controllers, acceptance plans, evidence publication, or later milestone code.
Every contract remains experimental and blocked until its own fresh reviews,
static checks, mutation tests, and exact-main validation pass. The machine
profile remains blocked until retained cloud evidence proves its exact
firmware, q35 topology, and PCI/AHCI inventory.

## P0 contract requirements

### ADR 0027 Alternative B

- Reserve the complete fixed Recovery bootstrap arena before any payload read.
- Capture the firmware-selected Root range only from the UEFI Loaded Image
  Protocol; keep Root's private stack and page-table slots distinct.
- Define the private-table/stack switch, one-way transfer, Recovery retirement,
  unmapping, TLB invalidation, zeroing, and final-map normalization order.
- Fix the exact q35 PCI-function and AHCI inventory, boot-device-to-BDF binding,
  engine-stop sequence, bounded idle waits, bus-master disable/readback, and
  rejection of missing, extra, active, or unverifiable devices.
- Re-hash all staged sources after closure and recheck the complete disabled
  vector immediately before entry. No PCI/controller authority crosses into
  Recovery, Nucleus, Core, or applications.

### ADR 0028 Alternative A

- Define versioned, length-framed, domain-separated SHA-256 preimages over
  exact immutable bytes; self-identity fields are excluded from their preimage.
- Provide a total producer, transport, verifier, expected-literal, retirement,
  and mismatch table for Root, Recovery, contracts, components, and both state
  services.
- Root retains the exact `RECOVERY.ELF` file bytes, remains the sole pre-load
  verifier and loader, and transports the bounded read-only source plus verified
  identity. Recovery performs only the post-entry secondary check and retires
  those bytes before Nucleus entry. Nucleus receives no file bytes and only
  compares the Recovery-authenticated identity with its reviewed literal.
- Bind each state-service identity to its distinct role, exact executable
  contract identity, and exact executable payload bytes.
- Require the service-first/literal-table/Nucleus/final-image build order and
  the three-way computed-versus-compiled-versus-outer comparison before mapping
  or state-slot attachment.
- Keep Nucleus limited to hashing a capability-contained byte slice; it does
  not become a component-bundle parser.

### ADR 0029 Alternative B

- Remove every Core-held readable-state or ticket design.
- Define exactly two Nucleus-held state slots and two opaque Core selectors.
- Define slot, selector, incarnation-bound redeem token, service incarnation,
  and derived-handle object kinds and rights.
- Fix the exact two positions and rights of the opaque selectors in Core's
  initial capability table.
- Fix states `unredeemed`, `bound`, `rebindable`, and `revoked` with a total
  event/state transition table and deterministic failure precedence.
- Inject a nondelegable redeem token directly into the verified service
  incarnation. Core cannot read, clone, delegate, redeem, retarget, or revoke.
- Specify one-winner concurrency, no-effect wrong/stale attempts, crash/exit
  cleanup, same-identity restart, the two terminal revocation causes, and
  mutable-region quarantine.

### Shared byte-contract closure

P0 must also correct ordinary incompleteness without changing an ADR choice:

- Outer-entry RSI uses exact `total_bytes`, validates every offset, length,
  embedded size and padding byte, and has no duplicated source-record length.
- Component-bundle grammar fixes dependency layout and kinds, flags, rights
  masks, ordering, bounds, duplicates, cycles, and payload ownership.
- State grammar fixes object kinds, name/payload tables, encoding, ordering,
  overlap rules, and identity inputs. The preserved positive fixture is exactly
  the approved bytes `abc`, never the obsolete 28-byte draft.
- Core-bootstrap ceilings cover mapped memory/BSS, virtual window, stack and
  guard, page tables, threads, capabilities, and IPC.
- UEFI normalization has one total memory-type by known-attribute decision
  table; unknown or conflicting values fail closed.
- One authoritative first-failure/no-effects precedence applies across all
  boot/platform validators.
- Executable mappings transition `RW+NX → remove write → TLB flush → RX`, with
  no writable alias; stack, data, and tables remain NX; required CPU controls
  are active before parsing.
- Complete positive fixtures cover Root, Recovery, platform, Core, component,
  initial system, initial preserved data, identities, manifests, q35, and AHCI;
  single-field mutations cover every field, enum, bit, length, overlap,
  ordering, identity, and precedence rule.
- Precedence fixtures cover every adjacent dual-invalid predicate plus all
  security- or authority-sensitive nonadjacent dual-invalid combinations, and
  assert the exact first error and an empty effect log.

## P0 validation and source-only merge gate

The contract change must prove:

- deterministic packer/inspector agreement and exact golden fixture digests;
- oversize, truncation, overlap, duplicate, cycle, unknown enum/bit, stale
  identity, alias, timeout, and precedence rejection with empty effect logs;
- ADR 0027 arena, retirement, PCI/AHCI, DMA-closure, mutation, alias, and
  determinism cases;
- ADR 0028 independent preimage vectors plus wrong domain, version, role,
  contract, length, source, literal, outer value, and build-order cases;
- ADR 0029 every state/event pair, wrong identity/role, stale selector/token,
  single concurrent winner, allocation failure at each authority-creation
  boundary, crash points, restart, terminal revocation, and proof that Core
  never receives state-readable authority; and
- clean architecture, correctness, and security review after one bounded
  remediation, green exact-head checks, conflict-free merge, and a distinct
  successful exact-main Specifications run.

No source marked `source-ready-pending-review` is acceptable. Readiness is
`blocked` or `pending-review` until the complete gate passes.

## P1 — Milestone A source, only after P0 and the cumulative source-start gate

P1 may start only after P0 is reviewed, merged, and exact-main validated **and**
every cumulative precondition that the owner-approved vertical packet and
Milestone A execution map require before creating Milestone A target files has
passed. This includes their exact SSD-profile/confinement evidence, local and
remote preflights, accepted earlier Alpha decisions, PR/controller/helper/Lab
readiness, gate-report state, required reviews, and clean verified `main`.
Requirements that inherently bind the resulting P1 source or artifact are
post-source activation gates, not circular prerequisites to authoring source.
A fresh writer then owns only:

- `Cargo.toml` and `rust-toolchain.toml`;
- `boot/**`, `recovery/**`, and `nucleus/arch/x86_64/**`;
- the minimum required `tools/sprint-alpha/**`;
- `tests/sprint-alpha/boot/**`; and
- `docs/sprint-alpha/boot/**` plus the exact milestone status references.

P1 implements source only for Milestone A: ADR 0027 and the
Root/Recovery/Nucleus portions
of ADR 0028 against the ready contracts. It may retain immutable state sources
and construct inactive slot descriptors, but it must not implement Milestone C
capability/IPC behavior or expose, inject, or redeem ADR 0029 selectors/tokens.
P1 receives architecture, correctness, and security review and merges only as
source after static/source-policy gates pass. No target build, link, image,
Development Probe, firmware load, or guest execution occurs until the
post-source activation closure below is also satisfied.

## Deferred milestone ownership

- Milestone C owns capability/IPC/component code, executable service identity
  verification, and ADR 0029 selector/token redemption and lifecycle behavior.
- Milestone D owns state parsing, mutable import, quarantine, reconstruction,
  and preserved-data recovery.

Milestone A may not pre-implement those behaviors, and C/D may not reinterpret
the merged boot/platform contracts.

## Post-source pre-build activation remains blocked until

Source-only merges are not activation. Before any Milestone A target build or
image creation, a separate reviewed pre-build activation change must prove all
of the following:

1. ADRs 0027–0029 are canonical accepted decisions, P0 and the P1 source-only
   change are reviewed, merged, and exact-main validated, and the complete
   owner-approved cumulative pre-A gate remains satisfied.
2. Gate-report v2 binds the exact accepted decisions, contract identities,
   fixtures, source revision, controller, profile, tool, firmware, trusted
   role-image, and helper identities that already exist before the target build,
   and reports ready without caller-selected authority. It must not invent,
   predict, or accept placeholders for target outputs that do not yet exist.
3. The approved Development Lab profile retains exact q35 firmware,
   PCI-function, AHCI, controller, timeout, resource, output, no-network,
   no-sharing, no-passthrough, and software-emulation evidence.
4. The trusted controller comes from reviewed `main`; the untrusted source
   branch cannot modify or select launch, verdict, or acceptance behavior.
5. Required architecture, correctness, security, static, mutation, dependency,
   reproducibility, and source-only gates pass with no conflicts or findings.
6. The exact resulting `main` Specifications run succeeds before a separately
   authorized cumulative Development Probe build is dispatched.

## Post-build launch and acceptance gate

The pre-build gate permits only the bounded controller-run cloud build/image
phase. After its required two reproducible builds, and before artifact freeze,
launch, guest execution, or acceptance, the trusted controller must:

1. derive each target artifact and complete-image identity from the actual
   controller-owned output bytes, bind both builds to the exact source and
   pre-build identities, and prove the required reproducibility relationship;
2. reject caller-supplied, predicted, placeholder, stale, or mismatched target
   identities and retain the exact build evidence;
3. transition any gate-report/contract fields needed to carry those identities
   only through a separate reviewed, merged, exact-main-validated change; and
4. pass the frozen-artifact, launch, cumulative evidence, review, and
   acceptance gates before any guest execution or completion claim.

Nothing in this packet authorizes local target compilation, linking, image
creation, firmware loading, VM/QEMU execution, raw disks, passthrough,
networking, or changes outside the approved SSD workspace.

## Stop conditions

Stop without broadening scope if work would:

- start D0 without the exact informed owner choice, P0 before D0 exact-main
  validation, P1 source before P0 exact-main validation and the authoritative
  cumulative source-start gate, any P1 build/image phase before post-source
  pre-build activation, or any freeze, launch, guest execution, or acceptance
  before the post-build gate;
- change a selected alternative, R0-002, trust boundary, persistence/data-loss
  promise, compatibility rule, dependency policy, or exact q35 topology;
- give Core readable state, token possession, redemption, retargeting, or
  revocation authority;
- create circular identities, ambiguous retained-byte retirement, incomplete
  DMA closure, or overlapping Milestone A/C/D ownership;
- transfer any commit, file, or fragment from the preserved historical
  contract worktree instead of independently re-deriving it from current main;
- weaken validation, isolation, signing, rollback, tests, or evidence to pass;
- proceed with a conflict, failing required check, missing evidence, or
  unresolved architecture/correctness/security finding; or
- compile, image, boot, or execute RAR OS locally.

No deadline, generic approval, passing CI, or preparation document overrides
these conditions.
