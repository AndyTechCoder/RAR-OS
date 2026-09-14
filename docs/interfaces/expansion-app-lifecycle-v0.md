# Experimental app lifecycle candidate

Status: source/model integration only, 2026-09-14. No native app activation.

The legacy ten-principal binding table and 368-byte Modern bootstrap remain
unchanged. Two separate app bindings use logical principals10/11 and physical
slots11/12. Settings still uses5/7; network remains logical7/physical10; idle15.
No additional native entry, syscall, device profile or mapping is enabled here.

A new explicit applications_bootstrap composition is only exercised by model
tests. After desktop publication, only Manager may atomically enable fixed
control channels: Shell→Manager, Manager→Shell/Storage, Storage→Manager/Notes,
Compositor→Manager/Notes/C-example. Apps receive no Shell, Manager, System,
device, framebuffer, raw input, network or agent handle. Notes gets UI+document;
the C example gets UI only. These are fixed Alpha limits, not arbitrary grants.

Manager-only preparation takes kernel-derived AppImage metadata. It must come
from the exact VerifiedApp immutable bank and a checked generation floor:
the mechanism record itself does NOT verify signatures. It prepares only local
caps and snapshots required peers. It does not publish a process or reserve a
physical root. Native integration must hold IF=0 across prepare, construction
and publish, with no IPC/disk wait. Failed construction must retire/scrub the
owned candidate before any scheduling. Native root construction is still pending.

The handover builds exact 256-byte SDK bootstrap bytes, validated by the shared
Rust decoder. Publication rechecks the global clock, required full peer
incarnations, vacancy and capability generations before any mutation.
Do not schedule an app before this publication succeeds. Service-control
queries authenticate the actual caller's self-receive grant; they return only
pre-granted fixed handles, not arbitrary capability minting. Owner namespace
digests are exposed only to Storage/Manager, redacted for Shell/Compositor.

App close requires the full current incarnation. Fault/close revokes every local
capability, clears the separate binding and purges old sender messages from all
queues. Reuse retains monotonic handle generations and fresh process incarnation.
App failure alone does not request OS recovery. Loss of Shell, Storage,
Compositor, Manager or System retires both apps and revokes additional control
channels; normal Settings replacement does not alter these app dependencies.
Kernel retirement/reconciliation must consume that model state natively before
this candidate can be activated.

Cloud source tests cover narrow channels, unchanged legacy bindings, withheld
publication, SDK bootstrap bytes, stale plans, exhaustion, contained app failure,
relaunch, late close/fault, full identity, queue budgets and all required peer
losses. These are not guest tests. Remaining: immutable app-bank validation,
private root builder, native app traps/Manager/Storage/GUI, independent executables
and actual persistence/fault/isolation tests under reviewed cloud profiles.
