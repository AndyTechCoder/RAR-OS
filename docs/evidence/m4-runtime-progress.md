# M4.3 repair core — source work in progress, 2026-09-12

M4.1 and M4.2 remain accepted at their exact development snapshots below.
M4.3 is not complete. The first coherent repair source batch adds an opaque
factory/content inspection/repair planner, explicit kind3 System selector
succession, and signed-fixture plus selector failure/tear tests under ADR0036.
It introduces no Data authority, native repair mode, new syscall, guest profile
or runtime claim. Independent source review and cloud checks are pending.

The planner refuses an intact active or named prior, binds exact record/role/
observed content, preserves high-water, and requires the exact authenticated
immutable factory generation1. Its complete sealed-read and trusted root-hash
provenance obligations remain native integration work, not established by a
caller-provided slice or source test. Old-reader handling is documented as an
unsupported downgrade, never an automatic migration or format.

Next: finish focused review/cloud source validation, integrate the immutable
root identity and sealed inspection/System-only repair transaction, then prove
fresh-VM repair and interrupted repair. Complete the full fault/release gates.
No local/SSD mutation, deletion, download, build or execution is authorized.

---

# Accepted M4.2 development snapshot — 2026-09-12

M4.2 signed live updates is complete at source
93ac75b086755a9b57a1e0998dc16a36eb147be0 with trusted controller
14d6612bd7e7f6593ce89b71a89a60d283027aa6. Independent focused
acceptance review found no remaining M4.2 blocker after the final controller
Specifications34664012701/job103472147122 passed at that exact revision. This supersedes historical incomplete
M4.2 checkpoints, not the safety boundaries or remaining M4.3 requirements.
M4.1 remains accepted at fd1982665b81ba2e672e0044165d68e09daa47f0.
PR158 remains draft/unmerged; this is not whole-M4/main-release acceptance,
v0.4 publication or production security certification.

## Actual behavior and exact evidence

Cloud run https://github.com/AndyTechCoder/RAR-OS/actions/runs/34664032961
(job103472206047) succeeded on 2026-09-12 in 3m54s. The completed log binds the
source and controller above and reports all five fixed scenarios independently
checked, followed by actual rejection of a correctly signed non-enrolled
publisher. Six scenarios use eighteen fresh VM lifecycles, with no retained RAM
or guest-network channel.

The reviewed producer and independent retained-evidence validator cover:
- Changed native Settings executable behavior and compact mode while peers live.
- Failed health, tampered signature, incompatible ABI and unknown publisher
  refusal; stale generation refusal after fresh boot.
- Post-cutover UD2 failure and fresh verified prior-component fallback.
- Kernel-observed old queue removal, stale endpoint/capability denial and
  consumed lifecycle-token rejection.
- Exact injected System selector-write EIO, canonical reconcile panic,
  one-shot final status barrier, bounded whole-VM/backend teardown, and fresh
  factory selection with retained synthetic file.
- Exact persistent Data-image identity across update/fallback/error paths.

Retained artifact10287894113, modern-signed-runtime-34664032961-1:
21 files, ZIP1467368 bytes, SHA256
0e7b3579141ff61bad35ac732ef1b67b798eeaf89a858fbe0fff820d498f484f.
Availability and digest were checked through GitHub metadata; no artifact was
downloaded locally. Detailed evidence validation ran in the cloud. Final
review uses completed sanitized logs, exact source/controller and prior focused
code review, not a claim of separately re-extracting the artifact locally.

Exact-source checks: Specifications34655109736/job103445692408,
Foundation34655109758, Platform34655109752 and Desktop34655109760 passed.
Controller correction PR198 headf21840d8226cc8acda535fd8a54a5bcbc459c1a8
passed full Specifications34662911092/job103468963988 and independent focused
review before merge. Its real backend test proves EIO precedes process shutdown.
The fix preserves exact fault-audit checks and correlates natural exit21 with
its terminal record; owned SIGKILL exit-9 need not invent a terminal record.

## Prior failures and disposition

Runs34657349650,34659553868 and34662264976 failed and remain failed evidence,
not acceptance. The diagnosed harness defects were respectively panic-frame
parsing, selector-specific event/barrier validation, and prematurely requiring
a process-terminal record for an operation EIO. PR196/197/198 corrected these
with focused negatives and independent review. No native target change was
needed between source93ac75b and this successful campaign. No failed check was
ignored, weakened into unconditional acceptance, or silently relabeled.

## Remaining M4.3 work and limits

Immutable System-only repair and restartable interrupted repair, the full
controller-owned fault matrix, final crypto comparisons/reproducibility/
regressions/reviews, exact-main acceptance and v0.4 release remain unfinished.
Existing prior-component fallback is not immutable recovery repair.
Laboratory keys/data are public fixtures. Rollback protection is relative to
intact local journal state, not a hardware monotonic anchor. No production
confidentiality, arbitrary hardware support or universal restart-free update
claim is made.

All repository mutations are GitHub-only. No Mac/SSD file creation, edits,
deletion, artifact downloads, builds, mounting or RAR/VM execution occurred.
Only reviewed disposable cloud profiles executed target tests.

## Next concrete action

Begin M4.3 by specifying and independently reviewing the immutable-factory
repair contract; implement the System-only repair path and restart tests,
then complete the final fault and
release gates. No additional owner permission packet is required for routine
safe implementation within the approved scope; independent boundary review
and actual cloud evidence remain mandatory.

---

# Historical checkpoints (preserved; superseded by acceptance above)

# Verified M4.2 source result — 2026-09-11

M4.2 remains incomplete; do not begin M4.3 or merge PR158.

Source4c5bcc6fbd97cc06b4511e011ea6ae13998dcb20 now passed all four cloud checks:
Specifications34568197376/job103164591601, Foundation34568197221,
Platform34568197197 and Desktop34568197181. Completed sanitized logs confirm
73 signed-fixture/core/session tests, including every media-operation error
campaign, exact prior fallback, cancellation, stale identity, proposal
substitution and lost/duplicate ACK tests. Native kernel/service Linux
object-only composition compilation and Settings interaction tests passed.
This is not UEFI build/link, stack-use, signed boot or live-update VM evidence.

The later immutable-input/provisioning source and checkpoint
927d84246f6b098422b268999db82508a7344c7e are being published to the same draft
branch for ordinary cloud source validation. The pairwise-window/provisioner
batch83e0e743 still has no independent review clearance: the reviewer remains
blocked by its thread-store ENOSPC rejection. No retry, alternate reviewer,
review bypass, local cleanup or deletion has been attempted.

The owner renewed approval for safe continuation, not waiver of review or VM
acceptance. Safe source validation may proceed; runtime activation and merge
remain gated. Signed bootstrap composition, actual target package builds and
causal VM acceptance remain unfinished as detailed below.

---

# Current continuation status — 2026-09-11

M4.2 is **not complete**. Independent review of the latest provisioning source
83e0e743cad22d1bc0c5d011944e88c33a1b7eb7 was blocked before source access:
Codex could not create its thread-store writer lock (ENOSPC). The tool rejection
forbids retries/workarounds. No cleanup, deletion, replacement reviewer or
independent-review bypass was attempted.

Review of 6dd3e99589c2dbd1320d72de5c41d43711019aab found no source blocker.
The later83e0e batch adds pairwise physical-window preflight and a pure existing-
root signer/inspector-based package/virgin-System constructor with tests.
These additions have not received independent clearance or cloud validation.
All are saved as GitHub commits in the continuing PR158 ancestry; do not lose
or recreate them, and do not call source construction a completed VM feature.

Reviewed4c5bcc6fbd97cc06b4511e011ea6ae13998dcb20 has three passing regressions;
Specifications34568197376 remains pending at this checkpoint. Read the actual
run outcome before continuing. The PR checkpoint comment is
https://github.com/AndyTechCoder/RAR-OS/pull/158#issuecomment-5630304226 .

The safe continuation is to retain the current cloud result, restore the
independent-review infrastructure, review the exact pending source, then finish
the native bootstrap selection and trusted cloud target build/provisioning/
causal VM evidence listed below. No new owner architecture permission is
needed for the approved scope. No local Mac/SSD repository operation, deletion,
download, build or RAR OS execution is authorized by this checkpoint.

---

# Latest M4.2 implementation checkpoint — 2026-09-11

**M4.1 remains complete. M4.2 is NOT complete.** No live-update VM acceptance,
PR158 merge, v0.4 release or production security is claimed.

The owner explicitly approved continuing M4.2, including the native loader/
lifecycle bridge, on 2026-09-11. Routine safe work within that approved scope does
not require another permission packet. All work below used canonical GitHub API
source operations and existing cloud checks. No Mac/SSD repository files were
read, changed, deleted, downloaded or built; no RAR OS ran locally.

## Implemented in this continuation

- Native exact-sealed-byte PE trial construction with minimal health authority,
  private initialized mappings/stacks and writable-aperture retirement.
- Prepared live ACTIVE handover and an all-or-nothing initial desktop plan.
  The plan stores capabilities, not copied peer queues; its 8 KiB bound is
  compile-time enforced. Initial desktop processes remain unscheduled until
  publication. The selected-boot composition is not yet activated.
- Kernel-authenticated Settings rebinding in shell/compositor, preserving peer
  state and committed pixels while rejecting old incarnations.
- System boot preparation, exact pending-token copy, cancellation and borrowed
  one-shot publication tokens. Pre-I/O policy rejection cannot strand the owner.
- Bounded authenticated System/manager wire/session implementation, exact
  package/record/signature verification and full durable-ACK comparison.
  Native syscall adapters and one-shot boot fallback routing are linked in
  source; legacy service entries remain selected pending provisioning.
- Manager-only whole-guest UPDATE-RECONCILE halt for uncertain publication,
  plus exact bounded release-status routing.
- Fixed immutable System-only laboratory package windows have been implemented
  as an unactivated source candidate. Their compile/review and trusted package
  provisioning remain pending; no host paths or arbitrary device selectors exist.
- A genuinely different Settings source variant: D toggles spacing. Default
  Settings behavior is preserved. Standalone role5 and failed-health variants
  exist for later signed cloud fixtures; they have not been executed.

## Review and cloud evidence

Source22ca84eee5675244c53785195d45b0f4c1500a5b passed all four source checks,
including Specifications34564109443/job103152583187. Source
5841c117b2177bd9b3725dac27ddfd8061461ddb passed all four checks:
Specifications34565615234/job103157012881, Foundation34565615263,
Platform34565615254, Desktop34565615241. Completed logs verify actual model/
support tests and Linux object-only native kernel/service composition compilation.
These do not constitute UEFI link, stack-usage or VM runtime evidence.

Consolidated current implementation:
4c5bcc6fbd97cc06b4511e011ea6ae13998dcb20. Independent read-only review found
no remaining source blocker after corrections. Foundation34568197221,
Platform34568197197 and Desktop34568197181 passed. Specifications34568197376
was still running when this checkpoint was prepared; do not infer its outcome.

The review found and the source fixes addressed stale peer snapshots,
consumed pending tokens after policy rejection, Stage Begin error atomicity,
and terminal routing. Cloud validation caught a private-method visibility
error, an oversized Process-based desktop plan, and an outdated exhaustive
named-send matrix. A subsequent duplicate-grant edit was caught and corrected
by review in4c5bcc6f. Failed or superseded runs are not counted as passes.

New signed-fixture transaction tests cover Boot/Install/Fallback, clean
cancellation, stale peers, malformed/substituted proposals, begin/finish/abort
failure, every observed media-operation error in all three modes, and
lost/duplicate ACK with zero reexecution. Their final cloud result remains
bound to the current exact-head check, not to earlier passing snapshots.

## Exact remaining work to finish M4.2

1. Integrate bounded immutable laboratory package inputs and factory System
   provisioning, preserving the separately immutable boot/recovery copy.
   Select the bootstrap-only native composition and actual service entries.
   Do not infer formatting permission from zero media.
2. Build/sign the actual standalone Settings variants in the pinned reviewed
   cloud tool. Validate every target cfg, final PE/image/stack bounds and
   reproducibility; source/object compilation is insufficient.
3. Demonstrate actual selected boot, changed-code live update with peers
   continuing, health failure and fresh-incarnation fallback, rejection cases,
   durable publication cuts, native mapping/revocation/zeroing and whole-guest
   reconcile halt. Bind exact unchanged Data and retained causal GUI evidence.
4. Close the final focused review and exact-head evidence. Keep PR158 draft
   until the applicable M4 integration/release gate; preserve released v0.3.

No new coordinator, repeated polling automation, permission-only PR or
re-run of already accepted M4.1 campaigns is needed.

---

# Latest M4.2 source integration checkpoint — 2026-09-10

M4.1 remains complete; M4.2 is NOT complete and no updated Settings VM proof is
claimed. The owner-approved staging syscall candidate is integrated. Its three
historical regression checks passed; Specifications34509010239 was superseded/
cancelled when the follow-up source head was pushed, not recorded as a pass.

System package adapter commitf912659711b9dd7679959212cbf246638f6c4da5 passed
Foundation34510775679, Platform34510775671 and Desktop34510775680.
Specifications34510775667 also completed successfully. All four exact-head
source checks passed; this does not validate the later changes below.
Independent review found two integration gaps: unbound generic readback and
missing existing-prior fallback. This change binds copy_prepared to pending
identity/selection with sticky sink failure and implements exact-prior,
no-package-write fallback publication. Matching fault/race/max-size tests are
included. Review of these fixes remains pending.

This change also implements native staging FINISH, System COPYING-only ABORT,
the manager-only RO/NX view and pre-trial rejection, all-root writable-alias
retirement/TLB ordering, and full non-elidable scrub before reuse. Exact
page-table byte tests and syscall/model tests accompany the contract. These
changes have not yet passed independent review, exact-head cloud checks or
native guest validation; they do not constitute M4.2 acceptance.

Remaining: fixed System/manager protocol and signed-byte verification; actual
trial PE construction, health, durable cutover and fresh-incarnation fallback;
visible changed Settings while peers remain alive; rejection/cut/rollback
evidence and exact Data preservation. No merge or v0.4 release is authorized by
this checkpoint. All work is GitHub API-only; no Mac/SSD file mutation, download,
build, deletion or RAR target/VM execution.

---

# Latest M4 checkpoint — M4.1 COMPLETE, 2026-09-10

M4.1 is COMPLETE at accepted development snapshot
fd1982665b81ba2e672e0044165d68e09daa47f0. Independent consolidated review found no
remaining M4.1 acceptance gap, the exact reviewed controllers are integrated,
and all four exact-head checks passed: Specifications34503312824,
Desktop34503312764, Platform34503312795 and Foundation34503312880.
This is not full-M4, PR158 merge, production security or v0.4 approval.
Final whole-M4 regression and release work belongs to M4.3, not an open M4.1
completion condition. This documentation-only closure changes no target bytes.

| Section | Outcome | Next |
| --- | --- | --- |
| M4.1 Persistent files | COMPLETE; demonstrated and integrated at the accepted snapshot | Preserve evidence for the later whole-M4 regression |
| M4.2 Signed live updates | Incomplete; source primitives only | Required System-only staging-copy approval, then real Settings replacement/health/fallback |
| M4.3 Recovery/release | Incomplete | Immutable System-only repair, unchanged Data, interruptions and final release proofs |

Runtime source633a9531f2731c69aeafcfd12a8789265a62f483 passed all four source checks.
Actual persistence/corrupt-mount run34497590722/job102939893132 passed under
controllerbab435bc8e973500aef37f54bae7f7072ef93083. Artifact10160495313:286406 bytes,
SHA25610e4382f2be65a3436527904d76cf98f82eade58b3f46de4cc52d7c306a68f84.

Actual mounted-write-error run34502568755/job102956693964 passed at the same
runtime source under controllerd70067ffc156757b12487caad34ec2c3fd7a3fb0.
Controller treea64aa05ca50c1b8281c79e8b7ebaca2a2da82fd2 passed independent review
and full sourceCI34500426129 before PR189 merged. Artifact10162511761:248682 bytes,
SHA2566fbab80ace5f135b89e2bf03c7df82c7fcc7bc468e30fc68f4fa123b60a5533a.
Its independent checker requires198 ordered pre-error reads,11 causal GUI frames,
first-Enter fault identity, same-VM LIST/locked WRITE/LIST/Files Unavailable,
zero post-error backend requests/remounts, joined VM/backends and unchanged disks.

The60-case fault campaign34359925328 is preserved. Complete Git-tree comparison
from its source5b8555e86b54056efa2927e094b3f1746e66d668 to633a9531 confirms all
services/modern/* and nucleus/modern/native_pio.rs blobs identical. Only two of
the15 inspected storage/app files differ: the reviewed app DEVICE-denial checks,
already exercised by current-source VM runs. Crypto324/972+9failure-probe run
34492109713 and retained inspection34492745752 passed, as documented below.

The guest DEVICE status checks prove common capability rejection before opcode
handling; they are not a literal read/write-opcode matrix. This scoped evidence
limit is not an outstanding M4.1 gap. No production secrecy or physical
power-loss claim is made. Superseded PR189 CI34499791651 was cancelled after
concrete review findings; both fixes and meaningful negatives passed before merge.

Evidence: https://github.com/AndyTechCoder/RAR-OS/pull/158#issuecomment-5622079061

This integration copies only19 exact reviewed PR188/189 controller blobs.
No target code, capability or device authority changes. PR158 remains draft,
v0.3 preserved; M4.2 permission denial is not overridden. No Mac/SSD file
mutation, deletion, download, build or target/VM execution.

---

# Prior checkpoints (preserved; superseded by the status above)

# Latest M4 checkpoint — 2026-09-10, crypto runtime evidence expanded

M4.1, M4.2, M4.3 and M4 remain incomplete. This supersedes older status
statements below without removing historical evidence. Supporting source tests
and controller work are not counted as implemented OS features.

| Section | Demonstrated behavior | Remaining outcome |
| --- | --- | --- |
| M4.1 Persistent files | Actual Terminal-to-Data persistence across full VM destruction and fresh firmware; all60 Data fault cases; fresh independent crypto comparison, nine actual process-failure probes and retained source/lifecycle inspection | Remaining storage authority acceptance; integrated-head confirmation |
| M4.2 Signed live updates | Reviewed staging and lifecycle primitives, including physical retirement source | System staging copy authorization remains unresolved; connect package I/O, signature verification, executable loader, health/cutover/fallback and prove visible Settings replacement with peers alive |
| M4.3 Recovery/release | Contracts and reusable cloud evidence machinery | Actual immutable recovery, System-only repair with exact unchanged Data, interrupted repair and final regression/reproducibility/release |

## Fresh crypto evidence, not a production security claim

Trusted-main comparison run34485256699 succeeded against target source
3d8ec7fe12ba42097cd06305993a4217a69123ca using controller
252684e2f194a54849d34dc6401f7b4d5d249a73. It performed324 comparisons and972
adapter calls, including fresh public hash/AEAD challenges. All RAR outputs were
retained before reference execution. Ed25519 coverage remains the fixed corpus.

Artifact10155500584 is9,613,731 bytes, SHA256
10128e9660bfc04d5fbe1df38329a5b722600f48815e31da2e5b513d5470f918.
Inspection34485914267 validated7319 inventory members, exact Git/source closure,
the frozen requests/results and three-way agreement. Inspection34487656249
reused that artifact under controllerbf6527f33e7fb138353731f2435e9e3a21d8cb2f,
checking all972 unique container lifecycles, confinement, stopped state and
confirmed absence. Neither inspection executes retained target source.

Evidence:
https://github.com/AndyTechCoder/RAR-OS/pull/158#issuecomment-5619883909
https://github.com/AndyTechCoder/RAR-OS/pull/158#issuecomment-5620117417

PR187 merged as9f7e09bc64bc35a46e7d1dd7e9c1e0590188bfd3 after final reviewed
head d602de8929e3c64b08ff022652c2262d7b0088d3 passed Specifications34489608594.
It consolidates the nine actual timeout/crash/output-limit probes, outer
controller integration, pure retained-evidence inspection and negative tests.
Actual campaign34492109713 / job102921198587 and retained inspection34492745752 /
job102923398054 succeeded. Artifact10158373977 is9,692,082 bytes, SHA256
945e514ef718727d11280b1f4eae1e62ba756d9bd5937e52f12b03e95fc441be.
The reader validated7392 inventory members,324 three-way comparisons/972
normal adapter calls,972 confined/stopped/removed lifecycles and9 actual
timeout/crash/output-limit probe records with unique identity and cleanup.
Probe evidence SHA256:
1b3046ea91f32b2e24d39aa1b90875d53fd440d62763255b7566f4b575e47259.
This closes the specific process-failure evidence gap, not a general sandbox or
production cryptographic-security assessment.
https://github.com/AndyTechCoder/RAR-OS/pull/158#issuecomment-5620794855

## Remaining M4.1 guest acceptance

Exact Data capacity already has causal guest evidence through IDENTIFY and the
successful mounted write/read path; wrong-capacity negatives remain source
tested. Do not weaken host preflight to attach a wrong-sized image.

A new Files/Terminal boot check invokes the existing DEVICE status operation
with the absent device capability and two unrelated issued capabilities. All
three calls must return exact Denied before the app can publish a view. This
adds no authority, disk command or write even if the negative check regresses.
Pure tests pin call order, exact errors and fail-closed boot shape. Actual
guest evidence remains pending a new cloud boot, so source tests alone do not
close the device-denial requirement.

Still needed: a controller-owned corrupt/ambiguous-header negative boot with
two same-incarnation file operations, sticky Unavailable output, no SAVED,
mount reads but zero writes/flushes, and unchanged exact Data hash. System
IDENTIFY success is currently discarded and is not guest-readiness evidence.

The last runtime Desktop check failed during pinned Debian snapshot package
acquisition before guest boot, not in the OS. This is not a passing regression,
and the failure has not been concealed or relabeled.

---

# Latest M4 checkpoint — 2026-09-09, actual Data fault campaign passed

M4 and all three sections remain incomplete. This checkpoint supersedes older
status statements below without removing their historical evidence.

| Section | Demonstrated or implemented | Remaining end-to-end work |
| --- | --- | --- |
| M4.1 Persistent files | Actual fresh-VM persistence; retained crypto/source inspection; all 60 fixed Data fault cases observed and content-checked | Close remaining crypto/confinement and storage authority acceptance, then rerun relevant proof at the integrated final head |
| M4.2 Signed live updates | Reviewed manifest/lifecycle foundations; physical retirement implementation with all four source checks passed | Connect immutable staging, manager/System protocol, actual signed Settings loader, durable cutover, peer continuity and fallback; demonstrate privileged retirement in the VM |
| M4.3 Recovery/release | Reviewed contract and reusable evidence infrastructure | Actual immutable System-only repair, interrupted repair with unchanged Data, final regressions/reproducibility/review and exact-main release |

## Actual 60-case Data fault evidence

Run [34359925328](https://github.com/AndyTechCoder/RAR-OS/actions/runs/34359925328),
job102493991562, completed successfully. Trusted controller:
9e8dd1522300b2b5a178596a1ca979407ab0696d. Target source:
5b8555e86b54056efa2927e094b3f1746e66d668.

Its completed log confirms all 60 fixed cloud Data faults were observed and
content-checked. The reviewed controller performs per-case evidence validation,
fresh image/key/challenge uniqueness checks, then independent complete-campaign
aggregation before emitting that result. Coverage is write/flush operations,
ordinals1..6, and before-cut/after-cut/error/torn-cut/short-error effects.
This is virtual-device crash-consistency evidence, not physical power-loss proof
or System update/recovery fault coverage.

Retained artifact10108193805, modern-data-faults-34359925328-1:
2,501,628 ZIP bytes; SHA256
23ff91f174dbc709973ca8b7ac9cb0b3b43d0a36ae34ea5ff803398034fb782e.
No artifact was downloaded locally. This result does not cover the later
physical-retirement source or establish final-head/release acceptance.

The separate retained reader run34350130709 successfully inspected the baseline
persistence and fixed crypto comparison reports. It confirmed the actual195-read
mount sequence and the retained288-case/864-response comparison with its Git-bound
source closure. Remaining crypto/confinement requirements are not waived.

## Integrated source checkpoint and next action

Physical retirement at6d62a066bc934736ea6507f539628e3f7531229b passed independent
source review and Foundation34358858179, Platform34358858336,
Desktop34358858199 and Specifications34358858139. These checks compile/test
the implementation but do not execute its privileged retirement path.

Bring exact reviewed trusted-main controller blobs into the continuing draft
implementation branch, retaining all target source and runtime-only tests.
No main merge or M4 release is authorized by this integration.

The next implementation outcome is the complete signed Settings path: bounded
immutable staging and reserved physical slot, manager verification, System
durable publication, isolated trial and prepared cutover, surviving peers and
verified fallback. Batch related code, tests and interface details in PR158.
Do not add authorization-only PRs, repeat the successful Data campaign unchanged,
or treat model/source checks as executable replacement proof.

All work remains GitHub-only. No Mac/SSD mutation, deletion, build or target/VM
execution. Certified disposable cloud execution only.

---

# Latest M4 checkpoint — 2026-09-09, fault-scenario source review

M4 is incomplete. This checkpoint supersedes older progress statements below;
it does not change the acceptance requirements or authorize local execution.

| Section | Evidence-backed progress | Next completion work |
| --- | --- | --- |
| M4.1 Persistent files | Retained successful two-VM Terminal/DataVault/Files proof; successful cloud fixed-corpus crypto handoff; reviewed fault-control integration and 60-case interrupted-save scenario source | Inspect retained crypto comparison evidence, complete remaining crypto/confinement cases, bind actual baseline block ordinals, validate retained fault receipts and run the actual guest fault campaign |
| M4.2 Signed live updates | Existing manifest/lifecycle models and reviewed design | Actual Settings executable staging/replacement, durable cutover, physical authority retirement, health/fallback and surviving peer apps |
| M4.3 Recovery/release | Existing recovery contract and fault infrastructure | Actual immutable System-only repair with Data unchanged, interrupted repair, full regression/reproduction/review and exact-main v0.4 release |

## New actual cloud result

Crypto run [34340985436](https://github.com/AndyTechCoder/RAR-OS/actions/runs/34340985436)
completed successfully (job102431464710), using controller
18146218af1c6f2a3dd629235cc17be0c964347b and RAR source
bfd8647b10e1daa38934b1e12363ba919938bbf7. Its controller requires two driver
constructions, two adapter compilations and the fixed 864 comparison calls
before reporting fixed-corpus success. Retained artifact10099863097 contains
6488 files, ZIP8803217 bytes, SHA256
a74400671ae13808b0755391e10018c05650312749d8a4421b923c52bc77db7c.

The retained manifest/comparison/inventory still require cloud-side inspection.
This is not full crypto interoperability acceptance, independent production
audit, current-runtime evidence, or M4.1 completion. No artifact was downloaded
locally. The earlier Docker private-client-directory and bounded memoryview
JSON defects were corrected in reviewed, tested and merged PR179/180; the
successful run used those corrections. Do not rerun the failed versions.

## Reviewed source work, not guest fault proof

Fault-control integration44765a41ebba13d8f0afafac22022a0a3b153695 passed
independent source review. Its Foundation34342143959, Platform34342144002 and
Desktop34342144020 checks passed; Specifications34342143948 was still running
when this checkpoint was written.

Scenario candidatee6212b25e16d550606fb276b597d75197b9fb41f passed focused
independent correctness/security review with no blocking source finding.
Its 60 cases cover six CREATE/WRITE write and flush boundaries, before/after
cuts, no-success errors and short/torn prefixes. Frozen authenticated state
must match exact committed/burned slots and next-slot position as well as file
contents; only then may a fresh read-only guest show the matching Files view.
Unexpected failures cannot substitute for a planned fault receipt. Inert tests
exercise orchestration without a real VM or file I/O.

Six proposed reverse-order cases were removed because this workload has one
dirty sector per flush and reversal would do nothing. Multi-sector ordering
remains a separate obligation, not claimed coverage. The scenario is not
activated. Required next work is retained-evidence validation, actual baseline
trace binding, trusted-controller integration and real faulted VM runs.

PR158 stays draft/unmerged. Preserve the released v0.3 graphical Alpha.
All repository mutation is GitHub API-only; no Mac/SSD files are created,
changed, deleted or downloaded, and no RAR OS/VM/reference adapter runs locally.

---

# Current M4 checkpoint — 2026-09-09

M4 remains incomplete. The approved delivery sections are M4.1 persistent
files, M4.2 signed live updates, and M4.3 recovery/release. This current
checkpoint supersedes the historical status text retained below. It does not
change any of the ten acceptance requirements or authorize local execution.

## What actually works

The cloud persistence diagnostic [34319265996](https://github.com/AndyTechCoder/RAR-OS/actions/runs/34319265996)
succeeded using trusted main controller
e4abb7ea966b5f65199c5f724db1633ef3dfcbb3 and target source
977aa66f8b4cc3d83370c10d88ea9763e31c11e7.
Terminal wrote the controller-generated challenge through the actual guest
DataVault path; the whole first QEMU was destroyed; a new QEMU with fresh
firmware state read the retained file through Files. Actual pixels and an
independent frozen authenticated Data-image oracle agreed. The retained
envelope also passed the positive revalidation and rejected 60 mutations.
These are envelope-corruption checks, not guest interrupted-write fault cases.

Run job102362089590 retained artifact10091254514 (7 files, ZIP223800 bytes,
SHA25686d6bfe56c595408925495f39fa413dd95d9041572fb6d12ae5c42cdfd3c52e1).
No artifact was downloaded to the Mac or SSD.

This proves that particular cross-reboot scenario. It does not establish
independent crypto interoperability, physical power-loss safety, the full
fault matrix, later runtime revisions, or M4.1 completion.

## Source validation and remaining integration

Runtime PR158 remains draft and unmerged. Source
4402f901bb0691061e48062573ffc501616abb2a passed all four cloud checks:
Specifications34323460813 (job102375185283), Foundation34323460734,
Platform34323460762 and Desktop34323460814. Actual specification logs include
the derived-image public roundtrip, compiler-owned lifecycle, bounded transport
cleanup, separate post-run diagnostics and overlapping-ELF-load regressions.
These source fixtures do not prove actual compiler or reference execution.

| Section | Verified progress | Still required |
| --- | --- | --- |
| M4.1 | Actual Terminal -> durable Data -> whole-VM destruction -> fresh-VM Files/oracle agreement; earlier reference/compiler candidate construction; source checks for handoff helpers | Real isolated RAR/OpenSSL/libsodium comparison, reproducible adapter handoff and confinement evidence; controller-owned guest write/flush/cut fault cases and remaining authority denials |
| M4.2 | Signed codec and lifecycle source tests; IRQ-budget integration source | Real executable staging/loading, live Settings replacement, durable cutover, complete physical revocation/unmap/zero-before-reuse, health/fallback and continued peer apps |
| M4.3 | Recovery contract and controlled block-fault infrastructure | Actual immutable System-only repair, identical Data hashes, interrupted repair, full fault campaign, final crypto/build/regression/review evidence and exact-main v0.4 release |

## Immediate next work

Complete one integrated cloud crypto handoff using the already-retained compiler
artifact10004371629 and reference artifact9967392023 after their fixed receipt
and independent inventory checks. Bind the exact five RAR source blobs, build
the reviewed driver and adapter twice, inspect before loading, stop the compiler
role before adapter execution, and freeze RAR results before either oracle.
Use the existing bounded runners and retain actual failure/cleanup diagnostics.
Do not treat more helper self-tests as completion of this runtime prerequisite.

The adapter-only image change is a pending source candidate, not an activated
image or a completed comparison. Afterwards complete the M4.1 guest fault cases,
then the actual M4.2 and M4.3 behavior above. Do not restart already-successful
construction or persistence runs without a concrete source/evidence reason.

All repository writes remain GitHub API operations only. No Mac/SSD file
creation, edit, deletion, download, build, packaging, mount or target/VM/reference
execution. Released v0.3.0 usable graphical Alpha is unchanged.

---

# Historical checkpoints (preserved; superseded where noted above)

# M4 runtime progress checkpoint

Incomplete development checkpoint: not v0.4.0, M4.1 completion, or Modern runtime
acceptance. Released v0.3.0 usable graphical Alpha remains unchanged. Delivery
is M4.1 persistent files, M4.2 signed live updates, M4.3 recovery/release; all ten
acceptance requirements in the milestone task remain controlling.

## Status by outcome

| Section | Implemented source | Missing runtime proof |
| --- | --- | --- |
| M4.1 Persistent files — in progress | Actual Modern kernel entry, full-width IPC/syscalls and fixed device authority; Data-only PIO service; Terminal/Files session wiring and sticky failure UI; bounded encrypted append-only snapshots; independent frozen-Data oracle; reviewed empty-image provisioner; actual reproducible UEFI kernel/service builds; controlled block backend and bounded child-process candidates | certified dual-PIO cloud profile, stack/syscall/framebuffer execution, fresh-VM persistence, bound frozen-image oracle and block-fault evidence |
| M4.2 Signed live updates — pending | Crypto, canonical manifest verification, System selector publication and incarnation/queue lifecycle model | System payload I/O, executable sealing/loading, real Settings replacement, health/fallback, stale-capability enforcement and GUI rebind |
| M4.3 Recovery and release — pending | Source-level fault tests and released M3 baseline | Immutable recovery integration, full block fault matrix, exact Data hash preservation, final reproduction/regressions/reviews and release |

No row claims running M4 behavior. Corrupt/ambiguous Data remains unavailable
without autoformat. Optional last-verified-prefix salvage is deferred.

## Integration evidence

At source33f7bd1be1d7f75077326b9ff770aaa36b6fd798:
- Specifications34228974378 passed, including full mutation checks.
- Desktop34228974302, Platform34228974334 and Foundation34228974332 passed.
- Actual Modern kernel/service entries compiled to Linux objects under the
  existing pinned cloud compiler. This was not a UEFI link/build or Modern boot.
- Independent source review of kernel integration and service/GUI/IPC wiring
  found no blocker after the persistent-warning visibility correction.

Modern source is selected separately from the released Desktop composition.
The old profiles do not acquire persistent devices. Device initialization and
guest execution still require the independently reviewed Modern cloud profile.

## Build-only tooling and runner transition

PR169 passed independent review and exact-head Specifications34235323255 at
f72b8672c7be9000854363bfcfa152034e4ba79e, then merged as
3d1d37860365f31de508bcdd1a6c027c31c1e3d5. Its exact reviewed merge tree is
dd00ae3552f221dd065ef3754e882e76d5009226. Exact-main validation34236807924 passed.

Actual Modern UEFI build34238883047 succeeded using that main controller and
reviewed source1769c12562fe9d752dcaeddf747a6222e76bad19. Both actual kernel and
service were compiled/linked twice, without the Linux compile-only fixture.
The controller confirmed identical binary hashes and bounded PE/W^X layouts,
including the128KiB service image limit. This is actual build/link evidence,
not a Modern VM boot, maximum-stack proof or encrypted runtime acceptance.

Cloud artifact10061103555 retains binaries/manifests/layouts/tool identities:
ZIP95236 bytes, SHA256947f08cef144f5b323b30a27cf7ae133397779e07865036f81a04f9a2682b0ae.
No artifact was downloaded to the Mac/SSD.

Runtime6ec7d3eee741f222273bcb3fe5757b711f6eade6 passed complete
Specifications34237072588 and Desktop34237072627, Platform34237072579,
Foundation34237072819. Actual Specifications runner was20260907.300.1, so this
now supplies full-suite operation evidence on that exact admitted hosted image.
The frozen oracle passed73 negatives,1026 marker prefixes,2 RFC vectors and2
capacity endpoints. The reviewed empty provisioner passed11 negatives,4096
fresh fixture allocations and2 independent empty-image parses.

The controlled block backend is new unactivated source. It binds one private
preconnected socket to one fixed regular synthetic disk; no listener, image
path input, device selector or VM launch. It records actual write/flush ordinals,
performs exact-count writes and fsync before flush ACK, rejects append/wrong-access descriptors before any ACK; supports bounded torn/
short/error/before/after cuts and reverse flush order, and forbids reconnecting
a volatile instance. Frozen reads require detachment and read actual file bytes.
The15-test backend candidate passed independent source review at6369251f430d92a88bb58f3c258932e9823d7a2c. Exact-head Foundation34242245906, Platform34242245927, Desktop34242245960 and full Specifications34242245898 passed; completed logs confirm all15 real file/socket tests.

The bounded backend child-process implementation at003755945731772e6a8bf5803f67f6e6ca94b4df passed independent review after fixing post-spawn constructor cleanup. Full Specifications34244394441, Foundation34244394528, Platform34244394519 and Desktop34244394508 passed. Job102122517517 logs confirm all7 actual process tests, including four injected startup-cleanup failures. This proves host-only child lifecycle, not whole-VM persistence.

New vm_profile.py and vm_session.py candidates now provide the concrete fixed paused QEMU composition, actual device/port/identity preflight, guarded QMP/input/capture path, continuously serviced backend deadlines and whole-VM-first kill/join. ADR0035 records the IDE read-only compatibility constraint: immutable boot/recovery Data retain O_RDONLY descriptors and reject every write even though the IDE-facing export advertises writable. A third private backend supplies immutable boot bytes, without an overlay or new DMA device. Eight extended process tests and pure profile/lifecycle fixtures await review/cloud validation. No Modern profile is activated; the outer trusted-main controller, image/workflow binding and causal GUI/disk proof are still required.

The follow-up review fixes add complete ten-driver/five-backend routing checks,
explicit same-model/same-address boot AHCI identity, all six boot-bus attachment
checks, measured firmware geometry, and constructor/teardown failure fixtures.
They remain unactivated and await independent review and exact-head cloud tests.

## Crypto prerequisite evidence

PR168 merged at d9cf06b87449391077f18566bffa1905fb3895bb after full source
CI34082160653 and independent review. Compiler construction34083184108 succeeded:
two no-cache candidates have identical inventories and image identity
sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914.
Artifact10004371629 retains construction evidence. This proves candidate
reproduction/inspection, not driver execution or crypto comparisons.

The driver recipe, source Git-object binding and source-layer inspector are
reviewed candidates. Actual compiler/adapter handoff and two pinned independent
reference comparisons remain prerequisites to encrypted runtime acceptance.
The new frozen-image checker does not replace either required reference.

## Next concrete actions

Finish focused review/cloud validation of the concrete block backend, then
integrate its private descriptors and fault records into the reviewed Modern
cloud controller. UEFI build tooling is complete; do not repeat its construction
as a substitute for the remaining runtime path.

Complete the separately reviewed dual-PIO cloud controller and causal
Terminal write -> entire VM destruction -> fresh firmware/VM Files read ->
independent frozen-disk agreement. Finish crypto/reference prerequisites before
accepting encrypted runtime evidence. Then deliver actual signed Settings
replacement and immutable recovery; do not relabel source tests as completion.

All repository mutations remain GitHub-only. No Mac/SSD files are created,
modified or deleted. No OS, emulator or adapter executes locally. No unreviewed
Modern VM profile is activated.

## Verified checkpoint after the first cloud diagnostic (2026-09-08)

This checkpoint supersedes the older "Next concrete actions" planning text,
not the acceptance contract. M4 remains incomplete in all three sections.

- Runtime source977aa66f8b4cc3d83370c10d88ea9763e31c11e7 passed
  Specifications34270243797, Foundation34270243824, Platform34270243879 and
  Desktop34270243822. Its signed-manifest compatibility field now requires
  exact kernel ABI1; source tests do not establish a working component loader.
- Tools PR170 merged at f441533bcb6763491ec00c9d280b0910826f5d58 after
  independent review and complete exact-head Specifications34261199628.
  Resulting-main Specifications34270145791 also passed in full.
- The first manual Modern persistence diagnostic34271876157 (job102215185348)
  used that trusted controller and fully checked source6fefabe4bac1fc967ba0c0b73743fe9c844964db.
  It failed in owned_identity on the first compiler container's pre-start
  inspection. No target build, VM start or cross-reboot persistence was proved.
  Failure artifact10074144358 has ZIP SHA256
  195e9479645b748cbf33e574695b6561210deacddb6bfb526a4596ab7784155f.
- Correction PR171 at96897b080660afad813d52ba2c1cec6f259b2d7a passed
  independent source review and full CI34272449883. It merged at
  a31b72eb3f8e18b855c642229bc0c82b64156c26; resulting-main validation
  34274475619 is pending. It binds the exact
  inherited image label map plus the reserved invocation label, preserves all
  ID/confinement/lifecycle checks and adds boolean mismatch diagnostics.
  Inherited labels are a plausible cause, not a recovered actual mismatch.
  Do not retry the unchanged failing controller.
- Existing reference construction33959130858 succeeded atacc027df17dfb2b497a78bbb41808968539b405f.
  Retained artifact9967392023 (22026442 bytes) has ZIP SHA256
  3f6113efaaa95f20807b5a0423ec710a607a88640775f36055abf715039bacbe.
  It proves candidate construction/reproduction only. Reuse it through bounded
  reviewed cloud acquisition and inventory validation, not local downloads or
  needless reference rebuilds.

### Requirement-by-requirement status

Numbers refer to the ten requirements in the active M4 contract.

| Requirement | Verified scope and remaining work |
| --- | --- |
| 1 Signed verification | Bounded codec/crypto and ABI compatibility tests exist; actual signed package and executable staging proof remain. |
| 2 Live Settings replacement | Logical lifecycle model exists; actual kernel staging, image mapping, manager transport and visible replacement remain. |
| 3 Failure/revocation | Model tests cover cap/queue/incarnation rules; actual timer budget, address-space retirement, rollback and app-continuity proof remain. |
| 4 Cross-reboot files | Real PIO/data path and cloud harness are implemented; first diagnostic stopped before execution, so persistence is unproved. |
| 5 Storage authority separation | Reviewed distinct cloud roles/descriptors and source checks exist; actual System-only recovery/Data-hash proof remains. |
| 6 Immutable recovery | Contract exists; actual repair/restart runtime and evidence remain. |
| 7 Interrupted publication | Journal/backend fault models exist; actual guest write/flush/cut/reboot campaign remains. |
| 8 Data encryption | RAR AEAD/vault implementation and source tests exist; crypto-reference closure and actual persistence acceptance remain; laboratory keys are public. |
| 9 Crypto independence | Reference/compiler candidate builds are retained; real isolated RAR/OpenSSL/libsodium comparisons remain. |
| 10 Reproduction/release | Earlier Modern UEFI two-build proof and regressions exist; final runtime/recovery/fault evidence, final-main reproduction and v0.4 release remain. |

### Immediate continuation

Validate the corrected trusted-main controller and run one corrected cloud diagnostic.
Retain actual failure or success evidence; success requires complete two-VM
pixel/block/frozen-image agreement and independent retained-envelope mutation
tests. Then finish crypto handoff/comparisons, actual signed replacement and
System-only recovery. Keep the complete original acceptance scope.

No local Mac/SSD mutation, build, packaging, image download or target/VM/reference
adapter execution has been authorized or performed by these steps.

### Completed-read boundary clarification

Inspection consumes opaque CompleteRead, which has no production constructor in
this unactivated batch. Only a test-only mock can issue one, after successful
exact-length input; partial or failed reads refuse a token. The future native
bridge must supply the independently reviewed production issuer. The planner's
matches_observations compares values only, not freshness; copied observations
cannot establish fresh I/O. Native one-shot receipt/seal/incarnation and exclusive
ownership remain required before any repair write. No claim of enforced fresh
native reads or guest repair is made by these source APIs.


### M4.3 continuation: strict registration and inspection staging preparation

Source47599's Specifications34666337685 stopped at the legacy fixed34-ADR count;
the other three checks passed. PR199 corrects optional proposed ADR0036
registration without granting native authority. Its reviewed remediation904b1fa
requires a regular non-symlink nonempty file, exact title/index/proposed status
and count34 without /35 with the proposal. Exact-main validation is still required
before that checker can validate this branch.

The prepared staging extension distinguishes non-executable whole-sector
inspection seals from executable package seals. Inspection bounds are512 through
2097664 bytes in whole sectors; executable bounds remain896 through2097536.
Neither view accepts the other purpose. Added source tests cover incomplete
copies, stale seals, zeroing, both maximum bounds and cross-purpose refusal.
No native syscall issues an inspection seal yet and CompleteRead still has no
production issuer. This is preparation, not actual recovery execution or M4.3
completion. ADR0036 now records the reviewed native bridge sequence and retains
proposed status.

The matching System read primitive inspects only the selected active or named
prior slot, with exact selector observations before and after complete-sector
streaming. Malformed headers yield one complete512-byte sector; parsed packages
include all rounded storage padding. Source tests forbid every write/flush,
cover minimum/maximum lengths, all read failures, sink-prefix failure and changed
selectors before/after copying. Every in-flight failure locks the owner without
issuing a receipt. No native caller or CompleteRead authority is added.

The pure repair classifier now consumes only complete whole-sector test receipts,
independently reconstructs framing/rounded length, rejects short or excess shapes
as missing evidence, verifies only the logical package and includes padding in
the observation hash. Tests cover valid non-sector-aligned packages, damaged
padding, malformed first sectors, short/excess receipts and maximum bounds.
No production CompleteRead issuer exists yet; these remain source preparations.


### Native inspection primitive preparation (supersedes earlier unreachable staging notes)

Prepared source now adds the distinct Inspection begin/metadata/reject syscall
routes and fixed kernel-derived immutable factory hash, with existing capability,
pointer, RO/NX mapping, purpose and scrub gates. ABI tests cover canonical bounds
and cross-purpose limits; the pure kernel test registry includes component-hash
bounds/padding/absence tests. No System/Manager Repair transaction or production
CompleteRead issuer is wired, and no new native scenario has executed.
The earlier f0143aad claim that no syscall can mint Inspection described that
earlier snapshot only; the new mechanisms require focused native-boundary review
and exact-head cloud validation before any activation or acceptance.


Prepared repair storage integration now retains the exact authorized Repair
record in Purpose::Repair, writes only the opposite slot, binds factory hash/
generation/digest and rejects kind substitution at publication. Source tests
cover all observed preparation/publication I/O failures, old selected bytes,
high-water/stale installs and invalid inputs without I/O. Prior content rejection
leaves read-only inspection possible, but transport/uncertainty remains sticky.
The native one-shot protocol and actual repair cloud scenarios are still absent;
no M4.3 completion or repaired-guest result is claimed.


### Exact-head source gate remediation

Candidate33292ba12bf4f66dcae18aee26062b1243e07c8b passed Foundation, Platform and
Desktop checks. Specifications34669503069 passed56 core tests, then correctly
refused the production library's unused unactivated repair helpers under
-Dwarnings. No warning suppression, public authority widening or test removal
is permitted. The pure planner module, repair-only journal constructors and
System inspection/repair storage mechanisms are now explicitly cfg(test) until
the native coordinator is implemented. Existing kind3 decoding remains intact;
native inspection syscall primitives remain as reviewed. Removing these gates
requires wiring and reviewing the real caller, not declaring recovery complete.

The same remediation adds repair-specific selector-race, partial-sink, stale
cancelled-token, every-copy-read-failure and maximum-slot-boundary tests in both
directions. Maximum-size storage fixtures are intentionally unauthenticated
framing tests, never native factory inputs. All tests remain enabled in cloud
test builds. These changes still require exact-head cloud validation.


The remediated e90026 source advanced through the core build and all75 kernel
tests, then the standalone kernel-library -Dwarnings step identified native-only
staging entry points and immutable-image helpers without their native caller.
The pure library wrapper now includes staging and lab_images under cfg(test);
the actual native main declares both independently and remains unchanged.
This preserves all kernel unit tests and the actual native object/build checks,
without changing syscall behavior or relaxing warnings. Exact-head validation
is still required. Specifications34669981342 is a failed run, not acceptance.


### Verified source checkpoint and next native-channel preparation

Source2ed4236eb7c1d226d52e82f2e735b3ef1e742789 passed all four checks:
Specifications34670391243, Foundation34670391286, Platform34670391123 and
Desktop34670391115. The completed logs include56 Modern core tests,75 kernel
tests and90 signed-layer tests, including the new repair copy/failure/bounds
cases. Review/evidence checkpoint is PR158 comment5643244194. This resolves
the two historical build-gating failures above, not M4.3 runtime acceptance.

The next source batch defines the separate exact128-byte RARREP01 inspection
channel: request/snapshot, full ordered hashed Record assembly, six inspection
phases with whole-sector bounds, and exact seal release messages. Pure binding
checks distinguish selected active/prior metadata from untrusted factory
metadata and reject ordinary executable-Transfer parsing. Four focused test
groups cover framing/order/identity/shape negatives. Native System/Manager
callers, opaque lease issuance and automatic repair remain to be implemented;
this codec does not produce CompleteRead or authorize writes. Exact-head tests
and independent source review are required for this new batch.


### Inspection codec cloud result and ordered-progress implementation

Exact source229e5d2ea6eacbc506fae4d7bfcd9615078c0b0a passed all four cloud checks:
Specifications34679147545, Foundation34679147518, Platform34679147519 and
Desktop34679147546. The codec source review found no concrete blocking issue;
its decoders are not native read receipts or repair authorization.

The next source checkpoint implements the Manager-side serialized inspection
progress guard: exact snapshot/record/incarnation, one outstanding phase,
strictly increasing seals, exact one-shot Release/Released progression, initial
versus fresh storage identity comparison, and permanent poison on any mismatch.
It adds six focused positive/negative test groups without changing native boot
or granting write authority. Exact-head validation and independent review for
this addition are pending at commit time.

M4.3 remains incomplete. Native System/Manager integration and the production
CompleteRead lease still need implementation, followed by actual cloud repair,
fresh-VM restart/fault evidence, final regressions/reproducibility and release
review. No host/SSD writes or target execution were used for this checkpoint.


### Ordered-progress accepted source checks and native integration candidate

Source8e510ec573e07dc3734ecd0302dab28dea472867 passed Specifications34680973843,
Foundation34680973844, Platform34680973847 and Desktop34680973845 attempt2.
Desktop attempt1 failed before guest execution because the pinned Debian
snapshot package download connection closed; one identical failed-job retry
passed. No dependency pin or safety check was changed. Independent exact-source
review found no blocking Progress defect. Its low-priority direct Released
length-coverage suggestion is included in the next integration tests.

The next candidate wires the actual signed Manager/System bootstrap Repair:
complete sealed inspections; independent kernel-derived factory hash; opaque
native receipt issuer; fresh observations; exact Repair proposal; opposite-slot
preparation/readback; and reuse of the existing health/durable-ACK/cutover path.
Core storage/planner gates are removed only for real linked callers. The pure
library still excludes the native unsafe issuer.

New tests cover the full four/six-phase System protocol with signed readback and
publication, preserved selected bytes/high-water, refusal before boot/after
success, early prepare/incorrect phase/bad peer, incomplete I/O/staging and
proposal/Release framing. They remain cloud source/model tests, not real native
health, successful repair or fresh-VM restart proof. Candidate exact-head checks
and focused independent native/unsafe review are pending at commit time.

M4.3/full M4 remain incomplete until actual cloud recovery/fault campaigns,
whole-M4 regressions/reproducibility/review and release gates pass. No release
or main merge is authorized merely by this implementation checkpoint.
