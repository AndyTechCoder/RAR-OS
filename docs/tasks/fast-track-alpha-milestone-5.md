# Fast-Track Alpha Milestone 5: Expansion

Status: IN PROGRESS — owner directed 2026-09-13.
Process: ADR0032. Proposed composition: ADR0037.
Baseline: main 996acacaf9ed80290ea95a4a9ac238cfe5023c33; preserve the published
v0.4.0-modern-alpha OS and proof assets unchanged.

## Product boundary

Deliver a connected, programmable Fast-Track VM Alpha on top of the actual
M1–M4 OS. This is not completion of the separate long-term Releases0–6 consumer
Alpha: Wi-Fi, general Internet/TLS, self-hosting, production identity, physical
hardware, a full app ecosystem and every architecture remain explicitly tracked
future work. Do not relabel unimplemented roadmap promises as polish.
Pal intelligence/model training and production cloud service wiring are excluded.

M5 closes only with integrated guest behavior and retained exact-source proof.
A packet parser, mocked transport, SDK header, UI screenshot or policy model
alone does not satisfy its corresponding runtime requirement.

## M5.1 — Connected virtual devices

RAR-owned bounded Ethernet/IPv4/UDP implementation and a narrowly granted
virtual NIC interface, in an isolated network service rather than application
or kernel networking policy. Two disposable cloud guests exchange a fresh
controller challenge through actual guest network I/O. Explicitly configured
peer addresses avoid implying DHCP, DNS, routing or general Internet support.

The reviewed controller joins only the two test guests within its private
network-disabled container boundary; no bridge, TAP, host forwarding, external
listener, slirp, DNS, Internet route, owner network or credentials. Concrete
NIC model, fixed launch arguments, buffers, syscall grants and peer transport
must receive independent review before activation. Existing M1–M4 profiles
remain network-disabled and reproducible.

Acceptance: a visible native network application sends and displays peer replies;
missing grants, wrong destinations, expired/revoked authority, stale process
incarnations, malformed/truncated/checksum-invalid packets, floods, dropped
packets and peer death cannot expand authority or hang the GUI. Payload and
queue budgets, deadlines and bounded retry behavior are observable. A pinned
host-only independent packet oracle agrees with wire bytes. Preserve privacy:
test public synthetic data only; UDP is not encryption or authenticated pairing.

## M5.2 — SDK and applications

A documented experimental language-neutral app contract, first-party Rust and
C bindings, reproducible cloud build/package/sign instructions and a small
independent example app which uses the contract rather than subsystem internals.
Use existing signature, lifecycle, W^X, capability and Data boundaries, extending
them explicitly where required. Do not advertise current private bootstrap
structures as a stable SDK.

Acceptance: rebuild an example from the SDK, verify/install its artifact and
launch actual isolated guest code from the GUI; demonstrate useful input/output,
a private persistent document across fresh boot, denied unrelated data/device
access and contained app failure. Include a usable small editor/notes app and
network app; existing Files/Settings/Terminal still work. Document exact supported
sizes and API/version errors. A second example in the other SDK language proves
the common ABI; generated declarations are checked for drift.

## M5.3 — Agents, profiles and integrated release

Implement an explicit provider-neutral agent tool interface with scoped grants,
expiration, revocation, resource budgets and content-minimizing attributable audit.
A deterministic provider is clearly labeled a test provider, not Pal intelligence.
Demonstrate actual agent requests in the guest: allow a narrowly granted action,
deny an ungranted action and deny again after revocation. No default network,
raw disk, signing, recovery or universal user-data authority for agents.
Essential local apps remain usable with the agent stopped.

Demonstrate a minimal headless ecosystem-node simulation and two named graphical
virtual presentation/resource profiles using shared RAR contracts. Preserve app
identity and document state during a compact/wide presentation transition.
A GUI-off desktop alone is not proof of tiny Tier0 support; actual bounded
portable execution/resource evidence is required for the node claim.
Profiles are not new incompatible OS editions. Do not claim ARM64, physical
phone, robot, car or drone support from x86 VM tests.

Run the integrated Alpha journey: cold boot -> GUI -> create/read document ->
launch SDK app -> communicate with isolated peer -> allowed/denied agent tool ->
profile adaptation -> signed component update -> rejected tamper -> rollback/
System-only recovery -> verify intact saved data after fresh boot.
Retain two-build reproducibility, isolation negatives, deadlines/resource
measurements, final independent review and previous milestone regressions.
Freeze exact source and trusted controller identities, merge only with passing
evidence, and publish a clearly experimental v0.5.0-expansion-alpha release with
durable proof assets and concise user testing instructions.

## Execution and safety

One writer and one reused read-only integrated reviewer, not a coordinator farm.
One continuing codex/m5-expansion branch and draft PR for implementation;
a concrete reviewed controller dependency may require a coherent integration,
but never an authorization-only chain. Batch fixes and diagnose failures before
one bounded rerun. Do not keep advancing a branch while its checks run.

All repository changes use GitHub APIs. No local files, SSD activity, builds,
target execution, artifact downloads or cleanup. Cloud execution is confined to
reviewed disposable profiles. No extra runtime dependencies without the normal
exception review. New contracts and controller authority require focused review
before use; routine safe work does not require owner reapproval.

Owned paths: new core/expansion, services/expansion, sdk/alpha, apps/expansion,
nucleus/expansion integration, tools/rar-lab/expansion and matching docs/tests;
touch existing code only for explicit reviewed integration. Preserve old public
formats and released profiles. No silent migration or Data autoformat.

## Progress record

See docs/evidence/m5-progress.md. Report implemented, cloud-tested, guest-proven,
reviewed and released separately. M5 remains incomplete while any runtime or
release requirement above lacks evidence.
