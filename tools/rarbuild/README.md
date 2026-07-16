# `rarbuild` Release 0 host CLI

Run `tools/rarbuild/rarbuild <command>` from the repository checkout root. The wrapper compiles
repository-owned host Rust with the pinned toolchain into `out/r0/host-tools/`; no Cargo
workspace member or third-party crate is used.

- `check` verifies the observed Rust binary/component hashes, Rust-bundled LLVM 22.1.2,
  and hashes any pinned external candidate without launching it; it reports missing required
  tools without installing or downloading anything. Exit 3 means certification prerequisites
  are unavailable.
- `build` writes a deterministic host-only build plan. No target source or target artifact
  exists yet, so it does not invoke a target linker.
- `image` writes a deterministic blocked image plan and exits 4. It creates no bootable image.
- `run` is refusal-only and exits 73 before tool compilation, executable lookup, record reads,
  or process spawning.
- `test` with no arguments runs only the standalone host suites. Every argument-bearing test
  mode is refused before tool compilation.
- `evidence` writes deterministic host bootstrap evidence and exits 4 while certification is
  impossible. Evidence names the exact configuration and target list directly.

Execution aliases, delegation names, arbitrary emulator names and arguments, unknown
commands, and wrong-arity host commands are classified before root discovery or compilation.
They cannot reach a resolver. The compiled CLI independently applies the same route
classification in case the wrapper is bypassed. No emulator process-spawning implementation
is shipped.
