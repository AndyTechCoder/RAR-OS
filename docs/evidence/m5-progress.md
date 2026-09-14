# M5 progress

## Latest consolidated status — 2026-09-14

M5 remains incomplete and PR204 remains draft. Do not mistake source completion
for actual guest proof or a published release.

- Full source CI passed at 5205da4c9ced22aa65d8b0c24c327696b16daeaa
  (34824000310) and corrected 932dde9bd9fceb5a9b8553509ef31bb0a15ba339
  (34825748882). These include native kernel/service object checks,
  independently linked C/signature conformance, Rust/C app tests, scoped-agent
  tests, portable-node tests and cloud evidence refusal checks.
- The independent reviewer closed the failure-receipt and negative-proof
  coverage findings. Controller-only PR207 passed exact CI34825922780 and
  merged as 807acd3a48cd59c1b8d0525ca85a1484bbcccacb. It contains no OS
  target implementation and does not establish M5 completion.
- Actual native first/fresh/node run34828014811 on that controller and target
  932dde9 failed before guest launch: service mapped143360 >131072. The failed
  receipt is retained remotely; no runtime success is claimed. The corrected
  source deduplicates control/transaction code, keeping the PE limit unchanged.
- The consolidated remaining source batch adds the fixed network-negative
  campaigns and same-data install/reject/fallback/repair/fresh-boot journey.
  See verification/m5-consolidated-campaigns.md. This batch still requires
  exact-source CI, independent review and actual confined-cloud results.

Release gates still open: actual independent-app/private-document/agent/node
proof, complete negative and integrated campaigns, final review, evidence-gated
implementation merge and v0.5.0-expansion-alpha release. No additional target
hardware, production Pal or general Internet support is implied.

## Historical checkpoint record (superseded where noted above)

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

Consolidated adapter review additionally required reserve enforcement during the
configured-but-uninstalled phase and an all-12-I/O failure campaign for explicit
installation itself. Both are now source tests. C host conformance is explicitly
not a final target undefined-symbol/link-map proof; no implicit helper-free claim.

## App kernel mechanism continuation

The next candidate adds a separate two-app binding table, Manager-only fixed
control channels, prepare/publish with stale-plan and peer checks, exact SDK
bootstrap encoding, app close/reuse and peer-loss revocation. The old ten-role
service ABI is unchanged. The new application composition is not selected by any
native entry. Its model tests and exact-source CI must pass before acceptance;
private roots, native syscall/service/GUI routing and independent executables
are still pending. See interfaces/expansion-app-lifecycle-v0.md.

## Native SDK and independent example source

The next candidate supplies Rust/C native SDK adapters over the existing int80
mechanism, documented UI/document payloads, standalone Rust Notes and standalone
C Counter entries, and pure state tests plus cloud object-only compilation.
Notes preserves edits across pending reads/writes, authenticates full service
identities and never automatically retries an accepted/uncertain write.
Counter is UI-only. These entries are not part of the shared service executable.
They still require independently linked/signed packages, native kernel/Manager/
Storage/Compositor wiring and actual cloud GUI launch/isolation/persistence proof.
No target entry is executed by Specifications. No new host tools, external target
dependencies, VM profile, native app activation or storage format is introduced.

Candidate service endpoints now consume the same SDK payloads: private document
dispatch with full identity and sequence replay refusal before Vault I/O; two
separate Compositor app surfaces with atomic commits and revocation pixel erasure.
Default native service loops still do not call these endpoints or bind apps.

The follow-on lifecycle candidate adds inactive app-aware mount, checked
same-owner/new-incarnation rebind, volatile sequence reset only on accepted new
identity, and old-recipient reply validity checks. C envelope negative tests and
stale-sender/new-surface staging regression close the focused source gaps.
Native service-loop/outbox and kernel app-root activation still remain pending.

## Exact continuation checkpoint — 2026-09-14

- Source c90c43ea0985032e5ff9f0fb4e0136ffe8b178d6 passed the complete
  [Specifications run34810372978](https://github.com/AndyTechCoder/RAR-OS/actions/runs/34810372978).
  This includes native Rust/C object compilation, pure Notes/Counter state tests,
  private-document dispatch and separate revocable Compositor surfaces.
- Source82d5093c00485ed4744afa09ff1833264dc2178c passed the primary validation
  phase of run34811348540; at this checkpoint the mutation-policy phase is still
  running. Do not infer its final conclusion from this document.
- Candidate442a4762dbb098e95debe129ab7c7c971c50ff43 has independent read-only
  source clearance conditional on exact-head CI. It adds a fixed freestanding C
  final link, two-link reproducibility, a bounded first-party ELF-to-private-PE
  converter and actual RAR PE/signature verifier checks on a public-lab signed
  Counter package. Its cloud result is not claimed before testing.
- All candidate changes remain in draft PR204. No M5 implementation merge or
  v0.5.0 release has occurred.

The concrete next implementation boundary is independent native app activation:
immutable verified package banks, private kernel roots and stack/ABI, Manager
launch/close, Storage binding and recipient-checked outbox, Compositor/launcher
routing, then actual cloud persistence, failure and authority-denial tests.
Source endpoints and signed byte verification are not substitutes for this.

M5.1 runtime negative networking campaigns, M5.3 broker/provider/profile work and
the integrated release journey remain required. No new owner input is needed
for safe source work. Never bypass a failing check, execute locally, or merge
this draft before the real acceptance evidence is complete.


### Fixed-image completion checkpoint (2026-09-14)

Source5f97512 CI34832036107 and controllerd9118fa CI34831996275 passed.
Focused independent review cleared the network-image split and controller-only
PR208 merged as6d8c7a93c4a56149e633a2650fb39446d86c2e7c. Actual native run
34834004462 stopped before guest launch because the common image mapped135168
bytes against the unchanged131072 limit. This is not M5 runtime acceptance.

Compositor/network regrouping was rejected by automated review and remains
uncommitted/unactivated. The smaller source-only alternative disables inlining
of unchanged rendering primitives to reduce duplicate code. The draft integrates
the reviewed main controller without changing image/role ownership. Actual
native apps, persistence, network/node/System journeys, final review and release
proof still gate M5 completion. No Mac/SSD activity or artifact download occurred.
