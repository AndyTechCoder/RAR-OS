# Release 0 host toolchain locks

`host-tools.manifest` declares the Class B host surface and `dependencies.r0` separates host inputs from target dependencies. `class-b-host-tools.v1` is the strict license/provenance/setup inventory for every selected platform tool group, OCI input, CI action, and orchestration boundary. The versioned `rar-host-tool-lock-v3` records are platform-specific:

- `host-tools.lock` records the proposed `aarch64-apple-darwin` inputs on the physical development Mac.
- `host-tools.x86_64-unknown-linux-gnu-ci.lock` records the executable test roots inside the Rust 1.95.0 OCI image pinned by digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`.

The macOS preparser uses the sealed-system shell, SHA-256 utility, and bounded-input helpers as the documented irreducible axiom. It bounds policy and lock files before shell `read`, rejects unknown fields, and hashes every selected non-root executable. The two closure manifests then pin the compiler driver, codegen backend, host standard/test libraries, selected bare-metal target libraries, Rust linker tools, component manifests, and SDK link stubs. They contain hashes and relative paths only—no compiler, SDK, target, firmware, or other binary payload.

macOS cannot execute the generated Mach-O through an already-open descriptor. Therefore Release 0 does not execute this local compiler closure: local accepted compile/test/planning/evidence routes refuse after verification and before `mkdir`, Rust, or linker execution. The Mac remains source/build storage. A future local route requires a separately reviewed descriptor-bound launcher or an equivalently immutable execution environment.

The Linux CI image digest plus an enforced read-only container userland is the selected tool closure. The complete CI lock is itself bound to a reviewed digest before parsing and at the shell-to-Rust handoff. The lock additionally records exact in-image paths and hashes for the shell, hasher, bounded-input tools, directory/cleanup tools, environment sanitizer, compiler, GCC driver, sysroot marker, Cargo, and Git. Rust sources and host scripts come from exact Git blobs; generated host binaries execute from `/proc/self/fd`. Hosted runner, kernel, and container engine remain explicit external non-certifying boundaries.

The Class B inventory records the upstream license and provenance source plus the exact repository setup/pin for each selected group. GNU/Debian package licenses remain available in the immutable image under the packaged copyright records; Xcode and Apple SDK use remains governed by the recorded Apple agreement. GitHub's hosted runner and container engine are explicitly `external-attested-noncertifying`: a separate host-runner job checks the observed `ubuntu-24.04` image version and passes those exact values through named job outputs to the read-only container job. The container and bootstrap refuse missing or mismatched handoff values instead of assuming GitHub propagates runner-only environment variables into a container. Those service layers are not folded into the OCI userland digest and cannot support target certification or the deferred artifact-reproducibility claim. A runner-image change fails CI until its inventory record is reviewed and updated.

Both locks keep external LLD, every QEMU backend, and both firmware inputs unavailable. `certifiable=false` remains mandatory. No command downloads or installs a tool, and no Cargo package, third-party crate, target-linked dependency, target artifact, target asset, firmware payload, or Dependency Exception Record is present.

The CI compiler-closure fields remain `none`. C2A's byte-pinned contracts are
exact-main validated, and C2B stages one main-push-only isolated observer
workflow. It runs the O001-O021 host harness before exactly one production
observation and may retain only four independently validated candidate files
through the pinned upload action. It cannot invoke the compiler, helper, target,
or update a lock, inventory, profile, gate, or readiness state. The candidate
remains untrusted until C3V re-observes and C3A separately accepts it.

The exact-set verifier is also present only as byte-pinned inactive source. It
is not invoked by local checks or workflows and has no reviewed runtime tool-pin
instance or candidate input. A later controller-bound review must supply and
pin those bytes before runtime testing; a mechanically matching candidate
remains non-reviewed and non-ready and cannot activate the CI lock or helper.

Its C1 controller test design is source-complete without adding a harness,
workflow, container launch, or execution authority. Exhaustive disposition,
precedence, injected-fault, evidence, closure-acceptance, and v1 helper-evidence
contracts are byte-bound. Runtime testing remains blocked on C3V fixtures, a
reviewed controller, fixture image, tool pins, exact subject pins, and wiring.

The deterministic predicate portion remains inactive and byte-pinned. C1 now
adds explicit instances for all 147 dispositions, 50 dual-invalid precedence
oracles, 12 fault injections, and canonical evidence/verdict schemas. Those are
contracts rather than runtime proof; C3V controller implementation, fixtures,
retained evidence, and wiring remain separate reviewed work. The diagnostic input domain is fixed by
the separate inactive contract described next.

The separate inactive input-domain contract fixes the controller-owned
environment, mount, file, token, mutation-target, and isolation domains without
creating fixtures or execution authority. The draft C1 contract set now binds
the constructible cases, dual-invalid oracles, injected faults, and
evidence/verdict grammar. Controller source, immutable fixtures, runtime
evidence, and workflow wiring remain blocked behind C3V/C3A review.
