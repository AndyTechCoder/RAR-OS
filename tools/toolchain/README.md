# Release 0 host toolchain lock

`host-tools.manifest` declares the Class B host surface. `host-tools.lock` is the canonical `rar-host-tool-lock-v2` record measured on the current `aarch64-apple-darwin` host. `dependencies.r0` separates host inputs from target dependencies.

Version 2 pins absolute paths and SHA-256 identities for the bootstrap shell, mkdir, Rust 1.95.0 compiler, Rust-bundled `rust-lld`, macOS SDK settings, and Cargo. It also pins Rust component manifests and represents external LLD, QEMU backends, and firmware as either complete `pinned` records or consistent `unavailable/none/none` records. `certifiable=true` is valid only when every required external record is pinned and target-linked dependencies remain `none`.

The bootstrap wrapper reads only the required root fields with shell builtins, rejects missing, duplicate, malformed, relative, aliased, or symlink paths, and invokes absolute roots. Because an executable cannot prove itself before first execution, the reviewed path/hash record is the explicit bootstrap axiom. The compiled verifier then streams and compares every pinned byte sequence before performing later subprocess work. No command downloads, installs, or invokes `rustup` or `git`.

The lock deliberately keeps external LLD, every QEMU backend, and both firmware inputs unavailable. A discovered but unpinned candidate remains unusable for certification and is never executed.

The official Rust 1.95.0 OCI image is pinned for portable CI host tests at `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. That image is a CI test root only. Linux `rarbuild check` remains unsupported until a separately measured, reviewed Linux lock records exact host paths, executable/component hashes, linker and SDK/sysroot inputs, plus the required LLD/QEMU/firmware pins.

No Cargo package, third-party crate, target code, target-linked dependency, binary blob, target asset, firmware payload, or Dependency Exception Record is present.
