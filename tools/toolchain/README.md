# Release 0 host toolchain lock

`host-tools.manifest` declares the permitted Class B bootstrap surface.
`host-tools.lock` records only values observed in the current repository checkout's
`aarch64-apple-darwin` host toolchain. The Rust executable and installed-component
manifest hashes were measured locally; they are not inferred release hashes.

The canonical lock also pins Rust-bundled LLVM 22.1.2 and verifies it through verbose
`rustc` output. External LLD, each QEMU backend, and both firmware inputs have separate
status, version/identity, and SHA-256 fields. Version 1 permits either consistent
`unavailable/none/none` records or complete reviewed `pinned/<identity>/<sha256>` records.
`certifiable=true` is valid only when every required external record is pinned and target
dependencies remain `none`.

The lock deliberately records external LLD, QEMU, and firmware as `unavailable` and
contains no digest for them. A discovered but unpinned replacement remains unusable for
certification. Discovery hashes pinned candidates without launching QEMU. `dependencies.r0`
is the complete current dependency inventory: standalone
host Rust uses only `std`, and no target code or target-linked third-party dependency exists.

The lock is host-specific. Linux or another macOS architecture must receive a separately
measured, reviewed lock entry before it is supported; copying these binary hashes to another
platform is invalid. No command in this directory installs or downloads tools.
