# Host Mac Safety Policy

Status: Mandatory and effective immediately

## Absolute invariant

RAR OS source, specifications, build artifacts, and VM disk images may be stored on this Mac. RAR OS boot code, Nucleus code, drivers, services, and applications must never execute natively on this Mac and must never replace, modify, extend, or participate in booting macOS.

RAR OS may execute only inside a reviewed and approved RAR Lab virtual-machine profile. Building a bare-metal target binary does not execute that binary; only host compilers and validation tools run during a build.

Codex is authorized to modify any file inside this repository and its Git metadata, including destructive rewrites. Automatic review may approve repository-confined work and publication to the canonical `AndyTechCoder/RAR-OS` GitHub repository. All other external effects fail closed under `.codex/config.toml` and `.codex/rules/host-safety.rules`.

## Forbidden actions

Agents, tools, scripts, and contributors must never:

- Install RAR OS onto an internal or external physical disk from this Mac.
- Write RAR images to `/dev/disk*`, raw devices, partitions, EFI System Partitions, recovery volumes, or macOS system volumes.
- Run `dd`, imaging tools, partitioning tools, or filesystem formatters against host devices.
- Use `bless`, `nvram`, Startup Disk changes, firmware updates, boot-policy changes, kexts, DriverKit/System Extensions, launch daemons, or login items for RAR OS.
- Load RAR executables through macOS, Rosetta, kernel extensions, or native process APIs.
- Use QEMU/VM raw-disk, physical USB, PCI, GPU, network bridge, host filesystem, clipboard, camera, microphone, or other device passthrough.
- Run a RAR VM as root or with unsandboxed elevated privileges.
- Start QEMU or another emulator directly outside the approved RAR Lab launcher.
- Enable VM networking before the profile and guest network behavior have a dedicated approval.
- Change these rules merely to make a test easier.

## Allowed host activity

- Read and edit source and documentation.
- Run Git, text-processing, documentation, lint, static-analysis, compiler, linker, and packaging tools.
- Compile bare-metal target artifacts that macOS cannot load as native applications.
- Inspect target binaries without executing them.
- Run host-only tests that contain no target OS code.
- Run an approved RAR Lab launcher after its profile has passed the certification below and the owner has authorized guest booting.

Repository command-prefix rules are defense in depth, not an exhaustive command-family parser. Absolute executable paths, shell or environment wrappers, build-tool indirection, launcher scripts, and unknown emulator names receive no implicit exception: the automatic reviewer denies uncertain parsing or non-canonical destinations, and R0-000 validates the fully resolved executable and argument vector before any process spawn.

## Certified VM profile

“Certified” here means approved for safe development-host execution, not independently security-certified.

Before first guest execution, a profile must prove:

- Pinned emulator and firmware hashes.
- CPU emulation/virtualization isolated from native execution.
- No raw or physical host storage.
- Only disposable image files under the workspace output directory.
- No host device, filesystem, clipboard, or credential sharing.
- Networking disabled by default.
- No elevated privileges.
- Explicit memory, CPU, runtime, and output limits.
- Emulator sandbox enabled where available.
- Full command line produced by an allowlisted launcher, never arbitrary string interpolation.
- Static negative tests rejecting forbidden arguments and paths.
- Timeout and forced termination controlled by the host launcher.
- Evidence identifying profile, hashes, command, artifact, and source revision.

The first certification review inspects the launcher and generated command without booting a RAR artifact. The owner separately authorizes the first actual guest boot.

## Initial execution phases

1. **Documentation/scaffold:** no RAR executable exists.
2. **Static build:** bare-metal artifacts may be compiled but not executed.
3. **Launcher certification:** unsafe VM configurations are rejected; no guest boot yet.
4. **Owner-authorized VM boot:** RAR runs only within the approved isolated profile.
5. **Expanded devices/network:** each new passthrough-like capability requires separate review; physical passthrough remains prohibited on this Mac.

## Physical hardware work

Physical-device testing happens later on dedicated test hardware, never by repartitioning or changing the boot configuration of this Mac. Image-writing and device-flashing tools must refuse host system disks and require a separately documented hardware-lab workflow.

## Incident response

If any command attempts a forbidden action:

1. Stop immediately.
2. Do not retry with elevated privileges.
3. Preserve the command and relevant logs.
4. Report whether any external state changed.
5. Require owner and security review before continuing.
