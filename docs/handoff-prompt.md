# Initial Codex Handoff Prompt

Status: Approved for Prompt 2 after repository publication

Use this prompt for the first implementation task:

> You are beginning RAR OS Release 0 in the `AndyTechCoder/RAR-OS` repository. Read `AGENTS.md` and every document it requires before taking action. Work only on R0-000 and R0-001 in `docs/tasks/release-0.md`; do not implement later tasks or stable interfaces they do not require.
>
> The physical Mac is source/build storage only. RAR OS must never run natively, modify macOS, access raw disks, change boot/firmware configuration, install system extensions, use physical device passthrough, or launch through direct/unapproved QEMU commands. During this task, do not boot or execute any RAR target artifact at all. First implement and statically test the certified RAR Lab launcher boundary; the owner will separately authorize the first guest boot later.
>
> Establish the pinned host-tool manifest, reproducible output layout, dependency inventory, safe `rarbuild` command skeleton, VM-profile schema, command allowlist, and negative tests that reject every forbidden host integration. Host-only validation is allowed; target OS execution is not.
>
> Use GPT-5.6 Sol Ultra as a coordinator only. Subagents may perform read-only specification review or work in isolated worktrees on non-overlapping host-only files. One agent owns each path. Do not let subagents change shared architecture contracts, approval records, or host-safety rules.
>
> Run the required specification and host-only checks. Update documentation with any discovered limitation. Commit the coherent change, push a `codex/r0-host-safety-bootstrap` branch to GitHub, and open a draft pull request. Do not merge it. End with exact evidence, changed files, checks, security review, limitations, and the next approval required.

## Follow-up sequence before Release 0 completion

Each follow-up is a separate reviewed branch/PR:

1. Review and merge R0-000/R0-001 host safety and bootstrap.
2. R0-002 hardware-description and boot-handoff contracts; still no guest execution.
3. Owner authorizes the first certified VM boot profile.
4. R0-003 x86-64 and R0-004 ARM64 may proceed in parallel worktrees.
5. R0-005 Tier 0 experiment proceeds separately.
6. R0-006 integrates portable memory/thread execution on both full architectures.
7. R0-007 implements and validates capability/IPC proof.
8. R0-008 closes deterministic lab evidence and fault scenarios.
9. R0-009 performs independent documentation/conformance review and cuts Release 0.

No single initial Codex task is expected to produce Release 0. The first complete version is reached through these reviewed increments.
