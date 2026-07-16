# RAR Lab Specification

Status: Gate 0 approved direction — 2026-07-16

## Purpose

RAR Lab is the host-side environment for building, booting, connecting, transforming, inspecting, recording, and damaging simulated RAR devices. It is a laboratory, not the target OS and not a substitute for guest behavior.

## Initial machine profiles

- `micro-arm`: Tier 0 ARM microcontroller-class machine with timers, storage, sensor, actuator, radio, watchdog, and reduced-assurance Vault.
- `device-arm64`: Tier 1 ARM64 machine with MMU, block storage, network/Wi-Fi, sensors, actuators, power, and optional display.
- `personal-arm64`: Tier 2 phone/tablet profile with touch display, battery, cameras/sensors, audio, radios, and hot-pluggable external display/input.
- `desktop-x64`: Tier 2/3 x86-64 UEFI profile with RAR virtual GPU, storage, wired/Wi-Fi, keyboard, pointer, audio, and multiple displays.
- `compute-arm64` and `compute-x64`: Tier 3 profiles with additional CPU, memory, storage, accelerator, and development devices.

Profiles declare capabilities and resources; guest code never branches on profile names.

## Virtual devices

RAR-owned virtual contracts cover Vault, block storage, GPU/display, input, audio, camera, sensors, motors, battery, charging, thermal, entropy, wired network, Wi-Fi/AP, Bluetooth controller, USB controller, clock, and fault controller.

QEMU may provide processor/platform emulation. RAR Lab adapters expose reproducible device control and record all external events needed for replay.

## Live controls

- Resize, attach, detach, rotate, and change density of displays
- Add/remove keyboard, pointer, touch, controller, storage, network, sensors, and peripherals
- Change battery, charging, thermal, signal, bandwidth, latency, packet loss, and connectivity
- Constrain CPU, memory, storage, and energy where the backend permits
- Connect multiple virtual devices and access points
- Install/remove tiers and layers through real guest update interfaces

CPU architecture cannot change live. Cross-architecture scenarios use separate connected VMs and stateful handoff.

## Fault injection

Inject component crashes, hangs, invalid IPC, storage corruption, torn writes, power loss, full disk, memory pressure, dropped packets, hostile peers, clock jumps, invalid firmware response, device removal, thermal shutdown, failed update, failed migration, revoked signing key, and damaged Recovery slot.

Faults are scenario files with deterministic seeds and expected guest-visible outcomes.

## Record and replay

Capture virtual device inputs, network packets, external timing, fault events, nondeterministic entropy test stream, selected scheduling decisions, system-graph transactions, and component lifecycle events. Sensitive user content is excluded or encrypted by default.

Replay must reproduce the failure or report the first divergence with relevant trace IDs.

## Guest boundary

RAR Lab may power-cycle, hot-plug, constrain, observe approved debug channels, and deliver external events. It may not edit guest manifests, declare an update successful, repair guest state, bypass signatures, or simulate a permission decision. Those behaviors belong to RAR OS.

## Interface

Provide a versioned scenario format and CLI first. A graphical controller may follow. Required commands cover create, boot, connect, transform, inject, record, replay, snapshot-host-state, and collect-evidence.

## Acceptance scenarios

1. Tier 0 sensor pairs with Tier 2 and survives failed update.
2. Tier 1 robot rejects an unsafe agent request and recovers a crashed driver.
3. Tier 2 phone gains a display and activates desktop presentation without reboot.
4. x86-64 and ARM64 devices hand off a portable component through state restart.
5. Recovery reconstructs damaged System Store while Data Vault hashes remain unchanged.

Physical hardware support remains a separate claim and test suite.
