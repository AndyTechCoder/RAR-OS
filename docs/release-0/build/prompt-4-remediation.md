# Prompt 4 Bootstrap Remediation Record

Status: Implementation record; exact-head CI and independent review evidence are external PR #3 merge gates

Base: GitHub `main` commit `2678a91996fbcbb1666fb008ecc1a347d7ba49e7`

Branch: `codex/r0-bootstrap-remediation`

Scope: R0-000/R0-001 remediation only

## Premature merge record

PR #2 merged before its mandatory Prompt 3 review, remediation, clean re-review, and complete currently applicable acceptance evidence. The post-merge audit comment marked it `changes-required`. PR #2 is historical implementation input, not a satisfied Prompt 3/4 or Release 0 gate. No later task, target execution, certification, or boot authorization may rely on that merge as acceptance evidence.

## Audit finding closure map

| Audit finding | Remediation | Defensive evidence |
| --- | --- | --- |
| Review and acceptance gate bypassed | This record and ADR 0011 preserve the failure history; the remediation uses a new branch/PR and requires fresh correctness/security review before merge | GitHub PR #2 audit; new review receipts on the remediation PR |
| Resolver trusted claimed emulator hash/path | Gate independently opens the canonical regular non-symlink executable, streams fresh bytes, checks stable identity, and passes the same descriptor to the spawner boundary | Resolver lie, nonexistent, symlink, wrong-byte, stable-handle, and positive mock tests |
| Accepted commands executed ambient tools | Version-3 platform locks, bounded preparser roots, pre-execution hashes, complete closure evidence, and an immutable CI image replace ambient lookup; pinned Git runs only after verification | Every accepted route, including `test`, runs under poisoned caller `PATH`; wrong-byte root/closure canaries |
| Output creation was pathname check-then-use | Durable writers use held descriptors and verify staging before rename; generated CI host binaries execute through `/proc/self/fd`; macOS refuses because equivalent descriptor execution is unavailable | Concurrent writers, parent replacement, interruption/fault cleanup, private-directory cleanup, and descriptor execution |
| Governance ownership evidence missing | ADR 0011 records the owner-directed coordinator write set and historical basis for the four already-merged governance files | ADR index and specification checks |
| Build/evidence state was hard-coded | Versioned renderers derive state from one verified clean commit/tree, lock, probe, and source snapshot | Contract conformance plus nonexistent-object, dirtiness, lock-swap, and source-mutation tests |
| File bounds and timestamps were incomplete | Tool-lock file reads are bounded before allocation; hashing streams; dates use Gregorian month/leap rules | Oversized on-disk lock, multi-buffer hash, invalid month-day, century, and valid leap-day tests |
| CI omitted host suites and artifact criterion was physically premature | Pinned OCI CI runs both host suites; ADR 0011 proves deterministic planning now and retains identical unsigned artifacts as a blocking pre-R0-close gate | Workflow, two-plan equality, and deferred-mandatory evidence marker |

## First re-review remediation

The first correctness and security re-reviews examined head `5df49b052f2e2e4e99750ee448c3111145765f34` and returned changes-required. Their accepted findings are closed in the later PR head as follows:

| Re-review finding | Closure |
| --- | --- |
| CI used unstable `gnu-lld` flavor | Separately measured Linux lock uses stable GCC 14 in the digest-pinned image; exact-head CI must pass |
| Artifact/firmware/disk/emulator pathname races | The pathless spawner boundary consumes all four verified no-follow handles by value; four replacement-race tests preserve original bytes |
| Compiler/linker executed before verification; closure incomplete | Bounded preparser hashes every non-root executable and checks Rust/SDK closure manifests; CI transitive closure is the immutable image digest |
| Linux accepted-route and `test` poison coverage missing | CI has a platform lock and exercises check/build/image/evidence/test under poisoned caller `PATH` without a platform skip |
| Concurrent output deletion and leaked private directories | Same-descriptor pre-rename verification, no post-commit unlink, propagated cleanup errors, competing-writer tests, traps, and unique cleanup directories |
| Darwin FFI mode ABI mismatch | Platform `mode_t` aliases, promoted variadic types, compile-time size assertions, and mode/failure tests |
| Versioned CLI contracts changed in place | New check/test/plan/evidence schema identities and field-order conformance files |
| ADR 0011 absent from approved specification set | Task packet approves ADRs 0001–0012 and CI checks the indexed approved range |
| Claimed Git revision could be nonexistent or mixed with mutable state | Pinned Git verifies commit/tree objects and cleanliness; one snapshot is captured and revalidated before output |
| Host tests reopened a hashed script pathname | Bounded captured script text is passed directly to the pinned shell; pathname replacement test proves byte continuity |
| Shell records were unbounded before Rust | Verified byte/line helpers bound lock and policy files before shell `read`; oversized/unknown record tests fail before compilation |
| Incremental SHA-256 short-read coverage incomplete | Boundary, fixed chunk, deterministic randomized chunk, official vector, and read-fault tests cover the streaming carry path |
| Build/evidence could publish a stale captured tool probe | Every revalidation reruns and compares the complete locked probe; the atomic writer invokes the full snapshot guard after staging and immediately before rename |
| Class B license/provenance/setup evidence was incomplete | A strict 15-entry inventory covers selected platform tools, SDK/image/action inputs, and external CI boundaries; shell and Rust checks enforce canonical complete records |
| Git object validation assumed SHA-1 only | Source snapshot validation accepts canonical lowercase 40- or 64-hex Git object IDs and tests both formats |
| Single-launch authorization could be replayed | Consumption is now a mandatory pre-resolver boundary with replay/failure semantics; rollbackable repository markers were rejected, no production consumer is shipped, and a monotonic owner-reviewed authority remains a blocking first-launch prerequisite |
| Lock-selected matching executable path/hash pairs were self-authorizing | The preparser binds each complete platform lock to a reviewed whole-file SHA-256; the compiled verifier requires the same digest handoff before any lock-controlled process use |
| Host Rust sources could change before compiler reads | The shell authenticates pinned Git, verifies the exact clean workflow head, and materializes every compiler input from commit blobs into an exclusive directory before Rust runs |
| CI image identity could be spoofed by an environment string | The workflow pins the OCI digest, mounts userland read-only, verifies protected tool mounts and runner evidence, and keeps runner/kernel/container layers explicitly non-certifying |
| Bootstrap directory creation and recursive cleanup followed mutable ancestors | Creation traverses one component at a time from bound current directories; private allocation is parent-relative; cleanup removes an exact regular-file allowlist and never recursively deletes |
| Generated check certifiability omitted verified bootstrap statuses | Hasher, cleanup tool, environment sanitizer, closure, and Git now participate in the same all-required status predicate with one-at-a-time negative coverage |
| Commit and tree could observe different `HEAD` values | Tree identity is resolved from the already captured commit object, never from a second `HEAD` observation |
| Host scripts could swap before capture | Production test scripts and their bootstrap library are loaded from the captured Git commit and supplied as exact text to the pinned shell |
| Special-file opens could block before type rejection | Final regular-file opens use nonblocking no-follow flags and a FIFO test proves bounded refusal without a writer |
| Image-plan grammar lacked a versioned contract | `rar-image-plan-v3` has a canonical field-order contract and renderer conformance coverage |
| Git index flags could hide workspace changes | `assume-unchanged`/`skip-worktree` are rejected; source, manifest, and inventory hashes derive from captured commit objects; compiler inputs use the same commit blobs |
| Approval-marker reads could block or allocate without bound after a path race | Root validation now uses the bounded nonblocking no-follow reader; oversized and post-check FIFO replacement tests fail closed |
| Runner attestation was checked inside a container that does not inherit GitHub's runner-only image variables | A separate non-certifying runner job verifies the reviewed image, hands exact values to the read-only container through named outputs, and the bootstrap rejects missing, mismatched, or legacy-only environment variables |
| Approval-marker defensive tests omitted their synthetic `docs/tasks/` fixture directory | The fixture now creates the complete required marker tree before oversized-read and FIFO-replacement assertions execute in pinned CI |

## Owned paths and coordinator handoff

Implementation changes remain in R0-000 and R0-001 owned paths: `tools/rar-lab/safety/`, `tools/rarbuild/`, `tools/toolchain/`, `tests/host-safety/`, `tests/bootstrap/`, `docs/release-0/host-safety/`, and `docs/release-0/build/`.

The owner-directed remediation grants coordinator ownership only for the required governance/CI corrections in ADR 0011, `docs/README.md`, `docs/tasks/release-0.md`, `tools/ci/check-specs.sh`, and `.github/workflows/specifications.yml`. It records the historical ownership basis for `.codex/config.toml`, `AGENTS.md`, `docs/v1-alpha-execution.md`, and `tools/ci/check-host-policy.sh` without changing those four files in this remediation.

## Non-execution and dependency attestation

No target source, target artifact, target linker invocation, QEMU execution, firmware execution, VM image, boot image, networked VM, physical device, unsafe target code, assembly, target-linked third-party code, or Dependency Exception Record is present. Host-only unsafe is limited to documented Unix descriptor bindings. Prompt 5 and R0-002 have not begun.

## Deferred mandatory gate

After target artifacts exist, two clean builds from the same checkout and locked inputs must produce byte-identical unsigned target artifacts for every required Release 0 target. R0-009 cannot close Release 0 without exact byte-comparison evidence. Missing, skipped, unequal, or unexplained output blocks closure.
