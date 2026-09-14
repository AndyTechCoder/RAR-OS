# Experimental native app SDK and examples

Status: source/link candidate only. No installer, launcher or target execution.

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

Cloud Specifications runs pure state/codec tests and compiles both native
entries to relocatable objects using already pinned compilers. The final-link
candidate below additionally links and lab-signs Counter, inspecting it only as
data. Notes final linking, native image loader/Manager/Storage/Compositor routes,
installation and actual GUI launch remain required. Neither target runs in
Specifications.

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

The same bounded ELF input can be wrapped in the existing app envelope with the
fixed experimental Counter identity `rar.counter.v000`, generation1, UI-only
rights and 64KiB stack. It uses only the already-public laboratory key; this
confers no production trust. The actual Rust verifier checks this real linked
payload plus identity, budget, signature, rollback and tamper refusals in cloud
CI. No package is installed or published as a release artifact by this check.

## Native activation sequencing constraint

A constructed app waiting for Storage binding/installation acknowledgement must
be explicitly unschedulable. The existing native SEND wakeup scans Blocked
processes with Active model state; merely leaving a published app Blocked would
not safely hold it. The activation adapter must distinguish a held app from a
receive-blocked running app and prevent generic wakeup until the intended start.
Do not retain a kernel AppHandover over IPC: build/publish in one trap, then use
a separate held-native-context state while Manager awaits the authenticated
Storage acknowledgement. Required service loss must retire held and running
apps alike before any service reconstruction resets volatile grant watermarks.
This is an implementation requirement, not a claim that the adapter exists.

## Private native control candidate

Only the rar_applications composition exposes syscall14 to existing trusted
services. It is not part of the untrusted app SDK. RDI is the caller's own
SelfReceive handle except Manager actions, which require Manager. RSI selects:
0 enable catalog/controls (Manager, zero remaining arguments);
1 fixed channel query (RDX destination, R10 zero);
2 app catalog/live query (RDX index0/1, R10 complete128-byte writable output);
3 construct/publish held app (Manager, index, R10 zero);
4 start held app (Manager, index, R10 full incarnation);
5 close exact app (Manager, index, R10 full incarnation);
6 fixed service incarnation query (role0/1/3/8/9, R10 zero);
7 conditional app SEND (index, R10 readable136-byte expected-incarnation+frame).

Control Record128: RARACT00 at0; index u32 at8; logical principal u32 at12;
incarnation u64 at16; package generation u64 at24; rights u32 at32; state u32 at36
(0 absent,1 held,2 running); app ID16 at40; owner32 at56; payload digest32 at88;
120..128 zero. Owner is redacted except for Storage/Manager. Complete spans,
caller identity, index, fixed destination grants and incarnation are checked.

Internal control frames use RARACM00, operation at8/index at9, incarnation u64
at16; all other bytes zero. Operations1 launch (incarnation0),2 close,3 focus,
4 Storage synchronize,5 Storage ready,6 Storage failure (nonzero incarnation).
Only authenticated Kernel-stamped Shell/Manager/Storage routes interpret them.
Input relay uses RARAKY00, index at8/key at9/incarnation at16 and zero padding.
Compositor supplies fresh public SDK input sequence numbers and uses atomic
conditional SEND. A replaced app cannot receive an old queued input or reply.

Storage mounts app-aware with the verified catalog owner, without autoformat or
installation. Only the authenticated Manager synchronize request can bind and
explicitly install the held Notes record. The normal Files/Terminal protocol and
private shared-data projection remain separate. Failed/uncertain install closes
the app; there is no automatic accepted-write retry. Existing packages/generation1
are fixed laboratory input, not a persistent general-purpose installer.

F1/F2/F3 preserve existing apps. F4 launches Notes; F5 launches Counter; F6 closes
the focused/pending independent app. Notes Escape remains explicit reload.
Counter displays successful denial of disk/port/network and absent Storage
authority, and Q deliberately executes UD2 in that app only for the contained
failure demonstration. Public synthetic laboratory data only.


## Compact/wide presentation and cloud evidence
F7 toggles a320x260 or548x260 native-app surface in the fixed640x480 framebuffer.
It does not relaunch the app, change its identity, discard its document or claim
a physical-phone port. RARPRF00 is a private Shell-to-Compositor frame: byte8 is
the canonical boolean and all remaining padding is zero. Existing app payloads
and public SDK bytes are unchanged.

The proposed expansion-alpha workflow builds Notes and Counter independently
twice, signs fixed public-lab envelopes, embeds exact immutable kernel banks and
builds each closed peer twice. A new cloud-only first-use journey exercises
private Notes saves/relaunch, both profile sizes, the C app, scoped deterministic
agent and contained C fault, then the established paired network and shared file.
A separate fresh-container journey receives only exact independently validated
frozen Data bytes and tests persistence without injecting the document again.
Full screenshots, exact QMP keyboard plans, actual wire captures, authenticated
frozen Data and reaped process/backend receipts are independently checked.
This candidate is not activated and does not establish M5 completion. Network
negative campaigns and integrated signed-update/recovery remain separate gates.
