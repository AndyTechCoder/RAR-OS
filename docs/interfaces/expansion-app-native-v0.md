# Experimental native app SDK and examples

Status: source/object candidate only. No installer, launcher or target execution.

The app v0 frame/bootstrap from expansion-app-abi-v0.md is unchanged. The
following payload semantics complete its first UI/document subset. These are
experimental Alpha contracts, not a stable cross-release ABI.

All request statuses are0. Paint uses one nonzero frame sequence throughout:
begin payload[0,count] with count1..6; line[1,row,printable ASCII bytes] with
row0..5 and at most48 text bytes; commit[2]. Compositor must authenticate the
actual app incarnation, enforce begin/line count, uniqueness and monotonically
committed frame sequence, and publish pixels atomically only on complete commit.
These helpers do not by themselves add a Compositor route.

Input comes only from the kernel-authenticated Compositor peer in the app Boot.
Payload is one byte: Backspace8, Enter13, Escape27 or printable ASCII32..126.
Compositor supplies a fresh monotonically increasing nonzero sequence. Apps
refuse duplicate/stale input and full-incarnation mismatches. No raw input grant.

ReadDocument request has no payload; WriteDocument has0..64 raw document bytes.
A successful read reply contains0..64 bytes; successful write has no payload.
Every error reply has no payload. Statuses:0OK,1Invalid,2Denied,3Unavailable,
4ReadOnly,5Indeterminate,6Busy,7Exhausted. Replies require matching operation,
sequence and the full expected Storage identity. Service dispatch still must
authenticate the actual kernel envelope and configured private grant. This
candidate does not add the native Storage routing or select mount_private.

Rust native adapter and C native.h use only int80 Yield0, Send1, Receive2, Exit5
and Ticks6. Handles come from the separate Boot, never an app-selected service
name. A complete152-byte kernel envelope is checked; identities are not narrowed
before bounds checking. Nonzero high bits cannot masquerade as another principal.
The adapter does one send; example backpressure retries ONLY explicit Full,
meaning the kernel did not enqueue it, at most256 attempts and100 ticks.
There is no accepted-write retry, automatic reinstall, deletion or remount.

Unsafe scope: fixed initialized read-only bootstrap copy and the existing
kernel-preserved x86-64 int80 register ABI. Examples include RAR-owned memory
intrinsics for compiler-generated copy/zero calls. Source/object compilation
does not prove final UEFI link closure or absence of unresolved helper symbols;
those require the later target link map and immutable-package inspection.

## Independent examples

apps/expansion/notes is a distinct no_std Rust entry, not a new role in the shared
Modern service executable. It edits one64-byte printable private document.
Enter saves, Escape explicitly loads, Backspace edits. It authenticates replies,
retains edits typed during a read/write, and reports uncertainty after100 ticks
without retrying a possibly committed write. Binary/nonprintable stored content
is refused without rewriting it. Capacity and sequence exhaustion are bounded.

apps/expansion/counter is a distinct freestanding C entry. It receives UI-only
rights, counts0..9999 and supports Plus/Space, Minus and0 reset. It cannot send to
Storage or choose another app's document. It stages canonical Paint messages.

Cloud Specifications runs only pure state/codec tests, then compiles both native
entries to relocatable objects using already pinned compilers. It does not link,
install or run these targets. Native image loader, Manager/Storage/Compositor
routes, independent PE packaging and actual GUI launch remain required.

## Service-side candidate integration

Store::process_app authenticates the actual full sender/incarnation against its
configured private grant before parsing a document operation. Malformed frames
and old/duplicate sequences are ignored. A fresh sequence is consumed before
I/O, so an uncertain write cannot be replayed. Success is emitted only after the
existing Vault durable acknowledgement. No request can install, choose another
owner/path, obtain shared records or add a device handle. Existing shared
Files/Terminal protocol remains unchanged.

Compositor keeps two separate app surfaces. Only the fixed kernel binding query
may set their current incarnations. Unknown/stale identities cannot alter
current staging. Current-app malformed/partial frames abort staging; a complete
valid commit publishes all rows at once. Revocation/reuse clears private app
pixels and retains the incarnation high-water mark. Released legacy surfaces
and window protocol are unchanged. No native loop calls the new methods yet.

## App close/relaunch transition

The app-aware mount validates the fixed verified owner and reserves capacity,
but gives no app a live grant and performs no installation/write. A trusted
kernel-binding transition must revoke the old grant first, then rebind only the
same owner/principal with a strictly newer full incarnation. Failed rebind does
not change grants, sequence/high-water state or Data. Successful rebind alone
resets the per-incarnation request sequence. Fresh boot resets volatile endpoint
state while preserving the persisted owner/document records.

A reply retains its original recipient principal/incarnation. Before SEND, the
runtime must check Store::app_reply_current and the current kernel binding;
revoked or old-incarnation queued replies are discarded, never sent through a
logical handle to a replacement app. The native outbox is still pending.

Rust and C now share the malformed-envelope policy: discard a dequeued malformed
frame without state changes, but terminate on kernel receive failure. The C
152-byte envelope parser is safe host-testable code in app.h, used by native.h;
host tests cover short/invalid returns, zero/full-width identities and malformed
frames. This does not substitute for native register/span/runtime evidence.

## Independent C link candidate

The fixed Counter now has a proposed cloud-only freestanding final link and
RAR-owned ELF-to-private-PE converter. The pinned cloud cc/ld link uses no startup
objects, target libc, libgcc or dynamic runtime. No target entry is executed.
The converter requires bounded static x86-64 ELF, defined symbols, one exact
entry, page-separated R/RX/RW segments, consistent allocated section bytes and
no dynamic/relocation/TLS section. Only inert metadata sections are discarded
by the link script. The existing kernel PE parser independently checks output.
Two independently linked ELF byte streams must match. Pure malformed fixtures
exercise rejection. No external packager or linked target dependency is added.

This is not a generic ELF loader, Rust final-link proof, signed app installation,
native app activation or actual GUI acceptance. Those remain required separately.
The immutable cloud image pins the linker through the already-used cc toolchain.
All output remains bounded ephemeral cloud scratch, never the Mac or SSD.
