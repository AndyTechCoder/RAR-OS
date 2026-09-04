# Foundation Milestone 1 evidence

Status: runtime and independent evidence review passed on 2026-09-04.
Final delivery is the reviewed merge of PR #149 and publication of the experimental
release below. This record does not claim GUI, applications or production readiness.

## Exact first successful proof

- Source: [42be166b38790f18cf5cb7d2d1b4632e06f87e33](https://github.com/AndyTechCoder/RAR-OS/tree/42be166b38790f18cf5cb7d2d1b4632e06f87e33).
- Trusted controller: `309667c6e0cc4a44a7a24b8883df3089c377fed1`.
- Cloud [run 33923002436](https://github.com/AndyTechCoder/RAR-OS/actions/runs/33923002436),
  attempt 1, job `foundation` / `101185266388`.
- Runner image: `20260831.293.1`, GitHub-hosted ubuntu-24.04 x86_64.
- [Recorded manifest](foundation/33923002436-manifest.json).
- Original Actions artifact: `9955811367`, `120948` bytes.
- Archive SHA-256: `412f4393d41e0aa2860906677cc05ecf5ccd062e672bb7b9f06ba1aef8ad84db`.
- Durable [experimental release](https://github.com/AndyTechCoder/RAR-OS/releases/tag/v0.1.0-foundation-alpha);
  asset `foundation-33923002436-evidence.zip`. Staged as a draft until the final gate.
  The release upload returned the same archive digest; no archive was rewritten.
- Immutable [dependency inventory](https://github.com/AndyTechCoder/RAR-OS/blob/42be166b38790f18cf5cb7d2d1b4632e06f87e33/nucleus/foundation/dependencies.json)
  and [unsafe invariants / implementation guide](https://github.com/AndyTechCoder/RAR-OS/blob/42be166b38790f18cf5cb7d2d1b4632e06f87e33/nucleus/foundation/README.md).
  These source records complement the binary/log bundle.

Two independent clean builds produced equal SHA-256 digests for all six outputs:

| Profile | EFI SHA-256 | FAT image SHA-256 |
| --- | --- | --- |
| normal | `d92e5c7cdaee5770f7cac634e21ed8f3443debcf789833884285c4eac126a7b7` | `cb37a6f3814d61f4059eaaead808ced719a98fde1e384143200c0ced09334cd0` |
| panic | `37bcf6bc44eb55d1630cadce20c0b706d08e45f25aa213f12f9bd947bc79862e` | `44403a44048d427b622f8cabf289f34a27a9473481c68855ac332db02f97fd10` |
| exception | `ab4021764bd704606388e4aa73ec34fb5b815a901cd4d4a38b54c871c4b79b77` | `3e26ab53002e6485280a6c515df08d6d0305cbb3a59bd9dce7d3b0729245dbd2` |

Each build passed all 11 model/image tests, including malformed UEFI table
headers, invalid/overlapping memory regions, forbidden mappings, noncanonical
unmap aliases, frame restrictions, allocator misuse/exhaustion and 4096 mixed
heap operations. The trusted controller passed 26 rejection tests; focused
launcher checks also protect immutable input, firmware and disposable scratch.

## Observed behavior

The normal VM emitted exactly once, in order:

```text
RAR-BOOT:UEFI
RAR-KERNEL:ENTRY
RAR-MEMORY:READY
RAR-ALLOCATOR:READY
RAR-INTERRUPTS:READY
RAR-TIMER:READY
RAR-FOUNDATION-READY
```

The memory marker follows a fresh physical frame mapped at a high virtual
address, a write/read check, unmap and release. Allocator readiness follows
aligned allocation, memory access, free and double-free rejection. Timer
readiness follows at least three real IRQ0 ticks; it is not a static banner.

The panic profile stopped after allocator readiness with
`RAR-PANIC:CODE=SELFTEST`. The exception profile executed UD2 after IDT setup,
reported `RAR-EXCEPTION:VECTOR=6` and `RAR-PANIC:CODE=EXCEPTION-06`.
Both emitted the ordered BEGIN/HALT panic records and never emitted ready.
Each guest remained halted until its 25-second timeout (expected exit 124).
Complete serial logs, their hashes and both build manifests are in the bundle.

Independent read-only review checked the integrated source, all six retained
binary hashes, all three serial hashes and transcripts, archive digest, both
11-test logs and tool identities. No blocking code finding remained at this
proof revision. Final documentation deltas require their normal focused review;
they do not erase the exact source identity above.

## Safety and limitations

All source edits were GitHub API operations. No local Mac/SSD file was created,
changed, moved or deleted; no RAR target was built or executed on an owner device.
Builds and boots used the [documented cloud profile](../../tools/rar-lab/foundation/README.md):
unprivileged bounded networkless containers; no credentials, raw disks, host
shares or passthrough. The generated disk remained an immutable input, with
guest writes confined to a fresh bounded disposable overlay. This is a project-
reviewed experimental profile, not an independent security certification.

The kernel has one CPU, supervisor-only mappings, a bounded bootstrap allocator,
fatal exceptions, PIC/PIT timing and serial output. No process isolation claim,
persistent user data, rollback demo, signed-layer activation, GUI, applications,
network stack, AI runtime or physical-device support is made. Future signed
updates and recovery/data-separation promises remain intact but unimplemented.

## Reproduce, debug and extend

Use the GitHub Foundation workflow on main, or an ordinary same-repository
Foundation implementation PR. Only the trusted main controller may provision
and execute the fixed build/boot commands. Do not invoke build.sh, launch.sh,
rustc, an image builder or QEMU on the owner's Mac or SSD.

Inspect the run manifest first, then the matching profile serial log and recorded
tool identities. A passing workflow without matching source/artifact/transcript
evidence is insufficient. Preserve failures: runs 33918813996 and 33919158210
exposed nondeterministic PDB identifiers; 33921010406 proved reproduction but
failed on read-only IDE before guest boot. The linker and bounded-overlay fixes
were independently reviewed; no target gate was relaxed.

Keep future boot, paging, allocator and interrupt changes in their existing
owned modules with tests and invariants. Controller authority changes require
focused independent review before they become trusted on main. Do not revive
the superseded packet/activation/retirement chains. The next milestone is
Platform; the graphical Usable Alpha remains a later distinct milestone.
