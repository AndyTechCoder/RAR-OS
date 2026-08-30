# ADR 0024: Alpha Controller Helper Build Trust

Status: Accepted — 2026-08-29
Decision: Alternative A

Approval basis: explicit owner approval of the repository's exact five-choice
sentence on 2026-08-29. Acceptance selects the time-bounded Alpha helper build
trust path; it grants no compiler identity, credentials, provisioning,
execution, readiness, or merge authority by itself.

## Context

Accepted ADRs 0017 and 0020 require the trusted outer controller to copy
untrusted role output through a descriptor-based stop/open/copy/recheck
primitive. The reviewed contract and dependency-free Rust codec source now
exist, but no executable helper is authorized. Running a compiler selected from
the Mac or runner `PATH`, trusting a version string without a digest, or locally
executing changed repository tests would violate the host-tool and Mac-safety
rules.

The helper must run on the outer Linux controller, before the launch role, so it
cannot simply live inside the isolated build, reference, or launch image. Its
build identity is therefore a controller trust-chain choice, not an ordinary
implementation detail.

## Decision drivers

- Never compile or execute this changed host helper on the Mac.
- Bind every executable compiler and helper byte used by the controller.
- Preserve the three disjoint runtime roles selected by ADR 0020.
- Keep untrusted target source away from controller-helper build authority.
- Require two reproducible helper builds and independent conformance evidence.
- Avoid turning an Alpha host tool into a shipped RAR OS dependency.

## Considered options

### A. Build twice on the approved runner with a pinned compiler closure

The trusted default-branch controller downloads or restores an exact
digest-pinned Linux Rust compiler closure, verifies every file before use,
compiles the helper twice in fresh bounded directories with networking disabled,
and requires byte-identical binaries. This is the smallest Alpha change and
keeps the helper outside target images, but the outer runner temporarily holds
compiler authority and the closure acquisition/verification path becomes part
of acceptance evidence.

### B. Store a prebuilt helper binary in the repository

Commit one reviewed Linux helper binary beside its source, digest, provenance,
and two-build reproduction evidence. Probe runs only verify and execute those
bytes. This removes probe-time compiler authority, but makes a binary artifact
part of ordinary source review and requires a separate trusted process whenever
the helper changes.

### C. Build in a distinct pinned controller-tool image

Use a fourth, trusted build-only image containing only a digest-pinned Linux
Rust closure. It receives the trusted controller-helper source, never the
untrusted target checkout, builds twice in fresh network-disabled instances,
and emits one byte-identical helper plus build evidence. The outer controller
copies and verifies that helper before any target role starts. This gives the
strongest separation and clearest production migration, but requires another
image, inventory, reproduction proof, and provisioning step before Milestone A.

## Decision

Select Alternative A for the time-boxed Alpha. It preserves the accepted three
runtime roles and can later migrate to Alternative C without changing the
handoff manifest or any RAR target interface. The runner may use the compiler
only after its complete closure, license, provenance, and digest are reviewed;
both fresh builds must be byte-identical; the helper source and binary hashes
must be retained; compilation and tests must run only in the approved isolated
Linux cloud job; and no compiler or helper enters a RAR OS image.

This decision grants no cloud credential, provisioning, deployment, VM launch,
Mac execution, target compilation, or merge authority.

## Consequences

- The controller profile gains an exact Linux host compiler closure and helper
  source/binary identity, separate from the target compiler.
- Probe preflight reproduces and verifies the helper before phase 1 succeeds.
- Changed helper tests never run locally; cloud evidence becomes mandatory.
- A future production lab can move the same source into a controller-tool image.

## Security and data impact

The compiler sees only trusted default-branch helper source, not target source,
owner data, credentials, role outputs, or launch authority. Build directories
are bounded and disposable. The produced helper gains only the exact filesystem
descriptor authority specified by the handoff contract; container, network,
cloud, GitHub publication, and target-launch authority remain outside it.

Two builds from one closure prove reproducibility, not independence from a
compromised compiler. Alpha accepts that bounded residual risk only with
authenticated closure acquisition, a separately identified verifier, a
source-only mount scope, and adversarial compiler/helper evidence. Production
migration still requires the stronger isolated controller-tool path.

## Compatibility and migration

This is an experimental host-controller choice. It changes no target ABI,
package, filesystem, tier, application, or persistent-data promise. A later
certified controller-tool image can replace the runner build while retaining
the same manifest bytes and negative cases.

## Validation

- The complete compiler closure and licenses are pinned and verified before use.
- Two fresh network-disabled builds from the same trusted source are byte-identical.
- The golden manifest vector, SHA-256 vectors, all valid combinations, malformed
  encodings, and descriptor-race cases pass in the isolated Linux job.
- The final helper identity is bound into the ready profile and retained evidence.
- Tests prove the helper has no process, network, container, cloud, credential,
  source-checkout, target-launch, or publication capability.
- Local policy proves the Mac never compiles or executes the helper.

## Replacement path

Move helper compilation into a digest-pinned controller-tool image, reproduce
the binary twice, update only the reviewed host profile/inventory, and preserve
the target-independent manifest contract.
