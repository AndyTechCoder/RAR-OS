# ADR0039: Concrete native Alpha cloud controller

Decision candidate, 2026-09-14, within delegated safe M5 completion under ADR0032.
This is implementation of a host controller, not an M5 acceptance declaration.

## Scope
A separate trusted-main expansion-alpha workflow accepts one exact source SHA.
It uses the existing pinned compiler/UEFI and QEMU/OVMF images and the existing
read-only, network-none, unprivileged, bounded disposable cloud containers.
No Mac/SSD execution, owner path, raw disk, external guest network or new target
dependency is authorized. Existing released workflows are not redirected.

The controller independently builds Rust Notes and C Counter twice, uses the
existing distinct app signature domain and public laboratory key, and embeds the
exact fixed packages into two reproducible closed-peer Alpha builds. It executes
one first-use and one fresh-container journey. The second receives byte-for-byte
copies of independently validated frozen synthetic Data, never the source
document as keyboard input. Complete retained pixels, planned QMP commands,
immutable hashes, actual PCAP, decrypted authenticated synthetic Vault contents,
full-incarnation lifecycle markers and actual guest/backend joins are required.
Corrupt evidence copies are rejected across all provenance and causal boundaries.

## Failure behavior
A failed scene emits a bounded differently shaped failure receipt containing
phase/reason, public synthetic serial tails and pixel-difference bounds. It
cannot pass the success schema. The outer controller saves the complete bounded
receipt before validation. Guest/backend teardown is attempted in every path;
cleanup failure emits failure, never a success-shaped record. Exact owned
container cleanup remains in the trusted outer finally block. No automatic
target retry or branch advance is introduced.

## Foundation-only node
The same approved Foundation serial machine is used in a new isolated cloud
container: q35/TCG/qemu64, one CPU,256MiB, no NIC, fixed read-only boot backing,
private firmware copy/snapshot, QEMU sandbox and25-second timeout. The timeout
wrapper must reap and report expected124; independent serial checks still require
the complete Foundation lifecycle and actual timer-derived bounded script result.
The executable is independently built twice and limited to256KiB. The4MiB kernel
arena and<=128-byte script state are distinct from firmware VM memory. This is an
x86-64 headless simulation, not an MCU port, real sensor driver or Tier0 release.

## Alternatives and gates
Reusing an old paired workflow with unreviewed source-selected launch flags is
rejected. Locally launching a VM or moving owner data is forbidden. A second
generic orchestrator is unnecessary. One concrete controller dependency PR is
permitted by the M5 task packet, followed by the existing implementation PR.
Before activation: exact-head source CI and independent unsafe/security boundary
review must have no blocking findings. Actual runtime evidence is required before
accepting functionality or merging the M5 implementation. Public lab signing and
tiny file/profile/agent limits remain explicit; no production trust is claimed.
