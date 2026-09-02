# Sprint Alpha  C3V exact-set verification child packet

Status: authoritative source-only child packet; implementation is phase-gated.

Parent: `docs/tasks/sprint-alpha-controller-helper-integration.md`

## Purpose

C3V independently verifies the exact controller-helper closure produced by the trusted-main C2 observer. It does not accept the closure, authorize a compiler, create a bootable image, or declare readiness.

The C2 prerequisite is satisfied only at exact main revision `70a683dfb6dbde03f0f884ddc16ac2a2680a4f4f` with passing Specifications run `33608694457` and Observer run `33608694456`. The retained C2 artifact is supporting provenance only. C3V MUST re-run the observer at the exact C3V merge revision immediately before verification in the same trusted-main job; it MUST NOT reuse an earlier receipt as verification input.

## Mandatory two-merge sequence

C3V is split into two separately reviewed and merged subphases:

1. **C3VA  contract and evidence activation.** Activate the test-plan and evidence contracts, validators, bounded fixtures, and policy tests. No workflow is added and no verifier runtime is activated.
2. **C3VB  trusted-main verifier activation.** Only after C3VA is merged and exact-main Specifications plus mutation validation pass, add the single verifier workflow and its bounded harness/runtime support.

C3VA and C3VB MUST each use a fresh branch from current GitHub `main`, exact-head independent architecture/correctness/security review, exact-head required checks, an exact-head merge, and exact-main validation. A later phase MUST NOT absorb an earlier phase.

## C3VA owned paths

Only these paths may change in C3VA:

- `spec/alpha/lab/controller-helper-closure-verifier-test-plan-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields`
- `tools/ci/check-controller-helper-closure-verifier-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-evidence-source.sh`
- `tools/ci/verify-controller-helper-closure-verifier-evidence.sh`
- `tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh`
- `tools/ci/fixtures/controller-helper-closure-verifier/evidence-valid.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/evidence-malformed.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/evidence-cases.v0`
- verifier-evidence policy-test modes, runners, and confinement declarations that are directly required by the preceding files
- `tools/ci/check-alpha-preimplementation.sh`
- `tools/ci/check-static.sh`
- `tools/ci/check-specs.sh`
- `docs/lab/README.md`
- `docs/lab/alpha-status-dashboard.md`
- `docs/status.md`

The existing verifier runtime and all C2 production paths remain read-only during C3VA.

## C3VB owned paths

Only these paths may change in C3VB:

- `.github/workflows/controller-helper-closure-verifier.yml`
- `tools/ci/run-controller-helper-closure-verifier.sh`
- `tools/ci/controller-helper-closure-verifier-harness.sh`
- `tools/ci/check-controller-helper-closure-verifier-policy.sh`
- `tools/ci/test-controller-helper-closure-verifier-policy.sh`
- `tools/ci/check-controller-helper-closure-verifier-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-evidence-source.sh`
- `tools/ci/verify-controller-helper-closure-verifier-evidence.sh`
- `tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh`
- `tools/ci/fixtures/controller-helper-closure-verifier/tool-pins.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/base-closure.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/cases.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/fixture-plan.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/residual-proofs.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/expected-verification.receipt.v0`
- `spec/alpha/lab/controller-helper-closure-verifier-test-plan-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields`
- verifier policy-test modes, runners, and confinement declarations that are directly required by the preceding files
- `tools/ci/check-alpha-preimplementation.sh`
- `tools/ci/check-static.sh`
- `tools/ci/check-specs.sh`
- `docs/lab/README.md`
- `docs/lab/alpha-status-dashboard.md`
- `docs/status.md`

The existing `tools/ci/verify-controller-helper-closure-candidate.sh` is read-only in C3VB. Every semantic contract or path not named above remains read-only.

## Permanent denied authority

C3V MUST NOT change the closure-acceptance instance, controller/helper source, target source, observer production files, locks, inventories, approved profiles, release gates, compiler policy, boot policy, or readiness state. It MUST NOT compile or execute any helper, target, firmware, guest, kernel, bootloader, QEMU workload, or RAR OS artifact. It grants no compiler-use, target-execution, acceptance, release, or readiness authority.

No C3V file may claim `reviewed`, `accepted`, `ready`, `bootable`, or an equivalent state. Only the later C3A acceptance gate may accept this closure or authorize the next capability.

## Trusted-main workflow contract

C3VB adds exactly one `push`-to-`main` trusted workflow. It has `contents: read`, no write permission, no pull-request trigger, no dynamic action reference, and no credentials exposed to verifier-controlled data. Every action is pinned to a full commit SHA.

The workflow MUST:

1. attest the approved hosted runner identity and architecture;
2. acquire the repository through the same bounded immutable acquisition pattern already approved for C2;
3. bind all scripts, manifests, fixtures, contracts, and action pins to reviewed digests;
4. run the C2 observer at the workflow's exact merge revision;
5. verify the fresh four-file observer artifact before using it;
6. run two stable topology/manifest passes;
7. execute the complete C3V matrix in its fixed order;
8. independently validate the canonical receipt, normalized evidence, and verdict;
9. upload only the exact retained set in a final step with a 14-day retention.

The exact retained set is:

1. `controller-helper-closure.sha256`
2. `controller-helper-closure.receipt`
3. `controller-helper-closure-verification.receipt`
4. `controller-helper-closure-verifier.evidence.v0`
5. `controller-helper-closure-verifier.verdict.v0`

No caches, workspaces, logs, archives, binaries, helper outputs, unnormalized streams, or credentials may be retained.

## Verification matrix

The verifier MUST execute exactly:

- 117 disposition runtime cases;
- 37 precedence runtime cases;
- 12 fault runtime cases;
- 166 runtime cases total;
- 43 residual proofs: 30 disposition and 13 precedence;
- 209 logical relationships in the canonical order `V001..V147,Q001..Q050,X001..X012`.

No sampled, skipped, retried, reordered, synthesized, or dynamically discovered case is acceptable.

Each runtime case is bounded to 2 CPUs, 2 GiB memory, no swap, 256 PIDs, a 64 MiB fixture, a 32 MiB tmpfs, and 30 seconds. The complete verifier is bounded to 20 minutes. Network access, package installation, credentials, `rustc`, `cargo`, linkers, helper execution, target execution, firmware, QEMU, and guest execution are forbidden.

## Evidence contract

The verification evidence MUST conform exactly to `controller-helper-closure-verifier-evidence-v0.fields`, including its header and per-case runtime/residual fields.

Bounds are:

- header: at most 8192 bytes;
- each case record: at most 2048 bytes;
- complete evidence: at most 524288 bytes;
- combined stdout and stderr retained per runtime case: at most 65536 bytes.

The normalized verdict is exactly one of:

- `mechanically-verified-not-reviewed-not-ready`
- `normalized-not-ready`

The canonical verification receipt is exactly 31 lines and uses the status `candidate-exact-set-verified-not-reviewed-not-ready`. Any malformed, missing, duplicate, out-of-order, oversized, unstable, or contradictory evidence fails closed.

## C3VA completion gate

C3VA is complete only when:

- every change is within its owned paths;
- contract and evidence validators fail closed;
- positive, malformed, mutation, and confinement policy tests pass;
- exact-head architecture, correctness, and security reviews report no blocker;
- the exact reviewed head passes all required checks and is merged unchanged;
- exact-main Specifications and the complete mutation suite pass.

## C3VB completion gate

C3VB is complete only when:

- C3VA's exact-main gate is already green;
- every change is within C3VB owned paths;
- the fresh observer/verification chain is exact-revision bound;
- both stable passes and all 209 relationships pass;
- resource, time, output, network, credential, tool, and retention constraints fail closed;
- exact-head architecture, correctness, and security reviews report no blocker;
- the exact reviewed head passes all required checks and is merged unchanged;
- exact-main Specifications, the complete mutation suite, and the verifier workflow pass;
- independent final review confirms the five retained files satisfy this packet.

Completion leaves the candidate mechanically verified but not reviewed, not accepted, and not ready.

## Host and storage safety

All work is repository-confined and GitHub-hosted. RAR OS target code MUST NOT be built, run, booted, mounted, or executed on the Mac or SSD. No local or SSD file may be created, changed, moved, or deleted. The SSD may be used only as separately authorized source/worktree storage; this packet grants no such authorization.
