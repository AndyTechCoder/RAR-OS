# R0-001 reproducible host bootstrap remediation

Status: Prompt 4 remediation implemented; exact-head CI and independent review remain mandatory PR merge gates

## Historical correction

PR #2 (`codex/r0-host-safety-bootstrap`) merged as `2678a91996fbcbb1666fb008ecc1a347d7ba49e7` before Prompt 3 review, Prompt 4 remediation, and currently applicable acceptance evidence. Its description explicitly said review had not begun and it must not merge. The post-merge audit therefore remains a recorded process failure; that merge did not authorize R0-002, target execution, VM certification, physical-device access, or later Release 0 progression.

This branch remediates only R0-000/R0-001. ADR 0011 changes only timing: deterministic build planning is proved while no target artifact exists, and two clean builds producing byte-identical unsigned target artifacts remain mandatory after artifacts exist and before R0-009 closes Release 0. ADR 0012 records the corrected host trust roots, platform lock separation, clean Git snapshot, and versioned host receipts.

## Command surface and execution hosts

The host command names remain:

```sh
tools/rarbuild/rarbuild check
tools/rarbuild/rarbuild build
tools/rarbuild/rarbuild image
tools/rarbuild/rarbuild run
tools/rarbuild/rarbuild test
tools/rarbuild/rarbuild evidence
```

`run`, aliases, delegation names, arbitrary commands, and argument-bearing `test` modes return 73 before root discovery or host-tool execution.

Executable accepted routes run only in the official Rust 1.95.0 OCI image pinned by index digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. CI checks out the exact PR head, selects the separately measured Linux lock, runs every accepted route under a poisoned caller `PATH`, and executes generated host binaries through `/proc/self/fd`.

The physical Mac verifies the bounded policy records, proposed macOS roots, and both closure manifests, then returns exit 2 with `reason=local-bootstrap-execution-awaits-descriptor-bound-launcher`. macOS does not provide the required descriptor-execution primitive for generated Mach-O files, so Release 0 does not reopen a mutable generated pathname and overstate its safety. This is a deliberate fail-closed limitation, not target non-execution alone.

## Bootstrap trust and closure

`rar-host-tool-lock-v3` separates the local diagnostic lock from the executable Linux CI lock.

Before shell parsing can allocate an unbounded record, the reviewed preparser axiom verifies exact hasher, byte-count, and line-bound helpers. It limits the lock to 16 KiB and 512-byte lines, limits approval/task/safety records, and rejects unknown fields. Before any selected non-root executable can run, the wrapper hashes its exact canonical path.

The macOS closure manifests pin:

- `rustc`, the compiler driver dylib, codegen backend, and Rust linker tools;
- host Rust standard/test libraries;
- the selected AArch64, x86-64, and Tier 0 target libraries and component manifests;
- Cargo;
- SDK settings and every SDK `.tbd` link stub in the selected closure.

The CI image digest is its complete transitive closure. The Linux lock additionally records exact hashes for Dash, SHA-256, bounds helpers, `mkdir`, `rm`, `env`, Rust, GCC 14, the sysroot marker, Cargo, and Git. `rustup` is never invoked and no command downloads or installs a dependency.

Wrong-byte fixtures prove that compiler, linker, compiler-driver, and standard-library changes fail before a canary can execute. Normal-exit traps remove private bootstrap/test directories; unique exclusive names avoid stale PID collisions.

## Clean source snapshot and versioned receipts

Pinned Git runs only after the compiled verifier has authenticated it. Planning and evidence require:

1. `HEAD^{commit}` and `HEAD^{tree}` resolve to existing objects;
2. tracked and untracked source state is clean;
3. one lock/probe/Git/source-input/manifest/inventory snapshot is captured;
4. the same lock, commit, tree, and source hashes still match before publication.

An archive containing only a claimed `.git/HEAD` value cannot emit evidence. Tests cover nonexistent objects, dirty source, lock replacement, and source mutation.

Corrected host schemas are `rar-host-check-v2`, `rar-host-test-v2`, `rar-build-plan-v3`, and `rar-build-evidence-v3`. Their field-order contracts are test fixtures under `tools/rarbuild/contracts/`; strict consumers do not reinterpret older schemas.

## Output and host-script safety

Durable plan/evidence output uses descriptor-relative no-follow directory traversal, exclusive mode-`0600` staging, file synchronization, same-descriptor rewind/hash verification, atomic rename, and directory synchronization. Newly created directory entries synchronize their parents. Pre-commit cleanup failures propagate. After rename, no failure path unlinks the destination because another writer may already have replaced it with valid evidence.

Competing-writer, parent-replacement, interruption, write/fsync/rename/unlink fault, and post-commit replacement tests cover those semantics.

`rarbuild test` reads each bounded test script once through a no-follow descriptor, hashes those captured bytes, and passes the exact text to the pinned shell using `-c` with the canonical script path as `$0`. Replacing the pathname after capture cannot change executed script bytes.

## Acceptance mapping

| R0-001 acceptance | Current evidence | State |
| --- | --- | --- |
| Report unavailable prerequisites without installation or host mutation | CI `rarbuild check` emits `rar-host-check-v2`; external LLD/QEMU/firmware remain unavailable | Applicable CI gate |
| Refuse every unauthorized execution route before resolution/spawn | Wrapper/compiled route matrices and poisoned-path canaries | Applicable CI gate |
| Deterministic planning while no target artifact exists | Two clean `rar-build-plan-v3` generations compare byte-for-byte and state `target_artifacts=not-produced`, `worktree_state=clean`, and `execution=forbidden` | Applicable CI gate |
| Bind evidence to tools, source, target, and configuration | `rar-build-evidence-v3` derives all values from one revalidated snapshot | Applicable CI gate |
| Two clean builds produce identical unsigned target artifacts | ADR 0011 requires exact byte comparison after artifacts exist and before R0-009 closes Release 0 | Deferred-mandatory; not passed |

## Unsafe and dependency review

No unsafe target code or assembly exists. Host-only unsafe remains isolated to `tools/rar-lab/safety/src/unix_fs.rs`. Platform-specific `mode_t` and variadic promotions have compile-time assertions; descriptor ownership, pointer lifetime, no-follow traversal, synchronization, and injected syscall failures have focused tests on supported CI/macOS configurations.

No third-party crate, Cargo package, target-linked dependency, binary payload, target asset, firmware, VM image, or Dependency Exception Record was added. Closure manifests contain hashes and paths only.

## Validation and remaining gates

Durable executable validation is the exact-head GitHub workflow:

```sh
tests/host-safety/run.sh
tests/bootstrap/run.sh
tools/rarbuild/rarbuild test
tools/rarbuild/rarbuild check
tools/rarbuild/rarbuild build
tools/rarbuild/rarbuild image
tools/rarbuild/rarbuild evidence
tools/ci/check-specs.sh
```

Local host-only validation is limited to shell syntax, specification/policy checks, closure verification, and the expected fail-closed wrapper refusal. No RAR target artifact, QEMU process, firmware, VM, networked guest, physical device, or target linker is executed.

The PR may merge only after exact-head CI succeeds, current `main` is conflict-free, and fresh independent correctness and security reviews report no blocking findings. Even after merge, external LLD, QEMU, firmware, profile certification, owner boot authorization, real-spawner lifecycle controls, and target-artifact reproducibility remain unsatisfied gates. Prompt 5 and R0-002 do not begin here.
