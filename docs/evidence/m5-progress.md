# M5 progress

Baseline: main 996acacaf9ed80290ea95a4a9ac238cfe5023c33.
M4 release: v0.4.0-modern-alpha (preserved).

- M5.1: candidate codec, channel, NE2000 driver and bounded service implemented in source.
  No NIC, service or cloud peer transport is active; no guest networking proof.
- M5.2: candidate standalone Rust protocol SDK implemented; native SDK packaging,
  C bindings, independent apps and runtime integration remain unimplemented.
- M5.3: agent broker, profiles and integrated Alpha acceptance not yet implemented.
- M5 release: NOT COMPLETE; no new runtime claim or release publication.

Next: cloud conformance for the codec, focused independent review, then concrete
isolated NIC/service/controller integration on the same implementation branch.
Existing M1–M4 functionality must remain reproducible. Source/model success is
not substituted for networking, SDK, agent or profile guest evidence.

## Initial source review

Independent read-only review of bb4490751a2825b4fe198dacf8e6477aac6053f4
identified one grant-expiry watermark defect. The consolidated correction retains
expired clock observations, tests an expiry20 -> stale13 sequence with unchanged
budgets, and adds a fixed frame, padding boundary, checksum KAT and513-length
cross-language conformance. Revised cloud results remain pending until recorded.
No source review result activates a network device or closes M5.1.

## Codec checkpoint accepted for further implementation

Exact head0676b2906e40f3e9a1fe5a817c74c7c3ad450870 passed full cloud
Specifications run34745592948. Independent source re-review found no remaining
blocking codec finding. This is source/conformance evidence, not guest networking.

The next change implements service-owned packet/grant coupling, full wire-byte
budgets, four-entry send/receive queues, revocation/expiry cleanup, copied ingress
and single-attempt transmission. Its native adapter and NIC remain unimplemented.
No additional guest or host authority is activated by pure channel tests.

## Native integration candidate

Independent review identified physical slot7 as reserved for Settings rollback;
networking must use an explicit distinct physical/logical task mapping.
The NE2000 candidate now implements actual register/ring/PIO driver logic through
an Io trait, with model tests and no native port adapter or activated profile.
Channel remediation makes driver uncertainty and ingress exhaustion sticky,
clearing pending traffic. Exact revised source checks/review remain required.

## Driver and service composition checkpoint

Driver correction bc5e4221f24ff72a5694ac30c1c46393569c3914 fixes reset register
page selection and adds direction-aware register transcripts, per-operation
I/O fault injection, stalled clocks/transfers and exact ring boundaries.
Independent source review found prior findings closed. Cloud run34762233569
passed primary validation; its full conclusion must be checked before acceptance.

Candidate service841baf0e5194ebe85aa6cba76dfb9fc18e4ce239 plus test follow-up
8661d5cd7115aa36376497b74c76834c3049fca7 connects driver and channel ownership,
fixed128-byte IPC, bounded receive/transmit polling, idle expiry, and sticky
failure cleanup. Independent source review found no blocker, conditional on
exact-commit cloud checks. No kernel dispatcher or native device is activated.

SDK candidate7d94e0d60c34f5b902a297bd14cc2fe68838dde7 extracts a standalone no_std
Rust client, exact reply validation against kernel-supplied peer/incarnation,
bounded outstanding requests and no automatic retries. Native SDK/apps and
guest evidence remain outstanding. Source review and exact CI are pending
until separately recorded; do not infer pass from this progress entry.

### Next concrete integration boundary

Implement the explicit logical principal7 -> physical slot10 composition,
distinct Network capability and typed syscall/PIO adapter, native dispatch and
bounded scheduler integration, plus the independently checked closed two-guest
cloud profile. Settings physical slots5/7 and M4 Data/System formats stay intact.
All native composition pieces must be reviewed together before activation.
Then obtain actual causal guest exchange evidence; model tests cannot close M5.1.
