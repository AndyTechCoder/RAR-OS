# Sprint Alpha Completion Evidence Map

Status: Non-authoritative preparation — no completion evidence exists yet

This map connects the eight owner-approved completion items in
`../sprint-alpha.md` to the evidence that Milestone G must retain. It defines no
target marker, wire format, controller schema, ABI, device authority, or
readiness state. The authoritative vertical packet, accepted ADRs, reviewed
active evidence protocol, and trusted controller win on any disagreement.

At the current source-only checkpoint, none of these items is implemented or
proven. A guest marker is one correlated observation, never sufficient proof by
itself.

## One retained demonstration boundary

The final evidence set must bind one exact source SHA and G checkpoint tag to:

- the reviewed active acceptance-plan schema and digest;
- the trusted default-branch controller/workflow SHA and run/attempt identity;
- every pinned runner, OCI, compiler, linker, firmware, QEMU, machine-profile,
  helper, reference, and input identity;
- two independent clean build receipts and their byte-identical unsigned target
  artifact hash;
- the single frozen artifact actually launched, its boot/session identity,
  complete ordered input transcript, serial offsets, captures, fault records,
  final exit status, and retained controller verdict; and
- the exact documentation and limitation set shipped at that source SHA.

Missing, stale, duplicated, reordered, mutable, cross-run, or zero-step evidence
fails. The final `integration:completion-contract-pass` marker is valid only
after the trusted verifier proves every item below from the same bound evidence
set. It cannot waive or synthesize another result.

## Item-by-item proof

### 1. Reproducible RAR-owned build

Required proof:

- two fresh network-disabled build receipts use the same source and locked
  target-affecting inputs and produce identical unsigned artifact bytes;
- a complete target dependency report shows no linked target code except
  RAR-owned source, compiler-provided `core`, approved freestanding built-ins,
  and any explicitly approved Dependency Exception Record; and
- the independent image inspector agrees with the RAR-owned packer on every
  byte-producing field and embedded payload identity.

Rejected substitutes: one build, matching filenames, a source-tree hash alone,
or a host-created/unbooted image. Primary milestone: A; retained closure: G.

### 2. Root → Recovery → Nucleus boot

Required proof:

- the frozen artifact launched only through the approved cloud profile;
- ordered Root, Recovery, and Nucleus entry observations correlate to the same
  boot/session and structured trace;
- exact R0-002 source/validation identities and malformed-input no-authority
  evidence are present; and
- the launch receipt proves disabled networking, passthrough, host sharing,
  credentials, raw devices, and elevated execution.

Rejected substitutes: parser-only tests, generated logs, screenshots, firmware
entry without Nucleus entry, or any local Mac/SSD execution. Primary milestone:
A; retained closure: G.

### 3. Interactive framebuffer GUI and input

Required proof:

- guest-rendered framebuffer captures are bound to exact post-input markers and
  serial offsets for keyboard and pointer actions;
- the profile/peripheral-grant identity and guest graphics/input service
  identities match the reviewed contract; and
- negative authority evidence proves apps cannot access the raw framebuffer or
  input transport and graphics/input cannot cross-access each other's device
  capability.

Rejected substitutes: a host-rendered page, prerecorded animation, marker-only
success, QMP input without guest consumption, or a capture without trace/input
correlation. Primary milestone: E; continuity replay: F–G.

### 4. Launcher, terminal, settings, and two native apps

Required proof:

- six ordered user actions produce distinct correlated captures for launcher,
  terminal, settings, demo 1, and demo 2, including the pointer observation;
- each surface is backed by the expected RAR component/bundle identity and
  receives only app-facing surface/input handles; and
- the retained limitations state that Alpha proves opening these minimal native
  experiences, not a complete terminal environment, design system, or broad app
  ecosystem.

Rejected substitutes: one shell painting five labels, duplicated captures,
host applications, or uncorrelated markers. Primary milestone: E; retained
closure: G.

### 5. Capability IPC, crash containment, and restart

Required proof:

- all forged, stale, over-rights, oversized, full-queue, timeout, cancellation,
  closed-peer, and peer-crash cases produce the reviewed deterministic outcomes;
- component identity, container/address-space identity, capability set, crash,
  revocation, replacement identity, and restart order are retained; and
- cumulative E–G evidence shows the real GUI plus another component remain
  responsive after the same correlated crash/restart sequence.

Rejected substitutes: restarting the whole OS, reusing stale authority, a
synthetic pre-GUI marker, or liveness from a different run. Primary milestone:
C; GUI continuity becomes provable at E under the reviewed active protocol.

### 6. System/preserved-data separation and recovery

Required proof:

- immutable source and non-aliased mutable runtime-region identities, bounds,
  owners, and capabilities are retained;
- the exact three-byte Alpha fixture `abc` has the fixed pre/post SHA-256 while
  deliberate system corruption, isolation, Recovery activation, reconstruction
  write set, and atomic outcome are correlated; and
- negative evidence proves the corrupt system writer and reconstruction path
  cannot write preserved data or reinterpret an unverified region as success.

Rejected substitutes: a production persistence/filesystem claim, a different
fixture, hashing without retained bytes/regions, or rebuilding preserved data.
Primary milestone: D; retained closure: G.

### 7. Signed layer activation, tamper rejection, and rollback

Required proof:

- exact manifest, signed preimage, public Alpha key, payload, generation,
  signature, component, and prior/candidate state identities are retained;
- official vectors plus isolated reference comparisons bind the RAR-owned
  cryptographic result, and the one-byte tampered candidate rejects before any
  executable mapping or authority grant;
- activation changes the component identity without re-entering the boot chain,
  failed health triggers rollback to the recorded prior identity/state, and an
  unaffected component remains responsive in the same boot/session.

Rejected substitutes: host-only signature verification, accepting then
quarantining tampered code, whole-OS reboot/rebuild, or rollback from another
run. Primary milestone: F; retained closure: G.

### 8. Exact identities and usable documentation

Required proof:

- source/tag, tools, firmware, controller, profile, artifact, plan, evidence,
  and review identities are complete and mutually consistent;
- build, boot, user operation, debugging, recovery, update/rollback, and
  app/extension documentation is present at the retained source SHA; and
- limitations distinguish demonstrated Alpha behavior from production
  security, physical-device support, durable storage, networking, adaptive
  tiers, broad SDK/apps, Pal, self-hosting, and Releases 0–6 completion.

Rejected substitutes: a README alone, mutable latest-version links, missing
commands/identities, or claiming V1/production completion. Primary milestone:
G.

## Closure rule

Milestone G may report Alpha 0.1 complete only when the trusted verifier maps
all eight items to present, exact, same-run evidence; every selected observation
in the reviewed active protocol passes in order; all required reviews/checks are
green; the PR is conflict-free; and the exact merge is verified on GitHub.
Anything less remains incomplete, regardless of the final guest marker.
