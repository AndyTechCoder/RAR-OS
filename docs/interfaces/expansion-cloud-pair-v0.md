# Expansion closed-pair cloud candidate

Status: **inactive source candidate**, not runtime approval or M5 completion.
The existing Modern VM profile and all M4 evidence stay network-disabled.

## Fixed topology and authority

Two single-vCPU TCG x86_64 guests retain the reviewed Modern firmware, private
System/Data block backends and immutable boot NBD backend. Each gains exactly
one RAR-driven ISA NE2000 at ports 0x300–0x31f, IRQ 5. The kernel requires IRQ 5
masked before exposing bounded PIO. There is no bus-master DMA or NIC boot ROM.

Guest A uses MAC 02:00:00:00:00:01; B uses 02:00:00:00:00:02. Each QEMU receives
one end of a newly created connected, unnamed AF_UNIX/SOCK_DGRAM socketpair.
There is no listener, host NIC, TAP, slirp, port forwarding, DNS, routing or
Internet authority. The outer container must still use network none.
The socket buffers are explicitly bounded and verified. QEMU inherits only
its own network endpoint and its own three private block client descriptors.
Both parent endpoint copies close immediately after their respective spawn.
No guest receives the other guest's endpoint or disk descriptor.

`expansion_profile.py` composes, but does not replace or relax, all legacy paused
System/Data/boot/firmware checks. Additional checks require exact network client
inventory, the NE2000 QOM type/port/IRQ/MAC/backend binding and its exclusive flat
I/O aperture before either guest may start. Both guests must pass while paused.
Boot images are fixed separate artifacts at /artifact/peer-a/boot.img and
/artifact/peer-b/boot.img. Arbitrary image paths or models are not accepted.

## Lifetime and failure domain

`expansion_session.Pair` borrows four distinct owner-private regular Data/System
image descriptors, never raw devices; identities cannot alias across roles or
guests. It owns all newly created network endpoints and two VM lifecycles.
Every synchronous VM service pass also pumps the other guest's bounded channels
and backend watchdogs with a recursion guard. Both guests share a 120-second
deadline. Peer death, panic, descriptor/transport error, partial construction or
deadline failure closes the whole pair. Teardown first signals both QEMUs, then
closes parent sockets and joins each VM and its three backends. It collects
cleanup failures and never reports success if a child could not be joined.
Repeated destroy returns the same terminal receipt; it is not a retry.
The trusted outer caller must also use unconditional final teardown.

No reconnect, in-guest reset, device reinitialization, state handoff, host
networking or general network testing mode is authorized. Guest grant checks
and service expiry remain independent of the outer pair deadline.

## Evidence required before activation

Pure tests cover command preservation, fixed inventory/aperture refusal,
descriptor/image aliasing, wrong socket types/buffers, reentrant fair service,
peer failure/deadlines, partial construction and aggregate cleanup failures.
These are source tests with fake processes/descriptors, not a live certificate.

Independent review must cover the exact controller and outer confinement before
trusted-main activation. Actual pinned QEMU must then prove both paused
inventories, endpoint inheritance, UEFI link/boot, RAR PIO, unpredictable
cross-guest challenge and the negative/fault/teardown cases. QEMU formatting was
checked against upstream v7.2.0 primary sources; exact installed pinned QEMU
behavior is still a runtime gate, not assumed from synthetic parser fixtures.

Relevant upstream primary source: [ISA NE2000](https://github.com/qemu/qemu/blob/v7.2.0/hw/net/ne2000-isa.c),
[network inventory](https://github.com/qemu/qemu/blob/v7.2.0/net/net.c),
[UNIX datagram backend](https://github.com/qemu/qemu/blob/v7.2.0/net/socket.c).

## Review hardening

The pair additionally checks the two retained boot descriptors as fixed-size
read-only regular files and requires all six image inode identities to differ.
An identical peer-a/peer-b artifact inode cannot masquerade as separate builds.
Paired CONT requires a transient authorization for the exact member currently
being started by Pair.start; direct member start is refused. Direct destruction
of a registered member routes to aggregate pair teardown.

The inert real-VM constructor harness now covers legacy, A and B on successful
paused setup and every post-spawn initialization failure. It asserts exact
argument preservation, three versus four inherited transport descriptors,
peer-specific boot paths, parent endpoint closure, paired start gating and all
child/descriptor cleanup. Combined preflight is exercised through both actual
base and Expansion validators, including missing/extra evidence refusals.
