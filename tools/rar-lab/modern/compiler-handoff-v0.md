# Modern compiler handoff candidate

Unactivated host-only infrastructure for M4 crypto comparisons. This is not
RAR OS linkage, guest execution, or a change to the accepted compiler baseline.

The reproduced compiler parent is
sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914
from cloud run34083184108 at main d9cf06b87449391077f18566bffa1905fb3895bb.
The derived image must have a new independently verified identity. Its exact
ordered parent layers and files remain unchanged; no retagging, whiteouts,
replacement paths, links or inherited extra authority are allowed.

## Driver layer

compiler_driver_layer.py constructs and independently inspects the one-file
canonical USTAR layer for /rar-compile-driver: root-owned0555, fixed timestamp,
at most2MiB static x86-64 ET_EXEC. It reuses the independent ELF byte parser,
requires exactly one NX stack and an entry in exactly one executable file-backed
load, rejects PT_INTERP/PT_DYNAMIC, W+X, malformed loads, and unbounded mapping.
Canonical byte equality rejects hidden concatenated entries and extra padding.
It has no filesystem write operations and refuses execution unless Python is
isolated with bytecode writes disabled. It activates no image. Construction
callers must provide read-only exact-main tool sources, not a mutable directory.

The driver has a distinct RAR path, not an upstream Rust toolchain identity.
Its fixed rustc command clears inherited environment and sets TMPDIR=/build;
there is no fallback writable /tmp or HOME. Existing source and output bounds,
nonroot/capability/NNP/seccomp guards, exact five-input inventory, and no output
execution remain. Parent-enforced isolation is still mandatory.

Source revision/source hash/recipe hash in the layer report are provenance
labels only. The future trusted-main construction must bind them to exact Git
blobs, build the driver twice with the pinned private bootstrap and network off,
verify identical outputs, and bind output hashes independently. No caller report
can substitute for that construction proof.

## Remaining integration gates

- Bind the exact five source files to a coordinator-selected immutable proposal
  commit/tree/blob identity and source-layer bytes, not arbitrary checkout
  content, moving references or caller labels. Control-plane code stays on
  exact reviewed main; proposal source need not be merged before testing.
- Inspect the full new config/rootfs: exact parent prefix, this driver layer,
  exact source layer, no other additions, exact process environment/entrypoint.
- Independently bind required Rust-std/musl and RAR notices to driver provenance.
  The final static adapter image must carry its applicable notices too; notices
  in the compiler parent do not automatically satisfy separate distribution.
- Run the driver only in a reviewed cloud compiler role: fixed nonroot identity,
  read-only root, no network/IPC/devices/host mounts, bounded noexec/nosuid/nodev
  /build tmpfs, resource limits and verified effective daemon configuration.
- Accept output only after clean process completion, bounded EOF and independent
  static-ELF inspection. Never execute it inside the compiler role.
- Reproduce adapter bytes and its new scratch image containing only the adapter
  and required inert notices. Stop the compiler container before adapter runs.
- Freeze corpus and RAR results before invoking either independent reference.
  Oracle output never enters compiler source, adapter construction or RAR input.
- Retain actual confinement/timeout/cleanup and three-way comparison evidence.

Current tests are source-level layer, framing, refusal and command construction
tests in the existing cloud Specifications sandbox, not runtime acceptance.
No compiler runtime, adapter runtime, Modern VM or disk profile is activated.

The layer report records SHA256 and Git blob identities of both this helper
and its adjacent compiler_elf.py parser. Those measured identities must match
the trusted-main tree before construction/inspection; labels alone cannot
establish provenance. The caller's read-only source mount prevents changes
between identity measurement and import. Both sources are bounded at128KiB.
Focused tests cover aggregate multi-LOAD limits, address/offset congruence,
ambiguous executable entry mappings, no-bytecode refusal and the exact2MiB limit.

## Proposal source object binding

source_snapshot.build_from_objects verifies the selected commit's raw Git object
identity, its root-tree relationship, bounded raw tree/blob identities, and every
component of each fixed path. Intermediate entries must be directories and all
five leaves must be regular100644 blobs. Missing/extra objects, wrong hashes,
duplicate names, malformed trees, executable/link/submodule source entries and
LFS pointers fail. No checkout, filters, hooks, attributes or LFS resolution runs.
The source-layer report binds the commit SHA256, root tree, five blob identities,
per-file SHA256/size and canonical layer digest.

The controller selects the immutable canonical-repository proposal commit and
obtains raw objects with replacement objects disabled. This pure function does
not authenticate the repository or choose/approve a revision. Compiler helper,
driver, parser, inventories and launch commands remain exact reviewed-main code.
The proposal contributes only bounded source bytes, never paths or commands.

Before release, compare final main's five blob IDs/SHA256 values with the tested
proposal and rerun comparisons if any differ. Rust compile-time built-ins can
read the compiler role's visible files, so its complete positive inventory must
contain no secrets, credentials or reference oracles. Compiled output remains
hostile and separately confined. These boundaries avoid merging untested OS
implementation merely to make testing possible.

The report also inventories every used intermediate tree object by Git OID,
SHA256 and size, in canonical OID order, and records commit byte length. The
trusted parent must retain the raw commit and used tree/blob objects with the
evidence so the complete root-to-leaf chain can be reconstructed independently.
Unused objects and count/per-object/aggregate bounds have focused source tests.

## Private driver construction recipe (not invoked or activated)

compiler-driver.Containerfile builds only the reviewed RAR host-driver source.
The trusted controller must supply a newly owned local parent tag already bound
to the exact reproduced compiler image, inspect that association, and invoke
BuildKit with network disabled, no cache, no pull, a fixed local output path and
a minimal context of this recipe plus the exact reviewed driver blob. No
proposal source, reference code, credentials or owner storage enters that build.

The recipe takes only the verified musl sysroot from the compiler parent and
uses the already pinned Rust1.95 private bootstrap. A fixed direct rustc command
has a cleared environment, fixed linker/sysroot/static target and reproducible
paths/metadata. Per-process CPU and file-size ceilings apply; the outer trusted
job must additionally enforce wall-time, output and disk budgets. No driver or
compiled adapter is executed during construction.

The export contains only the bounded driver and all captured parent notices;
its deliberately nonexistent entrypoint is not a runtime profile. Independently
verify export paths/types/metadata/content, complete notice identity against the
parent, static ELF framing, actual compiler/tool/source/recipe identities and
two-build equality before making the canonical driver layer. The derived
compiler image preserves all parent notices; the later adapter-only image
requires its own separately inventoried notice set. This recipe alone does not
establish legal sufficiency, role confinement, reproducibility or compilation
success. Actual construction remains pending.

## Source layer consumption

Before derived-image assembly, source_snapshot.inspect independently parses the
bounded source USTAR and compares every file to the separately Git-object-bound
size/SHA256 inventory. It requires the fixed five paths and their exact readonly
root-owned directory/file metadata, then requires the canonical encoding. Extra
paths, replacements, links, alternate metadata, changed content, concatenated
archives and trailing padding fail. The expected inventory must come from the
trusted parent's raw-object verification, never a manifest supplied by the layer.
This closes source-layer consumption validation only; no build, derived image,
compiler invocation, adapter comparison or Modern VM is activated by this API.
