# Compiler notice-layout correction (construction only)

Run34019740918 at8afd8a429c117888549f0aece8e55ee22b7dd2cf passed the
ELF search-directory correction and failed because share/doc/rust/LICENSE-APACHE
does not exist. This is a wrong export-path assumption, not permission to omit
the notices. No final compiler image, adapter or RAR target was executed.

The retained artifact9985084200 was inspected entirely in memory. ZIP SHA256:
d80167c4d164c05f6cd2ffa050f96ce9c0b998fb922c6be5aa4e76080c4d677b.
Pinned inventory SHA256:
d723682434bc5d2bb9b35a135eb873245ba14b2977dc89d665a2af4db2586efc.
The archive SHA256 remains
aee540abf132920f791ef781489851a078d69dff493fb628d49c1d573f92bb3a.

Its exact root rust-std-1.95.0-x86_64-unknown-linux-musl contains:
- LICENSE-APACHE:9723bytes, SHA25662c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a.
- LICENSE-MIT:1068bytes, SHA256b71bd43a069ca0641a9ecfe585ca7b3c53b5cc1608f8b68321168698e28b5ea1.
- COPYRIGHT:1571bytes, SHA256172020dbfd5b53a226dfde77616190a48dcff519b0bc0e6deb91a8450782c4af.

Pinned upstream packaging distinguishes archive-root legal overlays from installed
notices: [overlay](https://github.com/rust-lang/rust/blob/1.95.0/src/bootstrap/src/utils/tarball.rs),
[installed compiler notices](https://github.com/rust-lang/rust/blob/1.95.0/src/bootstrap/src/core/build_steps/dist.rs),
[generated copyright filenames](https://github.com/rust-lang/rust/blob/1.95.0/src/bootstrap/src/core/build_steps/run.rs).
The compiler installs COPYRIGHT.html and COPYRIGHT-library.html under
share/doc/rust, and SPDX texts under share/doc/rust/licenses.

The exporter now captures all three verified archive-root notices from the exact
already extracted pinned input, rejects redirected archive notice paths, and
checks each exact size/hash before export. This is not a broad /build read grant.
It also requires both generated installed copyright files, captures the bounded
installed license tree, and requires Apache-2.0.txt, MIT.txt and LLVM-exception.txt.
The existing file/count/aggregate bounds, readonly final metadata and independent
complete image inventory remain unchanged. Per-notice source/size diagnostics
make any remaining packaging mismatch explicit.

Pure tests cover exact notice path/identity admission and unknown-path/size/hash
rejection. Exact cloud CI and independent review are required before merge.
Actual complete notice capture, candidate construction/reproducibility, static
compilation and runtime comparison remain pending. Capture is not legal
certification or a source-offer compliance opinion. No dependency policy,
persistent-data contract, runtime profile or target linkage is changed.
