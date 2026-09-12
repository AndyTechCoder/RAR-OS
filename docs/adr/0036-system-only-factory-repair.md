# ADR 0036: Bounded immutable-factory System repair

Status: proposed implementation contract; independent review required before
native activation. Direction: owner-authorized M4.3, 2026-09-12.

## Decision and alternatives

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

## Experimental persistent representation

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

## Source and runtime gates

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

## Safety and limits

GitHub-only mutation; cloud-only approved execution. No Mac/SSD edits, deletion,
downloads, build or execution. No new target dependency, guest networking,
raw disk, device passthrough, Data-write authority or production key.
Root is immutable laboratory boot input, not hardware immutability. Public
fixture keys and rollbackable local journal state retain existing limitations.
