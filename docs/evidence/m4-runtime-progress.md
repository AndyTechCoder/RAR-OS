# M4 runtime progress checkpoint

This is an incomplete development checkpoint, not v0.4.0 or runtime acceptance.
The released v0.3.0 usable graphical Alpha remains unchanged.

## Implemented source, still requiring runtime integration

- RAR crypto, canonical signed Settings manifest verification and System selector.
- Lifecycle/incarnation/queue replacement model.
- Verified bounded PIO transport and its explicit-flush vault connection.
- Encrypted append-only DataVault with crash recovery and second-reboot tests.
- Alternate System selector publication with flush/readback, uncertain-write
  refusal, protected-record checks, fallback and fault tests.

These pieces are unactivated. Model/source tests are not guest evidence.
Corrupt or ambiguous Data remains unavailable without autoformat; optional
last-verified-prefix salvage is deferred and is not needed for M4.

## Critical remaining path

1. Reproduce and independently inspect the corrected pinned compiler candidate.
   PR168 is merged at d9cf06b87449391077f18566bffa1905fb3895bb after full
   source CI34082160653 and independent review. Construction34083184108 was
   dispatched once; its outcome must be checked, not assumed.
2. Complete the separate reference-free RAR adapter compiler/runtime handoff
   and real comparisons with both pinned crypto references.
3. Integrate Modern kernel/process sealing, fixed independent System/Data
   adapters, storage services, replacement Settings and update/health handover.
4. Review and activate the concrete cloud-only Modern device/controller profile.
5. Prove real fresh-VM persistence, live update, tamper rejection, fallback,
   interrupted commit and immutable recovery with preserved Data hashes.
6. Reproduce images, pass retained regressions and final independent review,
   then exact-main evidence and release.

No Mac/SSD files are created, modified or deleted. No OS, emulator or adapter
executes locally. No unreviewed Modern VM profile has been activated.
