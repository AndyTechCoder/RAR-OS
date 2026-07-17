# ADR 0011: Release 0 Reproducibility Gate Phasing

Status: Accepted — 2026-07-16

## Context

PR #2 merged R0-000/R0-001 before its required Prompt 3 review and Prompt 4 remediation. Its documentation correctly reported that no RAR target source or artifact existed, but the R0-001 packet still required two clean builds to produce identical unsigned target artifacts. Treating that physically impossible check as passed would weaken the release promise; treating all useful bootstrap planning as blocked would prevent the prerequisite work that makes later artifact comparison possible.

The owner explicitly directed Prompt 4 to re-phase, not remove or weaken, this requirement: deterministic build planning is proved during R0-001 remediation, and byte-identical target artifacts remain mandatory after target artifacts exist and before Release 0 closes.

## Decision drivers

- Preserve the exact two-clean-build, byte-identical unsigned target-artifact promise.
- Keep acceptance statements truthful when no target artifact exists.
- Permit only R0-000/R0-001 host remediation now, without beginning R0-002.
- Give the Release 0 gate an executable schedule that cannot be mistaken for a waiver.
- Record the premature PR #2 merge and the corrected review and acceptance sequence.

## Considered options

- **Mark target-artifact reproducibility passed from build-plan equality:** rejected because plans are not target artifacts.
- **Remove the target-artifact requirement:** rejected because it weakens Release 0 reproducibility.
- **Keep R0-001 permanently blocked until later target code exists:** rejected because it prevents accepting the safe bootstrap mechanisms needed to create those artifacts.
- **Split planning proof from the deferred Release 0 artifact gate:** selected because both claims remain precise and independently testable.

## Decision

R0-001 remediation must prove that two clean planning runs from the same checkout and validated tool lock produce byte-identical canonical build plans without compiling, linking, loading, or executing target code.

The original artifact requirement remains mandatory: after Release 0 target artifacts exist, two clean builds from the same checkout, locked inputs, target, and configuration must produce byte-identical unsigned target artifacts. R0-009 must not close Release 0 until this comparison passes for every required Release 0 target artifact. A missing artifact, skipped comparison, mismatch, or unexplained nondeterminism is a blocking gate failure, never a limitation converted into a pass.

This scheduling correction does not authorize a target build in Prompt 4, any target execution, R0-002 work, emulator launch, physical-device access, or VM boot authorization.

## Consequences

- R0-001 can close its currently applicable planning acceptance with truthful evidence.
- The artifact comparison is deferred only in time, not in scope, strength, or release criticality.
- Build evidence carries an explicit deferred-mandatory marker until target artifacts exist.
- R0-009 owns final closure evidence and must link the two clean build outputs and byte comparisons.
- Any task that first produces a Release 0 target artifact must preserve enough clean-build inputs for the final comparison.

## Security and data impact

No RAR target artifact is executed or authorized by this decision. The certified-VM and separate owner-authorization boundary remains unchanged. Reproducibility evidence continues to bind source revision, host-tool roots, hashes, targets, configuration, dependencies, and generated inputs. No user-data or persistent target format is introduced.

## Compatibility and migration

Build-plan and evidence schemas use version 3 so they can distinguish current deterministic planning from the deferred mandatory artifact gate, report pinned tool states truthfully, and bind one revalidated source snapshot. Versions 1 and 2 remain historical remediation inputs and are not accepted as Prompt 4 closure evidence.

The owner-directed remediation grants coordinator write ownership only for the governance and CI paths required to record this correction: this ADR, `docs/README.md`, `docs/tasks/release-0.md`, `tools/ci/check-specs.sh`, and `.github/workflows/specifications.yml`. It also records the historical coordinator ownership basis for the four already-merged PR #2 governance files: `.codex/config.toml`, `AGENTS.md`, `docs/v1-alpha-execution.md`, and `tools/ci/check-host-policy.sh`. No later-release implementation ownership is transferred.

## Validation

- Run the planning command twice from the same clean source and lock inputs; compare canonical plan bytes.
- Confirm both plans state that target artifacts were not produced and target execution was not attempted.
- Before R0-009 closure, perform two clean target builds for each required Release 0 target and compare unsigned artifact bytes.
- Fail the Release 0 gate on a missing artifact, skipped comparison, mismatch, unexplained nondeterminism, or incomplete evidence mapping.
- Require clean independent correctness and security review of the Prompt 4 remediation before merge.

## Replacement path

Future build orchestrators may replace the R0 host scaffold if they preserve the versioned evidence semantics and pass the same planning and artifact reproducibility gates. A later ADR may refine clean-room mechanics or artifact normalization, but it cannot silently weaken byte identity or move the gate beyond Release 0 closure.
