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
