# ADR 0021: Release 0 Certified Execution-Host Boundary

Status: Accepted — 2026-07-18

## Context

The build runner can prepare evidence but is not automatically a safe execution host. Launch-affecting host kernel, container, resolver, wrapper, and policy state must be explicit.

## Decision drivers

- Prevent certification from silently inheriting mutable host state.
- Bind the descriptor used for resolution to the descriptor passed to the spawner.
- Enforce timeout, termination, output, and cleanup controls without direct launch paths.

## Considered options

- **Certify only the emulator binary:** rejected because the surrounding host can change launch behavior.
- **Treat any GitHub or developer host as equivalent:** rejected because service and kernel boundaries differ.
- **Certify one exact execution-host descriptor:** selected.

## Decision

Certification binds one execution-host descriptor: OS/kernel class, architecture, container/runtime identity, immutable closure, resolver/spawner/wrapper hashes, environment allowlist, resource controller, timeout, termination escalation, output cap, cleanup policy, network refusal, device refusal, and direct-launch refusal. The descriptor is an independently hashed, typed leaf in the prepared identity graph, avoiding circular self-reference. Attestation, authority, resolver, spawner, and consumption cross-check the same named graph edges. A stateful enforcing machine permits only prepared → authorized → resolving → running → exit/termination → cleanup → terminal consumption/refusal. Illegal order, repeated terminal events, retry, identity drift, absent cleanup, timeout, output exhaustion, kill, crash, or transition uncertainty refuses or quarantines. The resolver opens the pinned executable without following links and passes the exact full descriptor to a synthetic spawner. Prompt 7A supplies only schemas, validation, and synthetic/refusal implementations; it certifies no actual execution host.

## Consequences

Prompt 7 must review the exact future host and every launch-affecting hash. A green build CI job is not execution certification.

## Security and data impact

Direct emulator invocation, arbitrary environment, devices, networking, elevation, path reopening, and unbounded runtime remain forbidden.

## Compatibility and migration

Any host, runtime, kernel class, wrapper, resolver, or lifecycle-policy change invalidates certification.

## Validation

Host-only tests use fakes to prove descriptor binding, no spawn before complete authorization, timeout escalation, interrupted launch handling, cleanup/quarantine, and failure closure.

## Replacement path

A different execution host may be certified through Prompt 7 with equivalent isolation and exact closure evidence.
