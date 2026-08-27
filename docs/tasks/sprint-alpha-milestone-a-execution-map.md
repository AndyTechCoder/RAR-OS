# Sprint Alpha Milestone A Execution Map

Status: Non-authoritative preparation — implementation remains blocked

This map turns the owner-approved Sprint Alpha vertical packet into a single,
ordered writer checklist. It adds no interface, byte format, architecture
decision, dependency, execution authority, or acceptance claim. If it conflicts
with `sprint-alpha-vertical.md`, an accepted ADR, or a reviewed contract, those
sources win.

## Start gate

Do not create Milestone A target files or image recipes until every cumulative
precondition in `sprint-alpha-vertical.md` passes. In particular:

- GitHub Actions must run real steps at the exact PR head;
- PR #7 must be green, reviewed, merged, and remotely verified;
- ADRs 0023 and 0024 must be unambiguously accepted;
- the Alpha boot contract must be reviewed and marked `ready`;
- the v2 Lab profile, controller, and helper evidence must be genuinely ready;
- the SSD confinement profile and capacity gates must have retained evidence.

No local Mac command may compile or link a RAR target, create an image, load
firmware, invoke QEMU or another emulator, or execute a RAR artifact.

## One-writer ownership

The Milestone A writer owns only the paths assigned by the vertical packet:

- `Cargo.toml` and `rust-toolchain.toml`;
- `boot/`;
- `recovery/`;
- `nucleus/arch/x86_64/`;
- the minimum Milestone A additions under `tools/sprint-alpha/`;
- `tests/sprint-alpha/boot/`;
- `docs/sprint-alpha/boot/`;
- only the exact Milestone A checkpoint update in `SPRINT_STATUS.md`.

The writer does not edit `spec/alpha/boot/`, R0-002 contracts, trusted
controller/workflow files, later-milestone paths, or another task's worktree.
Any required change there stops the writer and returns to the owning review
track.

## Ordered work packets

### A0 — Reconfirm immutable inputs

- Record the exact accepted boot-contract, R0 handoff, RHD, machine-profile,
  compiler, linker, firmware, controller, and source identities.
- Confirm target-linked dependency count remains zero.
- Confirm the clean source SHA is the SHA dispatched to the trusted controller.
- Stop if any identity is unavailable, stale, mutable, or disagrees with its
  reviewed inventory.

Evidence: one controller-owned preflight record bound to the exact source SHA.

### A1 — Freestanding target skeletons

- Establish three distinct RAR-owned artifacts: Root, Recovery, and Nucleus.
- Keep target code `no_std`; admit assembly or unsafe Rust only at documented
  machine-entry, firmware-call, register, page-table, or port-I/O boundaries.
- Give every unsafe boundary explicit preconditions, postconditions, aliasing
  rules, ownership transfer, failure behavior, and focused negative tests.
- Keep firmware types and calls inside Root. Recovery and Nucleus must have no
  firmware pointer, runtime, allocator, or hidden host ABI dependency.

Evidence: cloud-only freestanding compile/link records and artifact identities;
these are not boot evidence.

### A2 — Shared bounded parsers and arithmetic

- Implement checked integer/range operations used by ELF, entry-blob, memory-
  map, and R0 production paths without ambient allocation or undocumented
  cross-stage state.
- Implement one strict ELF64 subset parser for the already-reviewed contract;
  Root applies it to Recovery and Recovery applies it to Nucleus.
- Reject before copying or mapping on malformed, overflowing, overlapping,
  excessive, dynamic, writable-and-executable, or invalid-entry inputs.
- Keep error selection deterministic and bind every rejection to the reviewed
  `boot:error:<stage>:<code>` table once that table is ready.

Evidence: pure contract fixtures in the cloud plus proof that rejected inputs
produce no mapping, entry, or authority effect.

### A3 — Root

- Enter only when firmware selects the fixed Root path. Root itself reads only
  the fixed Recovery and Nucleus payload paths.
- Read Recovery and Nucleus with exact size ceilings and exact-read semantics.
- Validate and map Recovery; stage Nucleus as inert bytes and hash its exact
  file bytes with the RAR-owned SHA-256 implementation.
- Construct the reviewed Root-to-Recovery blob, obtain the final firmware map,
  and follow the bounded ExitBootServices retry rule without allocations after
  the final map.
- Transfer only the reviewed registers, mappings, and ownership; never return.

Evidence: byte-exact blob fixtures, bounded firmware-call trace, and Root-stage
negative cases.

### A4 — Recovery

- Validate the entire entry blob before interpreting a section.
- Recompute the Nucleus file digest, validate its ELF, zero BSS, and install
  only W^X mappings inside the reviewed slots.
- Convert every supported UEFI type/attribute combination according to the
  total reviewed table; reject unknown, conflicting, overflowing, overlapping,
  or unusable ownership.
- Canonicalize memory, carve every owned/device range, and produce RHD plus the
  unchanged R0-002 sources deterministically.
- Complete all source writes, revoke producer and DMA writes, establish required
  immutability, and identify Recovery as the sole producer before entry.
- Verify NX/WP/control-register and timer-profile requirements exactly as fixed
  by the ready contract; never infer or silently repair missing platform state.

Evidence: deterministic R0 bytes, source-ownership trace, W^X/control-state
negative cases, and Recovery-stage rejection-with-no-entry evidence.

### A5 — Nucleus x86-64 entry

- Accept only the unchanged R0-002 x86-64 entry register/state contract.
- Copy and validate untrusted entry bytes before constructing authority.
- Emit the structured Milestone A trace only after R0-002 validation succeeds.
- On malformed R0 input, create no capability, mapping, device access, thread,
  or observable success marker.
- Halt through the reviewed bounded failure path until Milestone B introduces
  runtime scheduling.

Evidence: valid entry trace and malformed-entry no-authority trace.

### A6 — Deterministic image tooling

- Build the reviewed private Alpha image using RAR-owned serialization.
- Keep image creation in the untrusted cloud build role; the trusted launch role
  receives only a frozen, reverified artifact and no source or build output.
- Require the independent read-only inspector to agree on every byte-producing
  field, range, file identity, padding rule, and computed checksum.
- Produce two clean, byte-identical unsigned images from the same exact inputs.

Evidence: both artifact hashes, independent inspection report, and exact input
identities. A local image or skipped comparison is failure, not evidence.

### A7 — Trusted cloud boot scenario

- Dispatch only through the merged default-branch controller for `milestone-a`.
- Launch only the digest-pinned software-emulated profile with networking,
  passthrough, sharing, credentials, and unrelated access disabled.
- Require the observed Root → Recovery → Nucleus trace and exact final source
  SHA/artifact/profile identities.
- Exercise failures through immutable inputs selected by the controller; never
  let the source branch alter the trusted launcher or verdict.

Evidence: retained controller verdict, complete logs, structured trace, exact
exit status, frozen-artifact identity, and immutable checkpoint.

### A8 — Documentation and checkpoint

- Document build architecture, stage ownership, memory map, unsafe invariants,
  error interpretation, debugging, reproducibility, limitations, and the exact
  cloud-only reproduction route.
- Record tests and evidence honestly; do not call an unrun or unavailable check
  passed.
- Obtain correctness and security review plus architecture review of every
  contract/trust-boundary interpretation.
- Resolve accepted findings in one bounded repair, rerun the full Milestone A
  cloud gate, push, and create the append-only `sprint-alpha-0.1/A` checkpoint
  only after all evidence is green.

## Mandatory case coverage

The implementation must cover all 41 rows in `spec/alpha/boot/cases.v0`; the
table below is a completeness map, not a replacement fixture.

| Stage | Cases | Owning packet |
| --- | ---: | --- |
| Integration success | 1 | A7 |
| Root file handling | 6 | A3 |
| Root ELF validation | 7 | A2/A3 |
| Root firmware exit | 1 | A3 |
| Recovery entry validation | 8 | A2/A4 |
| Nucleus identity | 1 | A4 |
| Recovery ELF validation | 7 | A2/A4 |
| Firmware-map conversion | 4 | A4 |
| R0 production/authority | 5 | A4 |
| Nucleus R0 rejection | 1 | A5 |
| **Total** | **41** | |

Every reject case must prove the forbidden next-stage effect did not occur.
The integration row requires a real cloud boot and cannot be satisfied by a
host parser, mock, generated log, screenshot, or unbooted image.

## Stop conditions

Stop the active writer without broadening scope if work would:

- invent or change a byte layout, error precedence, trust boundary, target ABI,
  persistent format, dependency, or tier meaning;
- weaken R0-002 validation or reinterpret descriptive records as authority;
- require a local target build/run, unapproved cloud access, raw device,
  passthrough, networking, elevated execution, or host sharing;
- modify the trusted controller from the source branch;
- proceed with a missing identity, red check, unresolved review finding, or
  non-reproducible artifact.

No schedule pressure converts a stop condition into permission.
