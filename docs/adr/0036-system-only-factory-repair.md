# ADR 0036: Bounded immutable-factory System repair

Status: Proposed — independent native integration review pending

Direction: owner-authorized M4.3, 2026-09-12.

## Context

M4.2 can select authenticated active content and a named prior. M4.3 needs a
bounded recovery path when neither stored copy is intact, without formatting
System or modifying Data. The pure planning core is not native repair authority.

## Decision drivers

Preserve Data, rollback high-water, immutable factory provenance, bounded memory,
exclusive System ownership and fail-closed handling of uncertain transport.

## Considered options

- Refuse all damaged-System boots: safe but leaves recovery unavailable.
- Format or reset the journal: rejected because it loses state and high-water.
- Treat factory as a newer install: rejected because generation1 is not newer.
- Explicit opposite-slot Repair after authenticated damage proof: selected.

## Decision

Add explicit repair planning and a Repair selector transition. Do not pretend
a generation-1 factory is a newer Install, lower the high-water mark, overwrite
the selected payload, or format ambiguous media. Ordinary verified-prior
fallback remains preferred. Rebuilding every System unit is unnecessary for the
single replaceable Settings unit in this Alpha.

The pure core planner verifies the exact immutable factory package against a
trusted full-package SHA256 identity, the enrolled public laboratory signing
root, all existing manifest/PE policy and exact factory generation1. A valid
signature alone is insufficient to select arbitrary recovery code. The native
caller must obtain expected identity from immutable boot provenance, never
from the package, writable System selectors or an IPC claim.

Read-only content inspections bind the complete selected Record, Active/Prior
role, exact referenced identity and observed byte hash. Inspections are made
only from complete bounded sealed read results, never from I/O errors or partial
transport buffers. Authenticated intact active bytes refuse repair. An intact
authenticated prior requires ordinary fallback, not overwriting that prior.
Only content-invalid active with absent or content-invalid prior can produce
a plan. Health failure alone is not damaged-byte proof and does not authorize
factory repair.

The plan retains exact observations and requires unchanged reobservation before
use. It is a pure decision, not an I/O, kernel, signing or lifecycle capability.
The eventual native transaction must additionally bind request/seal/incarnation,
exclusive System ownership and the same selection across inspection, staging
and publication; consume its one-shot pending state and fail closed on races.
No large stack/scratch allocation or new kernel window is introduced here.

### Experimental persistent representation

Keep RARSYS00/version0,512-byte records, geometry, checksum and fields unchanged.
Allocate kind byte3 to Repair. It has sequence>=2, parent sequence=sequence-1,
nonzero exact parent hash, active in the opposite slot, active generation1,
no previous pointer, and highest>=1. A legal immediate Repair successor retains
the predecessor highest exactly and binds its exact sequence/hash. Its active
manifest must independently match the immutable authenticated factory. The
journal checks structural succession, not root provenance, damage or health.

Preparation writes only the opposite System slot's exact factory package
sectors, flushes and verifies readback. Preserve the selected selector/payload,
unrelated System sectors and all Data. Verify/health-test sealed replacement and
complete fallible boot preparation before alternate-selector publication.
Indeterminate I/O halts; a new VM boot remounts and classifies actual bytes. No
in-process publication retry, implicit remount, autoformat or generation reset.

## Compatibility and migration

Existing Factory/Install/Fallback bytes and behavior remain unchanged; new
readers accept them without writing/migrating anything. Old readers reject
kind3 but may select the older valid selector. They still must authenticate its
referenced package; this extension does not prevent downgrading to old software.
Use matched new boot/controller for repaired fixtures. There is no deployed
Modern user dataset migration or older-runtime security guarantee. Data format
and encryption are unchanged. Preserve historical accepted source/releases.

## Validation

First coherent source batch: typed factory/inspection/plan, Repair journal
codec/succession, signed-package and stale/substitution negatives, selector
publication fault/tear coverage. It does not add a native Repair mode or enable
a new cloud scenario. Independent source checks are not guest repair evidence.

Native integration must provide independently bound immutable factory identity,
complete sealed content inspections and exact reobservation, System-only
staging, verified health and durable ACK. A valid prior remains fallback;
transport I/O, timeout, changed media, missing root, invalid/ambiguous selectors
and counter exhaustion stop without repair.

Actual M4.3 acceptance must include corrupt factory; corrupted installed and
prior packages; verified prior fallback; bad root/ambiguous selectors denied;
every repair write/flush/selector interruption followed by fresh restart;
high-water/stale rejection after repair; exact Data identity and full process/
backend joins. Complete the remaining Data/update fault matrix, crypto,
reproducibility/regression/review, exact-main gates and v0.4 publication.
No M4.3 completion is claimed by this ADR or source batch.

## Security and data impact

GitHub-only mutation; cloud-only approved execution. No Mac/SSD edits, deletion,
downloads, build or execution. No new target dependency, guest networking,
raw disk, device passthrough, Data-write authority or production key.
Root is immutable laboratory boot input, not hardware immutability. Public
fixture keys and rollbackable local journal state retain existing limitations.

### Completed-read boundary clarification

Inspection consumes opaque CompleteRead, which has no production constructor in
this unactivated batch. Only a test-only mock can issue one, after successful
exact-length whole-sector input; partial, unaligned or failed reads refuse a token. The future native
bridge must supply the independently reviewed production issuer. The planner's
matches_observations compares values only, not freshness; copied observations
cannot establish fresh I/O. Native one-shot receipt/seal/incarnation and exclusive
ownership remain required before any repair write. No claim of enforced fresh
native reads or guest repair is made by these source APIs.

## Consequences

Repair has its own journal transition and native bootstrap state machine rather
than bypassing install validation. This adds focused inspection and interruption
tests, but limits writes to the damaged replaceable System unit. The factory
image and signing fixture remain laboratory roots, not production secure boot.

## Replacement path

Replace the factory provenance provider or storage transport behind the reviewed
interfaces, preserving complete-read, one-shot, exact-record and Data-isolation
contracts. Any persistent representation change needs explicit compatibility
review. Do not reinterpret this proposed ADR as activation or release approval.

### Native bridge implementation contract

The reviewed integration direction uses private Repair mode3 once during
bootstrap only, never through Terminal or live-update commands. Starting consumes
the one-shot gate. Transport, channel, changed-media, native or indeterminate
errors halt; only a fresh VM can create another bootstrap attempt.

Inspection uses a distinct non-executable kernel seal, not a relaxed executable
Transfer. Read a complete first512-byte sector. If framing is invalid, exactly
that sector proves invalid content. Otherwise read the complete sector-rounded
package representation, at most2097664 bytes, including final storage padding.
Manager independently validates framing, exact logical length and zero padding;
a successful prefix or failed read never issues CompleteRead. Normal executable
seals retain their896..2097536-byte bound and cannot consume inspection seals.

Every inspection frame must bind request, seal, selector sequence, role/phase,
referenced slot/generation/digest, exact length and hash. Fixed phases are active,
optional named prior, immutable factory, then fresh active/optional prior/factory.
Manager—not a System boolean—decides whether stored content authenticates.
Usable active refuses repair; usable prior takes ordinary fallback. Fresh
observations must match the initial plan before any System write.

Manager obtains the expected factory package hash independently from immutable
boot/build provenance, never from System IPC or the package being verified.
Only System maps laboratory input4. Factory signature, generation1 and PE policy
remain independently required; this is a public laboratory root, not hardware
secure boot.

System reobserves the exact selected journal, prepares only the opposite slot
from the immutable factory, flushes and verifies complete readback. The pending
Repair state retains the exact authorized next record. Manager verifies the
sealed disk readback, completes health and desktop preparation, then publishes
through the existing exact durable-ACK barrier. No Data handle, large service
stack buffer, new memory window or in-process repair retry is introduced.

Fresh-VM acceptance must verify the published Repair selector, factory UI,
unchanged high-water, stale-install refusal and byte-identical Data. Each
interrupted write/flush/publication case must stop and join the old VM/backends
before examining media and restarting. These are required future runtime tests,
not evidence supplied by the current pure staging/planner code.

### Implemented pure storage-shape classifier

Before any native issuer exists, the pure classifier now independently checks
the reader's exact shape: malformed framing permits only a complete512-byte
sector; parseable framing requires exactly the sector-rounded declared package.
Unexpected short or extra sectors are missing evidence and return Identity,
not a damage observation. Nonzero final padding is content damage. Verification
uses only the logical package; observation equality hashes all stored bytes.
This does not provide native provenance, fresh I/O or repair authority.


### Native primitive allocation

Private STAGE_COPY operation4 begins Inspection; shared Append's framing ceiling
is2097664 while the Buffer enforces each reservation's exact purpose/length.
Private STAGE_VIEW10 reads Inspection metadata,11 rejects/scrubs only Inspection,
and12 derives the exact logical input4 SHA256 from the containing trusted boot
image. Existing executable views and trial paths reject Inspection. See
modern-runtime-v1.md for exact argument, identity and safety rules. This reviewed
root-provider refinement avoids a generated Manager constant while remaining
independent of System IPC. It does not create a CompleteRead issuer, implement
the Repair transaction or constitute native runtime acceptance.
