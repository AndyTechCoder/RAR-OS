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

## Lifecycle events and advisory RTC changes

Run34313003802/job102343354846, controller94fbb6b7c75e0307e79b4e1f5a3cd885f963a2bb,
captured RESUME in the continue reply and two RTC_CHANGE events in running
replies. Their data fields were exactly offset:int and qom-path:str. The retained
artifact is10089064023, ZIP215060 bytes, SHA256
4ca17240b654edf561d937d1d1091df0443daa70de70cd3c765943c61b172b4a.
The complete EOF/drain proof reached the checker; full persistence acceptance
still failed and is not claimed.

QEMU's official QMP reference defines RTC_CHANGE as a guest RTC time change,
with signed offset seconds and an RTC object QOM path. It is rate-limited and
is not RESET, STOP, RESUME, disk mutation or a new firmware instance:
https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html#event-RTC_CHANGE

The correction requires exactly one canonical RESUME as the first retained
event, with no preflight receipt. Subsequent events may only be RTC_CHANGE,
within the existing total32-event limit. Each has exact event/timestamp/data
fields, a bounded canonical timestamp, exact signed64-bit offset and one
bounded canonical machine QOM identifier; all RTC events in a VM refer to the
same identifier. This is an object identifier, never a filesystem path or
permission to open anything. No offset value changes the acceptance verdict.

RTC receipts may be continue/running/post-reap observations after RESUME in the
stream. Post-reap receipt means buffered stream data was parsed after the cut,
not that the guest executed afterward. No events are discarded. Duplicate
RESUME, reset, stop, shutdown, suspend, watchdog, panic, block-error,
device-change and unknown events remain rejected.

QOM syntax validation alone is not device attestation. The trusted image,
unchanged fixed launch arguments and independently checked paused topology
remain mandatory. RTC notifications are not used as boot/reboot, persistence,
elapsed-time or device-authority evidence. Whole QEMU destruction, fresh
firmware, all command/backend checks and frozen disk/pixel comparisons remain
unchanged. A source-only event fixture is not a runtime pass.

Logs expose bounded exact uppercase QMP event names, sanitized property names,
data-field type names, event hashes and derived receipt phases. Arbitrary data
values and strings/paths remain only in the bounded retained artifact; they are
not copied into log messages. This replaces unhelpful OTHER labels without an
unbounded/raw-value log channel. A hash is diagnostic identity, not acceptance.

Pure cloud tests cover request-time receipts, buffered/post-reap event records,
partial EOF, unexpected replies/errors, missing EOF, cumulative limits,
one-pass cleanup after drain failure, no freeze after failure, receipt ordering
and redacted diagnostic values. No local Mac/SSD files are written, downloaded
or executed. Actual corrected full-checker behavior and all remaining M4 acceptance
requirements still need cloud evidence. Focused event mutations cover every
forbidden lifecycle class, missing/duplicate resume, preflight/invalid receipts,
unknown/extra fields, boolean/noninteger/overflow offsets and timestamps,
malformed/multiple QOM identifiers and event-count boundaries.
