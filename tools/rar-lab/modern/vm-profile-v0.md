# Modern cloud VM composition candidate

Status: unactivated source candidate for M4.1 through M4.3. Existing certified
Foundation/Platform/Desktop profiles and the v0.3 release are unchanged.
No workflow or executable Modern launch entrypoint is enabled by these helpers.

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
