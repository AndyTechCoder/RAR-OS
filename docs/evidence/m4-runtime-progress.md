# M4 runtime progress checkpoint

Incomplete development checkpoint: not v0.4.0, M4.1 completion, or Modern runtime
acceptance. Released v0.3.0 usable graphical Alpha remains unchanged. Delivery
is M4.1 persistent files, M4.2 signed live updates, M4.3 recovery/release; all ten
acceptance requirements in the milestone task remain controlling.

## Status by outcome

| Section | Implemented source | Missing runtime proof |
| --- | --- | --- |
| M4.1 Persistent files — in progress | Actual Modern kernel entry, full-width IPC/syscalls and fixed device authority; Data-only PIO service; Terminal/Files session wiring and sticky failure UI; bounded encrypted append-only snapshots; independent frozen-Data oracle candidate | Actual UEFI build/link/layout evidence, certified dual-PIO cloud profile, stack/syscall/framebuffer execution, fresh-VM persistence, bound frozen-image oracle and block-fault evidence |
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

Draft PR169 adds two independent real UEFI builds using the reviewed pinned
Desktop toolchain recipe and unchanged networkless Foundation sandbox. It does
not start a VM, package disks, execute target binaries or grant device authority.
Its exact-head build tooling and focused overlap-test/evidence corrections are
independently reviewed. The69 pure controller tests passed in34230717029.

That run later failed the legacy host-safety runner guard on newly published
Ubuntu24 image20260907.300.1; the successful integration run used20260831.293.1.
The narrow admission of exactly20260907.300.1 was independently reviewed in
863f5bbe2704d34109fa28a7f3af4b217be7e106. OCI/executable hashes, read-only mounts,
source checks and unknown-version rejection remain. Corrected source validation
34232075836 is pending at this writing. No tooling merge/dispatch is claimed.

The frozen-Data oracle has independent byte framing and a bounded public-lab
AEAD reader, with no target codec/crypto imports, filenames or write operations.
Its source is a review candidate; in-memory fixtures are not real disk evidence.

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

Finish PR169 exact-head validation/review, merge only that tooling and verify
main, then obtain two actual UEFI builds of the reviewed Modern source without
activating the VM profile. Preserve the128KiB service image budget and report
stack headers as diagnostics, not maximum-stack proof.

Complete the separately reviewed dual-PIO cloud controller and causal
Terminal write -> entire VM destruction -> fresh firmware/VM Files read ->
independent frozen-disk agreement. Finish crypto/reference prerequisites before
accepting encrypted runtime evidence. Then deliver actual signed Settings
replacement and immutable recovery; do not relabel source tests as completion.

All repository mutations remain GitHub-only. No Mac/SSD files are created,
modified or deleted. No OS, emulator or adapter executes locally. No unreviewed
Modern VM profile is activated.
