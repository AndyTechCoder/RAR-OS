# Modern persistent file-service adapter candidate

Unactivated M4.1 implementation in services/modern/store.rs. This is a private
service API, not a replacement for the historical Desktop/Platform wire ABI.
It does not grant device authority or claim cross-VM persistence.

## Existing operations, durable state

The adapter mounts DataVault; it never calls DesktopStore::new, seeds welcome,
formats, repairs, or writes during mount. The vault's verified snapshot is the
only file state. CREATE, WRITE, READ and LIST reuse existing 128-byte canonical
request validation, limits and success/error payloads. The Modern shared
namespace is four files, 12-byte names, 64-byte values and 128 aggregate bytes.
Names retain Desktop letters/digits/dot/hyphen, excluding dot and dot-dot.
A valid vault with unsupported names fails the service mount without rewriting
or omitting entries. LIST uses the vault's canonical lexical name order, not
Desktop insertion order. Released Desktop-v0 remains unchanged.

CREATE and WRITE remain two distinct durable operations: interruption between
them may leave a committed empty file. This compatibility behavior must be
explicit in the Modern UI and runtime tests. An identical WRITE of an already
verified committed value acknowledges that existing state without consuming
another append-only slot; it does not bypass an unavailable session.

For mutation, copy the snapshot, validate quotas, then publish through the
vault. Only a successful durable publication produces OK. Clean slot exhaustion
retains reads and refuses changed writes. Any I/O/authentication/publication
failure makes the session unavailable until a new independently mounted service;
there is no automatic retry/remount/format. Indeterminate publication is reported
distinctly because a reboot can recover a new value despite no success reply.

## Authority and failure delivery

Only Files role4 and Terminal role6 share this namespace. Their expected nonzero
incarnations are supplied by trusted Modern bootstrap/grants; received identity
is taken from the kernel envelope, never request bytes. On authority/incarnation
changes, recreate the adapter from renewed grants. Requests cannot choose a disk,
owner, physical address or raw device. Actual kernel isolation still has to
supply only the fixed Data Block implementation; a Rust trait is not confinement.

Failure::ReadOnly, Unavailable and Indeterminate are private typed results.
They are NOT silently mapped to historical QUOTA/INVALID/OK status bytes.
The future Modern transport/receiver validator and UI must be implemented
together, show save uncertainty distinctly, and handle a durable ACK lost in
delivery without retries or false SAVED messages. The current uncorrelated
Desktop polling loop must not be used as runtime acceptance of this adapter:
bounded request correlation/late-reply rejection remains required before wiring.

## Validation and remaining integration

Focused cloud source tests cover mount without writes, Terminal-style create/
write and Files-style read/list after a fresh model mount, quota/existence
failures without mutation, canonical messages, sender/incarnation rejection,
every publication I/O failure, sticky unavailable behavior, uncertain commit,
read-only exhaustion, identical-write slot preservation and unsupported/corrupt
mounts without formatting. These are model tests, not real VM evidence.

Next: integrate the Modern-only transport/UI failure and correlation contract,
kernel-mediated fixed Data authority, real PIO storage service loop and reviewed
cloud disk profile. Complete required crypto interoperability and causal
boot1-write/full-VM-destruction/boot2-read/frozen-disk-oracle proof. None of these
runtime gates is satisfied merely by this adapter or its source tests.

## Correlated transport candidate — not activated

Decision for the separate Modern composition: retain bounded 128-byte messages
but add explicit correlation rather than reuse Desktop's uncorrelated polling
or introduce a new durable transaction-ID database. This does not change the
historical Desktop/Platform ABI. services/modern/transport.rs implements the
candidate codec and client/server state; real runtime/UI and kernel epoch
enforcement remain mandatory before activation.

Frame bytes0..80 contain the existing operation body; 80..112 are zero;
112..120 contain a nonzero little-endian u64 request ID; 120..128 are the eight
ASCII bytes RARFIO01. These fields are domain/version separation and correlation,
not authentication. Sender and incarnation always come from the kernel envelope.
Before passing a request to Store, copy only0..80 into a zeroed128-byte body.
Malformed outer framing and unauthorized envelopes never advance sequence state.

Within one live service epoch, each authorized Files/Terminal incarnation has an
independent sequence beginning at1. Only the exact next ID is accepted; advance
before Store execution, even if a canonical outer frame has an invalid body.
Stale, duplicate and gap requests are dropped without I/O or re-execution.
Lost sends can therefore leave a gap: fail closed and renew sessions through
the kernel lifecycle, never replay the uncertain request. Across roles, receive
order supplies serialization, not an additional shared transaction-ID scheme.

Replies echo ID/magic. Status0..4 retain operation-specific body meanings.
Modern-only status5 is ReadOnly,6 Unavailable,7 SaveUncertain; error bodies are
otherwise all zero. Receivers validate full canonical bytes and operation/status
compatibility, bounded READ, sorted unique LIST, padding, storage identity/
incarnation and pending ID. Unknown/malformed/stale replies never complete a
different operation. Each client has at most one pending operation and IDs never
wrap or reset during its incarnation.

The client burns an ID before its sole send attempt. It never automatically
retries a failed/ambiguous send. Mutation timeout or SaveUncertain locks ALL
CREATE/WRITE operations for that incarnation, including apparent no-ops; READ/
LIST and late success replies cannot clear the lock. ReadOnly/Unavailable also
lock mutations. An expired read/list reports Unavailable without claiming a save.
The runtime/UI must show these distinctions and must not say SAVED on uncertainty.

Each begin fixes an absolute deadline of4096 monotonic kernel ticks and a total
budget of65536 received messages. Neither invalid traffic nor partial progress
extends these bounds. Runtime must invoke tick even when no messages arrive;
the real kernel tick source/unit, bounded receiver integration and user-visible
deadline still require concrete runtime/profile validation. These source limits
are not a measured latency guarantee and do not reuse the old256-poll app loop.

### Restart boundary (still requires kernel implementation/proof)

Sequences are volatile, NOT durable transaction IDs or cross-reboot exactly-once
execution. Never reset a server under the same live client grants, nor reset a
client's IDs under the same incarnation. On service failure, old endpoints and
queues must be revoked/drained, affected client sessions reincarnated, and all
new nonwrapping identities bound by the kernel before remount/recreation. Until
that is proved, the profile stays unactivated; use controlled recovery, not
automatic remount/reset. A constructor or caller label cannot prove this policy.

Source tests cover real adapter invocation with a model Block, durable reply
loss followed by duplicate rejection, per-role serialization, unsupported
envelopes/framing/gaps, burned invalid requests, late/wrong/malformed replies,
canonical error/LIST bodies, explicit failure states, absolute deadline and
stale-message budget, mutation lock retention, and ID/clock overflow. These
tests do not prove actual process-restart revocation, device isolation, real
durability or the GUI flow; those remain M4.1 integration gates.

## App session integration

`services/modern/session.rs` is the Modern-only ring3 polling policy around the
correlated transport. The real syscall adapter is still required before runtime
activation; source tests use a fake runtime and are not VM evidence.

The adapter supplies checked monotonic kernel ticks, a single nonblocking send
attempt using the fixed storage grant, one nonblocking receive using the app's
own grant, and CPU yield. It never retries queue-full. Any send error permanently
closes this session, preventing a burned request ID from causing later successor
gaps. No constructor/reset may renew IDs under unchanged live grants. Kernel
reincarnation and queue/grant revocation remain prerequisites for renewal.

Clock, receive or yield errors after a mutation was submitted report uncertain
save and close the session; failure before submission reports unavailable.
Backward time closes the session. Absolute deadlines are checked before and
after polling, with a second fixed 65,536-iteration ceiling even for an empty
mailbox and stalled clock. Deadline expiry after accepted send can leave reads
available, but never clears uncertain-write lockout. A ReadOnly or Unavailable
server response locks writes; a local read timeout alone does not imply a
permanent server read-only state.

Only full envelopes from the expected current shell are offered to the bounded
input queue. Every dequeued message spends the transport budget, including UI,
stale, short and malformed traffic. Input queue overflow sets a sticky input-loss
indicator instead of panicking the app or retrying storage. UI consumers must
still validate their own event shapes. Transport outcomes remain typed: the GUI
must explicitly render ReadOnly, Unavailable and SaveUncertain, and may say SAVED
only for a validated successful mutation reply. The session does not implement
the future syscall adapter, kernel clock, UI labels or service restart policy.

## Kernel incarnation width

Modern lifecycle endpoints use nonzero 64-bit incarnations. Store authorization,
transport clients and servers, and the app session's stamped envelopes preserve
that full width, including current shell/storage expectations from bootstrap.
No cast to Desktop-v0's 32-bit generation field is permitted. These are private
in-process interfaces; the 128-byte request/reply framing and on-disk formats
are unchanged. The eventual Modern syscall envelope must preserve all 64 bits.
Focused tests exercise incarnations above 2^32 and u64::MAX and reject otherwise
identical lower-32-bit identities; zero remains invalid bootstrap material.


## Initial syscall, service and UI composition

The source composition now connects core/modern/main.rs,
services/modern/runtime.rs and apps/modern/runtime.rs to the kernel candidate.
This supersedes the earlier statements that the syscall/UI adapters are absent;
it does not supersede any activation, crypto, epoch or causal-runtime gate.

Data uses its own fixed bootstrap device grant with verified IDENTIFY, then
Store::mount and Server. It never instantiates DesktopStore, creates a welcome
file, formats, repairs or automatically remounts. Exact bootstrap Files/Terminal
u64 incarnations are checked before request handling. A failed identify/mount
leaves a failure-only receiver that echoes canonical correlation with status6
Unavailable after sender authentication, never an OK or I/O attempt. No
operation is executed in that state, including on duplicate IDs. A mounted
Server retains the existing strict successor/once-per-epoch policy.

FileRuntime performs exactly one nonblocking storage send, checked monotonic
kernel ticks, one152-byte nonblocking own receive and yield. Session remains
the bounded deadline/work/correlation policy; its state is constructed once
per Files/Terminal process and never reset under live grants. The existing
bounded-retry send utility is only used for GUI messages, not file requests or
storage responses. Storage responses get one send attempt, so dropped ACKs
cannot cause a resend/re-execution shortcut.

Files and Terminal use typed outcomes, not invented historical error bytes.
apps/modern/model.rs supplies tested sticky error labels and the exact canonical
ACK rule used by the actual Terminal SAVED branch. ReadOnly, Unavailable,
SaveUncertain, malformed response or failed request never permits SAVED.
After uncertain mutation, later reads cannot clear the write lock or warning.
Pending shell input is shape checked and bounded; overflow is visibly reported.
Settings needs no file session or storage grant.

Files can display all64 value bytes across two rows, and Terminal READ uses a
second value row when necessary. The GUI labels the public laboratory data as
not private. Terminal introduction/help explicitly explains separate CREATE
and WRITE commits and that interruption may leave an empty file. Only a
validated durable WRITE ACK prints SAVED; CREATE alone does not.

The compositor checks full current peer incarnations without converting them
to Desktop's u32 identities. Its initial expected peers come from the immutable
Modern bootstrap. M4.2 must still implement authenticated peer/surface rebinding
for real Settings replacement; that behavior is not supplied by this initial
M4.1 composition. System only identifies once without writing and awaits its
future protocol; manager awaits future lifecycle messages. Neither is an update
or recovery implementation yet.

Pure GUI/status/correlation tests and Linux object-only compile checks cover
source integration. They are not an EFI target build or VM persistence proof.
The exact pinned UEFI image, stack/image resource checks, certified dual-PIO
profile, full VM destruction/fresh boot, independent frozen-Data oracle and
retained fault/regression evidence remain completion requirements.
