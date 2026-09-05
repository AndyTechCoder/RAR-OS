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
