# Modern cloud VM composition candidate

Status: unactivated source candidate for M4.1 through M4.3. Existing certified
Foundation/Platform/Desktop profiles and the v0.3 release are unchanged.
A manual trusted-main workflow and fixed entrypoint now exist as source candidates.
Neither has been deployed to trusted main or invoked; the earlier helper-only
stage descriptions below do not imply activation.

## Fixed device and process layout

The pinned existing QEMU/OVMF tool recipe remains the intended runtime. QEMU is
q35/TCG, qemu64, one CPU,256MiB, software VGA, no guest NIC, no user config or
human monitor frontend, and starts with -S. The only HMP operation allowed through
QMP is the fixed read-only diagnostic info mtree -f -o. It is not a general
monitor-command interface.

Data uses ISA IDE0x1f0..0x1f7/control0x3f6/IRQ14; System uses
0x170..0x177/control0x376/IRQ15. Each has one master, fixed512-byte sectors,
exact independent identity and capacity. Neither can select another export.
Boot uses the already established q35 firmware IDE bus. Every block path is a
fixed raw layer over one preconnected private NBD socket: no raw device, image
path, overlay, dynamic image format, second connection or writable clone.

Three independently killable backend processes hold only their assigned image
descriptor and server socket. QEMU inherits only the three client sockets plus
its bounded standard streams. Boot's actual descriptor is O_RDONLY from the
read-only /artifact/boot.img mount. Recovery Data is also O_RDONLY. Because IDE
requires a writable-looking graph, these immutable inputs use ADR0035's explicit
write-refusing mode; export flags are not the physical access authority.

## Paused preflight, not guessed compatibility

vm_profile.py constructs fixed argv and a fixed query plan. Before the first
cont it checks query-status is stopped; actual QOM port/control/IRQ properties;
disk serial/model/unit/512-byte geometry and drive binding; parent bus links;
exactly child[0] as an ide-hd on each PIO bus; block driver/size/graph read-only
flags; and actual flattened I/O range ownership. The latter rejects overlap,
wrong extent, priority, type or owner. The diagnostic format derives from the
pinned upstream source links below, with strict refusal rather than fallback.

The actual patched Debian binary remains unproven until this preflight runs
inside the reviewed cloud container. A mismatch ends the session without
continuing guest CPUs. Preflight does not replace firmware DMA-retirement,
kernel capability checks, DataVault authentication or corruption handling.

- [ISA IDE](https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/isa.c)
- [IDE regions](https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/ioport.c)
- [QOM bus naming](https://github.com/qemu/qemu/blob/v7.2.0/hw/core/bus.c)
- [Bus links](https://github.com/qemu/qemu/blob/v7.2.0/hw/core/qdev.c)
- [Flattened map and ownership](https://github.com/qemu/qemu/blob/v7.2.0/softmmu/memory.c)
- [Fixed diagnostic syntax](https://github.com/qemu/qemu/blob/v7.2.0/hmp-commands-info.hx)

## Concrete lifecycle helper

vm_session.py has no VM-launch CLI. Its guarded VM API requires the immutable
Modern cloud tool image, nonroot65532, isolated Python and cloud markers; those
are defense in depth, not substitutes for trusted-main dispatch/confinement.

The outer session creates exactly /tmp/rar-modern with mode0700. Each bounded
sequence creates a new vm-N directory without reuse; copies pristine firmware
variables; starts and verifies all three backend children; then constructs the
paused QEMU. It continuously drains serial/QMP and services backend watchdogs
during QMP replies, startup, keyboard delays and captures. No shell is invoked.
Every constructor failure attempts exact QEMU kill/reap and all backend joins.

The VM deadline is120 seconds, backend deadline130 seconds with external service,
serial limit64KiB, QMP aggregate2MiB/record256KiB, at most32 asynchronous events
and512 exact-ID requests. QMP allows fixed preflight queries, one authorized
continue, bounded literal keycodes and a capture only to that VM's frame.ppm.
Reset, quit/graceful flush, savevm/loadvm, arbitrary commands, paths and key chords
are unavailable. Captures must be exact bounded640x480 RGB PPM regular files.

destroy kills/reaps the ENTIRE QEMU first, then kills/drains/joins all three
backend children. It preserves actual retained System/Data files, never flushes
their volatile state on teardown, never reconstructs bytes and never creates
a replacement connection. Cleanup failures prevent a joined result and any
frozen-image authority; the outer disposable-container deadline is the final
boundary for an uninterruptible host operation. No owner file is retired.

## Still required before runtime acceptance

The outer trusted-main controller and immutable launch-image/workflow integration
are not yet supplied by these helpers. They must provide source/artifact/tool
digest binding, effective container confinement, exclusive System/Data files,
fresh per-image keys, independent frozen-image verification, whole-session
budgets, complete typed audit/command/serial validation and artifact retention.

M4.1 must actually type an unpredictable boot1-only write through Terminal,
capture the exact SAVED view, destroy every VM/backend process, freeze and verify
real bytes, start fresh firmware/backend processes over the same retained images,
and show the value in Files without retyping it. No host-generated screenshot,
serialized Python object, QMP reset or model state counts. Crypto interoperability
with the two independent references remains required before encrypted acceptance.

M4.2 actual signed Settings code replacement and M4.3 immutable System-only repair
and data-hash preservation remain separate full outcomes. Pure tests and these
candidate helpers do not mark any section complete.


## Review closure fixtures

The paused-machine validator now checks the complete block graph, including
explicit firmware nodes and exact root/file edges; named-node metadata alone is
insufficient. The existing AHCI model is explicitly named at the same 00:1f.2
address. All six boot-controller buses and the actual boot parent/child linkage
are checked, alongside its PCI model and the two independent PIO buses.

Pure lifecycle tests inject eight failures into the actual constructor control
flow through inert adapters (stream setup, selector registration, QMP connect,
greeting and request). No test starts QEMU. Five teardown failures cover VM
wait timeout/error and failure of each of the three backend joins. Every other
child must still be attempted, and no failing teardown may return joined=True.
Passing these fixtures will not replace actual cloud process lifecycle evidence.


## Independent visual expectation

visual_oracle.py implements four fixed public scenes independently of target
imports: initial workspace, initial Terminal, SAVED after the write, and Files
in a fresh VM. Its 32 lowercase a-p challenge encodes 128 random bits supplied
only after the first actual home capture. The second VM's input plan contains
only F1. Every capture is compared in full as a bounded 640x480 RGB PPM; serial
claims or a matching filename alone are insufficient. The oracle's expected
pixels are never guest screenshots and must never be emitted as captured evidence.
Its self-tests use synthetic frames only and claim no guest execution.


## Two-VM persistence scenario candidate

persistence.py now supplies the concrete scenario API, with no activated launch
CLI. It creates one empty Data image and one zeroed System image exclusively in
the private disposable cloud directory. It retains separate O_RDONLY observer
descriptors to the same fixed inodes. The host has no post-provision write API.

The public random challenge is generated only after VM1's actual home frame.
Terminal receives the write; exact SAVED pixels must appear. All QEMU/backend
processes are killed and reaped before observer reads. The independent frozen
oracle must find exactly note=challenge, revision2 and committed physical slots
0/1 with no burned slot. A fresh VM with new firmware and backend processes
receives only F1 and must render that value in Files. Data must remain byte-
identical across the second boot, while System and immutable boot hashes remain
unchanged throughout.

The baseline audit validates typed readiness/request/completed-event order,
operation geometry and monotonically counted ordinals. System/boot writes,
uncompleted mutations, injected faults and device failures are rejected. A
deliberate whole-VM cut may leave a final read or transport-EOF terminal record;
neither is treated as a successful write. Complete raw evidence is retained with
the summaries. This baseline is not the M4.3 fault-injection acceptance suite.

Output retains actual captured PPM bytes, serial/QMP/topology/child-cut records,
frozen Data bytes and hashes under a64MiB bound. The future trusted-main outer
controller must independently bind/recheck that evidence. The output explicitly
does not claim crypto interoperability or milestone completion. No candidate
source publication activates QEMU or a workflow.


The second VM receives the retained O_RDONLY Data observer descriptor with
readonly_data=True; its physical-readonly readiness must agree, and any Data
WRITE request rejects the baseline even if the immutable backend refused it.
The normal first-VM audit conservatively marks every completed WRITE dirty and
clears it only on a completed FLUSH. A dirty cut cannot pass. Focused negatives
cover write-without-flush, another write after the last flush, writable readiness
in VM2, and a VM2 Data WRITE; write followed by flush is a positive fixture.


## Cloud packaging and entrypoint integration candidate

launch.Containerfile reuses the existing pinned Debian/QEMU/OVMF/Python tool
inputs. Apt reads only the fixed snapshot source list (sourceparts disabled);
it does not delete the base image's configuration. Only explicitly named reviewed
host helpers are copied into /opt/rar-modern. The image defaults to UID/GID65532.
The outer controller must still enforce network-none, read-only mounts/root,
resource limits, credentials/device denial and exact image/helper identity.

launch.py has no options or source-selected imports. It loads only fixed tool
siblings, applies the cloud VM guard, invokes the two-VM scenario, and emits a
single bounded canonical JSON result. This is a candidate entrypoint, not an
activated workflow or authorization to run the image locally.

pack.sh uses the already pinned cloud Rust image to compile the trusted-main
RAR-owned nucleus/foundation/image.rs mounted read-only as /packager.rs. The
bounded previously inspected UEFI binary is mounted read-only at
/artifact/modern.efi. The host packager runs only in the disposable container and
produces the fixed16MiB FAT16 boot image; no target code executes during packaging.
boot_image.py independently reconstructs the fixed FAT metadata and binds every
payload/zero-padding byte to that exact UEFI input, rejecting changed geometry,
directories, FAT entries, payload, padding or transfer framing. Its tests are
bytes-only synthetic fixtures, not boot evidence.

No Modern workflow or outer-controller dispatch is introduced by this candidate.
Effective confinement, exact source/build/helper binding, independent retained
envelope checks and actual cloud execution remain required before acceptance.


## Retained evidence checker candidate

runtime_evidence.py takes only bounded canonical JSON bytes plus independently
trusted boot digest and firmware sizes. It checks all five full actual frames,
the authenticated frozen Data snapshot and initial empty-image binding, unchanged
System digest, distinct fresh QEMU PIDs, the same three separate disk inodes,
typed backend audit, exact paused preflight, exact fixed QEMU arguments, and the
ordered input/capture plan. VM2 may receive only F1. The sole expected QMP event
is RESUME; panics, unexpected events, alternate paths and extra commands fail.

This is a content checker, not provenance or milestone acceptance. Its output
explicitly leaves provenance_validated and milestone_complete false. Only the
future trusted-main outer controller may bind the actual container/process,
source/build/tool identity and independently measured firmware geometry.
Current pure tests cover malformed framing, argument changes and command/capture
drift; a complete successful retained envelope still requires actual cloud VM
execution and independent review. No synthetic success envelope is boot evidence.


## Trusted-main outer integration candidate

runtime_controller.py and modern-persistence.yml now connect the reviewed pieces.
The workflow is manual-only, canonical-repository/main-ref constrained, and uses
pinned checkout/upload actions with read-only repository permissions. It does not
run automatically on a branch push. The controller additionally requires Linux,
isolated Python, the exact trusted-main workflow revision and clean exact source/
controller snapshots. Ambient Docker endpoint/configuration overrides are denied;
all Docker operations use a fresh private cloud configuration and fixed local
cloud-daemon endpoint.

Every compiler, packager, identity reader and VM container is created stopped.
Before start, inspect evidence must match the exact image/name, nonroot user,
fixed command/environment, network-none/read-only-root,1GiB memory and swap,2CPU,
64PID limit, dropped capabilities, no-new-privileges, no device/host namespace
access, exact256MiB private tmpfs and only the specified read-only bind mounts.
The runtime container receives the built artifact directory, never source or a
Docker socket. Container inspect records before and after execution are retained.

Two actual UEFI builds must match. The RAR host packager's result must satisfy the
independent whole-image check. Pinned runtime tool hashes and measured firmware
sizes are retained before the scenario runs. The emitted bounded actual envelope
is independently revalidated against those build/tool inputs. Successful content
validation is still not complete M4 or crypto acceptance. All containers owned by
this cloud invocation are terminated on exit; cleanup failure prevents success.
No owner files, source trees, images, volumes or local/SSD files are deleted.

Independent review and trusted-main tooling integration are still required before
the first bounded diagnostic VM run. That first real envelope must then pass full
independent success-path validation and field/state mutation coverage before its
runtime evidence can be accepted. Requiring a real envelope before allowing the
first reviewed diagnostic run would be circular; no such gate is introduced.

Backend termination content admits exactly (-9, backend-failed) or
(21, backend-failed), matching the actual observed process wrapper. Both impossible
nonzero/None combinations are negative fixtures. The first full actual envelope
will additionally exercise and mutate disk hashes, frames, input chronology,
preflight, readonly Data mode, inode/PID binding and termination states.

### Exact cloud container ownership remediation

The outer controller creates each container with an invocation-bound ownership
label, restart disabled and daemon logging disabled. It validates the returned
full container ID against an ID-based inspection, including name, image and
label, before recording cleanup authority. Start, post-execution inspection and
cleanup use only that immutable ID. Confinement is checked before first start.
An ambiguous create or mismatched identity fails the disposable hosted job;
it never grants name-based cleanup authority or retries creation. Unknown
objects are left to final hosted-runner teardown. Mock lifecycle checks cover
ambiguous creation, identity/label/name mismatch, confinement rejection, start
failure/timeout, swapped post-inspection and cleanup failure. These fixtures
prove control flow only, not an actual Docker or VM execution result.
