# Experimental Platform runtime

Status: implementation proposal; cloud compilation, boot and independent review
are required before claiming Milestone 2 completion.

The nucleus owns page tables, privilege transitions, process/capability state,
bounded IPC and the narrow PS/2 read broker. Actual volatile storage, keyboard
decoding and pixel drawing execute as separate ring3 roles from
`services/platform/runtime.rs`. The native fixture/client runtime is under
`core/platform/`; it is not a stable application ABI.

Sixteen fixed processes use private address spaces, private writable PE sections
and user stacks. Eight adversarial fixtures cover kernel-memory access, NX,
user-stack guards, port access, text writes, another process's private memory and
a privileged CR3 read, and an invalid user return stack pointer. A non-yielding task and two clients exercise preemption
and post-fault service communication. A two-phase assembly fixture checks all
general registers, stack, condition flags/DF, XMM0–15, MXCSR and x87 over real
timer preemption, credited separately only while each phase's sentinels are live.
A third check verifies supported context across an actual yield syscall and a genuinely blocked/woken receive.
A deliberately backpressured client exits with outstanding storage requests;
the invalid-return client faults with a pending request. Shared storage drops
undeliverable replies and continues serving the healthy client's retained state.

Kernel and user stack guard pages remain unmapped. User executable storage has
no writable supervisor alias in any active process root; original bootstrap
aliases are retired before ring3 entry. Fixed capability tables carry rights and
nonwrapping slot generations; peer death invalidates endpoint generations.
Sender identity is stamped by the kernel, not accepted from payload bytes.

Read [the private fixture contract](../../docs/interfaces/platform-runtime-v0.md)
for bounds, error results and unsafe invariants. All source is RAR-owned apart
from declared compiler support in `dependencies.json`.

Never build or execute this target on the owner's Mac or SSD. Builds and guest
execution are performed only by the reviewed trusted-main cloud controller.
No real user data, persistent disk storage, networking, GUI, stable app ABI or
production security is claimed.
