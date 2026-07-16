# `rarbuild` Release 0 host CLI

Run `tools/rarbuild/rarbuild <command>` from a canonical repository checkout. The wrapper has a closed command classifier and compiles repository-owned host Rust with the exact paths in `tools/toolchain/host-tools.lock`. It never resolves a compiler, linker, `rustup`, or `git` through ambient `PATH`.

- `check` streams and verifies the pinned bootstrap shell, mkdir, Rust compiler, Rust-bundled linker, macOS SDK settings, Cargo binary, Rust component manifests, and any present external candidate without launching QEMU or installing anything. Exit 3 means certification prerequisites remain unavailable.
- `build` writes deterministic `rar-build-plan-v2` planning evidence only. It does not compile or link target code.
- `image` writes a deterministic blocked `rar-image-plan-v2` and exits 4. It creates no bootable image.
- `run` is refusal-only and exits 73 before root discovery, record reads, executable lookup, or spawning.
- `test` with no arguments runs the two host suites through the verified local bootstrap roots. Every argument-bearing mode is refused before compilation.
- `evidence` writes `rar-build-evidence-v2` and exits 4 while target artifacts and certification inputs are absent.

Source revision is read through bounded direct Git metadata parsing; no Git process runs. Durable plan/evidence writes use descriptor-relative no-follow traversal, exclusive temporary creation, synchronization, atomic rename, cleanup, and fresh post-commit hashing.

The wrapper's first shell/compiler/linker execution is the documented bootstrap trust root. That root is an owner-reviewed absolute path/hash record; the compiled verifier independently checks its bytes before any later subprocess action. A pinned Linux OCI image provides the separate CI test root and is not a Linux host-support claim.

ADR 0011 requires deterministic planning now and retains two byte-identical clean target builds as a mandatory Release 0 closure gate after target artifacts exist.
