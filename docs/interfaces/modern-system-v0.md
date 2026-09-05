# Modern-v0 signed layer and System journal

Status: experimental candidate contract; not an active disk/profile authority.
Applies only to the M4 laboratory Settings component. This is not stable RLM,
RSM, RME or RCI. Existing Desktop-v0 and historical Alpha bytes are unchanged.
ADR0034 remains proposed until the complete runtime/device/lifecycle boundary is
reviewed. This file specifies the standalone codec/model before integration.

## Representation and limits

Unsigned integers are little-endian and occupy exactly the stated width.
Byte arrays have no implicit terminator, alignment padding or host layout.
Trailing bytes, truncated fields and nonzero reserved bytes fail closed.
All inputs are borrowed immutable byte slices. No allocation, unsafe code,
I/O, keys with real secrecy, kernel capabilities or execution occurs here.

## Layer manifest: exactly 384 bytes

| Offset | Bytes | Meaning |
| --- | --- | --- |
| 0 | 8 | ASCII RARMODL0 |
| 8 | 2 | version 0 |
| 10 | 2 | length 384 |
| 12 | 4 | flags 1: PUBLIC LABORATORY FIXTURE ONLY |
| 16 | 24 | ASCII rar.alpha.ed25519.v0 followed by four zero bytes |
| 40 | 4 | component logical principal 5, Settings |
| 44 | 2 | architecture 1, x86-64 |
| 46 | 2 | component interface version 0 |
| 48 | 4 | profile 1, Modern laboratory only |
| 52 | 4 | component persistent state schema 0: stateless |
| 56 | 4 | exact PE file length, 512 through 2097152 |
| 60 | 4 | maximum mapped PE bytes, positive multiple of 4096, at most 131072 |
| 64 | 8 | declared capability ceiling 7; phase grants are a strict subset |
| 72 | 8 | positive signed update generation |
| 80 | 32 | SHA256 of ASCII RAR-MODERN-SETTINGS-HEALTH-V0 plus one zero byte |
| 112 | 32 | SHA256 of exact PE file bytes |
| 144 | 32 | SHA256 of the exact laboratory public key |
| 176 | 20 | nonzero source commit identity, binary Git SHA1 identifier, not a security hash |
| 196 | 32 | nonzero SHA256 identity of the declared build inputs |
| 228 | 4 | required Modern kernel ABI version 0 |
| 232 | 4 | declared trial CPU budget, 1 through 100 preemptions |
| 236 | 4 | dynamic heap budget 0 |
| 240 | 4 | guarded user stack budget 16384 bytes |
| 244 | 44 | zero reserved bytes |
| 288 | 32 | nonzero manifest digest |
| 320 | 64 | pure Ed25519 signature |

The digest is SHA256(bytes[0..288]). The digest and signature fields are entirely
absent from that preimage, not replaced by zeros. Signed bytes are exactly the
18 ASCII bytes RAR-LAYER-ALPHA-V0, followed by one zero byte (a 19-byte
domain prefix), then the 32-byte manifest digest: 51 bytes total, as required by ADR0019. This is PureEd25519, not Ed25519ph/ctx.

The laboratory public key is RFC8032 section7.1 TEST1's
d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a.
Its published private seed is public test data. It cannot authenticate RAR
production releases or protect against anyone who can sign with that fixture.
Unknown keys/algorithms are rejected; there is no implicit owner-root enrollment.

Ceiling bits are 0: shell send, 1: compositor send, 2: one-shot trial health.
Trial receives only bit2, bound to its exact incarnation and consumed on report.
Production receives only bits0/1. No storage, input, framebuffer, device, broad
IPC or lifecycle authority follows from the manifest. The future kernel/broker
must enforce these grants; this codec cannot grant capabilities.
Source/build IDs are signed provenance labels, not proof of reproducibility;
the cloud controller must verify actual source/build identities independently.

## Validation and immutable-byte boundary

Precedence is framing, reserved/flag encoding, algorithm, known publisher,
manifest digest, strict signature, compatibility/provenance/health identity,
resource budget, positive trusted minimum generation, exact bounded payload
length, payload SHA256, then bounded PE/W^X parsing and declared image budget.
Public parse alone returns UNTRUSTED metadata. Only verify returns a
VerifiedLayer borrowing the same immutable manifest and payload. Its fields have
no public constructor. It has no execution authority.

Reuse the existing RAR PE parser: fixed virtual base 0x400000, mapped image at
most 128KiB, at most16 sections, no imports/relocations/TLS/delayed imports,
bounded headers/ranges and W^X sections. A mapped executable is not a UEFI
application invocation: the eventual kernel constructs a protected user process.
Before any runtime mapping the kernel must own and seal the exact verified bytes,
prevent writable aliases, construct the process unschedulable, and recheck all
privilege/layout conditions. A borrowed slice does not seal a mutable disk.

Install uses highest_committed_generation + 1 as its minimum. Boot/fallback use
the immutable lab floor1 plus an exact committed manifest identity/generation;
they never accept any arbitrary lower signed image merely because its signature
passes. Addition overflow retires updates rather than wrapping.

## System selector: exactly one 512-byte sector

Two sectors hold alternate selector records. These are UNTRUSTED checksummed
metadata, not signatures, hardware counters or independent monotonic storage.

| Offset | Bytes | Meaning |
| --- | --- | --- |
| 0 | 8 | ASCII RARSYS00 |
| 8 | 2 | version0 |
| 10 | 2 | length512 |
| 12 | 1 | kind: 0 factory, 1 install, 2 authorized fallback |
| 13 | 1 | active slot: 0 A, 1 B |
| 14 | 1 | prior slot: 0 A, 1 B, 255 absent |
| 15 | 1 | zero |
| 16 | 8 | positive selector sequence |
| 24 | 8 | highest committed signed generation, never lowered by fallback |
| 32 | 8 | fixed laboratory root floor1 |
| 40 | 8 | active signed generation |
| 48 | 8 | prior signed generation, zero when absent |
| 56 | 8 | parent selector sequence |
| 64 | 32 | active nonzero manifest digest |
| 96 | 32 | prior nonzero manifest digest, zero when absent |
| 128 | 32 | SHA256 of the entire preceding512-byte selector; zero only factory |
| 160 | 320 | zero reserved |
| 480 | 32 | SHA256 of bytes[0..480] |

Factory has sequence1, activeA, no prior/parent, and highest=active generation.
Install has sequence>=2, opposite active/prior slots, new generation strictly
greater than the preceding high-water mark, prior equal to preceding active,
highest equal to new generation, and exact preceding sequence/hash.
Fallback has sequence>=3, active equal to preceding prior, no new prior,
unchanged highest, and exact preceding sequence/hash. The rejected current image
is not installed as another fallback candidate. A second fallback without a new
successful install fails. All generations are positive; prior is below active
for an install. No counter saturates or wraps.

Decode validates shape, checksum and semantic bounds. With two valid records,
equal sequences, gaps, forks or an illegal transition are ambiguous and require
read-only recovery. Otherwise select the higher legal successor. With only one
valid record, select it provisionally. With none, do not autoformat.
Before using any selected record, separately verify referenced signed manifests,
exact generations/digests, payloads and floor. A selector checksum is not
authorization. A maliciously rewritten/co-rolled-back complete disk cannot be
detected by this format and is explicitly not an Alpha claim.

## Publication, failure and repair obligations

The current library only plans records; no disk driver or durability claim exists.
The future System service must write only the inactive payload/manifest slot,
flush and read back/verify it, run candidate health, and publish the next record
into the older selector sector only after required lifecycle preparation.
Flush and read back that selector before reporting durable commit. Never mutate
the current selector sector or current image during candidate preparation.

If an inactive slot previously held a fallback, overwriting it temporarily
removes that fallback. Until the new selector commits, the current active image
must remain intact. If it independently fails during this window, recovery uses
separately immutable material, not the overwritten prior slot.
Failure after publication must be reconciled from durable state on reboot;
post-cutover failure uses a newly health-tested previous image and a new
incarnation, not resurrection of a killed process. Runtime/lifecycle sequencing
and fault injection remain separate pending contracts.

System recovery may repair only identified signed System units. It receives no
Data write authority. Corrupt/ambiguous Data is never formatted by this module.
Nothing here defines or writes the future encrypted Data schema.

## Conformance, migration and limitations

Tests cover canonical framing, every truncation, every reserved/algorithm byte,
signed-preimage exclusion, unsigned/unknown-key rejection, policy bounds,
deterministic malformed inputs, journal roundtrip, one-byte corruption, every
prefix tear of a selector write, legal/illegal chains, fallback high-water
retention and counter exhaustion. Prefix-tear tests are models, not proof of
device flush/reorder semantics or cross-reboot persistence.

Positive signed package interoperability, two independent crypto references,
full fuzz/resource closure, the device/flush backend, real process replacement,
Data encryption, full crash/recovery campaign and runtime proofs remain pending.
Do not claim this component closes M4. No external target code is added.

There is no existing Modern user dataset to migrate. These candidate bytes are
not installed yet. Future incompatible versions must be explicitly rejected and
require a reviewed read/export/migration path; never silently reinterpret,
rewrite or erase unknown data. Codec, signature policy, journal planning,
block transport, lifecycle and UI remain separately replaceable.
