# Prompt 4 Bootstrap Remediation Record

Status: Implementation and host-only tests complete; independent re-review pending

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
| Accepted commands executed ambient tools | Version-2 lock defines the bootstrap axiom; wrapper uses builtins plus absolute pinned roots; compiled code hashes roots and parses Git metadata directly; `rustup`/`git` subprocesses are removed | Accepted-route poisoned-`PATH` canaries and closed route tests |
| Output creation was pathname check-then-use | Durable writers traverse/create/open/rename/unlink relative to held descriptors with exclusive no-follow temporary files and post-commit hashing; shell bootstrap claim is narrowed to unique private directories | Atomic replacement, parent replacement, interruption cleanup, and unsafe-invariant tests |
| Governance ownership evidence missing | ADR 0011 records the owner-directed coordinator write set and historical basis for the four already-merged governance files | ADR index and specification checks |
| Build/evidence state was hard-coded | Version-2 renderers derive unavailable/pinned and certification state from validated lock/probe values | Fully pinned synthetic plan/evidence tests |
| File bounds and timestamps were incomplete | Tool-lock file reads are bounded before allocation; hashing streams; dates use Gregorian month/leap rules | Oversized on-disk lock, multi-buffer hash, invalid month-day, century, and valid leap-day tests |
| CI omitted host suites and artifact criterion was physically premature | Pinned OCI CI runs both host suites; ADR 0011 proves deterministic planning now and retains identical unsigned artifacts as a blocking pre-R0-close gate | Workflow, two-plan equality, and deferred-mandatory evidence marker |

## Owned paths and coordinator handoff

Implementation changes remain in R0-000 and R0-001 owned paths: `tools/rar-lab/safety/`, `tools/rarbuild/`, `tools/toolchain/`, `tests/host-safety/`, `tests/bootstrap/`, `docs/release-0/host-safety/`, and `docs/release-0/build/`.

The owner-directed remediation grants coordinator ownership only for the required governance/CI corrections in ADR 0011, `docs/README.md`, `docs/tasks/release-0.md`, `tools/ci/check-specs.sh`, and `.github/workflows/specifications.yml`. It records the historical ownership basis for `.codex/config.toml`, `AGENTS.md`, `docs/v1-alpha-execution.md`, and `tools/ci/check-host-policy.sh` without changing those four files in this remediation.

## Non-execution and dependency attestation

No target source, target artifact, target linker invocation, QEMU execution, firmware execution, VM image, boot image, networked VM, physical device, unsafe target code, assembly, target-linked third-party code, or Dependency Exception Record is present. Host-only unsafe is limited to documented Unix descriptor bindings. Prompt 5 and R0-002 have not begun.

## Deferred mandatory gate

After target artifacts exist, two clean builds from the same checkout and locked inputs must produce byte-identical unsigned target artifacts for every required Release 0 target. R0-009 cannot close Release 0 without exact byte-comparison evidence. Missing, skipped, unequal, or unexplained output blocks closure.
