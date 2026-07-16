# Release 0 host toolchain locks

`host-tools.manifest` declares the Class B host surface and `dependencies.r0` separates host inputs from target dependencies. The versioned `rar-host-tool-lock-v3` records are platform-specific:

- `host-tools.lock` records the proposed `aarch64-apple-darwin` inputs on the physical development Mac.
- `host-tools.x86_64-unknown-linux-gnu-ci.lock` records the executable test roots inside the Rust 1.95.0 OCI image pinned by digest `sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`.

The macOS preparser uses the sealed-system shell, SHA-256 utility, and bounded-input helpers as the documented irreducible axiom. It bounds policy and lock files before shell `read`, rejects unknown fields, and hashes every selected non-root executable. The two closure manifests then pin the compiler driver, codegen backend, host standard/test libraries, selected bare-metal target libraries, Rust linker tools, component manifests, and SDK link stubs. They contain hashes and relative paths only—no compiler, SDK, target, firmware, or other binary payload.

macOS cannot execute the generated Mach-O through an already-open descriptor. Therefore Release 0 does not execute this local compiler closure: local accepted compile/test/planning/evidence routes refuse after verification and before `mkdir`, Rust, or linker execution. The Mac remains source/build storage. A future local route requires a separately reviewed descriptor-bound launcher or an equivalently immutable execution environment.

The Linux CI image digest is the complete transitive execution closure for CI. The Linux lock additionally records exact in-image paths and hashes for the shell, hasher, bounded-input tools, directory/cleanup tools, environment sanitizer, compiler, GCC driver, sysroot marker, Cargo, and Git. Generated host binaries execute from `/proc/self/fd`; captured host test script bytes are passed directly to the pinned shell.

Both locks keep external LLD, every QEMU backend, and both firmware inputs unavailable. `certifiable=false` remains mandatory. No command downloads or installs a tool, and no Cargo package, third-party crate, target-linked dependency, target artifact, target asset, firmware payload, or Dependency Exception Record is present.
