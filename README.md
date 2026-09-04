# RAR OS

RAR OS is a from-scratch, privacy-first, adaptable operating system intended to scale from tiny ecosystem devices to personal computers, robots, vehicles, servers, and future hardware categories.

Fast-Track Alpha Milestone 1 implements an experimental x86_64 UEFI foundation:
RAR-owned boot code, private page tables, memory/frame allocation, a bounded heap,
serial diagnostics, fatal exception handling and a basic hardware timer.
Acceptance requires the recorded reproducible cloud builds and three isolated
boot profiles; see the Foundation evidence record for the exact verified state.

This is not a usable desktop or production OS. GUI, applications, networking,
persistent storage, signed-layer updates and additional hardware come later.
Never build or execute RAR OS on the owner's Mac or SSD. Only the documented
isolated cloud profile is authorized for this milestone.

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
