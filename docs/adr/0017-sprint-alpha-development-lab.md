# ADR 0017: Sprint Alpha 0.1 and Cloud Development Lab

Status: Accepted — 2026-08-20

Approval basis: explicit owner approval of the Sprint Alpha 0.1 rebaseline and
Development Lab trust-boundary change on 2026-08-20.

## Context

Release 0 established careful non-execution, host-tool, hardware-description,
and handoff contracts, while draft PR #5 explored a production-style one-shot
authorization closure. That path accumulated production authority and cloud
service concerns before RAR OS had an authentic booting vertical slice. The
next fourteen days and three Codex usage resets are instead reserved for a
working, inspectable Alpha 0.1 demonstration without weakening the long-term
architecture, recovery, signing, or isolation promises.

The owner explicitly approved moving automated target compilation, linking,
image creation, and guest execution into an isolated cloud Development Lab.
The Mac remains source-editing and lightweight-static-check storage only.

## Decision drivers

- Produce observable Root → Recovery → Nucleus behavior early.
- Exercise contracts in working code instead of accumulating unused contracts.
- Keep every target build and execution off the owner's Mac.
- Retain deterministic tools, inputs, artifacts, traces, and failure evidence.
- Make routine probes quiet while keeping milestone failures truthful.
- Preserve the Releases 0–7 direction and production hardening path.

## Considered options

### A. Complete draft PR #5 and its production-style one-shot authority first

This retains the strongest pre-execution closure, but makes AWS-style authority,
credential, and orchestration work block the first authentic OS behavior.

### B. Permit local Mac target builds and VM execution

This shortens feedback, but expands host risk and contradicts the owner's
explicit Mac boundary.

### C. Use a repository-controlled cloud Development Lab

This keeps target effects off the Mac, allows automated iteration, and retains
bounded, hashed evidence. Production authorization remains future hardening.
This option is selected.

## Decision

RAR OS adopts a fourteen-day Sprint Alpha 0.1 vertical slice described in
`docs/sprint-alpha.md`. It overlays, but does not replace, Releases 0–7.
Prototype features may cross later-release themes only for the bounded Alpha
demonstration; they remain experimental until their normal release gates.

Target compilation, target linking, boot-image construction, firmware loading,
QEMU/emulator/VM execution, and guest integration, fault, recovery, or boot
tests occur only in GitHub Actions or another repository-approved cloud
Development Lab. They never occur locally on the Mac.

Every Development Lab execution must use:

- a repository-approved Linux runner whose actual image identity is recorded;
- immutable OCI, compiler, linker, emulator, and firmware identities before use;
- only repository-produced target artifacts;
- bounded disposable storage and explicit CPU, memory, output, and timeout limits;
- no guest networking initially, host sharing, passthrough, raw devices,
  elevated execution, production credentials, or unrelated external access;
- retained source, configuration, runner, tool, firmware, and artifact hashes;
- retained complete logs, serial output, structured result, and real exit status.

The `ubuntu-24.04` GitHub-hosted runner label is repository-approved only as an
orchestrator. Its observed `ImageOS`, `ImageVersion`, OS, and architecture are
attested and retained for each run. It grants no target-input authority. All
output-affecting tools and firmware must be pinned independently before target
compilation or execution.

Development Probes are manual, non-required workflows for iteration. A failed
probe remains failed and cannot satisfy a milestone. Required milestone CI is
strict, runs for pull requests and the resulting distinct `main` commit, and
uses concurrency cancellation to discard obsolete runs. Feature-branch pushes
do not duplicate the pull-request workflow for the same SHA.

Useful pinning, closure, profile, artifact, reproducibility, and safety work
from draft PR #5 may be selectively reimplemented or carried forward only when
the current or immediately following sprint milestone consumes it. AWS,
DynamoDB, KMS, production credentials, and production one-shot authority are
not Sprint Alpha dependencies. PR #5 remains unmerged until this replacement
is independently reviewed, merged, and durable; it is then closed as
superseded with its history and evidence preserved.

## Consequences

- Milestone A can begin immediately after this rebaseline merges.
- Cloud probe evidence is development evidence, not production certification.
- Cross-release Alpha prototypes need explicit limitations and later migration.
- Production-grade one-shot authorization remains required before production
  deployment or any future expansion beyond the Development Lab.
- The quiet pipeline reduces duplicate expensive runs without hiding failures.

## Security and data impact

The Mac boundary becomes stricter: local target compilation, linking, image
creation, firmware loading, and guest execution are forbidden. The cloud lab
receives no production credentials, guest network, host sharing, passthrough,
raw-device access, or elevated execution. Storage and evidence are bounded and
disposable except for retained repository-approved artifacts and logs.

This decision does not reduce capability isolation, target signing, rollback,
recovery, or user-data separation requirements. It grants no physical-device,
production-deployment, or unrelated cloud authority.

## Compatibility and migration

The long-term Releases 0–7 roadmap and stable Gate 0 commitments remain. Alpha
contracts are marked experimental and must either pass their normal release
gate or be replaced through a later ADR. Production authorization work may
reuse reviewed Development Lab evidence but cannot treat it as production
approval.

## Validation

- Local policy and documentation checks execute no target code.
- Workflow inspection proves expensive equivalent push/PR duplication is gone.
- Development Probes retain complete logs, structured results, and exit status.
- Each milestone maps observable behavior to strict required CI evidence.
- Exact runner, source, configuration, tool, firmware, and artifact identities
  are present before any cloud target execution is accepted.
- Review confirms local QEMU, firmware, emulator, VM, and target execution remain
  forbidden.

## Replacement path

After Alpha 0.1, the Development Lab policy may be replaced by a stronger
certified runner and production authorization design without changing target
interfaces. Draft PR #5 remains the historical record of the deferred one-shot
approach. Any local execution, physical hardware, networking, host integration,
or production authority requires a new explicit owner-approved decision.
