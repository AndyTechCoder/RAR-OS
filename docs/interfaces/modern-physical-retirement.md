# Modern deferred physical retirement

Status: implementation under review; no slot reuse, live replacement or M4
acceptance is claimed by source tests.

CPU scheduling state and private-memory state are separate. Logical death makes
a slot non-runnable and marks allocated memory Retiring, never Clean. Its own
trap cannot erase its current kernel stack: switching CR3 occurs before the
assembly return path switches RSP. Cleanup therefore runs only at the beginning
of a later validated ring3 trap in a surviving context, excluding current.

Each process owns the existing 2 MiB stride at arena + 4 MiB + slot * 2 MiB.
No arena size or historical profile changes. During construction a Modern-only
page-table helper reserves an empty leaf table for VA 16 MiB..18 MiB. This lies
above the allowed framebuffer span and below the minimum 32 MiB arena/kernel
image. No present/user mapping is installed by reservation.

At retirement the kernel verifies current CR3, IF=0, CR4.PGE=0 and PCIDE=0,
the victim's exact private root and stack geometry, logical vacancy/non-runnable
state, and the current owner's aligned preallocated leaf page. Unexpected
translation modes fail closed instead of assuming invalidation is sufficient.
Foundation already clears inherited PGE; the fixed qemu64 profile has no PCID.

The kernel clears the inactive victim PML4 first. Current/sibling address
spaces contain no user/executable aliases to that victim stride. Ordinary CR3
activation with global/tagged translations disabled already invalidated its
former user translations. Only then are supervisor RW/NX leaves installed in
the current aperture, invalidated, and used to clear all 2 MiB with volatile
writes. A volatile readback checks zero. All aperture leaves are removed and
invalidated before process metadata publishes Clean. The aperture page belongs
to the survivor, so clearing the victim cannot erase its own scrub mapping.

No disk, owner data, current stack, sibling private stride, global kernel image,
or firmware is cleared. The deletion prohibition for Mac/SSD/workspace files is
unchanged: these are exclusively owned volatile process pages inside a future
approved disposable VM.

Focused pure tests check bounds, disjointness, deferred-current exclusion,
retirement state and translation/interrupt conditions. Cloud host tests invoke
the actual page-table reservation in an owned aligned allocation without
privileged instructions. Existing cloud checks compile the actual kernel entry.
A cloud VM must still demonstrate the retirement path and surviving desktop;
later trial/replacement/reuse tests must prove stale accesses fail and no
candidate becomes runnable before complete construction. The serial marker
RAR-MODERN:PRIVATE-MEMORY-RETIRED follows physical completion, not logical death.

The process records both its exact reserved PT address and table allocation
count. Retirement walks the existing table path without allocation and checks
that it still resolves to that address, then repeats the empty-path check after
removal. Victim root clear and all removed PTEs receive volatile readback.
Before future begin_trial integration, every model-selectable vacant slot must
be physically Clean before staged state is consumed; checking only after model
allocation is too late. That allocation bridge does not exist in this change.
If no survivor reaches a trap, Retiring remains unavailable indefinitely; no
fallback path marks it Clean. Volatile VM-memory clearing is not a claim about
physical DRAM remanence or hypervisor memory sanitization.
