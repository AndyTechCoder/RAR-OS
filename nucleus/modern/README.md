# Modern kernel mechanisms

lib.rs/model.rs implement and test the unactivated Modern lifecycle model:
logical endpoints, caller-local capabilities, bounded stamped queues, restricted
trial health, checked incarnations, atomic model cutover, revocation and fault
recovery. See docs/interfaces/modern-lifecycle-v0.md.

This is not a kernel runtime, dynamic loader, sealed-memory implementation,
storage controller or boot proof. The model and ABI forbid unsafe code; the
new native_pio leaf contains narrowly scoped privileged x86-64 UEFI instructions.
There is no allocation, external runtime dependency or OS execution entrypoint. The real trap/loader
integration and independent runtime evidence must enforce the documented model.
Tests/no_std compilation run only in the cloud Specifications sandbox.

Staging records are private and currently populated only by test fixtures.
Production verification/sealing integration is still absent. Fault/timer events
are incarnation-bound; manager failure cancels pending work and requests
controlled recovery without disrupting the active Settings instance.

The M4.1 integration candidate includes the complete existing desktop named-send
graph and caller-local Data/System/Input/Framebuffer capability checks. Data
storage principal1 alone holds Data; System storage principal9 alone holds
System; manager8 and all apps hold neither. These methods derive device kind
from the kernel-owned table, not a userspace selector. Keyboard has no receive
grant. Trial and replacement Settings receive no device authority. Exhaustive
IPC-edge, cross-role/type denial and fault-revocation tests cover these additions.

The native_pio candidate maps authorized Data/System operations to separate
fixed registers; non-UEFI builds use inert denied stubs. Its public entrypoints
are unsafe and require the documented certified VM/kernel-context invariants.
Trap wiring, actual UEFI compilation/execution, certified register layout,
geometry/driver sequencing, framebuffer mapping and scheduling remain pending.
Model tests perform no native I/O. See modern-runtime-v1.md for the boundary.
