# Security Remediation Status

Status: Evidence-backed handoff, non-authoritative

This page prevents closed findings from being reworked and keeps owner-decision
items from stalling unrelated progress. It does not activate the Development
Lab, approve an ADR, authorize target execution, or claim that RAR OS exists.

The source scan was completed as
`5e2131da-6e16-49cd-9517-8073bd4a08a8` against
`e47a77c7f6077fa6177076ac25eeabf61efc04ca`. A finding is marked resolved only
when its reviewed PR merged and the exact resulting `main` commit passed the
Specifications workflow.

| Finding | State | Evidence or required next authority |
| --- | --- | --- |
| Host gate trusted `PATH` | Resolved | PR [#11](https://github.com/AndyTechCoder/RAR-OS/pull/11), merge `f2721ccaab408f2568b99a358d01488dea258c5a`, exact-main run `33272591571` |
| Git metadata could escape the SSD root | Resolved | PR [#11](https://github.com/AndyTechCoder/RAR-OS/pull/11), merge and run above |
| Proposal-controlled validators could self-attest | Reviewed, bootstrap-blocked | Draft PR [#13](https://github.com/AndyTechCoder/RAR-OS/pull/13) at `d222cf1e1989448002b8088bee3bd6088d92fe18`; requires explicit owner authorization for the one-time reviewed bootstrap merge, followed by an exact-main run |
| Specifications CI lacked resource limits | Resolved | PR [#12](https://github.com/AndyTechCoder/RAR-OS/pull/12), merge `2643309d078f3c8fcbbfd9ff292083dd974ff208`, exact-main run `33273672164` |
| Reference verdict is not bound to frozen-artifact execution | Owner decision required | Correct remediation changes the accepted ADR 0020 evidence boundary and public evidence format; no activating path may proceed meanwhile |
| Role inventory cannot prove whole-rootfs absence | Implementation in progress | The non-activating `tools/rar-lab/rootfs-proof/` foundation resolves bounded uncompressed ustar layers with OCI whiteout semantics and finds executable/ELF objects across `/`. OCI layout/manifest/digest resolution, compression, content hashing, evidence binding, and independent review remain required; keep v2 inactive |
| Controller identity is self-attested | Owner decision required | Correct remediation changes the Lab profile and retained-evidence trust contracts; all controller activation remains blocked |
| Release 0 fixture harness accepted escaping or unbounded reads | Resolved | PR [#14](https://github.com/AndyTechCoder/RAR-OS/pull/14), merge `f87dbb0b070a775931efc6f75decc9c0db0649b8`, exact-main run `33276136509` |
| Guest-authored markers could masquerade as acceptance proof | Owner decision required | Correct remediation changes the Alpha acceptance trust boundary and evidence format; the dormant launch path remains non-accepting |

## Additional hardening after the scan

- PR [#15](https://github.com/AndyTechCoder/RAR-OS/pull/15) binds each QMP
  operation to one cumulative deadline. Merge
  `75deeee20037aef8c4dd65305daf4cd535ccf970` passed exact-main run
  `33276908843`.
- PR [#16](https://github.com/AndyTechCoder/RAR-OS/pull/16) adds actual
  cloud-only execution of the 11 QMP Rust unit tests in a pinned,
  network-disabled, read-only-source container. Merge
  `a2df039274c905f2a62b67f0f4253994ac9a7a83` passed exact-main run
  `33277513266`.
- The effective-rootfs proof work is intentionally sliced without weakening the
  finding: an uncompressed layer resolver is useful implementation progress,
  but only a complete, digest-bound OCI image traversal may close the row.

## Continuation rule

Do not reopen resolved findings without new contradictory evidence. Skip an
owner-decision row while the owner is unavailable and continue only with work
that does not alter its public format or trust boundary. Do not merge PR #13
through an unreviewed dual trigger, force push, or synthetic check. Once the
required owner authority exists, merge it once, require the exact post-merge
`main` run, then proceed from that verified commit.

RAR OS target implementation remains 0 of 7 Alpha milestones. No host-tool,
schema, test, workflow, or green check is evidence of a bootable OS, GUI, app,
update, recovery, or signed-layer implementation.
