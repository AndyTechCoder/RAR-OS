# ADR 0035: Immutable IDE inputs through write-refusing private backends

Status: Candidate under the owner's delegated safe-direction authority;
independent review and concrete cloud profile validation required before use.
This does not activate a VM profile or modify target formats/production promises.

## Context

M4 needs a truly immutable boot input, independently writable System/Data during
normal operation, and no Data-write authority during recovery. The exact pinned
QEMU IDE implementation requests BLK_PERM_WRITE for ide-hd, even when its
configured block image is read-only. A writable overlay would conceal writes and
could create an unintended persistence channel. No such overlay is permitted.

Primary source at QEMU v7.2.0:
- [IDE realization](https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/qdev.c)
  passes readonly=false for hard disks.
- [Backend permissions](https://github.com/qemu/qemu/blob/v7.2.0/hw/block/block.c)
  consequently requests write permission.
These findings are design evidence, not certification of the patched binary.

## Alternatives

A. Preserve the existing IDE/firmware composition, but give QEMU only a private
NBD socket. The backend's actual image descriptor is O_RDONLY; every WRITE fails
with EPERM before any file mutation. The protocol may advertise a writable export
solely to satisfy IDE realization. Recommended.
B. A writable temporary overlay: rejected. Input immutability and causal
persistence evidence must not depend on ignoring a second writable store.
C. A different virtual boot device: deferred. Introducing another DMA controller
would require fresh firmware/runtime quiescence evidence and is unnecessary.
D. Change or rebuild third-party QEMU to accept read-only IDE: deferred to avoid
an additional unverified emulator fork and toolchain.

## Decision and invariants

Implement A as an explicit private write_refusing=true mode, legal only when
the actual Disk is read-only. It never turns a writable descriptor into a
read-only promise: constructor and every operation verify O_RDONLY/non-append
flags, inode/type/link count and exact size. WRITE is rejected before buffers,
dirty sectors, physical writes or success replies. Boot kind is always read-only,
has a fixed16MiB geometry and has no fault injection or mutable-data encoding.

The child's readiness evidence records BOTH physical readonly and export_readonly.
The paused QEMU graph's ro=false is expected in this mode, not proof of write
authority. The trusted controller must bind the actual inherited file descriptor,
backend readiness, observed write failures, effective read-only boot mount and
unchanged frozen hashes. A graph flag alone is never acceptance evidence.

Data/System remain separate fixed PIO controllers, sockets and backend children.
The boot-only IDE device remains on the existing q35 AHCI firmware path. No new
runtime device capability, DMA model, physical passthrough or host disk appears.
Recovery Data uses an O_RDONLY descriptor and write-refusing backend; the guest
recovery process must STILL lack a Data-write capability. Neither layer replaces
the other. Normal Data and System continue to use their own O_RDWR files.

The boot input is fixed /artifact/boot.img inside the immutable tool container
mount. QEMU never inherits an image descriptor or receives a writable file path;
it receives only the three preconnected socket endpoints. Its firmware variables
are fresh disposable bytes for each entire VM. No reset/snapshot/reconnect can
substitute for process destruction.

## Validation and consequences

Extend real cloud backend tests to open boot and Data fixtures O_RDONLY, negotiate
the IDE-compatible writable-looking export, attempt actual writes, observe EPERM,
and verify unchanged full-image hashes. Pure profile checks verify stopped guest,
actual QOM identities/ports/IRQs/master-only buses, flattened port ownership,
block-node geometry and fixed backend routing before the sole cont command.

The concrete VM/session, source/image binding, confinement, private fixtures and
whole-VM/backend kill/join require independent review and actual cloud evidence.
A candidate or passing parser fixture is not certification. No Mac/SSD operation
or production privacy/security claim is authorized by this ADR.


## Review refinement: complete routing and stable boot identity

The q35 built-in SATA instance is disabled and the identical ich9-ahci model is
instantiated explicitly at its existing PCI address 00:1f.2 with the stable ID
rar-boot-ahci. This adds no controller model or runtime DMA authority. Paused QMP
must prove the Intel 8086:2922 SATA identity at that address, the boot disk's
parent link and sole master child on port0, and empty ports1..5.

Firmware code/variables use explicit named raw/file nodes bound to
/machine/system.flash0 and flash1. Their geometry is measured from the pinned
tool-image files, not guessed. The read-only code node retains no write permission.
x-debug-query-block-graph must contain exactly ten block-driver nodes, five
backends and ten expected edges, including every raw-to-role-specific NBD/file
edge and every backend root. Extra, disconnected, backing or cross-role paths
fail preflight. The experimental query is deliberately tied to pinned QEMU;
unsupported output fails rather than weakening validation.

Primary definitions: [q35 device placement](https://github.com/qemu/qemu/blob/v7.2.0/hw/i386/pc_q35.c),
[AHCI buses](https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/ahci.c),
[firmware bindings](https://github.com/qemu/qemu/blob/v7.2.0/hw/i386/pc_sysfw.c),
[block graph schema](https://github.com/qemu/qemu/blob/v7.2.0/qapi/block-core.json).
These source-derived expectations still require actual cloud binary evidence.
