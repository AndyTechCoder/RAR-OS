# R0-000 host-safety remediation

Status: Prompt 4 host-only remediation implemented; guest execution remains impossible

## Boundary delivered

The RAR Lab safety library owns the strict version-1 VM profile, certification, and owner-authorization parsers; typed command planning; bounded resource limits; workspace-path validation; and the pre-spawn launch gate.

The shipped `LaunchPolicy` contains no approved certification or owner-authorization digest. `rarbuild run`, aliases, delegation names, arbitrary emulator arguments, and argument-bearing `rarbuild test` routes refuse before repository discovery, host-tool compilation, record parsing, executable resolution, or spawning. No real emulator resolver or process spawner is shipped.

## Certification and authorization separation

1. Certification binds canonical profile and command bytes, tool lock, emulator, firmware, target artifact, and source revision.
2. Owner authorization separately binds that certification, profile, and artifact to one permitted launch.
3. Content-addressed files have no authority by themselves; future reviewed launcher policy must pin both exact record digests.
4. The artifact, firmware, and disposable disk must be canonical regular non-symlink files in their class-scoped workspace locations before resolver delegation.

Profile input is bounded to 8 KiB, certification to 4 KiB, authorization to 2 KiB, and every record line to 512 bytes. Timestamps use actual Gregorian month lengths and leap-year rules.

## Self-verifying executable resolution

A resolver claim is no longer trusted as proof of an emulator. After all policy, record, and file bindings pass, the gate independently:

- requires an absolute canonical path;
- opens every path component with descriptor-relative no-follow operations;
- requires a regular final file;
- streams a fresh SHA-256 digest from the opened descriptor;
- verifies device, inode, size, modification time, and change time remain stable during hashing;
- compares the actual digest with the immutable emulator pin;
- rewinds and passes that same verified open descriptor to the spawner boundary.

The spawner therefore receives descriptor continuity, not a pathname and resolver-supplied hash assertion. Negative tests cover a lying claimed hash, nonexistent path, final symlink, wrong bytes, and mutation-sensitive metadata. The positive fixture creates non-executable synthetic bytes and reaches only mock resolver/spawner implementations.

## Streaming and descriptor safety

SHA-256 now uses fixed-memory streaming for files of any accepted size. Tool, firmware, artifact, source, and emulator inputs are never buffered in full by the hashing path.

The Unix descriptor module uses RAR-owned bindings for `openat`, `mkdirat`, `renameat`, and `unlinkat`. Unsafe code is confined to that file and documents these invariants:

- C pathnames are NUL-free single components;
- directory descriptors remain live for each call;
- returned descriptors transfer ownership exactly once to Rust `File` values;
- pointer lifetimes cover each call;
- flag constants are audited for the supported macOS and Linux ABIs.

The same module provides descriptor-relative atomic output for R0-001. Focused tests replace an output parent during staging and prove no write follows the replacement symlink.

## Acceptance mapping

| R0-000 acceptance | Evidence | State |
| --- | --- | --- |
| No target artifact executes | Refusal output and all suites state `target_execution=not-attempted`; no real spawner exists | Pass |
| Forbidden configuration classes are rejected | Typed profile, command, file, resource, and malformed-input negative corpus | Pass |
| Generated command references only allowlisted pinned resources and bounded limits | `generated_command_contains_only_the_typed_isolated_model` and strict `CertificationPins` validation | Pass for static construction; no command is certified |
| First guest execution is separately owner-gated | Independent certification and single-launch owner record schemas plus two immutable policy pins | Pass in implementation; later owner checkpoint remains mandatory |
| Fully resolved emulator is validated before spawn | Fresh descriptor-backed byte hash, stable identity checks, same-handle continuity, and resolver-lie tests | Pass |
| Unauthorized routes refuse before resolution or spawn | Resolver/spawner counters remain zero across all incomplete and mismatched states | Pass |

## Exact host-only validation

```sh
tests/host-safety/run.sh
tools/rarbuild/rarbuild run
tools/rarbuild/rarbuild test vm
```

The remediation suite contains 21 tests. The two CLI refusal commands return 73 and report `resolver_invoked=false`, `spawner_invoked=false`, and `target_execution=not-attempted`.

## Remaining risks

- QEMU, external LLD, and x86-64/ARM64 firmware remain absent and unpinned, so certification is impossible.
- No certification or owner-authorization record exists, and shipped policy pins neither.
- SHA-256 content addressing detects changes but does not authenticate an owner.
- Single-use authorization consumption, timeout enforcement, forced termination, and process-lifecycle cleanup belong beside a future reviewed real spawner.
- Descriptor continuity is implemented at the API boundary, but no subprocess receives or executes that descriptor because no spawner exists.

## Target non-execution attestation

No QEMU command, firmware, target linker, target binary, boot image, VM image, target artifact, emulator, or physical device was executed. Validation used only host Rust compilation, host unit tests, bounded file reads, streaming hashes, static command generation, deterministic plan/evidence output, and refusal paths. R0-002 and Prompt 5 have not begun.
