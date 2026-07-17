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

Executable accepted routes run only in the official Rust 1.95.0 OCI image pinned by index digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. CI checks out the exact PR head, mounts the container tool root read-only, verifies the reviewed hosted-runner identity, selects a whole-file-digest-bound Linux lock, runs every accepted route under a poisoned caller `PATH`, and executes generated host binaries through `/proc/self/fd`.

The physical Mac verifies the bounded policy records, proposed macOS roots, and both closure manifests, then returns exit 2 with `reason=local-bootstrap-execution-awaits-descriptor-bound-launcher`. macOS does not provide the required descriptor-execution primitive for generated Mach-O files, so Release 0 does not reopen a mutable generated pathname and overstate its safety. This is a deliberate fail-closed limitation, not target non-execution alone.

## Bootstrap trust and closure

`rar-host-tool-lock-v3` separates the local diagnostic lock from the executable Linux CI lock.

Before shell parsing can allocate an unbounded record, the reviewed preparser axiom verifies exact hasher, byte-count, and line-bound helpers. It limits the lock to 16 KiB and 512-byte lines, limits approval/task/safety records, and rejects unknown fields. The preparser also binds the complete selected lock to a separately reviewed SHA-256 before parsing; the compiled verifier requires that same digest handoff. Before any selected non-root executable can run, the wrapper hashes its exact canonical path.

`tools/toolchain/class-b-host-tools.v1` closes the Class B policy ledger for selected macOS roots, Xcode SDK, Rust/LLVM, OCI packages, `actions/checkout`, and CI service boundaries. Every row has a version/identity, integrity source, license, provenance URL, setup source, and explicit status. CI rejects missing, duplicate, malformed, or stale rows. The hosted runner and container engine are version-attested external service layers, not part of the OCI userland digest, and remain explicitly non-certifying.

The macOS closure manifests pin:

- `rustc`, the compiler driver dylib, codegen backend, and Rust linker tools;
- host Rust standard/test libraries;
- the selected AArch64, x86-64, and Tier 0 target libraries and component manifests;
- Cargo;
- SDK settings and every SDK `.tbd` link stub in the selected closure.

The CI image digest and enforced read-only userland form the selected tool closure. The Linux lock additionally records exact hashes for Dash, SHA-256, bounds helpers, `mkdir`, `rm`, `env`, Rust, GCC 14, the sysroot marker, Cargo, and Git. Hosted runner, kernel, and container-engine layers remain recorded non-certifying service boundaries. `rustup` is never invoked and no command downloads or installs a dependency.

Wrong-byte fixtures prove that compiler, linker, compiler-driver, and standard-library changes fail before a canary can execute, and a matching path/hash lock substitution fails against the immutable whole-lock digest. The shell verifies the exact clean workflow commit before compilation and materializes source blobs into an exclusive private directory. Its first external boundary after compiler return is opening the generated output; closure revalidation happens after that descriptor is held. Normal-exit traps remove only an exact regular-file allowlist relative to bound current directories; recursive deletion is not used. The CI route forbids introducing an unreviewed concurrent same-UID process.

## Clean source snapshot and versioned receipts

The shell authenticates pinned Git and establishes source identity before host compilation; the compiled verifier independently repeats the checks. Planning and evidence require:

1. `HEAD^{commit}` resolves to an existing object and its tree is resolved from that captured commit;
2. tracked and untracked source state is clean and no `assume-unchanged` or `skip-worktree` flags exist;
3. source-input, manifest, and inventory digests derive from the captured commit's tree/blob objects, while compiler inputs are materialized from those same blobs;
4. the complete locked tool/closure probe, lock, commit, tree, and source hashes still match after output staging and immediately before atomic publication.

An archive containing only a claimed `.git/HEAD` value cannot emit evidence. Both SHA-1 and SHA-256 Git object IDs are validated. Tests cover nonexistent objects, hidden index flags, dirty source, tool-probe drift, lock replacement, source mutation, and mutation at the publication boundary.

Corrected host schemas are `rar-host-check-v2`, `rar-host-test-v2`, `rar-build-plan-v3`, `rar-image-plan-v3`, and `rar-build-evidence-v3`. Their field-order contracts are test fixtures under `tools/rarbuild/contracts/`; strict consumers do not reinterpret older schemas.

## Output and host-script safety

Durable plan/evidence output uses descriptor-relative no-follow directory traversal, exclusive mode-`0600` staging, file synchronization, same-descriptor rewind/hash verification, atomic rename, and directory synchronization. Newly created directory entries synchronize their parents. Pre-commit cleanup failures propagate. After rename, no failure path unlinks the destination because another writer may already have replaced it with valid evidence.

Competing-writer, parent-replacement, interruption, write/fsync/rename/unlink fault, and post-commit replacement tests cover those semantics.

`rarbuild test` reads the bootstrap library and each bounded test script from the captured Git commit, hashes the script bytes, and passes the combined exact text to the pinned shell using `-c` with the canonical script path as `$0`. Replacing workspace paths before or after capture cannot change executed script bytes.

## Acceptance mapping

| R0-001 acceptance | Current evidence | State |
| --- | --- | --- |
| Report unavailable prerequisites without installation or host mutation | CI `rarbuild check` emits `rar-host-check-v2`; external LLD/QEMU/firmware remain unavailable | Applicable CI gate |
| Refuse every unauthorized execution route before resolution/spawn | Wrapper/compiled route matrices and poisoned-path canaries | Applicable CI gate |
| Deterministic planning while no target artifact exists | Two clean `rar-build-plan-v3` generations compare byte-for-byte and state `target_artifacts=not-produced`, `worktree_state=clean`, and `execution=forbidden` | Applicable CI gate |
| Bind evidence to tools, source, target, and configuration | `rar-build-evidence-v3` derives all values from one revalidated snapshot | Applicable CI gate |
| Two clean builds produce identical unsigned target artifacts | ADR 0011 requires exact byte comparison after artifacts exist and before R0-009 closes Release 0 | Deferred-mandatory; not passed |

## Unsafe and dependency review

No unsafe target code or assembly exists. Host-only unsafe remains isolated to `tools/rar-lab/safety/src/unix_fs.rs`. The Linux `mode_t` and variadic-promotion assertions, descriptor ownership, pointer lifetime, no-follow traversal, synchronization, and injected syscall failures are compiled and exercised by the pinned Linux CI route. The corresponding macOS constants, ABI bindings, and compile-time assertion are statically reviewed but remain uncompiled and unexecuted at this exact head because every local macOS Rust route refuses before compiler execution. Executable macOS evidence is deferred until a separately reviewed descriptor-bound launcher or equivalently immutable host-only route exists; ADR 0012 does not pre-authorize that route.

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
