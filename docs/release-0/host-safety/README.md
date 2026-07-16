# R0-000 host-safety scaffold

Status: Implemented host-only scaffold; guest execution remains impossible

## Boundary delivered

The RAR Lab safety library owns a strict version-1 VM profile parser, a typed command
plan, content-addressed certification and owner-authorization records, bounded resource
limits, workspace-path validation, and the pre-resolution launch gate.

The repository root must be an already-canonical absolute path with regular, non-symlink
`Cargo.toml`, `AGENTS.md`, approved Gate 0/Release 0 documents, and a real `.git` directory
or worktree file. Checkout-name assumptions, root aliases, and symlinked markers are rejected.
Profiles are bounded to 8 KiB, certifications to 4 KiB, authorizations to 2 KiB, and every
record line to 512 bytes before field parsing.

The shipped `LaunchPolicy` contains no approved certification digest and no approved
owner-authorization digest. `rarbuild run`, execution aliases, delegation names, arbitrary
emulator arguments, and every argument-bearing `rarbuild test` mode refuse before the
wrapper discovers the repository root, compiles a host tool, reads a record, resolves an
emulator, or calls a spawner. The compiled CLI independently classifies the same routes.
No emulator process-spawning implementation is present.

Certification and authorization are separate:

1. A certification record binds the canonical profile, generated command, tool lock,
   emulator, firmware, target artifact, and source revision. Its self-digest determines its
   path below `out/r0/evidence/certifications/`.
2. An owner record binds that certification, profile, and artifact for one launch. Its
   self-digest determines its separate path below `out/r0/authorizations/`.
3. Local files have no authority by themselves. A future reviewed policy must pin both
   exact record digests before the resolver can be called.

After both records bind successfully, but still before executable resolution, the gate
requires regular non-symlink firmware, artifact, and disposable-disk files at their exact
class-scoped `out/r0` paths. It freshly hashes firmware and artifact bytes and compares them
to the pins, request, and certification. Missing files, symlink ancestors/finals, root
aliases, and post-certification byte changes refuse with resolver/spawner counters at zero.

The SHA-256 implementation provides deterministic host-side content addressing and is
tested with public vectors. It is not a signature or proof of who approved a record.

## Exact host-only validation

From the repository checkout root:

```sh
tests/host-safety/run.sh
tools/rarbuild/rarbuild run
tools/rarbuild/rarbuild test vm
```

Observed results on 2026-07-16:

- `tests/host-safety/run.sh`: exit 0; 19 passed, 0 failed.
- `tools/rarbuild/rarbuild run`: exit 73; certification and owner authorization not
  approved; `resolver_invoked=false`; `spawner_invoked=false`.
- `tools/rarbuild/rarbuild test vm`: exit 73; execution-capable test mode refused;
  `resolver_invoked=false`; `spawner_invoked=false`.

The negative suite covers raw `/dev/disk*` and `/dev/rdisk*` paths, `/Volumes` and other
host paths, traversal and symlink ancestors, persistent/raw disks, host devices, USB/VFIO
passthrough, shared folders, clipboard, networking, native acceleration, graphical display,
elevation, missing sandboxing, unsafe serial modes, unbounded CPU/memory/runtime/output,
malformed/duplicate/unknown/reordered fields, emulator aliases, architecture mismatches,
arbitrary arguments, absent/mismatched pins, altered certification, altered authorization,
wrong content-addressed paths, mismatched artifact/source bindings, oversized records/lines,
missing backing files, root aliases, file symlinks, and changed artifact/firmware bytes.

## Acceptance mapping

| R0-000 acceptance | Evidence | State |
| --- | --- | --- |
| No target artifact executes | Both suites and all CLI output state `target_execution=not-attempted`; no spawner exists | Pass |
| Forbidden configuration classes are rejected | 19-test host-safety suite plus typed-command and file-backed gate inspection | Pass |
| Command references only allowlisted pinned resources and bounded limits | `generated_command_contains_only_the_typed_isolated_model`; profile parser and `CertificationPins` | Pass for static construction; no command is certified |
| First guest execution is separately owner-gated | Separate record schemas and two independently pinned policy digests | Pass in implementation; independent Prompt 3 review remains |
| Unauthorized routes refuse before resolution/spawn | Resolver and spawner counters remain zero for every absent or mismatched state; wrapper order test covers aliases and test modes | Pass |

## Security and unsafe review

- `unsafe` is forbidden at crate and test roots; there are no unsafe blocks or assembly.
- Host code uses Rust `std` only. There is no target code and no target dependency.
- Profile fields cannot express shell fragments, environment wrappers, helper programs,
  arbitrary QEMU arguments, devices, or delegation.
- Relative paths are class-scoped below `out/r0`; absolute paths, traversal, malformed
  components, root aliases, symlink ancestors/finals, and non-regular files are rejected.
- Certification checks bind canonical bytes to pins and records; actual artifact and firmware
  hashes and the disposable disk's regular-file status are checked before the resolver.
- A synthetic all-valid unit fixture reaches only mock resolver/spawner implementations. It
  creates bounded non-executable bytes under unique repository `out/r0` test directories,
  removes them afterward, and never resolves or executes a host program.
- SHA-256 tests cover empty, `abc`, and the standard long public vector.

## Limitations and remaining risks

- QEMU, external LLD, and x86-64/ARM64 firmware are absent and unpinned, so certification
  is impossible. No digest was invented.
- No certification or owner-authorization record exists, and the shipped policy pins none.
- Content addressing detects changes but does not authenticate an owner. The future owner
  checkpoint must pin the exact authorization digest through the approved governance path.
- Single-use authorization consumption, timeout enforcement, and forced termination belong
  beside a future reviewed spawner. No spawner is shipped now.
- Static path validation cannot replace descriptor-based anti-race handling in that future
  spawner; it must revalidate/open pinned files without following symlinks.
- Prompt 3 independent correctness and security review is still required. This implementation
  does not self-approve the safety boundary.

## Target non-execution attestation

During R0-000 implementation and validation, no QEMU command, firmware, target binary,
boot image, VM image, or RAR target artifact was executed. Only host Rust compilation,
host Rust tests, file hashing, static parsing, deterministic plan generation, Git revision
inspection, and the pre-spawn refusal routes ran.
