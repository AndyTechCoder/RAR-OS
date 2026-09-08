# Modern kernel mechanisms

lib.rs/model.rs implement and test the unactivated Modern lifecycle model:
logical endpoints, caller-local capabilities, bounded stamped queues, restricted
trial health, checked incarnations, atomic model cutover, revocation and fault
recovery. See docs/interfaces/modern-lifecycle-v0.md.

The distinct main.rs entry now connects the initial Modern policy to protected
CPU contexts, page tables, int80 IPC, keyboard, delivered timer ticks and the
native PIO adapter. It is an unactivated source candidate, not a boot proof or
completed dynamic loader. Foundation selects it only with rar_platform plus
rar_modern; rar_desktop is mutually exclusive. The existing Desktop profile and
runtime remain separate.

Model, ABI and support checks forbid unsafe code. The actual kernel entry uses
documented privileged/memory mechanisms; native_pio is its fixed storage leaf.
There is no external target dependency. Pure tests/no_std compilation run only
in the cloud Specifications sandbox; that does not compile or execute main.rs.
Actual pinned UEFI build and reviewed Modern controller execution remain gates.

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
The new entry supplies initial trap wiring, framebuffer mapping and scheduling;
actual UEFI compilation/execution, certified register layout, service/UI wiring
and runtime geometry/driver evidence remain pending. Model tests perform no
native I/O. See modern-runtime-v1.md for the boundary.

support.rs is used by the real entry and source tests: exact initial bootstrap
construction, full 152-byte user span checks before queue consumption, bounded
CPU selection, sticky tick exhaustion, and fixed candidate synthetic disk
expectations. Idle15 is a CPU context with no logical principal or grants.
M4.2 staging, trial construction/preemption/cutover and active bootstrap refresh
are not implemented by this initial M4.1 entry. No live-update claim follows.
