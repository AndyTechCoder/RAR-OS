# Modern cloud persistence diagnostic integration

This change installs reviewed host-only tooling for the owner-directed M4.1
persistence work. It does not merge the Modern target implementation, publish
v0.4, or satisfy a milestone by itself. The runtime remains in draft PR158.

## Scope

The fixed manual Modern cloud persistence workflow uses the canonical main
controller and an exact canonical source commit. It builds that source twice,
checks target images, packages the immutable boot image with the existing RAR
host packager and independently checks every boot-image byte. The isolated
runtime container owns fresh synthetic System/Data regular files. No owner
files, local machine, SSD, credentials, raw device or guest network is attached.

The controller implements the bounded Modern profile documented in
tools/rar-lab/modern/vm-profile-v0.md. Its initial permitted purpose is diagnostic
cloud validation following independent source/confinement review and source CI.
QEMU remains paused until the controller verifies its actual device graph.
A profile mismatch fails closed; it does not permit arbitrary launch arguments
or bypasses. Runtime compatibility is evidence to obtain, not a prior claim.

## Ownership and cleanup

All created cloud containers disable restart and daemon logging. Operations
and cleanup use only full IDs after exact ID/name/image/invocation-label
verification. An ambiguous create or identity mismatch is not resolved by name;
the job fails and final disposable hosted-runner teardown handles unknown
objects. Source, boot inputs and artifacts are never removed by cleanup.
The Mac and SSD receive no changes or execution.

## Acceptance still required

The first diagnostic run must retain actual two-VM serial, pixel, block and
frozen-disk evidence. A real positive envelope and systematic negative mutations
must pass the independent retained-evidence checker before persistence is
accepted. All crypto/reference, signed replacement, isolation/revocation,
rollback, System-only recovery, interrupted-write and reproduction requirements
in the active M4 contract remain binding. Tests of host helpers and inert
lifecycle fixtures are not evidence that RAR target behavior works.

The tools are experimental laboratory infrastructure; public test keys do not
provide production confidentiality. No change to stable OS formats, dependency
policy, tier meanings or persistent user-data promises is made here.
