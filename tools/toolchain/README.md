# Release 0 host toolchain locks

`host-tools.manifest` declares the Class B host surface and `dependencies.r0` separates host inputs from target dependencies. `class-b-host-tools.v1` is the strict license/provenance/setup inventory for every selected platform tool group, OCI input, CI action, and orchestration boundary. The versioned `rar-host-tool-lock-v3` records are platform-specific:

- `host-tools.lock` records the proposed `aarch64-apple-darwin` inputs on the physical development Mac.
- `host-tools.x86_64-unknown-linux-gnu-ci.lock` records the executable test roots inside the Rust 1.95.0 OCI image pinned by digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`.

The macOS preparser uses the sealed-system shell, SHA-256 utility, and bounded-input helpers as the documented irreducible axiom. It bounds policy and lock files before shell `read`, rejects unknown fields, and hashes every selected non-root executable. The two closure manifests then pin the compiler driver, codegen backend, host standard/test libraries, selected bare-metal target libraries, Rust linker tools, component manifests, and SDK link stubs. They contain hashes and relative paths only—no compiler, SDK, target, firmware, or other binary payload.

macOS cannot execute the generated Mach-O through an already-open descriptor. Therefore Release 0 does not execute this local compiler closure: local accepted compile/test/planning/evidence routes refuse after verification and before `mkdir`, Rust, or linker execution. The Mac remains source/build storage. A future local route requires a separately reviewed descriptor-bound launcher or an equivalently immutable execution environment.

The Linux CI image digest plus an enforced read-only container userland is the selected tool closure. The complete CI lock is itself bound to a reviewed digest before parsing and at the shell-to-Rust handoff. The lock additionally records exact in-image paths and hashes for the shell, hasher, bounded-input tools, directory/cleanup tools, environment sanitizer, compiler, GCC driver, sysroot marker, Cargo, and Git. Rust sources and host scripts come from exact Git blobs; generated host binaries execute from `/proc/self/fd`. Hosted runner, kernel, and container engine remain explicit external non-certifying boundaries.

The Class B inventory records the upstream license and provenance source plus the exact repository setup/pin for each selected group. GNU/Debian package licenses remain available in the immutable image under the packaged copyright records; Xcode and Apple SDK use remains governed by the recorded Apple agreement. GitHub's hosted runner and container engine are explicitly `external-attested-noncertifying`: a separate host-runner job checks the observed `ubuntu-24.04` image version and passes those exact values through named job outputs to the read-only container job. The container and bootstrap refuse missing or mismatched handoff values instead of assuming GitHub propagates runner-only environment variables into a container. Those service layers are not folded into the OCI userland digest and cannot support target certification or the deferred artifact-reproducibility claim. A runner-image change fails CI until its inventory record is reviewed and updated.

Both locks keep external LLD, every QEMU backend, and both firmware inputs unavailable. `certifiable=false` remains mandatory. No command downloads or installs a tool, and no Cargo package, third-party crate, target-linked dependency, target artifact, target asset, firmware payload, or Dependency Exception Record is present.

The CI compiler-closure fields remain `none`. A byte-pinned, source-only
observer and its experimental contract are checked in for later review, but are
not wired to a workflow and carry no execution authority. A future explicitly
authorized cloud observation may emit only a candidate manifest and a
not-ready receipt; the lock cannot change until the observer tools, complete
file set, and verifier are independently reviewed and pinned.

The exact-set verifier is also present only as byte-pinned inactive source. It
is not invoked by local checks or workflows and has no reviewed runtime tool-pin
instance or candidate input. A later controller-bound review must supply and
pin those bytes before runtime testing; a mechanically matching candidate
remains non-reviewed and non-ready and cannot activate the CI lock or helper.

Its inactive controller test design records an initial fail-closed risk catalog
without adding a harness, workflow, container launch, or execution authority.
It is intentionally incomplete and cannot produce an acceptance verdict.
Runtime testing remains blocked on separate complete error/precedence/fault and
evidence contracts, fixtures, a reviewed controller, fixture image, tool pins,
exact subject pins, and a later reviewed workflow change.

The representable deterministic predicate portion is now represented by
inactive byte-pinned validation, occurrence-qualified error-template, and
class-located source-order catalogs. Their 32-stage adjacent prefix is derived
from the declared control-flow order, but it is not an executable dual-invalid
oracle or fixture-reachability claim. The constructible runtime precedence and
runtime fault injection, explicit field/value case instances, canonical
evidence, and normalized verdicts remain absent and continue to block
controller implementation or wiring. The diagnostic input domain is fixed by
the separate inactive contract described next.

The separate inactive input-domain contract now fixes the controller-owned
environment, mount, file, token, mutation-target, and isolation domains without
creating fixtures or execution authority. Explicit field/value cases,
constructible dual-invalid oracles, injected faults, evidence/verdicts,
controller source, and workflow wiring remain blocked.
