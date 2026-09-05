# Platform trusted cloud controller

Status: controller/contract proposal. No Platform target exists yet; no Platform
boot or milestone-completion claim is made by this change.

This is the consolidated trusted-main controller extension for
[Milestone 2](../../../docs/tasks/fast-track-alpha-milestone-2.md).
It reuses Foundation's fixed container isolation and bounded subprocess runner;
Foundation's normal, panic and exception workflow remains an independent
regression requirement.

## Trust and execution boundary

Only the reviewed main-revision workflow and controller execute on the disposable
GitHub Ubuntu 24.04 x64 runner. The same-repository proposal is read-only source
inside an unprivileged, read-only, no-network, capability-dropped container.
No proposal shell script, workflow or Python file is executed on the runner.
Fixed compiler commands run inside the build container; target artifacts run
only in the isolated emulated guest. Never run these entrypoints on a Mac or SSD.

Rust 1.95.0, UEFI compiler support, Debian snapshot, QEMU and OVMF retain the
Foundation pins. The launcher adds snapshot-pinned Debian Python 3.11.2-1+b1;
exact interpreter/emulator/firmware binary hashes are retained with evidence.
Python, firmware and QEMU are host tools, not linked into RAR target images.

The guest has q35/TCG, qemu64, one CPU, 256 MiB, emulated PS/2 and standard VGA.
It has no network, passthrough, host directories, credentials, physical input,
VNC, SPICE, host display or raw disk. Boot media is an immutable 16 MiB artifact
with a disposable snapshot overlay; firmware variables and QMP/capture paths
exist only in private container tmpfs. The reviewed launcher exposes no options.

## Bounded proof sequence

Two clean fixed-command builds must produce byte-identical kernel, service
fixture and boot image. Both kernel and service model suites must pass.
The strict artifact transfer rejects arbitrary filenames, wrong sizes, missing
tests, noncanonical Base64 and incomplete transfers.

The private QMP Unix socket accepts only the trusted launcher's fixed sequence:
capabilities, one synthetic A key, one 640x480 PPM capture at a constant tmpfs path,
and quit. Guest serial data can only advance the ordered proof state; it cannot
supply commands, keys, paths or emulator arguments. Input proof before injection
is rejected. The PPM must have the exact header, length and every expected pixel.
The outer trusted controller independently revalidates serial and pixels.

A 25-second proof deadline, 30-second container entrypoint deadline and 35-second
outer launch deadline bound failures. Successful QMP quit must return zero;
a timeout is a failure, never Platform success. Output and file sizes are bounded. Failed launches retain bounded serial bytes
and a diagnostic in the evidence bundle; failure is never relabelled success.
Only this run's named disposable cloud containers are cleaned up; no repository,
branch, tag, owner data, Mac or SSD file is deleted.

## Validation and limitations

`check.sh` compiles the trusted Python sources and runs 63 negative tests for
sandbox policy, serial/frame evidence, fixed QMP command selection, artifact
framing and source classification. These checks execute in cloud CI only.
They do not prove target behavior. Actual ring3, preemption, IPC, storage, fault
containment, input and framebuffer evidence is required once target code exists.

A kernel-absent controller/contract-only change is labelled controller-only,
never passed Platform. Partial source or removal of an existing Platform fails.
No durable storage, disk driver, GUI, networking, production security or stable
application ABI is claimed.
