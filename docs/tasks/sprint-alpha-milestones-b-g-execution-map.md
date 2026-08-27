# Sprint Alpha Milestones B–G Execution and Evidence Map

Status: Non-authoritative preparation — implementation remains sequential and gated

This map organizes requirements already approved by
`sprint-alpha-vertical.md`, `../sprint-alpha.md`, accepted ADRs, and
`../../spec/alpha/evidence/acceptance-v1.plan`. It defines no wire format,
syscall, ABI, storage promise, device authority, package format, or stable API.
The authoritative sources win on any disagreement.

Milestones remain strictly ordered. Each writer starts only after the previous
checkpoint is green, reviewed, pushed, immutably tagged, and accepted by the
trusted cloud controller. Every probe is cumulative and must replay all earlier
acceptance rows.

## Rules shared by B–G

- One active writer owns only the current milestone paths plus its exact
  `SPRINT_STATUS.md` checkpoint update.
- Target-linked dependency count stays zero unless an approved Dependency
  Exception Record says otherwise.
- New Alpha contracts are deterministic, bounded, experimental version 0,
  replaceable, negatively tested, and reviewed before implementation relies on
  them. They do not become stable RAR promises.
- Apps and services receive explicit attenuated capabilities. No ambient
  syscall, administrator mode, raw device access, executable pointer, raw Rust
  ABI, or undocumented cross-subsystem call is allowed.
- Unsafe Rust and assembly carry adjacent invariants and focused tests, then
  independent security review.
- All target compilation, image creation, firmware loading, VM execution,
  fault injection, performance measurement, and acceptance happen only through
  the approved cloud controller. The Mac and SSD hold source only.
- Missing, duplicated, reordered, stale, extra, or unbound evidence is failure.
  A host mock, generated log, screenshot-only result, or skipped check cannot
  replace guest behavior.
- A contract, trust-boundary, dependency, persistent-data, tier, native-app, or
  release-commitment change stops the writer and returns to ADR review.

## B — Nucleus memory and execution

Start only from accepted checkpoint A. Own only `nucleus/portable/`,
`nucleus/runtime/`, `tests/sprint-alpha/nucleus/`,
`docs/sprint-alpha/nucleus/`, and the scoped status update.

Ordered implementation:

1. Page-frame accounting with checked ranges, deterministic exhaustion, no
   double allocation/free, and reserved/owned-frame exclusion.
2. Page-table construction and replacement with canonical-address, alignment,
   rights, ownership, alias, and W^X enforcement.
3. Separate address spaces with no implicit shared mapping and contained access
   faults.
4. Exception entry/return with bounded diagnostic state and no authority gain.
5. The already-authorized timer source, interrupt handling, and monotonic tick
   accounting without silently calibrating a different platform.
6. Minimal thread state, guarded stacks, lifecycle, and deterministic scheduler
   sufficient for later isolated components.

Required exact observations: 7 rows—allocator pass; invalid mapping contained;
cross-address-space access contained; exception contained; timer pass; threads
pass; scheduler pass with capture. Exhaustion and malformed state must stop
before mapping, scheduling, or authority effects.

Checkpoint evidence adds allocator/mapping fault results, exception/timer
traces, scheduling order, resource ceilings, unsafe invariants, and exact B
artifact identities to the cumulative A evidence.

## C — Capabilities, IPC, and component isolation

Start only from accepted checkpoint B. Own only `spec/alpha/capability/`,
`spec/alpha/ipc/`, `nucleus/capability/`, `nucleus/ipc/`, `core/registry/`,
`tests/sprint-alpha/isolation/`, `docs/sprint-alpha/isolation/`, and the scoped
status update.

Contract before code:

- opaque handles with non-increasing rights and deterministic stale-handle
  rejection without preselecting the internal lifetime mechanism;
- bounded messages and queues;
- timeout, cancellation, close, and peer-crash outcomes;
- component identity/lifecycle and restart notification needed only for Alpha.

The reviewed contract must fix bounds, validation order, error outcomes,
ownership transfer, cancellation races, queue teardown, and replacement notes
without introducing a general-purpose stable syscall ABI.

Implementation then establishes per-component handle tables, rights checks at
every operation, bounded copy/queue paths, deterministic cancellation and close,
and a registry that can restart one noncritical component without transferring
its stale authority to the replacement.

Required exact observations: 12 rows—forged, stale, and over-rights handles;
oversized message; full queue; timeout; cancellation; closed peer; peer-crash
notification; restart complete; GUI continuity marker; peer responsive with
capture. The fixed `component:gui-responsive` row currently conflicts with the
milestone order and ownership: A–C define no GUI, while graphics paths begin at
E. This map does not invent a pre-E presentation component or permit a synthetic
marker. Before C starts, an architecture-governed change must either authorize
and own a real earlier continuity witness or correct the acceptance row/order.

Checkpoint evidence adds contract fixtures, rights/queue exhaustion, race and
fault outcomes, restart identity, unaffected-component liveness, and exact C
artifact identities to cumulative A–B evidence.

## D — System/data separation and recovery

Start only from accepted checkpoint C. Own only `spec/alpha/state/`,
`spec/alpha/recovery/`, `core/state/`, `core/recovery/`, `services/storage/`,
`tests/sprint-alpha/recovery/`, `docs/sprint-alpha/recovery/`, and the scoped
status update.

Contract before code:

- deterministic experimental v0 system and preserved-data region framing;
- bounded records, checksums/identities, validation precedence, and corruption
  outcomes;
- recovery trigger, reconstruction scope, and atomic completion/failure state;
- an explicit replacement path that makes no production filesystem or general
  durability claim.

Implementation separates writable system state from preserved test data,
verifies both before use, isolates deliberate system corruption, activates the
minimal Recovery path, reconstructs only authorized system state, and never
rewrites the intact preserved-data fixture. The approved promise is limited to
the exact three-byte `abc` fixture and its fixed SHA-256 as documented by the
acceptance-plan README.

Required exact observations: 7 rows—regions distinct; fixed pre-hash; system
corruption isolated; Recovery activated; reconstruction complete; identical
post-hash; data preserved with capture. Recovery failure must not convert an
unverified region into success or broaden writes into preserved data.

Checkpoint evidence adds pre/post bytes and hashes, corruption location,
reconstruction write set, recovery trace, failure-injection outcomes, and exact
D artifact identities to cumulative A–C evidence.

## E — Interactive experience

Start only from accepted checkpoint D, accepted ADR 0022, and a separately
reviewed `ready` Alpha peripheral-grant contract. Own only
`spec/alpha/surface/`, `spec/alpha/input/`, `services/graphics/`,
`services/input/`, `apps/shell/`, `apps/terminal/`, `apps/settings/`,
`apps/demo/`, `tests/sprint-alpha/gui/`, `docs/sprint-alpha/gui/`, and the
scoped status update.

Contract before code:

- exact private Alpha peripheral envelope/profile binding and attenuated
  framebuffer/input authority;
- bounded surface buffers, geometry, format, damage/presentation, ownership,
  and failure behavior;
- bounded normalized keyboard/pointer events and routing;
- app-facing surface/input handles that never expose raw device authority.

Implementation keeps raw platform authority in the minimal adapter, grants the
graphics service only framebuffer authority, grants the input service only a
bounded event endpoint, and gives apps only surface/input handles. The shell
provides launcher and window/surface coordination; terminal and settings are
real native Alpha components; two demo apps exercise independent components.
Provisional accessible primitives are required, while the final design system
remains deferred.

Required exact observations: 6 captured rows—launcher, pointer accepted,
terminal, settings, demo 1, and demo 2. Captures must correlate with ordered
guest trace markers after the scripted input; host-rendered UI is invalid.

Checkpoint evidence adds input transcript, device/profile identities,
framebuffer captures, trace correlation, authority-isolation negatives,
resource bounds, and exact E artifact identities to cumulative A–D evidence.

## F — Signed layers, replacement, and rollback

Start only from accepted checkpoint E and a genuinely ready isolated reference
role/inventory. Own only `spec/alpha/layer/`, `spec/alpha/signing/`,
`spec/alpha/update/`, `core/crypto/`, `core/package/`, `core/update/`,
`tests/sprint-alpha/update/`, `docs/sprint-alpha/update/`, and the scoped status
update.

Contract before code:

- the exact ADR 0019 experimental manifest, signed preimage, key identity,
  payload identity, generation, resource, and health fields;
- deterministic encoding, length bounds, validation precedence, activation
  state machine, health timeout, rollback record, and replacement notes;
- explicit separation between laboratory signing identity and any future
  production trust root.

Implementation provides RAR-owned SHA-256, SHA-512, and RFC 8032 pure Ed25519;
canonical manifest verification; generation/downgrade checks; staged activation;
component replacement through Milestone C authority; bounded health evaluation;
and rollback that leaves unaffected components running. The public fixture
private key is test data and never a production secret or claim.

Required exact observations: 7 rows—valid signature accepted; valid layer
activated; one-byte tamper rejected before execution; component replaced
without reboot; health check failed; rollback complete; unaffected component
responsive with capture.

Checkpoint evidence adds official hash/Ed25519 vectors, malformed/invalid
cases, two digest-pinned isolated host-reference comparisons, bounded retained
fuzz seeds/results, constant-time and unsafe reviews, signed bytes, activation
and rollback traces, and exact F artifact identities to cumulative A–E evidence.

## G — One retained end-to-end Alpha

Start only from accepted checkpoint F. Own only `spec/alpha/integration/`,
`tests/sprint-alpha/end-to-end/`, `docs/sprint-alpha/`,
`evidence/sprint-alpha/`, `SPRINT_STATUS.md`, and `README.md`.

G adds no substitute implementation. It binds one clean exact-head run to all
eight completion items and all 45 ordered acceptance rows (A:5, B:7, C:12,
D:7, E:6, F:7, G:1). Two clean builds from the same locked inputs must produce
byte-identical unsigned target artifacts before the retained demonstration.

The final captured row is `integration:completion-contract-pass`; it is valid
only after every earlier row, required capture, fault result, identity, timeout,
resource bound, and review is present and consistent.

Closure documentation covers user operation, architecture, build, boot,
debugging, recovery, update/rollback, app/extension development, exact tooling,
known limitations, and the difference between demonstrated Alpha behavior and
unfinished production roadmap promises. The integration PR merges only when
required checks and independent reviews are green, conflicts are absent, and
the exact merge is verified on GitHub.

## Compact readiness chain

`A accepted → B nucleus → C isolation → D recovery → E GUI → F signed layers → G retained proof`

No later milestone may backfill an earlier missing result, and no calendar
deadline may collapse this chain into mocks or unreviewed interfaces.
