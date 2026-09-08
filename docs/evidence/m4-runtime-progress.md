# M4 runtime progress checkpoint

This is an incomplete development checkpoint, not v0.4.0 or runtime acceptance.
The released v0.3.0 usable graphical Alpha remains unchanged. The owner-approved
delivery is now M4.1 persistent files, M4.2 signed live updates, M4.3 recovery and
release; the complete acceptance contract remains in the milestone task.

## Status by outcome

| Section | Implemented source | Missing runtime proof |
| --- | --- | --- |
| M4.1 Persistent files — in progress | PIO transport, explicit-flush vault bridge, encrypted append-only snapshots, durable file-service adapter, correlated IPC, bounded app-session loop, crash and second-reboot source tests | Kernel device authority/syscalls, actual service/UI wiring, reviewed cloud disks, fresh-VM persistence and independent frozen-Data oracle |
| M4.2 Signed live updates — pending | Crypto, canonical signed manifest verification, System selector publication, incarnation/queue lifecycle model | System payload I/O, executable sealing/loading, real Settings replacement, health/fallback and stale-capability enforcement |
| M4.3 Recovery and release — pending | Source-level fault tests and existing M3 release baseline | Immutable recovery integration, full block fault matrix, Data hash preservation, target reproduction, retained regressions and final release evidence |

None of these rows claims running M4 behavior. Corrupt/ambiguous Data remains
unavailable without autoformat; optional last-verified-prefix salvage is deferred.

## Confirmed prerequisite evidence

PR168 merged at d9cf06b87449391077f18566bffa1905fb3895bb after full source
CI34082160653 and independent review. Compiler construction34083184108 succeeded:
two no-cache candidates have identical complete inventories and image identity
sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914.
Retained artifact10004371629 contains the construction evidence. This proves
candidate reproduction/inspection, not driver execution or crypto comparisons.

The private driver recipe, source Git-object binding and source-layer inspector
are reviewed source candidates. Actual separate compiler/adapter handoff and
two-reference interoperability remain prerequisites to encrypted runtime
acceptance, not evidence that the persistent-files section has completed.

## Next concrete implementation

The durable file-service adapter and correlated IPC have passed the primary
cloud source validation at checkpoints 2ec1e8cd and 4d32a364. Full run34215539991
passed at 2ec1e8cd; the later full run remains pending at this writing. The new
bounded app-session loop is a source candidate awaiting its cloud checks.
These are not runtime persistence evidence.

Connect these components to the actual Terminal and Files paths in the distinct
Modern composition, with durable acknowledgement and explicit unavailable,
read-only and uncertain-save UI behavior. Complete the kernel-mediated fixed Data
transport and reviewed cloud profile alongside it. Do not change released
Desktop-v0 into a persistent profile or inject reconstructed contents on boot.

Keep source CI and independent boundary review, but measure completion by the
boot1 write -> full VM destruction -> boot2 read demonstration and independent
disk evidence specified in the task. Later sections retain every original M4
security, update, fault, recovery and release requirement.

All repository mutations remain GitHub-only. No Mac/SSD files are created,
modified or deleted. No OS, emulator or adapter executes locally. No unreviewed
Modern VM profile is activated.
