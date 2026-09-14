# M5 progress

Controller main: 8787645fd82e945dcb426a8872c826ec7059c0b2.
Published M4 remains v0.4.0-modern-alpha, unchanged.
Working branch: codex/m5-expansion; implementation PR: #204.

**M5 is incomplete. The confined paired Expansion profile is activated and its
first actual bidirectional guest test passed. No v0.5.0 release is published.** Source review, source CI, guest evidence,
merge and release are separate states. Do not translate source-test success
into a runtime acceptance claim.

## Implemented source

- M5.1: RAR Ethernet/IPv4/UDP codec, grant/budget channel, bounded NE2000 PIO
  driver and isolated-service logic; distinct kernel Network capability and
  scalar PIO syscall; logical network principal7 mapped to physical slot10,
  preserving Settings slots5/7; gated native service composition.
- Network authority is denied after Terminal, Manager or System-service death,
  recovery escalation, stale incarnation or revocation. Driver uncertainty
  closes the channel; kernel reconciliation stops the device once.
- Closed two-guest cloud profile and paired lifecycle: private UNIX datagram
  socketpair only, exact NIC/QOM/I/O inventory, six-way image identity,
  explicit descriptor inheritance, both-paused preflight, owner-gated pair
  start, fair watchdog servicing and aggregate teardown. Actual pair proven below.
- Native Terminal network tool: NET SEND/RECV/CLOSE through the Rust SDK,
  bounded waits, exact reply identity, one send/no automatic retry, preserved
  keyboard input and bounded printable rendering. Actual guest exchange proven below.
- M5.2: first-party standalone no_std Rust and freestanding C network wire
  SDKs, framing/ownership contracts and cross-language byte tests.
  These are not a stable general application ABI or packaged example apps.

## Retained exact-source cloud results

| Source | Specifications run | Result |
| --- | --- | --- |
| 0676b2906e40f3e9a1fe5a817c74c7c3ad450870 | 34745592948 | Passed codec checkpoint |
| 287887ddc183f7d2939d3194658b87f07f4a1321 | 34760243466 | Passed channel checkpoint |
| bc5e4221f24ff72a5694ac30c1c46393569c3914 | 34762233569 | Passed driver checkpoint |
| 58f2290c7ba06dd950649e8eefd5f79573b6d05f | 34763120748 | Passed service/Rust SDK checkpoint |
| eb555649927ebbdc5060d4da418cdc9a8a4c248b | 34764620665 | Passed gated kernel object integration |
| 157ec29c39445c94f748042a4bf615d547f691f2 | 34765667196 | Passed client/control-plane revocation fix |

Additional exact-source CI: 1c4569e0d314e8e1a9ba1c26f4fd0257da5de1c9 passed
Specifications run 34770438147. Controller-only PR #205 merged after independent
review and exact-head CI 34770910627. Exact main a93b7e passed 34772014599.

Actual paired-cloud run [34772050322](https://github.com/AndyTechCoder/RAR-OS/actions/runs/34772050322)
passed on controller a93b7e and target 1c4569e. Both peer UEFI images rebuilt twice
identically. Eleven actual pixel scenes proved fresh A-to-B and B-to-A messages,
channel close/retirement and working Files; Data/System remained unchanged and
both guests plus six backends were reaped. Artifact 10321904618, ZIP SHA256
761d39be06f4f1f51e9022d2c1aa0fea1e6d7c63770075860bb3ae44f869b024.
This was positive networking evidence, not runtime fault or full M5 acceptance.

Captured-wire extension passed independent review and controller PR #206.
Exact controller 371226758e965c0c2a4e96a9b7af4030425d334f passed run 34773565172;
source f3dc3c3b699169835e8d888476b26b0bb23716f1 passed 34773754100.
Exact main 8787645 passed Specifications 34774701012 (confirmed 2026-09-14).

Actual captured-wire paired run [34774843591](https://github.com/AndyTechCoder/RAR-OS/actions/runs/34774843591)
passed on that controller/main and source f3dc3c3. Both peer images rebuilt twice
identically; eleven GUI scenes, fresh bidirectional messages, two exact 74-byte
frames in each capture, no extra post-close transmission, unchanged storage and
complete teardown passed. Remote artifact 10322769257, ZIP SHA256
cc93314acef029c507b66ed74f9c8b9c74ab1a6120b95f38a0a25743ad2f8885.
Artifacts remain remote and were not downloaded. This is not the runtime
malformed/flood/drop/expiry/revocation/peer-death campaign.

New source candidate: ADR0038 app signing envelope and private-document policy,
with bounded package fixtures and cloud CI registration. This commit's CI is not
claimed before it runs. No independent app is installed/executed by these tests;
native SDK, installer, Storage routing and runtime isolation remain pending.
The immutable CI receipts in PR #204 record subsequent run conclusions.
This document intentionally does not claim its own commit was tested before
that commit exists. No boot, UEFI linking, socket or PIO runtime claim follows
from the table.

## Independent source review

One reused read-only reviewer checked the codec, channel, driver/service, SDK,
kernel mapping/PIO boundary and integrated candidates. Corrections include:
expiry watermark retention; reset-page selection; physical/logical role
separation and unpublished-task protection; control-plane dependency loss;
six-way disk identity; actual inert constructor-path coverage; pair-owned start
and aggregate teardown; closed-channel acknowledgement ordering.

Review of 93dd30de1b2a751051e3a97180871b536ea7ef60 closed the concrete cloud-pair
findings and found no blocking native-client source issue, conditional on CI.
Review of eee396bb3d0a398a540674909ca61c67408c8239 found no blocking C SDK source
issue, conditional on CI. Neither review grants runtime activation or M5 closure.

## Remaining acceptance — not polish

1. Finish the actual malformed/flood/drop/expiry/
   revocation/peer-death negative campaigns. Positive paired exchange, exact
   linking/reproducibility and aggregate teardown are already proven above.
2. General experimental app contract, signed/installed independent Rust and C
   examples launched from the GUI, private persistent document, useful notes
   app and contained app-failure/data/device-denial proofs.
3. Provider-neutral agent broker and deterministic test provider, actual guest
   allowed/denied/revoked actions without ambient authority.
4. Bounded portable headless node and two graphical presentation/resource
   profiles, state-preserving transition and resource evidence.
5. Integrated M1–M5 journey, two-build reproducibility, security-critical
   regressions, final review, exact-main merge proof and experimental release.

No local files, SSD activity, builds, guest execution, artifact downloads or
cleanup are permitted. All writes remain GitHub API repository changes; actual
target execution remains confined to separately reviewed disposable cloud VMs.

## Consolidated app continuation (2026-09-14)

The next source candidate fixes both review gaps: full document capacity is
reserved at install and protected against later shared writes; signed-positive
manifest prefix/digest mutations and short/long framing are now tested.
The Store adapter adds explicit private install/read/write/revoke, atomic Vault
publication, shared-data projection and fresh-mount/fault-injection tests.
The default native Storage entry still selects the original shared-only mount.
No new disk format or runtime authority is activated.

A distinct 256-byte app bootstrap and 128-byte message contract now have first-party
Rust/C codecs, generated constant drift checks and streamed cross-language tests.
These are candidate bindings, not independently installed native applications.
The exact 64-byte, one-private-document limit remains explicit.

Pending before M5.2 completion: native kernel/Manager installer and app lifecycle,
SDK native entry/syscalls and useful Rust/C examples, GUI routing, actual guest
persistence, unrelated-device/Data denial and contained failure. M5.1 negative
campaigns and M5.3/integrated release remain open. No completion percentage is
inferred from this source batch.
