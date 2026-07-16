# ADR 0008: Hardware-Neutral OS with RAR Lab

Status: Accepted — 2026-07-16

## Context

RAR OS needs safe, reproducible multi-architecture development before physical reference hardware exists, without letting simulation redefine guest behavior.

## Decision drivers

- Keep the development Mac isolated from target execution and devices.
- Reproduce architectures, devices, faults, and timing inputs.
- Share contracts between virtual and physical drivers.
- Keep physical-support claims evidence-based.

## Considered options

- **Develop first on physical devices:** rejected as unsafe and poorly reproducible for early work.
- **Let host tooling simulate guest policy:** rejected because it would not test RAR OS behavior.
- **Pinned machines controlled by RAR Lab:** selected.

## Decision

Develop first against pinned x86-64, ARM64, and Tier 0 simulated machines controlled by RAR Lab. Physical and virtual devices implement the same RAR device-class contracts where practical. Guest updates, security, recovery, and tier changes remain real OS behavior rather than host simulation shortcuts.

## Consequences

- Development is reproducible and safe.
- Dynamic displays, sensors, failures, and multi-device scenarios are testable early.
- Simulation is not evidence of physical support.

## Security and data impact

Profiles forbid raw disks, host sharing, passthrough, networking, and elevation by default. Guest repair and permission decisions cannot be bypassed by the host.

## Compatibility and migration

Scenario and device contracts are versioned independently from the VM backend. Physical drivers implement the same device-class contracts where practical.

## Validation

- Profile commands contain only pinned, allowlisted workspace resources.
- Negative tests reject forbidden host integration.
- Record/replay reports first divergence.
- Physical support is claimed only after named-hardware testing.

## Replacement path

RAR Lab may replace its backend while retaining scenario and virtual-device contracts. Physical drivers slot beneath the same device-class RID interfaces.
