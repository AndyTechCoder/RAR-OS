# Modern diagnostic event receipts v1

The first two-VM runtime attempts reached actual saved-data and five-frame
agreement but failed the retained event audit. Run34277989413/job102235650460
reported three first-VM events: RESUME and two names redacted as OTHER.
Artifact10076516600 (ZIP214751 bytes, SHA256
9734d07a37b87f9aff22ea004fef77f97dd014647c9a7534c211c8d7961535fd)
retains that attempt. No inference about those two events authorizes acceptance.

## Complete bounded stream capture

The diagnostic envelope is now rar-modern-persistence-candidate-v1. Its VM proof
adds event_receipts and qmp_drained; previous v0 artifacts remain unchanged and
require their exact historical checker. They are not silently reinterpreted.
This is disposable laboratory evidence, not an OS persistent-data or stable API
format change.

Each event has exactly one receipt with contiguous zero-based event_index and
request_id. A nonnull request_id identifies the command reply loop that parsed
the record, not when the event occurred. Null identifies parsing during the
post-reap drain. The checker derives preflight/continue/running receipt phases
from the independently validated fixed command sequence and requires ordered
request IDs followed only by a null suffix. Producer labels do not choose phases.

After the whole owned QEMU is killed and reaped, the controller drains complete
buffered and socket QMP records to EOF using a separate nonblocking path with an
absolute one-second deadline. Existing cumulative bounds remain: 2MiB received,
262144 pending/record bytes,32 events and2048 serialized bytes per event.
Unexpected replies/errors, malformed/blank records, partial EOF, timeout and
budget excess fail. qmp_drained becomes true only after reap, EOF and an empty
pending buffer; it is mandatory in the independent checker.

A cleanup attempt becomes non-reenterable immediately. All backend stops and
descriptor closures are still attempted after drain failure. cleanup_succeeded
is set only after the full pass succeeds, and frozen-image access requires that
literal true flag plus qmp_drained and actual terminated VM/backend processes.
A failed cleanup cannot be retried by the scenario finally block or mistaken
for successful frozen-image authority. Never-started cleanup grants no drain
proof.

## Diagnostics do not relax event acceptance

The exact sole canonical RESUME requirement remains. A RESUME first parsed by a
preflight reply is rejected. Extra events remain rejected pending actual names,
exact public QEMU schema/semantics and appropriate fault/mutation evidence.

Logs expose bounded exact uppercase QMP event names, sanitized property names,
data-field type names, event hashes and derived receipt phases. Arbitrary data
values and strings/paths remain only in the bounded retained artifact; they are
not copied into log messages. This replaces unhelpful OTHER labels without an
unbounded/raw-value log channel. A hash is diagnostic identity, not acceptance.

Pure cloud tests cover request-time receipts, buffered/post-reap event records,
partial EOF, unexpected replies/errors, missing EOF, cumulative limits,
one-pass cleanup after drain failure, no freeze after failure, receipt ordering
and redacted diagnostic values. No local Mac/SSD files are written, downloaded
or executed. Actual revised stream-capture behavior and all remaining M4
acceptance requirements still need cloud evidence.
