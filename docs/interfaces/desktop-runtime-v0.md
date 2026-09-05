# Desktop-v0 experimental runtime and UI contract

Status: proposed for review alongside Milestone3/ADR0033; not a stable SDK,
RID/RCI/RBC contract, persistent format or public app loading convention.
All principals are fixed ephemeral boot-profile roles, not durable process IDs.

## Mechanisms and authority

Reuse Platform-v0 protected address spaces,208-byte trap,512-byte FXSAVE context,
guarded64KiB user/kernel stacks,128KiB bounded PE fixture image and private
INT80 operations0..5/errors-1..-6. Boot magic is Desktop-specific; roles/cap slots
below are a distinct profile, never reinterpreted by the Platform build.
The inherited16 task slots reserve roles0..7; unused slots cannot gain authority.

Roles: shell0,storage1,input2,compositor3,Files4,Settings5,Terminal6,idle7.
Each has private writable memory. Only3 maps validated640x480 framebuffer.
Only2 receives fixedPS/2 read capability. Idle yields without device authority.

Boot capability slots:
0 self-receive;1 send-shell;2 send-compositor;3 send-storage;
4 send-Files;5 send-Settings;6 send-Terminal;7 input-read;8 framebuffer;
9/10 empty. Absent slots are zero and never meaningful authority.

Exact grants:
- shell:0,2,4,5,6
- storage:0,4,6
- input:1,7
- compositor:0
- Files:0,2,3
- Settings:0,1,2
- Terminal:0,2,3
- idle:no grants
Compositor also receives framebuffer metadata and slot8. Each receive object is
self only. Endpoint grants are send-only except receive slots. Generation1
belongs to this boot; fault/exit revokes its table and endpoint generation.
Shared service queues0/1/3 use the inherited per-sender limit2, total4.

Kernel report instrumentation: code1 allowed only input2,code2 only compositor3.
After both report readiness the kernel emits RAR-DESKTOP-READY once.
Code255 reports failure. A terminal6 UD fault is recorded once as
RAR-DESKTOP:APP-FAULT=6 and kills6; other unexpected faults are explicit failures.
Reports do not grant policy or synthesize UI state. Kernel faults remain fatal.

## Common message validation

All messages are exactly128bytes in the inherited144-byte sender/generation
envelope. Sender is kernel-stamped, generation must be1 for live boot peers.
Integers are little-endian. Every unused byte must be zero; unknown type/value,
length, role, version, duplicate line or reserved data is rejected without
authority/state change. UI messages do not carry caller-selected surface IDs.

All loops/resource use are bounded. A send may retry Full at most256 scheduler
yields, then expose a failure; stale is not retried. Shared storage attempts a
reply once and drops undeliverable Full/Stale replies as Platform does.
Apps must not lose or misinterpret queued key messages while awaiting storage
responses: serialize requests and use a bounded pending-input queue or equivalent
explicit flow control. Full pending input is reported, not written out of bounds.
No exactly-once IPC or reliable storage reply promise is introduced.

## Input, shell and session messages

0x01 key: byte1 key code, all remaining zero. PrintableASCII32..126, Enter13,
Backspace8, Escape27, F1=0x81,F2=0x82,F3=0x83,Up=0x84,Down=0x85.
Only sender2 may deliver input to shell; only sender0 may deliver it to apps.
Input implements bounded Set1 make/break and extended Up/Down decoding; release
does not insert text. Auxiliary/error bytes and unknown sequences are ignored
with state reset, not interpreted as keyboard commands. No host capture/IME.

0x02 activate: shell to one app, no fields. App refreshes its view; Files refreshes
its listing/readback while retaining a valid selection. No new process is loaded.
0x10 shell composition: byte1 count0..3,bytes2..4 distinct roles4..6 in
back-to-front order with unused zero,byte5 light0/1,byte6 terminal-stopped0/1.
Only sender0 accepted by compositor. Focus is last visible entry or none.
0x12 theme: byte1 light0/1,all other zero. Only sender5 accepted by shell.
Settings alone owns changing the theme; shell propagates accepted session state.

F1/F2/F3 show/raise the corresponding window and send activate. Escape hides the
focused window and focuses the last remaining visible one. Other keys route only
to focus. Hidden app process/state remains alive. On Stale sending to Terminal,
shell removes6 from visibility, labels Terminal stopped and refuses to pretend
it restarted. This state cannot come from an app payload, timer or test script.
Future Files/Settings key routing and rendering must continue.

## Bounded surface transactions

Only senders4/5/6 may submit their own six-line surface model to compositor.
Title/chrome/clipping/z-order are compositor-owned, not app-controlled.
0x20 begin: byte1 line-count0..6,versionu32 bytes4..8,nonzero and strictly newer
than that sender's last committed version; other bytes zero.
0x21 line: byte1 row0..5,byte2 length0..48,versionbytes4..8 matching an open
transaction,data bytes8..8+length printableASCII32..126,rest zero.
Each declared row must arrive exactly once. Invalid messages leave committed
content intact; a replacement begin may abandon only that sender's staging.
0x22 commit:versionbytes4..8; all declared rows required, remaining bytes zero.
Commit atomically swaps that sender's bounded model; only then redraw.
No version wrap: exhaustion requires explicit future lifecycle handling.
App text is drawn/clipped only in its window content area. It cannot spoof the
top bar, dock, other windows or stopped banner.

## Desktop storage adapter and apps

The Desktop storage endpoint is distinct from the unchanged Platform service.
It reuses the bounded128-byte create/write/read/list encoding and Store model.
It checks sender4/6,generation1,then maps both to one fixed demo-workspace Owner.
No message can choose a namespace or principal. Other callers have no endpoint.
Original Platform-v0 per-caller behavior and its release regression remain intact.
Four files/128total bytes/64bytes per file and transactional failed-write behavior
are retained. Seed welcome with RAR OS ALPHA in this synthetic namespace.

Files lists names, selects with Up/Down, reads the selected value and refreshes
on activation. Terminal input is bounded64ASCII bytes with Backspace and Enter;
commands: help,list,read NAME,write NAME TEXT,crash. Missing/invalid arguments,
quota/storage errors and unknown commands have readable error output.
write creates if absent and then replaces data; existing-file errors do not
destroy content. read/list require actual storage replies, never app-side fake data.
crash deliberately executes an invalid instruction in Terminal only.
Settings Space toggles session appearance. No persistent settings are claimed.

## Visual and evidence contract

640x480,readable provisional RAR-authored5x7 ASCII glyphs; case maps to uppercase
for drawing only, not storage byte interpretation. Unknown printable glyphs draw?.
The trusted cloud oracle defines exact dimensions/colors/scenes independently
of target implementation. Normal UI behavior is generic: it must not inspect a
scenario index or synthetic challenge. No host-generated page replaces guest UI.
Twelve captures prove opening/hiding/raising apps, theme changes, input editing,
a controller-generated eight-letter file value, cross-app readback and continued
Files/Settings interaction after actual terminal death.

## Replacement and limits

This profile preserves architecture boundaries but is not the stable app model.
A later RID/RCI transition supplies explicit adapters, coexistence/conformance
tests and retirement of fixed boot identities/images. All Desktop-v0 state is
volatile and discarded on reboot/replacement; never promoted into real user data.
No pointer/touch, dynamic app install, restart, persistent filesystem, accounts,
networking, signing/update/recovery or production security completion is implied.
