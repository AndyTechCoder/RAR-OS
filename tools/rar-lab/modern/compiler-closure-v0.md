# Private compiler closure exporter v0 — not an activated image

This construction-only helper starts from fixed Rust1.95.0 rustc/rust-lld
locations in the already pinned provision base. It exports only their discovered
ELF runtime closure and the installed, separately hash-pinned musl target sysroot
into a fresh /compiler-root inside the private cloud construction container.
The musl sysroot is for a host-only test adapter, never RAR OS linkage.

The helper executes only fixed provision tools (rustc version, readelf, ldd),
never proposal source, an adapter, a reference executable or RAR target code.
Metadata/loader traces must resolve consistently and only into admitted library/
toolchain domains. Canonical source resolution, positive file inventory, SHA256,
size and modes are recorded; libssl/libcrypto/libsodium/reference paths or
dependencies fail closed. No symlinks are exported. The complete closure has
finite file/dependency/depth/byte budgets and fixed timestamps/modes.

This is not a complete reference-free compiler profile. Before image publication:
bind the exact base/musl archive and provisioner identity; retain complete
licenses/notices and compiler/runtime provenance; independently inspect the
actual final filesystem and ELF closure; verify loader environment and compiler
operation; reproduce the image; add the bounded source/scratch compiler runner;
build and inspect the static one-executable adapter image. Reference/target
runtime images, source mounts and activation are absent from this change.

The private report says private-closure-export-only and records captured notices without legal certification and leaves
accepted image identity false. Its own JSON is construction evidence,
not authority supplied by untrusted RAR source. The final image must not be
published/accepted from this report alone.

Pure self-tests exercise metadata/loader-path parsing, reference-path rejection,
unresolved/ambiguous dependencies and default-denied entry. They do not execute
provision tools or export files. Actual compiler/runtime dependency compatibility
remains unverified until the later reviewed cloud construction job runs.

Any separately loaded LLVM codegen backend is an additional closure root, not
assumed to appear in rustc DT_NEEDED. The fixed host codegen-backends directory
may be absent/empty for the builtin backend, or contain exactly one recognized
regular non-symlink backend. Any separate backend's ELF closure is inspected too. Malformed interpreter/NEEDED lines and unsupported
FILTER/AUXILIARY tags fail instead of being mistaken for an empty dependency
set. A real bounded musl compile remains mandatory before image usability.

## Rust1.95 builtin LLVM correction (pre-activation)

The pinned upstream source explicitly selects the builtin LLVM backend when the
`llvm` feature is enabled; external backend lookup is the alternate path:
https://github.com/rust-lang/rust/blob/1.95.0/compiler/rustc_interface/src/util.rs#L328 .
Thus requiring a separate backend file for all compiler distributions was an
incorrect packaging assumption. The exporter now accepts either no external
backend or one exact upstream-recognized LLVM filename; ambiguous, symlinked,
foreign or wrongly named files still fail. Optional separate backend files are
exported and recursively inspected just like the driver and linker.

Absence alone is never a usability claim: fixed bounded `rustc -vV` and musl
`--print target-cpus` probes must report the exact release/host, an LLVM version,
and x86-64 target support before export. These private construction probes take
no source input and generate no executable. Their hashes are recorded. The
driver/LLVM dynamic dependency graph remains fully inspected; no arbitrary
library-directory copy is added. A real isolated static compile and final-image
closure/reproducibility checks are still mandatory before image acceptance.
This correction does not activate the exporter or any runtime profile.

The exact CPU-probe header is bound to pinned upstream output:
https://github.com/rust-lang/rust/blob/1.95.0/compiler/rustc_codegen_llvm/src/llvm_util.rs#L540 .

## Construction recipe and notice capture (not activated)

The private compiler.Containerfile recipe uses the same pinned Rust base and
exact Rust1.95 musl std archive, SHA256
aee540abf132920f791ef781489851a078d69dff493fb628d49c1d573f92bb3a.
Before invoking it with network disabled, the controller must bound and inventory
the archive and provide only the archive, recipe and exporter as context. The
archive's own pinned installer is a ClassB bootstrap tool, not RAR target code.

The scratch candidate contains only the positive compiler/runtime/musl closure,
bounded inert notices, an evidence report and empty source/build directories.
The fixed process is nonroot rustc, with an absent executable-search PATH and
only the fixed toolchain library path. No shell, Python, package manager,
reference implementation or proposal source enters the final candidate. The
recipe does not compile or execute a RAR adapter and activates no runner.

Each external runtime file must have one identified installed package/version;
its copyright notice is captured. The Rust copyright/licenses and bounded
common-license texts are copied and hashed. Resolved notice sources are confined
to the fixed notice directories; their contents cannot add executable authority.
Capture is not legal certification: upstream distribution/source-offer
obligations still need a distribution check. The final independent inventory
must verify every runtime/notice byte, config and empty directory. Actual
isolated static compilation and reproducibility remain mandatory. No workflow
invokes the recipe in this source change.

### Embedded dynamic-loader authority

The exporter rejects ELF AUDIT/DEPAUDIT as well as FILTER/AUXILIARY tags.
RPATH/RUNPATH are parsed, not silently ignored: only the exact nonempty forms
$ORIGIN, $ORIGIN/../lib and $ORIGIN/../../.. are admitted. Every component is
expanded separately for canonical and exported alias locations, normalized,
checked against the immutable toolchain/system-library path domains, and required
to name a directory actually present in the positive export. Empty components,
CWD-relative entries, arbitrary tokens, writable source/build paths, malformed
tags and duplicates fail. This prevents a writable compiler scratch directory
from becoming library-search authority; it does not replace final-image inspection
and runtime confinement checks.

At most one search-path tag is admitted: simultaneous RPATH and RUNPATH are
rejected, avoiding cross-tag duplicates and ambiguous precedence. A mixed-tag
negative fixture enforces this restriction.

## Pinned archive inspection

compiler_archive.py supplies the pre-install byte gate for the exact musl input.
verify checks the fixed SHA256 before decompression; inventory is a pure helper
for synthetic tests, not an alternate production admission path. XZ decoder
memory is bounded at256MiB, compressed input64MiB, total expanded bytes512MiB,
8192 entries and256MiB per file. Truncated, trailing and concatenated XZ streams
fail. Every path must stay below the exact pinned root; duplicate entries, links,
devices, sparse members, special modes and group/world writable files fail.
Per-file hashes, metadata and payload totals are recorded, with required
installer and musl library anchors. Inspection never extracts or executes.
The cloud controller still needs to acquire the fixed URL under bounded HTTPS,
call verify, and keep the minimal construction context separate from RAR source.

## Independent ELF byte inspection

compiler_elf.py reads ELF64 little-endian x86-64 dependency metadata directly
from bounded candidate bytes, not readelf text or the exporter's graph. It checks
program-header/segment extents, unique dynamic/string-table load mappings,
bounded and terminated tables/strings, dependency names, interpreter framing,
SONAME and the same narrow search-path forms. Embedded audit/config/filter/
auxiliary loading, mixed search tags, text relocations, W+X and executable-stack
requests fail. This independently reports metadata for the final image inspector
to cross-check against its complete positive filesystem and construction graph.
It is not a loader, full ELF validity proof, or activation authority.

ELF dynamic-tag constants follow the public ABI definitions documented in
https://sourceware.org/git/?p=glibc.git;a=blob;f=elf/elf.h .
Synthetic tests cover malformed tables, mapped ranges, loading authority,
search-path rejection, bounds and header/segment mutations. Actual compiler-image
bytes and runtime dependency equivalence remain unverified until construction.

The byte inspector requires exactly one explicit read-write/non-executable
GNU_STACK declaration; missing or duplicate declarations fail. Dynamic and
string-table mappings must be readable, and the dynamic segment's file extent
must fit its memory extent. Forbidden reference-library names are rejected in
SONAME as well as NEEDED. Focused mutations cover each of these requirements.

## Complete candidate image inspection

compiler_inventory.py independently reads a bounded Docker-save archive without
extracting files or starting containers. It binds the image configuration digest
and each rootfs layer digest, rejects duplicate/replaced files and special
entries, and requires exactly the positive compiler/musl/notice/report inventory
plus its parent directories and the empty source/build directories. Bytes,
modes, owner, timestamp and aggregate budgets must match. Only build is writable,
and only by the fixed nonroot identity; no other payload is admitted.

The inspector parses each actual ELF through compiler_elf.py and cross-checks
direct dependency names, interpreter, exported aliases and search paths against
the construction graph and actual final files. Missing or ambiguous dependency
bytes fail. The graph alone cannot override the inspected bytes. Exact process
environment, entrypoint, working directory and absent additional runtime
configuration are required. This is still inspected-not-activated evidence,
not a complete loader proof, legal certification or approval to run an image.

Synthetic whole-image tests cover the positive composition and mutated process
authority, content, modes, ownership, graph, notices, special/extra entries,
duplicate JSON keys and image identity. They run only in the existing isolated
cloud Specifications job. No acquisition, extraction, compilation or image
activation occurs in these tests. Real compiler construction, reproducibility,
isolated compile and final static adapter checks remain outstanding.

## Manual trusted-main compiler candidate construction

The modern-compiler workflow and provision_compiler.py construct two no-cache
candidates only from an exact reviewed main checkout. The fixed Rust base is
pulled by digest, and the one fixed Rust musl HTTPS URL is bounded, hash-verified
and inventoried before its pinned installer runs inside network-disabled
construction. The context contains only the recipe, exporter and pinned archive;
RAR source, credentials and independent reference implementations are absent.

Like the reference provisioner, child Docker/Git configuration is private and
allowlisted, identities are retained, the default Docker BuildKit driver is
verified, no broad cleanup/retry is performed, and the hosted job owns all
temporary files and daemon state. No Mac/SSD operation is involved. The cloud
workspace requires16GiB free before work; candidate image size is capped at
768MiB and serialized archive at1GiB, alongside the source and exporter budgets.
Measured controller peak RSS and bounded logs/artifacts are retained.

Both candidates must pass independent complete image inspection and have equal
image/config, layer and file inventories. Construction runs fixed compiler
metadata probes, but does not start the final compiler image or compile/execute
an adapter or RAR target. Success remains candidate-reproduced-not-activated.
Real isolated compilation, static adapter inspection and three-way algorithm
comparisons follow separately. This manual workflow must be reviewed and merged
to main before dispatch; adding its source does not activate a guest profile.

### Independent filename resolution and provenance

A SONAME match is not accepted in place of the exact NEEDED filename. The
positive candidate must provide the actual filename in a direct immutable
search domain (fixed LD_LIBRARY_PATH, the object's admitted ORIGIN expansions,
or the listed x86-64 GNU scratch default-library domains). Every same-named
copy in the entire image must have identical bytes; differing shadows fail
regardless of path precedence or inherited RPATH. The loader-visible bytes
must also match a filename in the recorded trace. Cache-only and inherited-only
resolution are deliberately not admitted; a real compile still validates the
pinned loader's actual usability. No cache, preload file or writable library
search path is present.

This conservative gate follows GNU's requirement to install shared objects
under their soname filenames, not treat the embedded name as filesystem lookup:
https://sourceware.org/glibc/manual/2.40/html_node/Dynamic-Linker-Hardening.html .
It is not an emulator for every glibc loader extension.

Backend selection and positive probe digest/version shape are mandatory, as is
complete, unique package/version/file coverage of every non-toolchain canonical
runtime source with its captured notice. The inspector validates this evidence
structure and file coverage; it does not independently rerun a compiler probe or
legally certify a package. Dynamic synthetic ELF tests include logical/canonical
aliases, missing/renamed or unreachable filenames, shadow bytes, and provenance
coverage failures.

Backend selection also binds the complete fixed backend-directory inventory:
builtin permits no backend files; external permits exactly the selected node.
Mixed or additional backend payloads fail, with positive/negative fixtures.

## Inactive bounded compiler driver source

compiler_driver.rs is RAR-owned host tooling, not an OS executable or adapter.
It is not yet built into the candidate image or selected by any runner. Its
purpose is to compile the five exact RAR adapter/source files in a confined
compiler role, read one bounded regular ELF output from private tmpfs, and stream
those bytes to the controller without executing them or requiring a writable host
output mount. It admits no command-line parameters or input-selected tool/path.

The proposed parent must enforce and inspect nonroot65532, read-only root/source,
bounded noexec build tmpfs, no network, no capabilities, no-new-privileges, default
seccomp, CPU/memory/PID/time and stdout/stderr budgets before start. The driver
checks its fixed executable path and process status, an empty private build
directory, and a complete bounded readonly source tree not owned by its user.
It invokes only exact rustc/rust-lld/sysroot/flags with a cleared child environment.
The source tree can be owned by the hosted controller; its immutable readonly
mount and before/after byte identity remain mandatory parent responsibilities.

Output is opened with Linux O_NOFOLLOW, required regular/single-link/owned and
bounded, and checked for stable metadata and ELF magic before streaming.
The parent must independently inspect the full returned static ELF, bind it to
the exact source/compiler identity, reproduce it, and construct the separate
one-executable adapter image. ELF magic is not acceptance. The parent also owns
whole-container timeout/teardown, including compiler descendants, and must reject
partial stdout or nonzero status. This source creates no runtime authorization.

Pure driver tests cover fixed compiler arguments and refusal of privileged or
unconfined process-status fixtures; they do not call the production entry path.
Recipe integration and a focused driver/runner review remain outstanding.
