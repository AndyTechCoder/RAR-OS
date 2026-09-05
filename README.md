# RAR OS

RAR OS is a from-scratch, privacy-first, adaptable operating system intended to scale from tiny ecosystem devices to personal computers, robots, vehicles, servers, and future hardware categories.

Gate 0 was approved on 2026-07-16. That historical approval remains recorded;
ADR 0032 now governs the Fast-Track Alpha milestones.

RAR OS now has a real experimental x86_64 UEFI Platform: isolated user-mode
processes, timer-preemptive scheduling, capability IPC, volatile storage,
keyboard input and framebuffer drawing. Reproducible cloud builds, adversarial
fault/client tests and actual captured pixels are recorded in the Platform
evidence. Foundation normal/panic/exception boots remain regression-tested.
The release record binds final merge and exact-main validation.

This is not a usable desktop or production OS. GUI, applications, networking,
persistent storage, signed-layer updates and additional hardware come later.
Never build or execute RAR OS on the owner's Mac or SSD. Only the documented
isolated cloud profile is authorized for this milestone.

- [Platform evidence](docs/evidence/platform-milestone-2.md)
- [Platform implementation](nucleus/platform/README.md)
- [Platform private fixture contract](docs/interfaces/platform-runtime-v0.md)
- [Platform milestone](docs/tasks/fast-track-alpha-milestone-2.md)
- [Platform cloud lab](tools/rar-lab/platform/README.md)

- [Foundation evidence](docs/evidence/foundation-milestone-1.md)
- [Foundation implementation](nucleus/foundation/README.md)
- [Foundation milestone](docs/tasks/fast-track-alpha-milestone-1.md)
- [Foundation cloud lab](tools/rar-lab/foundation/README.md)

- [Specification index](docs/README.md)
- [Backlog](BACKLOG.md)
- [Constitution](docs/constitution.md)
- [Release roadmap](docs/release-roadmap.md)
- [Release 0 implementation tasks](docs/tasks/release-0.md)
- [Owner approval record](docs/approval-record.md)
- [Initial publication record](docs/publication-record.md)
- [Host Mac safety policy](docs/host-safety.md)
- [V1 alpha execution runbook](docs/v1-alpha-execution.md)

The canonical public repository is `AndyTechCoder/RAR-OS`.
