# Sprint Alpha 0.1 Vertical Implementation Packet

Status: Owner-approved execution contract — 2026-08-25
Deadline: 2026-08-30 23:59 America/Los_Angeles

## Objective

Deliver the minimum authentic x86-64 RAR OS described by the eight-item
completion contract in `../sprint-alpha.md`. A clean cloud run must build, boot,
accept input, open the required system surfaces and apps, contain a component
crash, preserve test data through recovery, and prove signed-layer rejection
and rollback. A host mock, Linux-hosted UI, or unbooted image is not acceptance.

## Approved specifications

- Constitution, from-scratch policy, architecture, security/recovery, formats,
  host safety, ADRs 0001–0021, and `release-0.md`.
- R0-002 boot and hardware contracts under `spec/boot/`, `spec/hardware/`, and
  `sdk/generated/release-0/` are authoritative and read-only dependencies unless
  an ADR-governed correction is independently approved.
- New Alpha-only contracts use an explicit experimental version under
  `spec/alpha/`; they are not stable RAR ABI, RID, package, storage, or update
  promises.
- The trusted cumulative A–G observation sequence is fixed by
  `../../spec/alpha/evidence/acceptance-v1.plan`; implementation cannot replace
  a required guest result with a generic ready marker.

## Preconditions

Implementation does not start until all of these pass:

1. The reviewed `.codex/rar-os-ssd-user-fragment.toml` is installed once in the
   owner's user-level Codex configuration, the repository-selected
   `rar-os-ssd` profile is selected in a fresh SSD-root task, and no legacy
   sandbox override is selected. The owner retains one-time product evidence
   that an in-subtree write succeeds and outside-subtree read/write attempts are
   denied; repository scripts cannot self-attest the active task's profile.
2. `tools/ci/check-local-sprint-preflight.sh` passes from a clean, fully pushed
   SSD worktree with at least 10 GiB free on the Mac's internal disk.
3. `tools/ci/check-remote-sprint-preflight.sh` proves the exact GitHub head,
   a successful required workflow with real steps, and an immutable checkpoint.
4. ADR 0020 is accepted and new reviewed Development Lab, image, and crypto
   inventory schemas bind the selected isolated reference topology. The active
   Lab profile becomes `ready` only with real reviewed identities for every
   build, reference, and launch role plus compiler, linker, QEMU, firmware,
   machine profile, QMP client, and reference executables. The ready controller
   must be merged to `main` before a source-branch Development Probe can run.
5. PR #7 is green, independently reviewed, merged, and verified on GitHub.
6. ADR 0021 and ADR 0023 are accepted, and the resulting Alpha boot contract is
   marked `ready` after fresh architecture, correctness, and security review,
   before Milestone A target files or image recipes are created.
7. ADR 0024 is accepted and real twice-reproduced helper build/test evidence,
   the ready v2 Lab profile, and the reviewed runnable v2 controller are merged
   before any source-branch Development Probe or untrusted target build runs.

PR #7 is also the one-time pre-A controller transition. Before any untrusted
Alpha source is built, `main` must contain the generic A–G dispatch controller,
role-separated build/launch images, frozen-artifact re-verification, sandboxed
trusted launcher, fixed A–G QMP scenario/evidence harness, dependency gate, and
Alpha-aware static check added by that PR. The A–G implementation branch
may not edit `.github/workflows/` or trusted `tools/ci/` controller code; any
later controller change is a separately reviewed ADR-governed `main` change.

No missing precondition may be converted into a mock or a local target build.

### Milestone-specific gate

ADR 0022 does not block Milestones A–D. It must be accepted, and its exact
experimental peripheral-grant contract must be marked ready after fresh review,
before Milestone E graphics or input target code begins.

## Branch, task, and checkpoint contract

- Use one branch `codex/sprint-alpha-vertical`, one worktree under the exact SSD
  `worktrees/` directory, one draft PR, one Medium-effort writer task, and no
  persistent goal.
- Finish A through G in order. Push a clean checkpoint after each milestone.
- Create annotated tags `sprint-alpha-0.1/A` through
  `sprint-alpha-0.1/G`. Push each once, verify its remote SHA, and never move,
  delete, force-push, rebase, or rewrite a published checkpoint.
- Diagnose a failure once and batch one repair. At most two retries are allowed;
  the third identical failure records one blocker in `SPRINT_STATUS.md` and
  stops without polling.
- Correctness and security reviewers remain read-only. Add architecture review
  for public-contract, trust-boundary, unsafe-code, or persistence changes.

## Scope and owned paths

The single writer owns only the active milestone paths plus shared manifests and
status files named here. Preserve unrelated files.

| Milestone | Required behavior | Owned paths |
| --- | --- | --- |
| A | Reproducible RAR-owned image; Root → Recovery → Nucleus; R0-002 validation; structured boot trace | `Cargo.toml`, `rust-toolchain.toml`, `boot/`, `recovery/`, `nucleus/arch/x86_64/`, `tools/sprint-alpha/`, `tests/sprint-alpha/boot/`, `docs/sprint-alpha/boot/` |
| B | Page allocator, mappings, protected address spaces, exceptions, timer, threads, scheduler | `nucleus/portable/`, `nucleus/runtime/`, `tests/sprint-alpha/nucleus/`, `docs/sprint-alpha/nucleus/` |
| C | Rights-checked handles, bounded IPC, timeout/cancellation, isolated components, crash/restart | `spec/alpha/capability/`, `spec/alpha/ipc/`, `nucleus/capability/`, `nucleus/ipc/`, `core/registry/`, `tests/sprint-alpha/isolation/`, `docs/sprint-alpha/isolation/` |
| D | Separate system/preserved-data regions; corruption isolation; reconstruction preserving the exact test file | `spec/alpha/state/`, `spec/alpha/recovery/`, `core/state/`, `core/recovery/`, `services/storage/`, `tests/sprint-alpha/recovery/`, `docs/sprint-alpha/recovery/` |
| E | Framebuffer GUI; keyboard and pointer; launcher, terminal, settings, and two native demo apps | `spec/alpha/surface/`, `spec/alpha/input/`, `services/graphics/`, `services/input/`, `apps/shell/`, `apps/terminal/`, `apps/settings/`, `apps/demo/`, `tests/sprint-alpha/gui/`, `docs/sprint-alpha/gui/` |
| F | Signed layer activation; tamper rejection; component replacement; failed-health rollback | `spec/alpha/layer/`, `spec/alpha/signing/`, `spec/alpha/update/`, `core/crypto/`, `core/package/`, `core/update/`, `tests/sprint-alpha/update/`, `docs/sprint-alpha/update/` |
| G | One clean retained A–F demonstration and complete user/build/debug/recovery/update/extension evidence | `spec/alpha/integration/`, `tests/sprint-alpha/end-to-end/`, `docs/sprint-alpha/`, `evidence/sprint-alpha/`, `SPRINT_STATUS.md`, `README.md` |

Shared manifest changes must be the minimum required for the active milestone.
No parallel writer may touch these paths.

## Dependencies

- Target-linked dependencies remain `none`. Rust compiler-provided `core` and
  approved freestanding compiler built-ins follow the existing from-scratch and
  toolchain records; adding any other linked code requires a Dependency
  Exception Record before use.
- QEMU, firmware, compiler/linker tools, GitHub Actions, and the OCI runtime are
  pinned host/lab tools, not RAR OS components. They never ship in target images.
- Initial platform is x86-64 `q35` under software emulation. No KVM, host device
  passthrough, guest networking, shared folders, cloud credential, or raw disk.
- UEFI and PE/COFF are external standards. RAR owns the target boot path and
  loader code after the firmware interface; no third-party bootloader is linked.

## Interface rules

- R0-002 input validation and failure codes are unchanged.
- Milestone C exposes only experimental capability/IPC semantics needed by the
  demonstration: opaque handles, non-increasing rights, bounded messages,
  bounded queues, timeout, cancellation, close, and peer-crash notification.
- Milestone D's on-image state format, Milestone E's surface/input contract, and
  Milestone F's layer manifest/signature/health record are experimental version
  0 contracts with deterministic encodings, length bounds, validation order,
  negative fixtures, and replacement notes in `spec/alpha/`.
- Milestone F uses only the laboratory Ed25519 signing and key model, exact
  signed-message construction, validation order, and evidence requirements in
  ADR 0019. It includes official RFC 8032 and hash vectors, two independently
  maintained digest-pinned host-reference comparisons, bounded fuzzing,
  constant-time analysis, and specialist cryptography/security review. The
  fixture private key is public test data and cannot support a production claim.
- Its two independent Class C references are OpenSSL 3.0.13 (Apache-2.0) and
  libsodium 1.0.19 (ISC), inventoried in
  `tools/sprint-alpha/alpha-crypto-references-v1.env`. Version 1 is permanently
  blocked and grants no executable authority. Before F, ADR 0020 must be
  accepted and a new reviewed inventory/topology schema must bind the isolated
  reference role, executable/harness identities, comparison evidence, and
  absence from the untrusted build image. The references are host-only
  comparison oracles and never link into or ship with RAR target code.
- No ambient syscall, global administrator mode, direct device access from apps,
  raw Rust ABI, executable pointer, or undocumented cross-subsystem call is
  allowed. All authority is explicit and least-privilege.

## Required failure behavior and tests

Each milestone includes deterministic success, boundary, exhaustion, malformed,
and fault tests. The retained end-to-end run additionally proves:

1. malformed R0-002 input grants no authority;
2. invalid mapping and cross-address-space access are contained;
3. forged/stale/over-rights handles, oversized messages, full queues, timeout,
   cancellation, closed peers, and a crashed peer fail deterministically;
4. a deliberately crashed noncritical component restarts while GUI and one
   other app remain responsive;
5. system-region corruption activates Recovery while the pre-hashed preserved
   test file has the identical post-recovery hash;
6. a valid signed layer activates, a one-byte-tampered layer is rejected before
   execution, a failed health check rolls back, and an unaffected component
   continues without a whole-OS reboot;
7. scripted keyboard and pointer inputs open launcher, terminal, settings, and
   both demo apps, with framebuffer captures and trace correlation; and
8. two clean builds from the same commit and locked inputs produce identical
   unsigned target artifacts.

Every Development Probe is cumulative: its exact evidence plan includes all
rows introduced by that milestone and every earlier milestone. The controller
binds each result to an ordered post-input serial offset, requires each marked
capture, rejects every extra descendant, and uses the same reviewed timeout for
evidence readiness and retained-evidence release.

Unsafe Rust and assembly require adjacent invariants, focused tests, and an
independent security review. Tests and evidence may not weaken checks, replace
guest behavior with host behavior, or treat missing evidence as success.

## Documentation and acceptance evidence

Every behavior change updates its specification, tests, user-visible guide, and
`SPRINT_STATUS.md` in the same checkpoint. Evidence records exact source and tag
SHA, workflow/run/attempt, runner image, OCI digest, compiler/linker/QEMU/
firmware hashes, build plan, artifact hashes, QEMU command plan, serial and
structured traces, input script, framebuffer captures, fault results, resource
bounds, timeout, and final exit status. Secrets and owner data are never inputs
or artifacts.

Milestone G closes only when the eight completion items in `../sprint-alpha.md`
map to retained evidence from one clean exact-head run, all mandatory checks
pass, reviews report no blocker, the PR is conflict-free, and the merge is
verified on GitHub. Limitations remain explicit; unfinished roadmap releases
remain open.

## Stop conditions and known risks

Stop for a constitutional, trust-boundary, persistent-data, tier, dependency,
public-format, native-app-model, or release-commitment change and propose an ADR.
Also stop for missing lab pins, a zero-step Actions run, internal disk below the
minimum, inconsistent workspace guard marker, escaped worktree, force-push requirement,
unbounded output/runtime, unexplained reset, nondeterminism, or evidence loss.

The calendar target is aggressive because no target OS implementation exists.
This packet makes the minimum vertical slice possible and measurable; it does
not turn the date into permission to fake, weaken, or overclaim behavior.
