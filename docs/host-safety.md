# Host Mac Safety Policy

Status: Mandatory and effective immediately

## Absolute invariant

RAR OS source, specifications, build artifacts, and VM disk images may be stored on this Mac. RAR OS boot code, Nucleus code, drivers, services, and applications must never execute natively on this Mac and must never replace, modify, extend, or participate in booting macOS.

RAR OS may not be compiled, linked, packaged into a boot image, or executed on
this Mac. Target compilation, linking, boot-image creation, firmware loading,
and guest execution occur only in the owner-approved cloud Development Lab
defined by ADR 0017. A future local RAR Lab profile would still require a
separate explicit owner decision.

Codex is authorized to modify repository working files and ordinary Git metadata.
It may not rewrite or discard published sprint history. Automatic review may
approve repository-confined work and publication to the canonical
`AndyTechCoder/RAR-OS` GitHub repository. All other external effects fail closed
under `.codex/config.toml` and `.codex/rules/host-safety.rules`.

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
- Run Git without force/history-rewrite operations.
- Run text-processing, documentation, lint, static-analysis, and reviewed
  repository scripts that are provably host-only.
- Compiler, linker, object-copy, packaging, firmware, and image commands may not
  run directly on the Mac. A reviewed host-only script may compile only its own
  diagnostic/test helper when it cannot select or emit a RAR target artifact.
- Inspect target binaries without executing them.
- Run host-only tests that contain no target OS code.

## SSD data-safety boundary

The SSD contains unrelated irreplaceable owner data. On the owner's Mac, the
only RAR OS workspace is the exact subtree
`/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha`.

- Do not read, list, search, size, hash, permission-change, move, or delete any
  parent, sibling, or unrelated SSD path.
- Do not run volume-wide cleanup, wildcard deletion, recursive permission
  changes, filesystem repair, formatting, or imaging.
- GitHub is authoritative, but the current owner directive is to delete nothing in the RAR OS workspace.
- No-deletion scope: files, directories, scratch, artifacts, and worktrees.
- No-overwrite scope: moving or copying over an existing path is forbidden.
- Duration: this remains in force after merge until explicitly lifted by the owner.
- Future removal rule: after an explicit lift, only one exact registered worktree may be removed after clean pushed commits, exact remote merge verification, and separate review.
- No failure grants permission to broaden cleanup.
- Under the current directive, the combined local gate is `/bin/sh
  tools/ci/check-local-readonly.sh`. It performs whitespace, shell-syntax,
  and host-policy checks without scratch or mutation. Its complete wrapper and
  sole executable policy dependency are digest-bound in CI.
- The production workspace-budget check is fixed to this exact subtree and to
  the reviewed 10-GiB free, 8-GiB workspace, and 512-MiB output ceilings. It
  rejects path arguments and environment overrides. Its numeric test helper
  accepts values only and performs no filesystem discovery.
- If the volume is absent, renamed, unexpectedly mounted, or the resolved path
  escapes the exact subtree, stop before any write.
- The repository selects the `rar-os-ssd` permission profile. Codex requires its
  definition to be installed once at user level from the reviewed
  `.codex/rar-os-ssd-user-fragment.toml`; repository config cannot grant itself
  a new machine permission profile. Until installed, task startup fails closed.
  The profile grants local commands only minimal runtime reads and read/write
  access to this exact subtree. Start every RAR OS task from its SSD worktree;
  do not select a legacy sandbox mode or add the historical Mac source folder
  as a runtime workspace root.
- A repository fragment or nested `codex sandbox` smoke test cannot prove which
  profile governs the current task. Before unattended implementation, the owner
  must install the exact reviewed fragment, start a fresh task from the SSD
  worktree with that named profile selected and no legacy override, and retain
  the one-time in-product confinement check: a write inside the exact subtree
  succeeds while a read and write outside it are denied. Until then this is a
  manual fail-closed blocker, not a mechanically satisfied preflight.
- `.rar-os-workspace-identity` is a path-continuity guard marker, not proof of a
  physical disk's identity. It detects an absent, renamed, or mismatched RAR OS
  workspace; it does not authenticate the SSD or unrelated content on it.

Repository command-prefix rules are defense in depth, not an exhaustive command-family parser. Absolute executable paths, shell or environment wrappers, build-tool indirection, launcher scripts, and unknown emulator names receive no implicit exception: the automatic reviewer denies uncertain parsing or non-canonical destinations, and R0-000 validates the fully resolved executable and argument vector before any process spawn.

## Local certified VM profile (not authorized for Sprint Alpha)

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

The first certification review inspects the launcher and generated command
without booting a RAR artifact. The owner separately authorizes any future local
guest boot. Sprint Alpha cloud Development Lab approval does not satisfy or
bypass that local authorization.

## Cloud Development Lab

ADR 0017 permits automated cloud target work only on repository-approved Linux
runners with pinned target-affecting OCI, compiler, linker, emulator, firmware,
and artifact inputs. Runs use bounded disposable storage and explicit resource,
output, and timeout limits; initially disable guest networking; prohibit host
sharing, passthrough, raw devices, elevated execution, production credentials,
and unrelated external access; and retain complete logs, structured results,
real exit status, serial output, and exact hashes. Cloud evidence does not
authorize execution on this Mac or constitute production certification.

## Initial execution phases

1. **Documentation/scaffold:** no RAR executable exists.
2. **Local static checks:** documentation, formatting, and host-only checks only;
   no target compilation, linking, image creation, or firmware loading.
   Mutation-based policy tests skip without writing on the Mac and SSD. They
   obtain a scratch path only from
   `tools/ci/require-ephemeral-policy-test-root.sh`, which admits the dedicated
   pinned validation container's bounded `/tmp` tmpfs after verifying its
   container shape, exact clean source revision, mount flags, and size. The
   workflow uses an independent exact-revision checkout that the earlier
   validation container cannot access, then mounts it read-only. `/tmp` is the
   only approved policy-test scratch path. Docker may expose isolated writable
   pseudo-filesystems such as `/dev`; they are not host-backed, are not accepted
   by the guard, and policy-test work paths cannot target them. Incomplete
   Linux/CI evidence fails closed. Each Specifications container is limited to
   2 CPUs, 2 GiB of memory with no additional swap, and 256 processes; the
   aggregate validation job is limited to 30 minutes. Pull-request orchestration
   is loaded from the trusted base and checks out the proposal only as untrusted
   input. Proposal validators execute only when the complete executable
   authority closure is byte-identical to the trusted controller; controller
   changes remain data-only until the resulting `main` commit validates itself.
3. **Cloud Development Lab:** bounded automated target builds and isolated guest
   execution under ADR 0017.
4. **Future local launcher certification:** unsafe VM configurations are
   rejected; no guest boot occurs.
5. **Separately owner-authorized local VM boot:** outside Sprint Alpha authority.
6. **Expanded devices/network:** requires separate review; physical passthrough
   remains prohibited on this Mac.

## Physical hardware work

Physical-device testing happens later on dedicated test hardware, never by repartitioning or changing the boot configuration of this Mac. Image-writing and device-flashing tools must refuse host system disks and require a separately documented hardware-lab workflow.

## Incident response

If any command attempts a forbidden action:

1. Stop immediately.
2. Do not retry with elevated privileges.
3. Preserve the command and relevant logs.
4. Report whether any external state changed.
5. Require owner and security review before continuing.
