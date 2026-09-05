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

The private report says private-closure-export-only and explicitly leaves license
closure and accepted image identity false. Its own JSON is construction evidence,
not authority supplied by untrusted RAR source. The final image must not be
published/accepted from this report alone.

Pure self-tests exercise metadata/loader-path parsing, reference-path rejection,
unresolved/ambiguous dependencies and default-denied entry. They do not execute
provision tools or export files. Actual compiler/runtime dependency compatibility
remains unverified until the later reviewed cloud construction job runs.

The explicitly loaded LLVM codegen backend is an additional closure root, not
assumed to appear in rustc DT_NEEDED. Exactly one expected regular, non-symlink
backend under the fixed host codegen-backends directory is required. Its ELF
closure is inspected too. Malformed interpreter/NEEDED lines and unsupported
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
