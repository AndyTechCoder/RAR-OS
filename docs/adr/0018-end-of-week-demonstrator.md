# ADR 0018: End-of-Week Alpha Demonstrator

Status: Accepted — 2026-08-25

Approval basis: explicit owner direction on 2026-08-25 to deliver a working,
bootable GUI Alpha by the end of the week while preserving the full vision.

## Context

Five sprint days were lost to infrastructure blocks and an unbounded blocked-
state retry loop. The existing plan remained technically useful but did not give
the owner an immediate visible product or a safe, interruption-resistant
execution contract.

## Decision drivers

- Produce authentic boot and GUI behavior immediately.
- Preserve the custom-OS, isolation, recovery, signing and data promises.
- Keep all target build and execution off the Mac.
- Protect unrelated irreplaceable SSD data.
- Make failure bounded, resumable and understandable.

## Considered options

### A. Continue the original fourteen-day sequence unchanged

This preserves sequencing but no longer reflects the remaining calendar time.

### B. Ship a graphical host mock

This is fast but is not a bootable RAR OS and cannot satisfy the owner.

### C. Build a minimum authentic vertical slice and retain later hardening

This demonstrates every central promise in narrow working form while retaining
the normal Releases 0–7 gates. This option is selected.

## Decision

Alpha 0.1 uses the completion contract in `../sprint-alpha.md` and targets
2026-08-30 23:59 America/Los_Angeles. Work remains one x86-64 cloud-only
vertical slice. Each required area must work minimally and truthfully; breadth,
production assurance, additional architectures, physical devices, networking,
Pal, and ecosystem scale remain later expansion work.

GitHub is the durable source of truth. The exact SSD RAR OS subtree may hold
temporary repository metadata, worktrees, scratch data and artifacts. No RAR
task may inspect or affect unrelated SSD content. Work is checkpointed to
GitHub before handoff, and failures use bounded retry rather than polling loops.

## Consequences

- GUI and native demos become part of the first visible Alpha acceptance run.
- Isolation, data preservation, recovery, signing, update and rollback are
  minimal demonstrations, not production-complete subsystems.
- ARM64, Tier 0, networking, browser RAR Lab, Pal and broad apps are stretch
  work until the mandatory x86-64 demonstration passes.
- The full Releases 0–7 roadmap remains unchanged.

## Security and data impact

Local target build and execution remain forbidden. The cloud Development Lab
retains its no-network, no-passthrough, bounded-resource rules. The SSD boundary
is strengthened: unrelated paths are outside authorization even for reads or
cleanup, and worktree removal requires verified GitHub durability.

## Compatibility and migration

All Alpha interfaces remain experimental. Later release gates may replace them
through documented migrations and conformance tests. No prototype becomes a
stable public or persistent contract merely by appearing in the demonstration.

## Validation

- One clean cloud run satisfies every item in the Alpha completion contract.
- Evidence proves RAR-owned boot and target behavior rather than a host mock.
- Negative demonstrations cover tampered layers, component crash and recovery.
- Static policy checks preserve the Mac and SSD boundaries.
- A failed external service or third identical error produces one checkpointed
  blocker and no automatic retry loop.

## Replacement path

After Alpha 0.1, normal Releases 0–7 replace experimental shortcuts with full
multi-architecture, security, migration, performance and hardware evidence.
