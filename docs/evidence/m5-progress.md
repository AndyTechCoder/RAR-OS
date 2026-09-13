# M5 progress

Controller main: a93b7e870e80a7a603a0c15dca017e5d8dd6958c.
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

The next capture enhancement adds independent full Ethernet/IPv4/UDP packet
comparison and a no-extra-transmissions check. Its code is a candidate until
its own review, source CI and actual cloud proof pass.
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

1. Finish captured-wire comparison and the actual malformed/flood/drop/expiry/
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
