# R0-001 reproducible host bootstrap remediation

Status: Prompt 4 host-only remediation implemented; independent re-review required before merge

## Historical correction

PR #2 (`codex/r0-host-safety-bootstrap`) merged as commit `2678a91996fbcbb1666fb008ecc1a347d7ba49e7` before Prompt 3 review and Prompt 4 remediation. Its own description said review had not begun and the PR must not merge. The post-merge audit therefore records PR #2 as not satisfying the Prompt 3/4 or R0 acceptance gate. That merge does not authorize R0-002, target execution, profile certification, VM boot, physical-device access, or later Release 0 progression.

This remediation starts from that exact current `main` commit. It fixes every accepted audit finding and preserves all RAR OS requirements. ADR 0011 corrects only the evidence schedule: deterministic build planning is proved now, while two clean builds producing byte-identical unsigned target artifacts remain a blocking requirement after target artifacts exist and before R0-009 closes Release 0.

The finding-by-finding closure and ownership evidence is recorded in [Prompt 4 Bootstrap Remediation Record](prompt-4-remediation.md).

## Command surface

The public host-only surface remains closed:

```sh
tools/rarbuild/rarbuild check
tools/rarbuild/rarbuild build
tools/rarbuild/rarbuild image
tools/rarbuild/rarbuild run
tools/rarbuild/rarbuild test
tools/rarbuild/rarbuild evidence
```

`run`, execution aliases, arbitrary absolute commands, delegation names, and every argument-bearing `test` route refuse before repository discovery or host-tool execution. Accepted routes use no ambient `rustc`, `rustup`, `git`, linker, or shell lookup.

## Bootstrap trust root

The unavoidable first host execution boundary is explicit in `rar-host-tool-lock-v2`:

- `/bin/sh` interprets the reviewed wrapper.
- `/bin/mkdir` creates repository-confined bootstrap directories.
- the exact Rust 1.95.0 compiler and Rust-bundled `rust-lld` paths compile host-only RAR tools;
- the linker flavor and macOS SDK settings identity are fixed;
- every root path is absolute, canonical, regular, and non-symlink;
- every root file or settings record has a reviewed SHA-256 pin.

The shell cannot cryptographically verify the executables that constitute its own root of trust. It reads the reviewed path/hash record with shell builtins, rejects malformed or aliased paths, and invokes only those absolute roots. The compiled verifier then independently streams and checks every pinned byte sequence before it performs any later subprocess action. This is the documented bootstrap axiom, not ambient `PATH` trust.

`rustup` and `git` are not executed. Rust and Cargo paths come directly from the lock. Source revision is read with bounded parsing of `.git/HEAD`, loose references, packed references, and worktree `commondir` metadata.

## Output and input safety

Version 2 plan and evidence writers use descriptor-relative Unix operations:

- every path component is opened with no-follow semantics;
- missing output directories are created relative to an already-open directory descriptor;
- temporary files use exclusive creation and mode `0600`;
- bytes and the containing directory are synchronized;
- commit uses same-directory `renameat`;
- failed staging and failed post-commit verification remove the temporary or destination entry through the held descriptor;
- committed bytes are reopened without following links and freshly hashed.

Focused tests replace the parent directory between staging and rename and prove that no file reaches the replacement target. Interruption hooks prove temporary cleanup. A hard process kill can leave an inert uniquely named temporary file in the original descriptor-bound output directory; it is never accepted as evidence and later writes never reuse it.

The initial shell-to-Rust bootstrap uses exclusive, mode-`0700` per-process directories under `out/r0/host-tools/` or `out/r0/host-tests/`. The shell stage cannot provide the same descriptor-relative guarantee as the compiled writer; its claim is deliberately narrower and depends on the reviewed bootstrap root plus a private repository checkout. All durable plan/evidence output uses the descriptor-relative writer.

Tool-lock loading is bounded before allocation. Artifact, firmware, tool, source, and emulator hashing is streaming with fixed memory. Source-input identity hashes path, byte length, and a streaming content digest rather than buffering the complete source tree.

## Pinned local inputs

Observed on `aarch64-apple-darwin` on 2026-07-16:

| Input | Identity | SHA-256 |
| --- | --- | --- |
| Bootstrap shell | `/bin/sh` | `523408f21ffe09778e70c2b6dce65904cde0d326bfb5bd4134a382fcd425c274` |
| Bootstrap mkdir | `/bin/mkdir` | `04400c35f60e7a27db6560e32e85f85a4921c0b7b1900f26759157f7eb6eae3d` |
| Rust compiler | 1.95.0, commit `59807616e1fa2540724bfbac14d7976d7e4a3860` | `b829b733131d4e1673eeebd1f34d06ae1e9ff4977b051313cf42e2a9e79ecf1c` |
| Rust-bundled linker | LLVM 22.1.2, `ld64.lld` flavor | `96df7b3559f741be99cc2047cfaff84eeb5367dc9268a87c22ac9d376d98c60b` |
| macOS SDK settings | Xcode `MacOSX.sdk/SDKSettings.json` | `2fa5c0ce1bbcd261b132b572b1a9eece3b5905b04640a44deae1a6a8812928fb` |
| Cargo | 1.95.0, commit `f2d3ce0bd7f24a49f8f72d9000448f8838c4e850` | `c512bff73c86143b557463f021d0c3d5b0490d97d65040ba59ea2b3427784758` |
| `rust-src` manifest | Rust 1.95.0 | `47b629523343fa73b4436080f660b510e0cd1c2553a94ba90ef8bdcc2e025ec1` |
| AArch64 target manifest | `aarch64-unknown-none` | `d2c67d85ffb386328781b6300ddfde93c9a500072a9e6e08eb3ff1fb0017375c` |
| Tier 0 target manifest | `thumbv8m.main-none-eabi` | `c12a52d6b268e44baf79e6ec56fe0f82b53587d2dee6b1694fda3ffb94720f2b` |
| x86-64 target manifest | `x86_64-unknown-none` | `a1c0aed6cf079827ac9ebc82faeea2b517aba581c240dfe84a31761a99068c75` |

External LLD, QEMU x86-64/ARM64/ARM, and both firmware inputs remain explicitly unavailable and unpinned. `certifiable=false` remains mandatory. No target-linked third-party code, target artifact, firmware blob, VM image, or Dependency Exception Record exists.

## Durable CI

`.github/workflows/specifications.yml` runs both Rust host suites in the official Rust 1.95.0 OCI image pinned by index digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. The image digest is the CI test bootstrap root. CI runs no target build or emulator and separately proves the execution routes remain refusal-only.

The canonical R0 host lock remains the measured ARM64 macOS lock. Passing portable unit tests in the pinned Linux container is not a Linux host-support claim; a supported Linux `rarbuild check` still requires a separately measured and reviewed Linux lock.

## Acceptance mapping

| R0-001 acceptance | Current evidence | State |
| --- | --- | --- |
| One command reports missing tools without host installation or mutation | `rarbuild check` hashes locked roots and reports unavailable LLD/QEMU/firmware; no downloader or installer exists | Pass |
| Every unauthorized execution-capable route refuses before resolution or spawn | Wrapper and compiled route matrices; poisoned-`PATH` canaries; resolver/spawner counters | Pass |
| Deterministic planning while no target artifact exists | Two clean regenerations produce byte-identical `rar-build-plan-v2` bytes and explicitly state `target_artifacts=not-produced` and `execution=forbidden` | Pass |
| Two clean builds produce identical unsigned target artifacts | ADR 0011 retains this exact requirement after artifacts exist and before R0-009 closes Release 0 | Deferred-mandatory; not yet applicable and not passed |
| Evidence records tools, hashes, target, configuration, and source | `rar-build-evidence-v2` derives tool and certification state from the validated lock/probe | Pass for the host scaffold |

## Unsafe and dependency review

There is no unsafe target code or assembly. Host-only unsafe is isolated to `tools/rar-lab/safety/src/unix_fs.rs`, which binds `openat`, `mkdirat`, `renameat`, and `unlinkat`. Its invariants require NUL-free single-component names, live owned directory descriptors, exact one-time descriptor ownership transfer to `File`, valid pointer lifetimes, and audited macOS/Linux flag values. Focused tests cover no-follow emulator opens, descriptor continuity into the spawner boundary, exclusive output creation, parent replacement, cleanup, and atomic replacement.

No third-party crate, package, host runtime library, target dependency, binary blob, target asset, or firmware was added. Host Rust uses `std`; the Unix calls are RAR-owned bindings to host system interfaces.

## Exact host-only validation

Run from the repository root:

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

The remediation suites currently contain 21 R0-000 tests and 23 R0-001 tests. All commands are host-only. Expected nonzero statuses are `check=3`, `image=4`, `evidence=4`, and refusal routes `=73`.

## Remaining gates and non-execution attestation

- No target source or artifact exists; Prompt 4 does not compile or link one.
- The identical target-artifact gate remains mandatory before Release 0 closes.
- External LLD, QEMU, firmware, profile certification, and owner boot authorization remain absent.
- Single-use authorization consumption, timeout enforcement, and forced termination remain attached to a future reviewed real spawner; no spawner is shipped.
- Fresh independent correctness and security reviews must be clean before this remediation PR can merge.

No QEMU executable, firmware, target linker, target binary, boot image, VM image, physical device, or RAR target artifact was resolved for execution, loaded, launched, or executed during this remediation. Prompt 5 and R0-002 have not begun.
