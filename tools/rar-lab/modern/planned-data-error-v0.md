# Planned Data I/O event correction

Failed cloud run 34352861329 (controller d6ceeb567ce8e3a7ab561378f2512788d695dcc4,
source 5b8555e86b54056efa2927e094b3f1746e66d668) retained case 0. The
reviewed read-only diagnostic run 34356806413 inspected artifact 10104503506
(digest cfdd3b4aed0a1e46cee41184e6f2cbe91ef04bc8d00aa4a63ca74ceb1339394e).
Its exact pre-kill fourth event was BLOCK_IO_ERROR, action report, empty
device alias, node-name rar-data, operation write, reason Input/output error.
The Data backend had the planned cut exit 20; peers and VM were still live.

The fault-only validator previously forbade all block error events despite
the profile deliberately using rerror=report,werror=report. Correct that
mismatch, not the OS or the persistent format. Accept zero or one matching
report only when the independent validator supplies a fixed mutation fault
plan. Zero remains valid when deliberate termination wins the event race.
Require the report's receipt during the final Return save command or the
pre-kill drain. Retain the exact pre-kill event-count boundary, one initial
RESUME and at most four chipset-bound RTC events. Reject other disks, reads,
stop/ignore actions, repeated reports, earlier-input receipts, post-kill
events and all unexpected lifecycle events. The fresh read-only recovery VM
and ordinary persistence validator remain unchanged and reject block errors.

The full evidence validator still checks the independent fault audit,
mutation prefix, complete byte replay, disk oracle, fresh-VM recovery and
unchanged System/Boot. The QMP report alone never establishes a valid fault
or successful recovery. Human-readable reason text is bounded but is not
used to decide error semantics.

QEMU 7.2 primary sources:
- https://github.com/qemu/qemu/blob/v7.2.0/qapi/block-core.json
  (BLOCK_IO_ERROR and BlockErrorAction).
- https://github.com/qemu/qemu/blob/v7.2.0/hw/ide/core.c
  (ide_flush_cb and ide_handle_rw_error: flush uses the write error path).

Tests cover write and flush cases, cut and error delivery, correct and
incorrect receipt boundaries, wrong devices/actions/operations, duplicates,
unexpected lifecycle events, mismatched fault delivery, and recovery-VM
refusal. No target code, workflow authority, disk format, or guest error
handling changes. An independently reviewed correction and exact-head
cloud checks are required before the next campaign attempt. M4 remains
incomplete until actual full acceptance, not merely these synthetic tests.
