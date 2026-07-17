# R0-000 host-safety remediation

Status: Prompt 4 host-only remediation implemented; guest execution remains impossible

## Boundary delivered

The RAR Lab safety library owns the strict version-1 VM profile, certification, and owner-authorization parsers; typed command planning; bounded resource limits; workspace-path validation; the atomic authorization-consumption boundary; and the pre-spawn launch gate.

The shipped `LaunchPolicy` contains no approved certification or owner-authorization digest. `rarbuild run`, aliases, delegation names, arbitrary emulator arguments, and argument-bearing `rarbuild test` routes refuse before repository discovery, host-tool compilation, record parsing, authorization consumption, executable resolution, or spawning. No production authorization consumer, real emulator resolver, or process spawner is shipped.

## Certification and authorization separation

1. Certification binds canonical profile and command bytes, tool lock, emulator, firmware, target artifact, and source revision.
2. Owner authorization separately binds that certification, profile, and artifact to one permitted launch.
3. Content-addressed files have no authority by themselves; future reviewed launcher policy must pin both exact record digests.
4. The artifact, firmware, and disposable disk must be canonical regular non-symlink files in their class-scoped workspace locations before resolver delegation, and their no-follow descriptors remain open through delegation.
5. After all policy, record, pin, path, and resource checks pass, `AuthorizationConsumer::consume_once` atomically consumes a key containing the validated authorization digest and nonce plus certification/profile/artifact bindings. Only then may the resolver run.

An authorization-consumer success is an irreversible commit. Resolver failure, emulator verification failure, spawner failure, launcher crash, or uncertain downstream state must never restore that authorization. Replay, consumer storage failure, and uncertain consumer commit state fail before resolver or spawner delegation. Repository-local marker files cannot meet that promise because the writable repository can be rolled back. The boundary therefore has only hostile-state test doubles. A future real launch path requires an owner-reviewed monotonic authority outside repository state; without it, a real resolver or spawner is forbidden.

Profile input is bounded to 8 KiB, certification to 4 KiB, authorization to 2 KiB, repository approval markers to 1 MiB, and every record line to 512 bytes. Root validation reads markers through nonblocking descriptor-relative no-follow traversal, so a post-metadata FIFO replacement refuses without waiting for a writer. Timestamps use actual Gregorian month lengths and leap-year rules.

## Descriptor-bound launch resolution

A resolver claim is not trusted as proof of an emulator. After all policy and record bindings pass, the gate independently opens the artifact, firmware, and disposable disk with descriptor-relative no-follow traversal and verifies their bindings. It then irreversibly consumes the authorization before invoking the resolver. The resolved emulator is independently opened with the same no-follow discipline. The gate hashes artifact, firmware, and emulator bytes from those same descriptors, checks stable descriptor metadata and pathname identity during opening, and rewinds the verified handles.

The `ProcessSpawner` boundary consumes a command by value containing:

- the four opened verified resources (firmware is optional for Tier 0);
- pathless typed argument markers that require the spawner to bind those exact handles;
- fixed resource limits.

It receives no workspace root, artifact path, firmware path, disk path, emulator path, or pre-rendered pathname argument vector. The static `CommandPlan` still renders canonical paths only for profile certification; it is not the spawn authority. Four deterministic race tests replace each pathname after every descriptor is verified and before mock delegation. The pathname then contains substituted bytes while the mock spawner reads the original verified bytes from every handle, proving the substituted object cannot enter the authorized command.

For the emulator, the gate additionally:

- requires an absolute canonical path;
- requires a regular final file;
- verifies device, inode, size, modification time, and change time remain stable during hashing;
- compares the actual digest with the immutable emulator pin;
- rejects resolver hash assertions, aliases, missing files, symlinks, wrong bytes, and metadata changes.

The positive fixtures are non-executable synthetic byte strings and reach only mock resolver/spawner implementations.

## Streaming and descriptor safety

SHA-256 uses fixed-memory streaming for files of any accepted size. Tool, firmware, artifact, source, and emulator inputs are never buffered in full by the hashing path. Tests cover public SHA-256 outputs, padding boundaries at 55, 56, 63, 64, and 65 bytes, fixed short reads of 1, 7, 63, and 65 bytes, deterministic randomized short reads over the million-`a` vector, and injected read faults.

The Unix descriptor module uses RAR-owned bindings for `openat`, `mkdirat`, `renameat`, and `unlinkat`. Unsafe code is confined to that file and documents these invariants:

- C pathnames are NUL-free single components;
- directory descriptors remain live for each call;
- returned descriptors transfer ownership exactly once to Rust `File` values;
- pointer lifetimes cover each call;
- Darwin fixed `mkdirat` mode uses 16-bit `mode_t`, and variadic `openat` mode uses default-promoted `c_int`;
- Linux fixed and variadic modes use `c_uint`;
- compile-time type/size checks and platform-gated runtime mode/failure tests cover the supported ABIs.

The same module provides descriptor-relative atomic output for R0-001. A temporary file is opened read/write, written and synchronized, rewound, and hashed through that same descriptor before rename. Every directory component created by `mkdirat` is synchronized through its parent descriptor. The parent binding is rechecked before rename, and the renamed directory is synchronized after commit.

Pre-commit failures remove and directory-sync the temporary entry; unlink or cleanup-sync failures are returned as `output-cleanup-failed`. After rename, failure paths never unlink the destination because another writer may already own that pathname. Injected write, file-fsync, rename, and unlink failures cover cleanup and propagation. Competing-writer and replace-after-commit tests prove an earlier writer cannot delete a later valid output.

## Acceptance mapping

| R0-000 acceptance | Evidence | State |
| --- | --- | --- |
| No target artifact executes | Refusal output and all suites state `target_execution=not-attempted`; no real spawner exists | Pass |
| Forbidden configuration classes are rejected | Typed profile, command, file, resource, and malformed-input negative corpus | Pass |
| Generated command references only allowlisted pinned resources and bounded limits | `generated_command_contains_only_the_typed_isolated_model` and strict `CertificationPins` validation | Pass for static construction; no command is certified |
| First guest execution is separately owner-gated | Independent certification and single-launch owner record schemas plus two immutable policy pins | Pass in implementation; later owner checkpoint remains mandatory |
| Fully resolved launch resources are validated before spawn | Fresh descriptor-backed hashes, stable identity checks, four pathname replacement races, pathless typed arguments, and same-handle continuity | Pass |
| Unauthorized routes refuse before resolution or spawn | Resolver/spawner counters remain zero across all incomplete and mismatched states | Pass |
| One-launch authorization is enforced at delegation | Required consumer call occurs before resolver; sequential/concurrent replay, consumer failure, and irreversible post-consumption resolver/spawner failure tests cover the protocol | Boundary implemented; monotonic production authority remains a blocking first-launch gate |

## Exact host-only validation

```sh
tests/host-safety/run.sh
tools/rarbuild/rarbuild run
tools/rarbuild/rarbuild test vm
```

The suite count is reported by exact-head CI. Under ADR 0012, local macOS invocation intentionally returns 2 before Rust compilation because generated Mach-O execution awaits a descriptor-bound launcher. The suite therefore requires the pinned Linux CI bootstrap for executable evidence. The two CLI refusal commands return 73 and report `resolver_invoked=false`, `spawner_invoked=false`, and `target_execution=not-attempted`.

## Remaining risks

- QEMU, external LLD, and x86-64/ARM64 firmware remain absent and unpinned, so certification is impossible.
- No certification or owner-authorization record exists, and shipped policy pins neither.
- SHA-256 content addressing detects changes but does not authenticate an owner.
- The approved schema does not content-pin the disposable disk; this remediation preserves the exact opened disk handle but does not invent a new stable certification field.
- Open descriptors defeat pathname replacement, but they do not prevent an independently writable same inode from being modified in place; ownership and immutability of future certified inputs still require launcher/profile review.
- No repository-confined implementation can provide rollback-resistant one-shot authority. A monotonic external authority requires separate owner review before any real launcher exists; this requirement is not waived or reported as passed.
- Timeout enforcement, forced termination, and process-lifecycle cleanup belong beside a future reviewed real spawner.
- Descriptor continuity is implemented at the API boundary, but no subprocess receives or executes those descriptors because no spawner exists.
- Linux ABI assertions and tests are cfg-valid but were not executed on this macOS host; Linux CI remains required evidence.
- If the final directory sync fails after rename, the writer returns failure while preserving the destination; callers must treat commit state as uncertain and inspect evidence instead of deleting the path.

## Target non-execution attestation

No QEMU command, firmware, target linker, target binary, boot image, VM image, target artifact, emulator, or physical device was executed. Pinned Linux CI may compile and execute only the host Rust tests; local macOS validation is limited to shell/specification/policy checks, bounded closure verification, and refusal paths. R0-002 and Prompt 5 have not begun.
