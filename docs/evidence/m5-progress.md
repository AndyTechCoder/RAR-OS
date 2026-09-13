# M5 progress

Baseline: main 996acacaf9ed80290ea95a4a9ac238cfe5023c33.
M4 release: v0.4.0-modern-alpha (preserved).

- M5.1: candidate bounded network codec and policy implementation underway.
  No NIC, service or cloud peer transport is active; no guest networking proof.
- M5.2: SDK, independent apps and runtime integration not yet implemented.
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
