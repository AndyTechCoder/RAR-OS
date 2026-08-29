# ADR 0026: Alpha Platform Payload and State Sources

Status: Proposed — owner decision required
Decision: Undecided

## Context

The current private Alpha boot image contains only Root, Recovery, and Nucleus.
Root accepts only the Recovery and Nucleus payload paths, and the final R0-002
entry grants device authority only for APIC and serial. Its boot-source record
is descriptive and grants no block-device access.

Later approved milestones nevertheless require independently restartable
components, native apps, a storage service, separate system/preserved-data
regions, recovery, and signed component replacement. The architecture forbids
putting GUI, apps, package management, or general services inside Nucleus.
There is currently no reviewed path that delivers those executable/data bytes
or grants the minimum state authority.

Patching Root/Recovery entry separately during C, D, E, and F would repeatedly
rewrite the trusted boot boundary after checkpoint A. Giving Core ambient access
to the boot volume would invent device authority. Linking everything into
Nucleus would erase the component boundary. This decision must therefore be
resolved before the final Alpha boot contract becomes ready and before A target
implementation starts.

## Decision drivers

- Preserve distinct Root, Recovery, Nucleus, Core, service, and app boundaries.
- Keep runtime block-device authority out of Alpha unless genuinely required.
- Make component and state bytes deterministic, bounded, inspectable, and
  independently replaceable.
- Give D an honest on-image source and separate system/preserved-data regions
  without claiming a production filesystem or shutdown persistence.
- Avoid changing the A boot handoff for each later milestone.
- Keep R0-002 unchanged and all new framing private, experimental, and removable.
- Keep target-linked third-party dependencies at zero.

## Considered options

### A. Let RAR Core open the Alpha boot volume directly

Add runtime block-device access and let Core discover component/app/state files.
This requires a controller, bus, DMA, and storage authority path not present in
R0-002; it also couples runtime behavior to the temporary FAT boot volume. It is
not acceptable without a much larger hardware/storage decision.

### B. Link components, apps, and state fixtures into Nucleus

Compile all later milestone behavior into one Nucleus image. This is initially
small, but independently restartable components and replaceable signed layers
would be labels inside one trusted binary. It violates the approved architecture
and cannot satisfy isolation or replacement honestly.

### C. Stage bounded private Alpha sources before Nucleus entry

Extend the reviewed private Alpha image with four deterministic bounded inputs:

- an immutable Core-bootstrap image containing only the first Core loader and
  its fixed Alpha identity;
- an immutable component bundle containing independently framed component/app
  payloads and identities;
- an immutable initial system-state source image; and
- an immutable initial preserved-data source image.

The exact paths, bytes, ceilings, alignment, checksums, and validation order are
fixed in a reviewed experimental contract before code. Root reads only those
fixed inputs, stages them in dedicated non-overlapping slots, and records exact
digests. Recovery validates only the fixed outer source-set record: record
count, order, purpose, byte range, digest, transfer rights, and non-overlap. It
does not parse the inner component-bundle or state formats. Recovery then
produces one private `AlphaPlatformEntryV0` around the unchanged `BootEntryV1`.

The envelope grants no raw storage device. It describes exact bounded memory
sources and rights. The Nucleus Alpha adapter validates one minimal,
version-fixed `AlphaCoreBootstrapV0`, maps it into a fresh address space, and
starts one initial thread with only the component-source read capability and
the minimum Nucleus IPC/capability mechanisms needed to establish Core. It does
not resolve dependencies, select ordinary components, broker policy, or manage
lifecycle. Those remain Core responsibilities. Component-bundle bytes remain
immutable and non-executable until that Core loader validates an inner entry.
Recovery remains the sole envelope producer and revokes its staging writes
before Nucleus entry.

The Core loader validates the component bundle, creates isolated components,
and grants only declared capabilities. The state and preserved-data services,
not Root or Recovery, validate their respective inner source formats before
making non-aliased mutable runtime copies in separately owned regions. The
immutable source images remain retained, read-only, and digest-verifiable. A
system reconstruction receives read authority to the immutable system source
and write authority only to a newly allocated system destination; it receives
no preserved-region write capability. The preserved-data service is the sole
normal writer of its runtime region, which becomes read-only to recovery during
reconstruction. Alpha does not promise writeback to the FAT volume, persistence
after VM shutdown, or a production filesystem.

ADR 0022's optional peripheral grant, if accepted, uses a separately framed
record in the same private envelope; it does not reinterpret component or state
sources. This option is proposed.

### D. Implement a production storage and package stack before Alpha

Define stable block drivers, discovery, filesystem, package, persistence, and
update formats now. This would solve the general problem but expands the
time-boxed Alpha into later releases and is not proposed.

## Proposed direction

Select Alternative C. It establishes one replaceable boot-to-Core delivery
boundary early, while keeping the temporary boot volume and all later payloads
outside Nucleus internals. The exact envelope and four source formats require
fresh architecture, correctness, and security review before implementation.

Acceptance authorizes only the matching experimental specification and source
work. It does not authorize local target compilation/execution, cloud
provisioning, credentials, VM launch, merge, production persistence, or a ready
state.

## Ownership and sequencing if accepted

- Architecture/specification work owns the envelope and source contracts before
  A implementation.
- Milestone A owns Root staging, Recovery outer-record production, the narrowly
  mechanical Nucleus Core-bootstrap mapper, deterministic image tooling, and
  empty/minimal canonical source fixtures within its already assigned paths.
- Milestone C gains narrowly scoped ownership of `spec/alpha/component/` and
  `core/loader/` for the reviewed component bundle and loader; it does not reopen
  A boot code.
- Milestone D uses its existing state/recovery/storage paths for system and
  preserved regions; it does not reopen A boot code or gain a raw device.
- E and F populate/replace reviewed bundle entries through their owned
  service/app/update paths and the Core loader contract, not Nucleus internals.
- Any later need to edit A-owned boot/adapter files requires explicit temporary
  ownership, full cumulative A-through-current revalidation, and architecture,
  correctness, and security review.

The authoritative vertical task packet must record these narrow ownership
additions in the same reviewed acceptance change; this ADR alone does not edit
the packet.

## Security and data impact

Every source has a fixed ceiling, checked range, digest, purpose, owner, and
rights. Ranges are page-aligned and pairwise disjoint. The outer parser never
interprets inner formats. The Nucleus bootstrap parser accepts only the single
fixed Core-bootstrap format; the Core loader exclusively parses bundle entries;
and each state service exclusively parses its own versioned state source.
Component bytes never become executable before canonical parsing, identity
verification, bounds, rights, and dependency checks succeed. Immutable sources
and mutable destinations cannot alias. State corruption cannot expand writes
into the preserved region. Descriptive metadata grants no device or DMA access.

The preserved fixture is public test data, not owner data. No owner path,
credential, host file, shared folder, or external storage is exposed to the
guest. The Mac remains source-only.

## Validation if accepted

- Two clean builds produce byte-identical unsigned images and source bytes.
- A RAR-owned packer and independent read-only inspector agree on every source
  byte, range, digest, entry, padding, and computed field.
- Missing, duplicate, reordered, oversized, overlapping, aliased, writable-
  executable, wrong-purpose, wrong-owner, and digest-mismatched sources reject
  before Nucleus entry or component execution.
- Core-bootstrap tests cover wrong identity, entry, segment layout, rights,
  digest, executable mapping order, initial capability set, and attempts to make
  the Nucleus adapter perform component selection or lifecycle policy.
- Bundle tests cover empty/minimal, multiple components, unknown critical entry,
  dependency cycle, excess authority, malformed executable, and stale identity.
- D proves initial state bytes came from retained immutable sources; source and
  destination regions are disjoint; reconstruction writes only a fresh system
  destination; the corrupt system writer cannot write preserved data; and the
  exact immutable and runtime preserved-fixture hashes remain unchanged.
- Tests prove no runtime boot-volume, block-device, bus, DMA, host filesystem,
  or firmware capability reaches Core, services, or apps.
- Cumulative A–G probes retain exact envelope, bundle, state, artifact, and
  source identities.

## Compatibility and replacement

`AlphaPlatformEntryV0`, the component bundle, and state images are private Alpha
formats. They never become R0, RID, package, filesystem, or update compatibility
promises. Later production boot discovery, component packaging, System Store,
Data Vault, drivers, and A/B recovery replace them through new versioned
contracts and migrations. Old Alpha bytes are rejected, never reinterpreted.
