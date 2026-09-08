# Modern UEFI build-only evidence

This trusted-main cloud tool builds the real Modern kernel and ring3 service
twice in independent networkless containers. It does not activate the Modern VM
profile, boot either artifact, package a disk, run target code, invoke a target
test binary, mount media, provision System/Data, or perform crypto comparisons.
The existing Desktop/Foundation/Platform launchers are unchanged.

## Inputs and authority

The manual Modern UEFI build workflow runs only from canonical repository main.
Its one input is a full lowercase 40-hex source commit from the same repository,
validated before checkout. Pinned checkout actions have credential persistence
disabled. Controller code, Containerfile and fixed build commands come only from
that main revision; the proposal is read-only compiler input, never outer-runner
code. Both clean checkout identities are checked; source symlinks/submodules are
rejected. Source provides no command line, host output path, image or extra mount.

The recipe reuses the reviewed Desktop Rust 1.95.0 OCI digest and independently
SHA256-checked x86_64-unknown-uefi standard-library distribution. Provisioning can
fetch those fixed tool inputs. Compilation has no network, credentials, raw
devices, passthrough, host write bind or privilege. It uses the unchanged
Foundation sandbox: uid65532, read-only root, private bounded tmpfs, 2 CPUs,
1GiB memory/no extra swap, 64 processes, all capabilities dropped and
no-new-privileges. The source checkout is its sole read-only bind.

Build containers run only fixed compiler commands and fixed artifact transfer.
The service is linked at 0x400000 without relocation; its actual bytes are
embedded in the Modern kernel. No rar_modern_compile_only fixture is selected.
The maximum build time is 300 seconds each, output transfer 6MiB, executable
file size 2MiB each; workflow lifetime is bounded at 30 minutes. Container cleanup
is limited to exact names generated for this disposable cloud run, never files
or volumes on the owner's machine.

## Evidence and limits

The trusted bounded parser independently checks AMD64 PE32+, zero timestamps,
UEFI subsystem, executable entry, page-aligned nonoverlapping virtual sections,
nonoverlapping bounded file spans, no writable-executable section and no imports,
TLS, delayed imports or debug directory. The ring3 service also requires the
runtime's fixed base, 128KiB image limit and no dynamic relocations. Kernel image
span is capped at 64MiB; this is a parser ceiling, not proof of runtime allocation.
Malformed or oversized artifacts fail, rather than relaxing the runtime budget.

Both builds' hashes and layouts, tool executable/image identities, source and
controller commits, runner/run identities, and final status are retained.
Reproducibility here means two isolated builds with the same pinned tool image;
it does not mean two independent compiler implementations.
PE stack-reserve/commit headers are reported, but are NOT a maximum call-stack
proof. Real guarded-stack execution, syscall/framebuffer behavior, device
certification, cold-reboot persistence, fault behavior, signed live updates,
recovery and independent crypto comparisons remain separate M4 requirements.
Passing this workflow must never mark M4 complete.

Pure negative tests run in the existing isolated Specifications validation.
A new controller must pass independent review and merge before dispatch; never
execute a proposal copy of this controller as the trusted outer orchestrator.
No local Mac/SSD build or execution is authorized by this document.
