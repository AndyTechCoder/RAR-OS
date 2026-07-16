# V1 Alpha Codex Execution Runbook

Status: Approved for execution; begins after repository publication and GitHub authentication

## V1 alpha definition

V1 alpha is complete when Releases 0–6 in `release-roadmap.md` pass. It includes all four simulated tiers, custom OS foundations, signed components/layers, data isolation, recovery, updates, networking/Wi-Fi, adaptive GUI, multi-user applications, continuity, agent readiness, SDKs, and self-hosting.

It excludes physical-hardware support claims, the final branded design system, Pal intelligence/model training, Linux compatibility, production cloud services/CDN, browser, broad app ecosystem, independent production security certification, and safety certification.

## Why prompts are staged

Late subsystem contracts depend on evidence from earlier releases. Writing every implementation prompt now would force later agents to guess around interfaces that do not yet exist. Therefore each release closes by producing and independently reviewing the next release's decision-complete task packet.

Expect roughly 70–100 task/review PRs before V1 alpha. The exact total is evidence-driven; it is not safe to cap defect fixes or split unrelated work merely to hit an arbitrary prompt count.

## Model policy

- Main release coordinator: GPT-5.6 Sol Ultra.
- Architecture/release planning: `rar_architect`, Sol Ultra, read-only.
- Focused implementation: `rar_implementer`, Sol Extra High (`xhigh`).
- Correctness review: `rar_reviewer`, Sol Extra High, read-only.
- Security/trust review: `rar_security_reviewer`, Sol Max, read-only.
- Exploration/log triage: `rar_explorer`, Terra High, read-only.
- Gate closure: `rar_release_manager`, Sol Ultra.

Ultra coordinates; it does not give multiple agents permission to edit the same paths. At most four threads run concurrently, one nesting level only.

## Git workflow

- One task packet per `codex/<task-id>-<slug>` branch and draft PR.
- One writer owns a path at a time.
- Every commit includes code/specification, tests, and documentation for one coherent change.
- Author never self-approves security-sensitive work.
- Review findings are fixed on the same branch, then re-reviewed.
- Gate integration uses a separate `codex/rN-integration` branch.
- Owner authorization dated 2026-07-16 permits automatic merges at the final
  review/remediation or release-gate step only.
- Before an automatic merge, required checks, independent reviews, acceptance
  evidence, documentation, and base-branch synchronization must all pass with
  no blocking findings or conflicts. Implementation authors do not self-approve.
- Use squash merge for a single task packet and a merge commit for a reviewed
  release integration unless repository history requires a documented exception.

## Required review levels

- Documentation/host tooling: correctness reviewer.
- Nucleus, unsafe Rust, assembly, capabilities, drivers, parsers, storage, update, recovery, identity, cryptography, networking, agents: correctness plus security reviewers.
- Public interfaces/persistent formats: correctness, security, and architecture reviewers.
- Release gate: all applicable reviewers plus release manager.

## Exact prompts through Release 0

Send these as separate Codex tasks. Do not paste all of them into one thread.

### Prompt 1 — Gate 0 approval and repository publication

> Read AGENTS.md and the complete specification index. Verify that the Gate 0 approval record matches the owner's approved direction, update its date/status only if approval is explicitly present, run specification checks, commit the complete handoff package, push it to the canonical GitHub repository, and open/record the initial PR or initial-main commit as appropriate for an empty repository. Do not implement OS code or execute target artifacts.

### Prompt 2 — R0-000 and R0-001

Use `docs/handoff-prompt.md` exactly.

### Prompt 3 — Bootstrap security review

> Review the R0-000/R0-001 draft PR. Spawn `rar_reviewer` and `rar_security_reviewer`, wait for both, and consolidate only evidence-backed findings. Verify repository-only effects, VM non-execution, dependency provenance, reproducibility, GitHub scope, and negative safety tests. Do not edit or approve the PR.

### Prompt 4 — Bootstrap remediation

> Address every accepted finding on the R0-000/R0-001 branch. Do not broaden scope. Rerun all host-only checks, update evidence and documentation, commit, push, and obtain clean independent re-review. If every acceptance condition and required check passes with no blocking findings or conflicts, mark the PR ready and merge it automatically. Target code must still not execute.

### Prompt 5 — R0-002 contracts

> Implement only R0-002 from the approved Release 0 task packet. Use `rar_architect` read-only before writing. Produce the hardware-description and boot-handoff contracts, generated Rust types, malformed fixtures, conformance tests, and documentation. Do not boot or execute target code. Commit, push, and open a draft PR.

### Prompt 6 — R0-002 review/remediation

> Independently review the R0-002 PR with architecture, correctness, and security agents. Fix accepted findings in the owning implementation thread, rerun conformance tests, and push. If every acceptance condition and required check passes with no blocking findings or conflicts, mark the PR ready and merge it automatically. Do not authorize VM execution.

### Prompt 7 — First VM authorization checkpoint

> Audit the certified RAR Lab profile evidence produced by R0-000 and the contracts from R0-002. Use read-only architecture and security agents. State whether the exact profile is safe for the first guest boot under docs/host-safety.md. Do not boot anything. If ready, produce the precise owner authorization text and immutable profile/artifact hashes.

The owner must explicitly authorize the first guest boot after Prompt 7. This is the only required manual execution checkpoint.

### Prompt 8 — Parallel platform bring-up

> After recorded owner authorization, implement R0-003, R0-004, and R0-005 in three isolated worktrees using one `rar_implementer` per task. No paths may overlap except approved generated contracts. Use only the authorized certified VM profiles. Commit and push one draft PR per task; do not integrate them in this task.

### Prompt 9 — Platform reviews

> Review the x86-64, ARM64, and Tier 0 PRs with separate correctness reviewers and one cross-cutting security reviewer. Compare semantic hardware contracts and ensure no QEMU/profile assumptions leak into portable code. Return findings per PR; do not edit.

### Prompt 10 — Platform remediation

> Route accepted review findings to the owning platform worktrees, fix them without cross-ownership edits, rerun the complete authorized boot matrix, update evidence, commit, push, and obtain clean re-review. Merge each platform PR automatically only when its complete acceptance evidence passes with no blocking findings or conflicts.

### Prompt 11 — R0-006 Nucleus execution

> Implement R0-006 against the reviewed platform contracts. Use one primary writer and read-only subagents for memory-model and portability review. Add isolation, stress, guard-page, fault-attribution, unsafe-code, and leak tests. Run only authorized VM profiles. Commit, push, and open a draft PR.

### Prompt 12 — Nucleus independent review/remediation

> Review R0-006 with correctness, security, and architecture agents. Trace every unsafe block and assembly boundary. Fix accepted findings in the owning branch, rerun both architectures, commit, push, and obtain clean re-review evidence. If every acceptance condition passes with no blocking findings or conflicts, mark the PR ready and merge it automatically.

### Prompt 13 — R0-007 capability and IPC proof

> Implement R0-007 only. Prove handle forgery resistance, rights-reducing delegation, cancellation, timeouts, backpressure, peer crash, and endpoint replacement on x86-64 and ARM64. Use independent correctness and security review before marking ready. Commit, push, and open a draft PR. Fix accepted findings and merge automatically only after clean re-review and complete acceptance evidence.

### Prompt 14 — R0-008 laboratory evidence

> Implement R0-008 after R0-007 review passes. Add deterministic scenarios, structured evidence, first-divergence reporting, timeouts, fault injection, and containment demonstrations. Do not add networking, storage, or GUI. Commit, push, and open a draft PR with evidence. Obtain independent correctness and security review, fix accepted findings, and merge automatically only after clean re-review and complete acceptance evidence.

### Prompt 15 — R0-009 gate closure and Release 1 packet

> Act as `rar_release_manager`. Independently review and integrate all approved Release 0 PRs, run the entire matrix from a clean checkout, verify every Release 0 promise, close documentation and limitations, and create decision-complete Release 1 task packets based on actual evidence. Commit/push the integration branch and open a draft Release 0 gate PR. If all gate evidence, required checks, and independent reviews pass with no blocking findings or conflicts, mark it ready and merge it automatically. Do not begin Release 1 implementation.

## Releases 1–6 loop

For each later release, use this four-prompt control loop plus one prompt per approved task packet:

1. **Plan review:** architecture/security agents review the generated packet; release manager fixes ambiguity before code.
2. **Implementation:** one prompt/branch/PR per task packet; parallel only for non-overlapping ownership.
3. **Independent review/remediation:** mandatory reviewers by risk class; repeat until no blocking findings.
4. **Gate closure:** clean integration, complete evidence, user-facing demonstration, limitations, and next release packet generation.

Never ask one task to “finish the rest of RAR OS.” The active task packet and release gate define completion.

## User involvement

Routine edits, tests, commits, pushes, PR readiness changes, and evidence-gated
merges use automatic review. Ask the owner only for:

- Gate 0 approval.
- First certified VM boot authorization.
- Constitutional or product-scope changes.
- New external target-code dependencies.
- Reduced privacy/security/data guarantees.
- Physical-device testing or host-access expansion.
- V1 alpha release approval.
