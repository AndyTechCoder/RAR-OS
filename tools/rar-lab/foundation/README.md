# Foundation cloud execution

This is the ADR 0032 / Milestone 1 disposable x86_64 UEFI profile. It is an
experimental development profile, not an independently certified security product.
Never invoke these tools on a developer Mac or SSD.

The workflow loads the controller from the PR base commit on main. Source is
an exact same-repository commit checked out without persisted credentials. The
proposal never selects runner commands, Dockerfiles, compiler flags, launcher
flags or credentials. Controller changes take effect after reviewed merge.

Two independent unprivileged, networkless build containers receive source
read-only and use a 256 MiB private tmpfs. They compile RAR-owned modules directly
with pinned rustc, without Cargo build scripts or downloaded crates. Host tests
and the RAR FAT image builder run within that same sandbox. No source program
runs directly on the runner. Output uses a bounded flat Base64 protocol; the
trusted controller permits only fixed names, exact image sizes and strict data.

The VM uses TCG, one virtual CPU, 256 MiB guest RAM, fixed q35 hardware, and no
network, host shares, passthrough or credentials. The launch container is
unprivileged, read-only, networkless, capability-free and bounded to 1 GiB / two
host CPUs / 64 processes. Only generated image files enter it read-only; these
are virtual storage inputs, never physical disks. RAR Lab fixes all emulator
arguments, verifies firmware and emulator checksums from the immutable tool
image, and stops each VM after 25 seconds with a two-second kill grace period.
The guest must remain halted; timeout status is expected and a success marker
alone never overrides a wrong transcript or process outcome.

Host tool inputs reuse the approved Rust 1.95.0 digest and hash-checked UEFI
component, Debian base digest, signed Debian snapshot, and exact QEMU/OVMF package
versions. Record resulting OCI IDs and executable hashes in every run. These are
host/bootstrap tools under ADR 0003; OVMF is VM platform firmware, not RAR code.
Compiler-provided core and compiler support must be declared in the target
inventory; no external allocator, kernel, boot library or OS is admitted.

The retained run bundle contains source/controller revisions, run/attempt/job,
runner image, sandbox policy, tool image IDs and hashes, both build digests,
focused test output, each boot image and EFI file, serial logs and classification.
GitHub artifact storage retains bundles for 90 days. Milestone closure must also
publish a durable release evidence bundle; an expiring CI upload alone is not
permanent release evidence.

Provisioning images has network access to pinned tool sources on the disposable
runner. Source compilation and VM execution have none. A Docker/kernel/emulator
escape remains a residual cloud-provider risk; no owner device participates.

Changing this controller requires review of the actual resulting commands and
negative tests. Do not broaden resource limits or omit failed records to obtain
a passing run. Foundation completion additionally requires integrated independent
review, malformed memory/handoff tests and documented unsafe-code invariants.

## Reproducible PE linking and firmware scratch

Rust 1.95's MSVC-style link path enables PDB output even when Rust debuginfo is
zero and symbols are stripped. The embedded CodeView identifier then depends on
temporary object paths. Cloud runs 33918813996 and 33919158210 identified this
mismatch; their PE headers and code-block fingerprints were otherwise equal.
The builder explicitly overrides this with /DEBUG:NONE while retaining the zero
COFF timestamp. No artifact bytes are normalized or excluded from comparison.
See [Rust linker behavior](https://github.com/rust-lang/rust/blob/1.95.0/compiler/rustc_codegen_ssa/src/back/linker.rs#L1073)
and [LLD option precedence](https://github.com/llvm/llvm-project/blob/llvmorg-22.1.0/lld/COFF/Driver.cpp#L1760).

Each launch copies the pinned OVMF variable-store template into private bounded
container tmpfs. The code pflash stays read-only; only this disposable variable
file is writable to firmware. Both templates are included in the recorded tool
hashes. Nothing is persisted across profiles or exposed to the owner device.

Controller-only bootstrap proposals are identified only when neither the trusted
base nor the proposed source contains a Foundation kernel. Their evidence says
controller-only and makes no build/boot claim. A proposal removing an existing
kernel is rejected. Any proposal containing the kernel must run the complete
reproducibility and boot gate. This avoids reporting a missing-source compiler
failure as a kernel failure while the first controller is being established.
