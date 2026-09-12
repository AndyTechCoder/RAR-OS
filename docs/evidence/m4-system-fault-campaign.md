# M4.3 System interruption campaign

Status: implementation awaiting independent review, exact-head cloud checks,
trusted-main integration and actual runtime results. No M4 acceptance claim.

## Fixed behavior and unchanged authority

The two literal Install/Repair jobs enumerate every native payload-sector write,
payload flush, selector write and selector flush for the matched signed laboratory
packages. Five effects cover each boundary: before-cut, after-cut, error, torn-cut,
and short-error; partial persistence is exactly a 255-byte prefix.

The existing certified Modern topology and confinement remain unchanged. Each
case gets a new isolated container and three fresh QEMU lifecycles. VM1 creates
an unpredictable synthetic note; Repair additionally performs an actual gen2
install. After all processes join, only Repair flips the existing two fixed System
header bytes. VM2 injects its single fixed System backend fault. Data is physically
read-only in VM2 and VM3. VM3 receives no note-creation input and displays the
retained note and the selected real Settings component.

The backend's stable/volatile semantics are unchanged. Exact audit matching
requires every mutation's order, sector address and complete payload hash,
including unflushed writes. No mutation retry is allowed after the selected fault.
Whole QEMU/backend joins precede frozen-image inspection and a fresh restart.
The complete frozen and restarted System bytes must match an independent oracle;
the complete Data image must remain identical. Normal setup and restart System
mutations are also restricted to the exact expected transaction.

A cut may restore the intact factory prior before Repair publication. Such a
fresh boot uses normal fallback (kind2), not an artificially forced second Repair
(kind3). Both retain high-water2. Neither controller-synthesized images nor
guest success markers count as actual runtime proof.

## Fault observation and limits

System gets an explicit role in the existing strict audit parser with exact
8MiB geometry; Data remains the default exact99328-byte role. The VM2-only
transaction option requires read-only Data and rejects simultaneous Data or
selector faults. A distinct exception carries the exact selected fault proof.
An acknowledged CONT marks the VM started before delivering an early boot fault,
so teardown cannot skip the QMP drain.

The System-only event policy allows only the fixed RESUME/RTC events and at most
one System write-error event in the final joined stream. Receipt-after-reap is
not represented as event occurrence before kill. The exact backend fault was
observed before destruction; process joins, stream completeness and fixed command
causality are separate requirements. Data's existing receipt policy is unchanged.
For non-cut error/short-error effects, immediate destruction is insufficient.
The controller continues observing for at most25seconds until the complete exact
native UPDATE-RECONCILE fatal frame is present, checking every intervening System
request and refusing any subsequent mutation. Missing/truncated/wrong reconcile
fails, and the independent retained validator requires the same complete frame.
Only then does the controller destroy the whole VM. True cut effects are immediate.
Unexpected exits, panics, peer failures, events or generic exceptions fail closed.

Each mode has at most256 cases; no caller-provided paths, offsets, devices or
launch flags. Each case has a600-second container limit and each existing VM
retains its120-second limit. The campaign starts no case after110minutes;
workflow timeout is150minutes. Two fixed modes may run concurrently; no agent
coordination or retry loop is introduced. Inputs stay read-only, networking stays
off, and UID65532/resource/ownership checks and cloud-only cleanup are unchanged.

Actual JSON captures are retained losslessly as deterministic gzip, with raw and
compressed lengths/hashes. Capture retention happens even if validation fails.
Accepted cases must have distinct image identities, lab keys and unpredictable
challenges; reused fixtures fail. Neither compressed artifacts nor target images
are downloaded to the owner's device.

## Tests and release gate

Focused inert tests cover every oracle boundary/effect, exact mutation hashes and
no-retry rules, representative full retained captures, forged lifecycle/Data/
identity/frame claims, System versus Data event roles, forbidden commands,
early boot fault cleanup/order, bounded dispatch selection and campaign
failure/no-retry/retention behavior. Existing Data, VM, signing and safety tests
remain enabled. No tests execute RAR OS outside the certified cloud profiles.

Independent review and passing source CI are required before this controller
becomes trusted main. Actual Install/Repair campaigns must then pass against the
reviewed native source. Required final regressions, conflict-preserving
integration, final-head/exact-main evidence and release acceptance remain
mandatory. This document itself grants no completion or production certification.
