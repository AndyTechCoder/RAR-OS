# ADR 0012: Release 0 Host Bootstrap Trust and Snapshot

Status: Accepted — 2026-07-16

Approval basis: the owner's Prompt 4 direction to remediate every accepted R0-000/R0-001 audit finding without weakening any RAR OS requirement.

## Context

The independent Prompt 4 review found that the original Release 0 wrapper executed an absolute compiler, linker, and directory tool before comparing their bytes, covered only a small part of the compiler's transitive inputs, and could combine mutable Git, lock, source, and output state in one evidence record. The same review found that the version-1 host check and test receipts had changed fields without changing schema identity.

These are host-bootstrap corrections inside R0-000/R0-001. They do not define a target ABI, authorize target execution, begin R0-002, or change the mandatory Release 0 target-artifact reproducibility gate in ADR 0011.

## Decision drivers

- No non-root bootstrap executable may run before its selected bytes are authenticated.
- The unavoidable first verifier must be smaller and more explicit than the compiler it verifies.
- Host compilation must bind the compiler driver, codegen, host standard library, selected target libraries, linker, and SDK link inputs that can affect generated host binaries.
- CI must use a separately reviewed Linux record rooted in the immutable container digest rather than pretending that the developer-specific macOS lock applies.
- Build plans and evidence must derive from one verified, clean Git commit/tree and one stable source/tool snapshot.
- Changed host receipt grammars require new schema identities and conformance tests.

## Considered options

- **Keep compiler, linker, and `mkdir` as unauthenticated path axioms:** rejected because a mismatched absolute path can execute before the later Rust verifier exists.
- **Require a complete RAR-native compiler before Release 0:** rejected because it contradicts the staged self-hosting decision in ADR 0010.
- **Treat only the compiler launcher and component-list manifests as the closure:** rejected because the launcher loads a compiler driver and the build consumes precompiled libraries and SDK linker inputs whose bytes are not authenticated by path-list manifests.
- **Use one lock on every host:** rejected because paths, ABI, sysroot, and tool bytes are platform-specific.
- **Selected approach — minimal bootstrap axiom, platform lock, closure verification, and immutable snapshot:** accepted.

## Decision

The local macOS preparser axiom is the sealed-system POSIX shell plus the exact system hasher and bounded-input helpers named and hashed in the repository-owned bootstrap code. The selected `rar-host-tool-lock-v3` record narrows all other tools to canonical absolute paths and hashes. The preparser bounds every policy/lock input, rejects unknown lock fields, verifies each non-root executable, verifies the closure-manifest files, and checks every file named by the Rust and SDK closure manifests. The macOS closure includes the Rust compiler driver, codegen backend, host standard/test libraries, selected bare-metal target libraries, Rust linker tools, component manifests, and SDK link stubs used by the host link.

macOS does not provide the descriptor-execution primitive required to bind the generated Mach-O object without reopening its mutable pathname. Therefore this Release 0 implementation treats the macOS lock and closure as diagnostic/preparation evidence only: local compile, test, build-plan, image-plan, and evidence routes refuse after verification and before compiler execution. The physical Mac remains source/build storage. A later macOS host route requires a separately reviewed descriptor-bound launcher or an equivalently immutable execution environment; this ADR does not pre-approve one.

The Linux CI record is separate. Its trust root is the official Rust 1.95.0 OCI image pinned by immutable digest in the workflow. The container root is mounted read-only, the bootstrap verifies that `/usr` and executable/library mount descendants are not writable, and the lock records exact in-image paths and hashes for the shell, hasher, bounded-input helpers, compiler, GCC driver, environment sanitizer, Cargo, Git, and sysroot marker. The image digest plus read-only userland is the selected CI tool closure; the environment string alone is not attestation, and the hosted kernel/container service remains the explicit non-certifying boundary.

The versioned Class B inventory records version/identity, integrity pin, license, provenance, setup source, and status for every selected host-tool group, SDK, OCI image, GitHub action, and CI orchestration boundary. The hosted runner and container engine remain external attested service layers rather than part of the OCI digest, so the workflow checks the reviewed runner-image version and records both layers as non-certifying. They may run current host tests but cannot support target certification or target-artifact reproducibility evidence.

Before parsing a selected lock, the preparser hasher compares the whole lock with its reviewed SHA-256. The compiled verifier accepts only the same hard-coded digest and requires the shell-authenticated digest handoff. A lock-controlled matching path/hash pair therefore cannot nominate a new executable authority. Git is also selected by that immutable lock and authenticated before use.

Before compiler execution, the shell verifies the exact workflow-selected clean Git commit and materializes every Rust source/fixture byte from that commit's blobs into an exclusive mode-`0700` directory. Rust never compiles mutable workspace source. After the trusted compiler returns, the shell's first external-action boundary is opening that private output; closure revalidation then occurs while the generated inode is already held, and execution uses that descriptor. Private-directory creation and cleanup operate relative to bound current directories; cleanup removes an exact regular-file allowlist and never uses recursive deletion. Production host test scripts and the bootstrap library are likewise read from the captured Git revision and supplied as captured text to the pinned shell; transient workspace replacement cannot substitute their executed bytes. This route assumes the isolated CI job does not introduce an unreviewed concurrent same-UID process; doing so invalidates the bootstrap boundary and requires new review.

The shell establishes exact Git source identity before host compilation; the compiled verifier independently repeats it. Planning/evidence routes require `HEAD` to resolve to an existing SHA-1 or SHA-256 commit, resolve the tree from that captured commit rather than a second mutable `HEAD`, reject `assume-unchanged` and `skip-worktree` index flags, and require tracked and untracked source state to be clean. The source-input digest is SHA-256 over the canonical recursive `ls-tree` blob/mode/path listing from the captured commit; manifest and inventory digests likewise come from that commit's blobs, never mutable workspace paths. The complete locked tool probe and every snapshot field are checked again after durable output staging and immediately before atomic rename. Dirty trees, hidden index state, missing objects, tool/closure drift, lock swaps, and source mutation fail closed.

The corrected host schemas are:

- `rar-host-tool-lock-v3`
- `rar-host-check-v2`
- `rar-host-test-v2`
- `rar-build-plan-v3`
- `rar-image-plan-v3`
- `rar-build-evidence-v3`

Their field contracts live under `tools/rarbuild/contracts/` or the canonical lock records under `tools/toolchain/`. They are host-only Release 0 contracts, not RAR target interfaces.

## Consequences

- The first-execution axiom is explicit, reviewable, and much smaller than the compiler closure.
- Local macOS routes verify the proposed closure but do not execute it; full executable host validation runs in the pinned Linux CI root.
- CI and macOS records can change independently only through a reviewed schema-preserving lock update or a versioned correction; Class B license/provenance/setup records change in the same review.
- Evidence refuses dirty or unverifiable Git worktrees; ad hoc archives with a claimed `.git/HEAD` value cannot emit release evidence.
- Adding a new output-affecting compiler, linker, SDK, target-library, or script input requires updating its closure evidence and tests.
- This does not make the VM alpha production-secure and does not satisfy the deferred two-clean-build target-artifact gate.

## Security and data impact

The decision narrows host execution authority, binds release evidence to an existing clean Git object, and prevents mutable path substitution for generated binaries and host test scripts. It does not grant access to RAR user data, host devices, physical media, networking, signing keys, or target execution. Closure manifests contain only hashes and relative paths for Class B host inputs; they contain no compiler or SDK payloads.

## Compatibility and migration

Version-1/2 host records remain historical evidence and are not silently reinterpreted. Strict consumers select the new schema names. The macOS and Linux CI locks are intentionally separate; replacing paths, hosts, compiler versions, image digest, or closure inputs requires remeasurement and review. No RAR target format or later-release interface migrates under this decision.

## Validation

- Wrong-byte absolute compiler, linker, driver, and standard-library fixtures fail before their canaries can execute.
- Matching path/hash lock substitutions fail against the separately reviewed whole-lock digest.
- Shell-preparser tests reject oversized files, oversized lines, unknown fields, and malformed records before compilation.
- Linux CI runs every accepted route under a poisoned `PATH` with the platform-specific lock.
- Missing Git objects, dirty source, lock replacement, and source mutation fail snapshot validation.
- Hidden index flags are rejected, and transient workspace bytes cannot change the commit-derived source-input digest.
- Tool-probe changes fail snapshot comparison, and source mutation at the writer's pre-commit boundary leaves no published destination.
- CI enforces read-only tool mounts; wrappers compile exact Git blobs and use non-recursive private cleanup.
- CI validates the complete Class B inventory and refuses an unreviewed hosted-runner image version.
- Contract tests compare canonical renderer field order with every versioned host field contract.
- No test executes a RAR target artifact or emulator.

## Replacement path

Release 6 replaces the host bootstrap with the approved self-hosting stages from ADR 0010. Before then, another host compiler or image may replace either bootstrap record only with equivalent provenance, closure, clean-snapshot, poisoned-environment, and reproducibility evidence.
