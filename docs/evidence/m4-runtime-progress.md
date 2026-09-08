# M4 runtime progress checkpoint

Incomplete development checkpoint: not v0.4.0, M4.1 completion, or Modern runtime
acceptance. Released v0.3.0 usable graphical Alpha remains unchanged. Delivery
is M4.1 persistent files, M4.2 signed live updates, M4.3 recovery/release; all ten
acceptance requirements in the milestone task remain controlling.

## Status by outcome

| Section | Implemented source | Missing runtime proof |
| --- | --- | --- |
| M4.1 Persistent files — in progress | Actual Modern kernel entry, full-width IPC/syscalls and fixed device authority; Data-only PIO service; Terminal/Files session wiring and sticky failure UI; bounded encrypted append-only snapshots; independent frozen-Data oracle; reviewed empty-image provisioner; actual reproducible UEFI kernel/service builds; controlled block backend and bounded child-process candidates | certified dual-PIO cloud profile, stack/syscall/framebuffer execution, fresh-VM persistence, bound frozen-image oracle and block-fault evidence |
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
3d1d37860365f31de508bcdd1a6c027c31c1e3d5. Its exact reviewed merge tree is
dd00ae3552f221dd065ef3754e882e76d5009226. Exact-main validation34236807924 passed.

Actual Modern UEFI build34238883047 succeeded using that main controller and
reviewed source1769c12562fe9d752dcaeddf747a6222e76bad19. Both actual kernel and
service were compiled/linked twice, without the Linux compile-only fixture.
The controller confirmed identical binary hashes and bounded PE/W^X layouts,
including the128KiB service image limit. This is actual build/link evidence,
not a Modern VM boot, maximum-stack proof or encrypted runtime acceptance.

Cloud artifact10061103555 retains binaries/manifests/layouts/tool identities:
ZIP95236 bytes, SHA256947f08cef144f5b323b30a27cf7ae133397779e07865036f81a04f9a2682b0ae.
No artifact was downloaded to the Mac/SSD.

Runtime6ec7d3eee741f222273bcb3fe5757b711f6eade6 passed complete
Specifications34237072588 and Desktop34237072627, Platform34237072579,
Foundation34237072819. Actual Specifications runner was20260907.300.1, so this
now supplies full-suite operation evidence on that exact admitted hosted image.
The frozen oracle passed73 negatives,1026 marker prefixes,2 RFC vectors and2
capacity endpoints. The reviewed empty provisioner passed11 negatives,4096
fresh fixture allocations and2 independent empty-image parses.

The controlled block backend is new unactivated source. It binds one private
preconnected socket to one fixed regular synthetic disk; no listener, image
path input, device selector or VM launch. It records actual write/flush ordinals,
performs exact-count writes and fsync before flush ACK, rejects append/wrong-access descriptors before any ACK; supports bounded torn/
short/error/before/after cuts and reverse flush order, and forbids reconnecting
a volatile instance. Frozen reads require detachment and read actual file bytes.
The15-test backend candidate passed independent source review at6369251f430d92a88bb58f3c258932e9823d7a2c. Exact-head Foundation34242245906, Platform34242245927, Desktop34242245960 and full Specifications34242245898 passed; completed logs confirm all15 real file/socket tests.

The bounded backend child-process implementation at003755945731772e6a8bf5803f67f6e6ca94b4df passed independent review after fixing post-spawn constructor cleanup. Full Specifications34244394441, Foundation34244394528, Platform34244394519 and Desktop34244394508 passed. Job102122517517 logs confirm all7 actual process tests, including four injected startup-cleanup failures. This proves host-only child lifecycle, not whole-VM persistence.

New vm_profile.py and vm_session.py candidates now provide the concrete fixed paused QEMU composition, actual device/port/identity preflight, guarded QMP/input/capture path, continuously serviced backend deadlines and whole-VM-first kill/join. ADR0035 records the IDE read-only compatibility constraint: immutable boot/recovery Data retain O_RDONLY descriptors and reject every write even though the IDE-facing export advertises writable. A third private backend supplies immutable boot bytes, without an overlay or new DMA device. Eight extended process tests and pure profile/lifecycle fixtures await review/cloud validation. No Modern profile is activated; the outer trusted-main controller, image/workflow binding and causal GUI/disk proof are still required.

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

Finish focused review/cloud validation of the concrete block backend, then
integrate its private descriptors and fault records into the reviewed Modern
cloud controller. UEFI build tooling is complete; do not repeat its construction
as a substitute for the remaining runtime path.

Complete the separately reviewed dual-PIO cloud controller and causal
Terminal write -> entire VM destruction -> fresh firmware/VM Files read ->
independent frozen-disk agreement. Finish crypto/reference prerequisites before
accepting encrypted runtime evidence. Then deliver actual signed Settings
replacement and immutable recovery; do not relabel source tests as completion.

All repository mutations remain GitHub-only. No Mac/SSD files are created,
modified or deleted. No OS, emulator or adapter executes locally. No unreviewed
Modern VM profile is activated.
