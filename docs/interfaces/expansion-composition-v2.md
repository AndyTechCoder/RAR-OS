# Candidate Expansion kernel composition v2

Status: source candidate only. The new rar_expansion build flag selects the
composition only alongside rar_signed_updates. No existing target build or
cloud profile enables that flag. Source CI emits Linux objects only; it does
not link, execute or activate a guest.
This is not authority to execute a new VM profile.

## Explicit mapping

The new model constructor starts only existing bootstrap principals8/9.
One atomic desktop plan prepares the existing roles plus logical principal7 at
physical slot10. Settings principal5 retains physical slots5 and7 for trials
and rollback. No logical/physical equality is inferred for these roles.
The native loader consumes the prepared role-to-slot mapping, constructs each
private root unscheduled, revalidates all roots and publishes runnable state
only after the complete authority plan is published. A failed slot10 reservation
or changed composition refuses publication before changing any binding.

Network is a distinct Object and right256; internal right storage widens to
u16. Existing bit values and caller-local handle encoding do not change.
Only active physical slot10 with logical principal7 and its exact current
binding may resolve Network. Storage Device, StageCopy, Input, Framebuffer and
Manager grants are not substitutes. No raw I/O is implemented by the model.
Network gets Receive, named-send to Terminal6, and Network in local slot11.
Terminal gains only named-send to principal7 in local slot5.
This is a bounded initial network-app channel, not a general grant-management API.

## Private bootstrap v2

The private368-byte Boot layout remains fixed; version2 identifies Expansion.
Existing version1 masks and peer7-zero rule are preserved.
Version2 Terminal adds send slot5; version2 Network has only slots0,1,11.
Network's own active role7 requires its exact nonzero self incarnation.
Other roles may observe peer7 absent after isolated Network failure; a stale
named-send grant grants no destination or new authority. They continue normal
offline work, including Settings update/rollback. Networking stays unavailable
until a separately controlled reboot; no automatic adapter reinitialization.
Bootstrap8/9 and idle15 may precede network publication. Settings trials retain
their single health grant and may precede publication. Trial activation
requires identical bootstrap version as well as identity/entry consistency.
No device geometry, framebuffer, kernel pointer or user-chosen port is exposed
to Network. Network's physical slot10 is not represented as logical role10.

This is an experimental private kernel/service contract, not a stable SDK or a
change to persisted Data/System files. Public app network bytes remain the
separate RNETv0 protocol.

## Revocation and validation

Fault cleanup stamps and purges by logical principal and full incarnation,
not physical slot. Killing Settings slot7 cannot revoke Network principal7.
Killing Network slot10 removes network grants/queued messages but preserves
Settings. Native device shutdown on owner death remains mandatory future work;
logical capability revocation alone is not proof that hardware has stopped.

Tests cover prepare-before-publication denial, cross-capability refusal,
wrong physical caller, stale handles, collision/revalidation failure, explicit
slot mapping, stamped IPC, network death, Settings update/cutover/fault while
network messages are queued, and all version2 descriptor masks, the exact prepared-root predicate used by
native synchronization, and successful Settings handover after Network loss. Existing
version1/kernel/source regressions remain required. Native root/PIO/scheduling
and causal paired-guest evidence are not replaced by these tests.

Existing builds still choose the legacy constructor. Candidate flag-selected
code supplies Network syscall13, an exclusive port leaf, revocation reconciliation
and role7 native service loop. Prepared native roots are protected by the plan's
explicit physical mapping, including slot10, while publication is pending.
Native service work polls hardware before at most one client message, including
while its single owned reply waits under bounded kernel-queue backpressure.
Full peer identity is checked before request dispatch. All I/O crosses the
kernel Network grant; no user-mode port instructions are introduced.

The A/B compile-time peer fixture chooses two static MAC/IP/port endpoints,
not production discovery or authenticated identity. Each profile has finite
packet/byte budgets and60,000 delivered-tick expiry. No automatic device reset,
grant renewal, reconnect or retransmit occurs. Service failure closes the link
and exits; kernel revocation reconciliation stops a previously bound NIC.

A fixed closed cloud profile, target linking/build closure and actual guest
proofs are still mandatory before selecting this source in a runtime workflow.
Do not infer native success from the object-only compile path.
