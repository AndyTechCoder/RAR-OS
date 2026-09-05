# Fast-Track Alpha Milestone 4: Modern Architecture

Status: owner-directed; contract design in progress, not implemented or complete.
Direction: 2026-09-05 UTC, "Perfect. So then let's continue with the next, milestone 4".

## Baseline and purpose

Start from published v0.3.0-usable-alpha at
06ecaaad61ab40f4c90ee73df85ee3493c89ccc1. Preserve that release and all earlier
Foundation/Platform proofs. The new milestone turns the working graphical
prototype into an experimental updatable, recoverable system. It is not the
entire production OS, a new hardware tier, or the Expansion milestone.

ADR0032 governs efficient delivery. ADR0034 records the new implementation
boundary and must be independently reviewed before its proposed authority is
used. Existing Desktop-v0 remains reproducible; its volatile data is not silently
converted into a persistent format.

## Completion means actual behavior

1. A RAR-owned verifier checks canonical bounded layer metadata, publisher key,
   Ed25519 signature, content hashes, interface/profile compatibility, resource
   budget and rollback policy before candidate bytes become executable.
   Unknown key/algorithm, malformed metadata, altered payload or stale generation
   is rejected without affecting the active component or stored data.
2. A real native component image is installed in an inactive system slot,
   health-checked and switched through a bounded lifecycle transaction.
   Use Settings as the first replaceable graphical component. Shell, compositor,
   Files and Terminal stay alive; a visible new Settings behavior proves that
   different executable code, not a host-rendered page or palette fixture, runs.
3. Failed candidate startup/health, crash or incomplete update keeps or restores
   the verified prior component and its compatible state. Old endpoint handles,
   queued messages and lifecycle tokens cannot gain the replacement's authority.
4. Synthetic files written through Terminal are read through Files after a
   completely new guest boot from the retained test Data image. Persistence
   requires guest block I/O and durable commit ordering, not retained RAM or
   controller-injected reconstruction of file contents.
5. Root/recovery inputs, writable system slots and Data occupy distinct authority
   and storage domains. Routine system update/repair cannot write Data. Recovery
   treats Data read-only, repairs only identified damaged system units and proves
   exact preserved Data-image identity across that operation.
6. Recovery starts from independently verifiable immutable laboratory material
   when a system slot is corrupt. Interrupted repair remains restartable.
   Root/kernel replacement may require a controlled restart; no universal
   restart-free kernel update or physically immutable software guarantee.
7. Persisted transaction records use explicit bounds, checksums/authentication,
   monotonically ordered generations and copy-on-write publication. Inject failure
   at every write/flush/commit boundary; reboot must select a complete old or new
   state, never a mixture. Ambiguous or corrupt data is not autoformatted.
8. Data encryption uses an established authenticated-encryption construction
   with a reviewed key/nonce lifecycle and explicit experimental schema.
   Laboratory keys and all test data are public fixtures, never production
   credentials or real user secrets. No production confidentiality claim.
9. RAR target crypto passes official vectors, negative/malformed cases, bounded
   deterministic fuzzing and interoperability with two independent pinned,
   host-only references in a separate cloud role. No reference library is linked
   into target images. Crypto and unsafe/device code receive focused review.
10. Two independent builds reproduce the target images. Actual cloud boot,
    cross-reboot persistence, live replacement, tamper rejection, rollback,
    interrupted commit and recovery proofs pass. Retain guest serial/screenshots,
    exact disk hashes, injected fault locations, reference/tool identities and
    source/controller revisions. Retained Desktop/Platform/Foundation regressions
    still pass. Publish v0.4.0-modern-alpha only after reviewed final-head and
    exact-main evidence; model-only tests cannot satisfy runtime requirements.

## Delivery and ownership

One main writer. Read-only independent architecture/correctness/security reviews
cover public/persistent contracts, crypto, device authority and lifecycle changes.
Use a small number of coherent feature changes, quick focused checks during work
and a consolidated remediation pass. No authorization-only PR chains, automatic
retry loops, self-approving reviews or evidence fabricated from source markers.

Expected implementation paths: core/modern/, services/modern/, nucleus/modern/,
target crypto modules, narrowly reviewed Desktop/Foundation integration,
tools/rar-lab/modern/, matching workflow and focused tests/docs. Paths and binary
formats become concrete in the implementation ADR/interface specification before
their code is accepted. Preserve historical experimental contracts unchanged.

## Host and cloud safety

All repository mutations are GitHub API operations. No Mac/SSD file creation,
edits, moves, deletion, builds, packaging, mounting or target/VM execution.
No local permission installation or migration is needed.

The existing Desktop profile does not authorize persistent media. A new reviewed
Modern profile may keep bounded synthetic System/Data regular-file images only
inside one disposable cloud test session, across guest process restarts. No
volume from the owner's machine is attached. Input boot/recovery artifacts stay
read-only; writable test images never share an inode or writable backing chain
with source, boot, recovery or one another. No external persistence service,
network listener, device passthrough, raw host disk, credentials or real user data.

The trusted-main controller chooses exact paths, sizes, commands, device model,
fault points, time/output limits and cleanup of its own disposable cloud session.
Proposal code supplies no host paths or launch arguments. Device/profile details
require the same focused review as the controller; do not run a provisional
launcher to discover whether it is safe.

## Limits and progress

This document records acceptance targets, not completion. Current missing work:
the Modern contracts/controller, cryptographic implementation/reference closure,
persistent block/storage path, component replacement, update/recovery wiring and
runtime evidence. Networking, SDK, additional hardware profiles, AI/agents,
production identity/hardware-backed secrets and an external cryptographic audit
remain later work. A failure to finish any requirement is reported explicitly,
not relabeled as successful Milestone 4.
