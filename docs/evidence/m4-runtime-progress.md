# M4 runtime progress checkpoint

Incomplete development checkpoint: not v0.4.0, M4.1 completion, or Modern runtime
acceptance. Released v0.3.0 usable graphical Alpha remains unchanged. Delivery
is M4.1 persistent files, M4.2 signed live updates, M4.3 recovery/release; all ten
acceptance requirements in the milestone task remain controlling.

## Status by outcome

| Section | Implemented source | Missing runtime proof |
| --- | --- | --- |
| M4.1 Persistent files — in progress | Actual Modern kernel entry, full-width IPC/syscalls and fixed device authority; Data-only PIO service; Terminal/Files session wiring and sticky failure UI; bounded encrypted append-only snapshots; independent frozen-Data oracle; empty-image provisioner candidate | Actual UEFI build/link/layout evidence, certified dual-PIO cloud profile, stack/syscall/framebuffer execution, fresh-VM persistence, bound frozen-image oracle and block-fault evidence |
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
3d1d37860365f31de508bcdd1a6c027c31c1e3d5. The merge tree exactly equals reviewed
dd00ae3552f221dd065ef3754e882e76d5009226. It adds build-only tooling; no Modern
VM or image provisioning is activated. Exact-main self-validation34236807924 is
in progress at this checkpoint. Real UEFI dispatch/results are not yet claimed.

The final source gate passed69 Modern controller negative tests and actual
bootstrap guard fixtures:3 exact admitted runner versions and7 malformed,
unknown or adjacent rejects. Actual run image was20260831.293.1. The one-literal
admission of20260907.300.1 preserves pinned OCI/tool hashes, source and read-only
mount checks; full-suite new-host operation remains unobserved, not claimed.

Runtime source1769c12562fe9d752dcaeddf747a6222e76bad19 passed full
Specifications34232725410 and Desktop34232725473, Platform34232725540,
Foundation34232725369. The independently reviewed frozen-Data oracle passed
73 negative tests,1026 marker-prefix cases,2 RFC vectors and2 exact capacity
endpoints. These are independent in-memory checks, not actual disk binding.

The new bytes-only empty-image provisioner is independently reviewed source.
It emits exactly194 sectors with two canonical identical headers and64 virgin
slots; it accepts no existing image, file or challenge. Session-local key/ID
reuse is refused even with a different nonce prefix. The fixed4096-image
session budget and independent empty-image parses are new cloud test candidates,
not claimed passed here. Future trusted-controller entropy, exclusive private
paths/inodes and writable-clone prevention remain required before activation.

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

Finish the already-running exact-main self-validation34236807924, then obtain
two actual UEFI builds of reviewed source1769c12 using merged tooling without
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
