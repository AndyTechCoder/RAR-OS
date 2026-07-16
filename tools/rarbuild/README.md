# `rarbuild` Release 0 host CLI

The public host-only command surface remains `check`, `build`, `image`, refusal-only `run`, host-only `test`, and `evidence`. Every execution alias and argument-bearing test mode refuses before repository discovery, record reads, executable resolution, or process spawn.

Executable accepted routes currently run only in the digest-pinned Linux CI image with its separately measured lock. The macOS wrapper verifies bounded policy records, exact proposed roots, and both closure manifests, then exits 2 with `reason=local-bootstrap-execution-awaits-descriptor-bound-launcher` before compiler execution. This preserves the physical-Mac boundary instead of claiming pathname verification is descriptor-bound execution.

Within CI:

- `check` emits `rar-host-check-v2` and exits 3 because external LLD, QEMU, and firmware remain unavailable.
- `build` writes deterministic `rar-build-plan-v3`; it does not compile or link target code.
- `image` writes blocked `rar-image-plan-v2` and exits 4; it creates no bootable image.
- `test` captures and hashes the two host scripts once, passes those exact bytes to the pinned shell, and emits `rar-host-test-v2`.
- `evidence` writes `rar-build-evidence-v3` and exits 4 while target artifacts and certification inputs are absent.

Pinned Git verifies that `HEAD` names an existing commit and tree and that tracked and untracked source state is clean. One `BuildSnapshot` captures lock, probe, commit, tree, source inputs, tool manifest, and dependency inventory; every field and the source snapshot are revalidated before output publication. Missing objects, dirty source, lock swaps, and source mutation fail closed.

Durable output uses descriptor-relative no-follow traversal, synchronized exclusive staging, same-descriptor byte verification, atomic rename, and parent synchronization. A post-commit failure never unlinks the destination because a concurrent writer may already own that pathname.

Field-order contracts live in `tools/rarbuild/contracts/`. ADR 0011 retains two byte-identical clean unsigned target builds as a mandatory Release 0 closure gate after target artifacts exist. No current command satisfies or waives that deferred gate.
