# R0-001 reproducible host bootstrap scaffold

Status: Host scaffold implemented; target builds and certification blocked by prerequisites

## Command surface

Use the standalone wrapper from the repository checkout root:

```sh
tools/rarbuild/rarbuild check
tools/rarbuild/rarbuild build
tools/rarbuild/rarbuild image
tools/rarbuild/rarbuild run
tools/rarbuild/rarbuild test
tools/rarbuild/rarbuild evidence
```

`Cargo.toml` intentionally remains an empty workspace with exactly `members = []`.
`rarbuild` is compiled directly with Rust 1.95.0 into `out/r0/host-tools/rarbuild`.
It uses repository-owned Rust plus `std`; it neither resolves Cargo dependencies nor
downloads or installs tools.

Before compiling, the wrapper classifies the complete closed command surface using shell
builtins. Run/alias/test execution routes exit 73, while unknown, absolute, or wrong-arity
host commands exit 64; none reaches root discovery or `rustc`. Accepted host commands then
require a canonical absolute checkout root with regular repository/approval markers and a
real `.git` directory or worktree file.

The output layout is repository-confined and separated from source:

```text
out/r0/
  host-tools/                 standalone rarbuild binary
  host-tests/                 standalone host test binaries
  build-plan/build-plan.txt   deterministic target-build plan
  image-plan/image-plan.txt   deterministic blocked image plan
  evidence/host/              deterministic host bootstrap evidence
  artifacts/                  reserved; no target artifact exists
  images/                     reserved; no image is created
  toolchain/firmware/         reserved; no firmware exists
  vm/                         reserved for disposable VM images only
```

Every writer rejects paths outside `out/r0`, traversal, and existing symlink ancestors.
Build-plan bytes derive from the source revision, an allowlisted source-input tree digest,
the tool-lock digest, fixed targets, fixed configuration, and the empty target dependency
inventory. The reproducibility test deletes the repository-confined plan between runs and
proves byte-identical clean regeneration.

## Pinned and discovered tools

Observed on `aarch64-apple-darwin` on 2026-07-16:

| Input | Version/state | SHA-256 or status |
| --- | --- | --- |
| `rustc` | 1.95.0, commit `59807616e1fa2540724bfbac14d7976d7e4a3860`, LLVM 22.1.2 | `b829b733131d4e1673eeebd1f34d06ae1e9ff4977b051313cf42e2a9e79ecf1c` |
| Rust-bundled LLVM | 22.1.2 | Required verbose-version match; `llvm=ok-rust-bundled-22.1.2` |
| `cargo` | 1.95.0, commit `f2d3ce0bd7f24a49f8f72d9000448f8838c4e850` | `c512bff73c86143b557463f021d0c3d5b0490d97d65040ba59ea2b3427784758` |
| `rust-src` install manifest | Rust 1.95.0 installed | `47b629523343fa73b4436080f660b510e0cd1c2553a94ba90ef8bdcc2e025ec1` |
| `aarch64-unknown-none` manifest | Installed | `d2c67d85ffb386328781b6300ddfde93c9a500072a9e6e08eb3ff1fb0017375c` |
| `thumbv8m.main-none-eabi` manifest | Installed | `c12a52d6b268e44baf79e6ec56fe0f82b53587d2dee6b1694fda3ffb94720f2b` |
| `x86_64-unknown-none` manifest | Installed | `a1c0aed6cf079827ac9ebc82faeea2b517aba581c240dfe84a31761a99068c75` |
| Apple Clang | 16.0.0, discovery only | Not output-affecting in this scaffold |
| External LLD | Unavailable | `status=unavailable`, version/hash `none` |
| QEMU x86-64/ARM64/ARM | Unavailable | Each has `status=unavailable`, version/hash `none` |
| x86-64/ARM64 firmware | Unavailable | Each has `status=unavailable`, identity/hash `none` |

The local lock digest observed after implementation is
`5f1878b789df0505304b3a417637f76e26428487261dec24228bd9143e24fc35`.
The dependency inventory states `target_linked_third_party_code=none` and there are no
Dependency Exception Records.

## Exact validation and exit meanings

```sh
tests/bootstrap/run.sh
tools/rarbuild/rarbuild check
tools/rarbuild/rarbuild build
tools/rarbuild/rarbuild image
tools/rarbuild/rarbuild evidence
```

Observed results on 2026-07-16:

- Bootstrap suite: exit 0; 17 passed, 0 failed.
- `check`: exit 3. Rust, bundled LLVM 22.1.2, Cargo, `rust-src`, and all target manifests match; LLD,
  QEMU, and firmware report `unavailable-required`; certification is impossible.
- `build`: exit 0. It writes a build plan with `target_artifacts=not-produced`,
  `target_linked_dependencies=none`, and `execution=forbidden`.
- `image`: exit 4. It writes only a blocked plan; `target_artifact=unavailable` and
  `firmware=unavailable`.
- `evidence`: exit 4. It records source/tool/dependency/plan digests, bundled LLVM,
  `configuration=release-0-host-scaffold`, the exact three-target list, impossible
  certification, absent authorization, and no target execution.

Exit 3 means required discovered tools are unavailable or unpinned. Exit 4 means a safe
plan/evidence file was produced but a target artifact or certification prerequisite is
blocked. Exit 73 is an intentional execution-route refusal.

## macOS and Linux host procedure

On ARM64 macOS, use the preinstalled Rust 1.95.0 toolchain and run `check`. The current
lock supports only the exact measured `aarch64-apple-darwin` binaries above. No setup
command installs missing tools.

On Linux, begin with a pre-provisioned Rust 1.95.0 toolchain containing `rust-src` and all
three target libraries, then run `check`. The macOS binary hashes must not be reused.
Linux remains unsupported until a coordinator-approved, locally measured Linux lock entry
records exact compiler/component hashes and the required pinned LLD, QEMU, and firmware.
`rarbuild` reports a platform/hash mismatch rather than relaxing the lock or downloading
anything.

## Acceptance mapping

| R0-001 acceptance | Evidence | State |
| --- | --- | --- |
| One command reports missing tools without host installation/mutation | `rarbuild check`; exit 3 with deterministic unavailable states | Pass |
| Every unauthorized execution-capable route refuses before resolution/spawn | Bootstrap route matrix, wrapper-order test, R0-000 resolver/spawner counters | Pass |
| Two clean builds yield identical unsigned target artifacts | No target implementation or target artifact exists in Prompt 2 | Blocked; two identical build plans proven instead |
| Evidence records tools, hashes, target, configuration, and source | `rarbuild evidence` and `build-plan.txt` | Pass for host scaffold |

## Security, unsafe, and dependency review

- All Rust roots forbid unsafe code; no unsafe block or assembly exists.
- No Cargo package, crate dependency, target runtime, binary blob, firmware, or target asset
  was added.
- Host tools are Class B inputs and remain outside any future target image.
- The lock schema can represent future `pinned` external inputs with safe version/identity
  and exact SHA-256 fields. `unavailable` requires `none/none`; `certifiable=true` is valid
  only when every required external pin is complete and target dependencies remain none.
- `check` invokes Rust/Cargo version operations and hashes discovered external candidates.
  It never launches QEMU, firmware, a target linker, target binary, or image. A present but
  unpinned executable remains unusable.
- `build` and `image` are plan generators only. They do not compile, link, package, load, or
  execute target code.
- The source tree and outputs reject symbolic-link escape and non-repository destinations.

## Limitations and next gate

- External LLD, all QEMU backends, and required firmware are unavailable; their versions and
  hashes remain absent rather than fabricated.
- No target implementation exists, so unsigned target-artifact reproducibility cannot yet be
  demonstrated. The scaffold records this as a blocked acceptance item.
- Only the current ARM64 macOS Rust installation is locked. Linux needs a separate reviewed
  platform lock based on observed binaries.
- Prompt 3 must independently review correctness, security, provenance, reproducibility, and
  the no-execution evidence. Prompt 2 does not authorize Prompt 3 remediation, merge, R0-002,
  profile certification, or any guest boot.

## Target non-execution attestation

No QEMU executable, firmware, target linker, target binary, VM image, boot image, or RAR
target artifact was executed. Permitted activity was limited to host Rust compilation and
tests, Rust/Cargo version and component inspection, local file hashing, Git revision reading,
static command/profile generation, deterministic repository output, and refusal paths.
