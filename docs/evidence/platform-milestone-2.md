# Platform Milestone 2 — runtime evidence and release gate

Date: 2026-09-05 UTC. Status: required behavior demonstrated on reviewed source;
final-head checks, exact-main validation and release publication remain mandatory
promotion steps. This document is not a substitute for their results.

## Immutable provenance

- Reviewed implementation: `d058cc32a7be13d2970c6c1be33bcf600abc2838`, PR #153.
- Trusted controller: `dcb61acf2c10344c0e0883cfae13138239f8e88d`, merged PR #152.
- Platform run: [33935588426](https://github.com/AndyTechCoder/RAR-OS/actions/runs/33935588426).
- Foundation regression run: [33935588441](https://github.com/AndyTechCoder/RAR-OS/actions/runs/33935588441).
- [Raw Platform manifest](platform/33935588426-manifest.json).
- [Raw Foundation manifest](platform/33935588441-foundation-manifest.json).
- Foundation baseline remains tagged `v0.1.0-foundation-alpha` at
  `27ce297f4ed67117bb9176b98fd3817d095dd29a`; no history/tag/branch was removed.

## Demonstrated behavior

| Requirement | Actual evidence, not just initialization |
| --- | --- |
| User execution/isolation | Sixteen ring3 fixture processes, private writable images/stacks; kernel, peer-memory, NX, guard, text-write, I/O and CR3 faults contained |
| Invalid return isolation | RSP=0 INT80 client revoked without stopping kernel or healthy services |
| Preemption/context | Non-yielding peer preempted; both distinct live-sentinel timer phases observed; GPR/stack/condition flags/DF/XMM0–15/MXCSR/x87 checked |
| Syscall context | State checked after immediate Yield and kernel-observed blocked/woken Receive |
| Capability IPC | Rights/generations, kernel sender identity, stale/dead peers, invalid/overflow/cross-page buffers, message/queue bounds |
| Shared service availability | Per-sender request quota; full response queue and stale replies to exited/faulted clients; no storage retry-loop or service exit |
| Storage | Separate ring3 create/write/read/list, namespace isolation and quotas; healthy read/list of retained RAM state after all adverse cases |
| Input | Actual trusted QMP A make/break delivered through emulated PS/2 and decoded in ring3 |
| Display | Ring3 GOP drawing, exact 640x480 PPM capture, every pixel independently validated |
| Reproducibility | Two clean builds with identical kernel, service fixture and 16 MiB boot image |
| Regression | Foundation normal, deterministic panic and invalid-instruction profiles all passed; two clean matching builds |

Each Platform build passed 12 kernel/loader/display/context/queue model tests and
5 service model tests. The trusted controller passed 63 negative tests.
Foundation retained 11 model tests per build and 26 controller negative tests.

The final ordered record is `RAR-PLATFORM-READY`, after all earlier behavioral
records. Successful Platform termination is trusted QMP quit with exit zero;
timeouts never count as Platform success. Foundation's expected bounded halt
profiles retain their separately classified timeout-124 termination.

## Exact reviewed-source hashes

| Artifact | SHA-256 |
| --- | --- |
| platform.efi | 9fbb03bfec3192e25a50f91fb63b916cc57c87905eac2e34c5336bc02229f000 |
| platform-service.efi | 222085311ddaf596913f6a8722e502a4a49e6e204af7a4814fdc0a68c4d41dc6 |
| platform.img | a17d5be016eabea88f0db5eac72b159c0c5beccdc46089e582229d35d63cc5d1 |
| captured frame.ppm | b0fcaaf70dc00a65765312436037dee2409a3514321be14107769e5c8fa7ed27 |

The reviewed kernel file is 91,648 bytes; the 36 MiB runtime arena is a bounded
prototype allocation, not the eventual tiny-device footprint.

## Independent review and remediation

Independent architecture/correctness and security/service reviewers examined
the implementation, then verified one consolidated remediation at the exact
reviewed source above. All actionable findings were closed:

1. Invalid user return registers previously caused a kernel-wide panic. They now
   cause process-local death/revocation; corrupted kernel-owned frames remain fatal.
2. Shared storage previously retried full replies indefinitely or exited on stale
   clients. Replies now use one bounded attempt; dead/full clients cannot stop it.
3. Context evidence previously covered preemption but not explicit syscalls or
   both timer phases. The strengthened runtime proves immediate/blocked syscall
   preservation and distinct live-sentinel timer phases.

No unreviewed code may be folded into the release under this evidence. A
documentation-only closure head must retain the reviewed source bytes and pass
its own relevant checks. Any code change requires affected review and new proof.

## Durable proof and promotion

The release `v0.2.0-platform-alpha` must bind the exact merged revision and retain
reviewed-source, final-head and exact-main proof bundles. Original reviewed bundles:

- `platform-reviewed-proof-33935588426.zip`: 97,024 bytes,
  SHA-256 `a804d61b08ba322cc40115561c3259865378c3cdce4b889df8091205b5a76899`.
- `foundation-reviewed-proof-33935588441.zip`: 121,149 bytes,
  SHA-256 `72f7efe54ab1486e3b5143b0fe04074d28d0770d2cefe71581098e6895fcee94`.

These were streamed directly from Actions into an unpublished GitHub release
draft, with returned asset size/digest verified. Final publication follows
passing final-head checks, evidence-gated merge and exact-main validation.
Release assets supplement the expiring Actions artifacts; they do not change
the manifests' original source/controller identities. The release body records
the final gate/run identities without rewriting historical proof.

## Safety and remaining limits

All repository mutations and evidence publication used GitHub APIs. No local
Mac/SSD file creation, edits, deletion, builds, packages, mounts or target/VM
execution occurred. Review was read-only; no local security-scan artifacts were
created. Cloud containers and guest profiles remained pinned, bounded,
unprivileged and networkless, with no credentials, passthrough, host shares,
physical devices or owner data exposed.

This is a genuine protected Platform prototype, not the complete RAR OS vision.
It uses one qemu64/TCG CPU and fixed native fixtures, no restart/rebinding API,
no TLS/AVX, no general device discovery and no production security certification.
Dead-task private memory is not reused or zeroized in this milestone.
Storage is explicitly volatile; dropped replies have no exactly-once guarantee.
GUI/apps are Milestone 3. Persistence, signed updates/rollback/recovery and Data
Vault demonstrations remain Milestone 4; networking/SDK/AI/hardware expansion
remain Milestone 5. None is silently counted as implemented here.
