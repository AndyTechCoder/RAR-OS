# M4.3 fixed automatic-repair cloud scenario

Status: Proposed controller candidate; exact-source review and cloud checks required.
No M4.3 completion or runtime result is claimed by this document.

## Scope and authority

Add one literal repair-both case to the existing signed-runtime campaign.
It reuses immutable boot/input4, the existing System/Data/boot devices,
pinned tools, disposable cloud UID65532 confinement and three fresh VM lifecycles.
No additional device, host path, guest networking, dependency or Data authority.
No execution, file mutation or deletion on the owner's Mac or SSD.

The source under test must contain the reviewed native bootstrap Repair
implementation and remediation (initial candidate00c6b5e93f99200eb36f799d25753510a0a9baed).
Controller changes are not runtime authority until independently reviewed,
validated and merged to trusted main. Dispatch remains manual, exact-source
and main-only, with retained evidence and no automatic rerun loop.

## Exact scenario

1. VM1 boots pristine factory System, creates an unpredictable note through
   Terminal, then performs a valid generation2 install. Actual updated Settings
   pixels and update lifecycle markers are required. Join the VM and all three
   backends before freezing media. Independently verify encrypted Data and the
   exact installed System image.
2. The controller invokes one narrow System-fixture method. It checks the
   already-joined owner/descriptor identity and exact expected installed image,
   then flips only byte0 of each package header at byte offsets1024 and2098688.
   It consumes a one-shot flag before the first attempted one-byte write,
   refuses short/error writes without retry, fsyncs, and compares the complete
   damaged System image. No caller-selected offset or path is accepted.
3. VM2 starts from that exact damaged image, with physically read-only Data.
   No update or repair command is supplied. Native boot must classify both
   stored units, authenticate immutable factory input4, repair the opposite
   slot and publish kind3 before its GUI appears. Require factory Settings,
   the original note, exact repaired System and byte-identical Data.
4. VM3 is a new QEMU/firmware process on the retained images with read-only Data.
   Require factory Settings and the same note; generation2 update must now be
   rejected by the retained high-water2. No further System/Data mutation.

The pure oracle derives installed, damaged and repaired bytes independently
from the matched build's signed factory/update packages. Repair selects
sequence3, kind3, generation1, slotA, no prior, high-water2 and the exact parent
hash of sequence2. Corrupted slotB and all non-target bytes must remain unchanged.
Retain the complete damaged/final System bytes, damage receipt, Data hashes/
authenticated frozen image, actual frames, command transcripts, process/backend
joins, fixed topology, immutable boot hash and exact source/controller identities.

## Validation and limits

Pure tests exercise exact oracle bytes, illegal mutations, fixed corruption
geometry, joined-owner refusal, first/second short or raised writes, failed flush,
sticky one-shot/no-retry behavior after every partial mutation, independent
command/serial plans and evidence framing. Existing five cases and independent
unknown-publisher validation are retained. The new case makes six primary
scenarios plus the separate unknown-publisher composition.

The no-command plan and exact final image prove automatic repair. Ordering of
publication before GUI startup is established by the reviewed native boot path;
this capture does not provide a cross-stream timestamp barrier for that ordering.

This positive scenario is not the interrupted-repair fault matrix. M4.3 still
requires interrupted System writes/flush/publication with fresh restart, remaining
negative/fault evidence, crypto/regression/reproducibility, final reviews,
exact-head/exact-main release gates and v0.4.0-modern-alpha publication.
