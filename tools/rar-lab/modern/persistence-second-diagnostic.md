# Modern second persistence diagnostic

Run34276025198/job102229117475 used reviewed main controller
a31b72eb3f8e18b855c642229bc0c82b64156c26 and checked runtime
977aa66f8b4cc3d83370c10d88ea9763e31c11e7. It reached the retained-evidence
checker after the runtime container completed successfully.

The checker passed its canonical envelope, frozen Data authentication/content,
initial image/System hash and all five actual pixel-frame checks. It then
rejected the first VM proof at the exact-single-RESUME event-list requirement.
Remaining topology/command/backend proof revalidation had not completed; this
run does not establish accepted persistence, crypto closure or M4 completion.

Artifact10075764053 retains six files, including the actual persistence envelope:
ZIP214680 bytes, SHA256
caa037ab8ff83320c9b4d88b62520d0d867d858e525c62c0f18a4475ea715af0.
Nothing was downloaded to the Mac or SSD.

The next correction adds only bounded failure diagnostics to that unchanged
rejection path: event count, at most32 names from a fixed public vocabulary,
and a truncation flag. Unknown names/objects are replaced by OTHER; arbitrary
event fields and values are not printed. The exact-one-RESUME requirement,
timestamp/shape checks, preflight, input plan, confinement, backend audit,
cryptographic checks and acceptance flags remain unchanged.

This does not guess whether RESET, a duplicate event or an empty event list
caused the failure. A subsequent reviewed cloud run must expose the actual
mismatch before any justified event-contract correction. Diagnostic names do
not establish event ordering, provenance or acceptance.
