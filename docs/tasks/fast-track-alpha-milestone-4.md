# Fast-Track Alpha Milestone 4: Modern Architecture

Status: M4.1 COMPLETE at accepted development snapshot
fd1982665b81ba2e672e0044165d68e09daa47f0 (2026-09-10).
M4.2, M4.3 and full Milestone 4 remain incomplete. This is not main-release
acceptance or production certification. See docs/evidence/m4-runtime-progress.md
for exact runtime evidence, source identities, review and integration checks.
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

## Owner-approved three-section delivery

The owner approved this organization on 2026-09-08. It replaces the delivery
sequence, not the ten completion requirements above, ADR0034 review boundaries,
or any host/data safety constraint. M4.1 is complete at the accepted development
snapshot above; M4.2 and M4.3 remain incomplete.

### M4.1 — Persistent files

Status: COMPLETE at fd1982665b81ba2e672e0044165d68e09daa47f0.
Independent consolidated review found no remaining section acceptance gap.
The completion basis is actual behavior and passing integrated-source checks,
not documentation alone. Unfinished M4.2 work in draft PR158 is not approved
for merge by this disposition.

Deliver the actual Terminal -> storage service -> kernel-mediated Data device ->
durable DataVault -> fresh-VM Files path. Reuse the working Desktop UI without
changing its historical volatile profile. Finish the concrete bounded Modern
device/syscall contract, separately enforce System/Data authority, wire the
existing PIO and vault implementation, and mount rather than recreate contents.

Acceptance: a controller-generated unpredictable value typed only during boot1
survives complete QEMU destruction and fresh firmware state; boot2 Files and a
separate frozen-image oracle agree. Include read/write denial for unauthorized
roles, exact capacity checks, sticky failure/no autoformat, and representative
controller-owned interrupted-write tests. Finish the reviewed crypto/reference
and cloud-profile prerequisites before accepting encrypted runtime evidence;
do not postpone an unsafe prerequisite merely because final release is later.
This section primarily covers requirements4,7,8,9 and the storage isolation in5.

### M4.2 — Signed live updates

Connect canonical signed metadata and inactive System payload storage to the
actual component loader and lifecycle mechanisms. Replace Settings executable
code while shell, compositor, Files and Terminal continue. Trial health has no
production authority; atomic publication and incarnation/queue revocation must
be enforced by the running kernel, not only by the lifecycle model.

Acceptance: visibly different Settings behavior; tampered, unknown-key,
incompatible and stale candidates rejected; failed startup and post-cutover
failure restore the verified prior component without granting stale handles
authority. Verify preserved persistent Data. Covers requirements1,2,3, the update
portion of5, and the related System transaction cases in7.

### M4.3 — Recovery and release

Boot independently verifiable immutable laboratory recovery, identify and repair
only damaged System units, and preserve the exact Data-image hash. Demonstrate
restartable interrupted repair and complete the full controller-owned fault
matrix across persistence, update and recovery. Run final crypto comparisons,
two independent target builds and retained Desktop/Platform/Foundation
regressions; retain actual serial output, screenshots, image/source/tool hashes
and causal test evidence. Complete focused independent reviews/remediation,
final-head and exact-main gates, then publish v0.4.0-modern-alpha.

Covers the recovery portion of5, requirement6, full7/9 regression coverage and10.
No section is complete until its actual cloud behavior is demonstrated; passing
source/model tests, a compiler image, documentation or a prompt count is not the
completion measure.

### Execution discipline

One main writer; one integrated implementation and outcome demonstration per
section, with quick focused checks and a consolidated independent review near
completion. Review security-critical contracts/authority before use and fixes
before meaningful merges. A section can use coherent dependency changes where
trusted-main execution requires them, but not authorization-only PR chains.

Maintain docs/evidence/m4-runtime-progress.md with what works, exact evidence,
remaining integration and the next concrete action. Shared compiler/reference
work belongs to M4.1's required crypto validation and final M4.3 regression; it
is not a fourth open-ended milestone. Preserve already-reviewed work, avoid
duplicate live runs, diagnose a terminal failed/cancelled run before one bounded
retry, and never weaken checks simply to report progress.

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

M4.1's acceptance targets are complete at the recorded development snapshot.
Remaining Milestone 4 work is actual signed component replacement, update/fallback
integration, immutable System-only recovery and the final whole-M4 runtime,
regression/reproducibility/review/release gates. Networking, SDK, additional hardware profiles, AI/agents,
production identity/hardware-backed secrets and an external cryptographic audit
remain later work. A failure to finish any requirement is reported explicitly,
not relabeled as successful Milestone 4.

## Required causal persistence and fault evidence

For the persistence scenario, kill the entire QEMU process and launch a fresh
process against the same launcher-private synthetic images. Fresh OVMF variable
bytes are created for each boot. No RAM snapshots, savevm/loadvm, retained NVRAM,
TPM state, writable boot overlays or alternate cross-boot channels may carry the
test file. No writable runner/source/host bind is allowed for System/Data.

Generate an unpredictable challenge outside proposal authority, prove it absent
from initial disk state, and type it only in boot 1. Freeze exact disk bytes
after the cut. Boot 2 receives neither that challenge nor a command that can
reconstruct it. A distinct trusted read-only oracle parses the frozen Data image
and verifies the same committed value that the restarted guest displays.
Retain exact process/command/input/fault evidence and whole-image hashes.

Fault cuts must be controller-owned and observed at the block boundary; guest
success markers do not schedule or prove durable writes. A reviewed bounded
virtual backend defines short/error/torn/reordered writes and flush ordering.
Corruption mutation is a separate test, not a substitute for interruption.
The claim is virtual-device crash consistency, not physical power-loss safety.

This Alpha demonstrates interrupted/failed-update recovery and rollback
generation enforcement relative to intact local committed metadata. It does
NOT detect wholesale rollback of all System/Data images or claim a persistent
hardware/virtual monotonic trust anchor. That stronger claim needs a separately
reviewed non-co-rollback Vault and remains future work; a counter on the same
rollbackable disk must never be described as such an anchor.
