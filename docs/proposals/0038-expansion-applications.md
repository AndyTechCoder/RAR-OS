# ADR0038: Independently signed applications and private documents

Status: Proposed source candidate — 2026-09-14. No runtime activation.
Owner directs safe M5 completion under ADR0032. The concrete boundary requires
independent review before activation; this proposal does not change M4 releases.

## Problem and options

The existing network SDK is a message binding, not a general installed app SDK.
Settings has a privileged fixed-role update path that is inappropriate for
untrusted apps. The storage service currently exposes only shared bounded files.

A. Separate app envelope/domain, existing PE/W^X loader mechanisms, narrow app
grants and a validated private namespace in unchanged Vault snapshots.
B. Reuse privileged Settings identity and treat an arbitrary filename as private.
C. Rewrite the entire filesystem and application runtime before any SDK example.

A is the implementation candidate. Reject B: neither a filename nor a signature
grants authority. C remains an expansion path but adds unnecessary migration risk
to this bounded Alpha. No external target implementation is introduced.

## Decision candidate

Use docs/interfaces/expansion-app-package-v0.md as the explicit experimental
contract. Source verification and document policy are allocation-free safe Rust.
The existing public lab signer gets only a distinct fixed app message domain.
No cryptographic primitive changes or production signing claims.

No existing persistent bytes change. Private record install is explicit, atomic
and quota-checked, not a boot-time migration. Shared records are retained.
Whole-OS downgrade to old M4 readers after install is unsupported and fail-closed;
component Settings rollback must keep new Storage. No file deletion, Data
autoformat, private record exposure or owner-input requirement is introduced.

## Before native activation

Review concrete Manager-only immutable package access, kernel-mediated private
construction/publish, actual stamped caller routing, grant revocation and
shared/private Storage separation. Preserve the fixed 368-byte private Modern
bootstrap for existing services; do not relabel it as the public app ABI.

The app SDK needs a separate language-neutral experimental bootstrap and bounded
messages; both Rust and C examples must compile independently and execute in
separate protected guest tasks. Source-only codec fixtures do not satisfy this.
Demonstrate private-document persistence, unavailable-space refusal, contained
app failure, unrelated Data/device denial, and update/recovery preservation.

## Consequences

The initial private document is only 64 bytes, one owning app, within the existing
four-record/128-byte value budget. This is visible Alpha functionality with clear
limits, not scalable app storage. Future storage and stable SDK expansion require
new contracts and migration evidence. Public lab keys remain insecure for real
secrets; isolated synthetic cloud tests only.

## Native SDK/examples continuation proposal (2026-09-14)

This extends the same source candidate, not the stable native application model
and not permission to launch new code. M5.2 explicitly requires separate Rust/C
bindings and a useful Notes app; the existing 256-byte experimental Boot and
128-byte RAPP frame remain unchanged. Existing Modern public bytes are untouched.

Proposed A: implement first-party adapters over the existing int80 Yield, Send,
Receive, Exit and Ticks calls; define bounded UI/document payloads within the
existing candidate operations; compile distinct Rust Notes and C Counter entries
only as cloud relocatable objects. Notes has UI+private-document rights; Counter
has only UI. Before runtime use, require focused review of the exact native
adapter, operation semantics and service/loader integration. This source tree
does not select a new constructor, link/execute an app, change the controller,
grant devices/network, or migrate Data.

Alternative B: embed example logic as more privileged built-in Modern roles.
Rejected: that would not prove the independent language-neutral app contract.
Alternative C: call platform/libc or another OS SDK. Rejected: unnecessary target
dependency and incompatibility with the from-scratch/native requirement.

Unsafe preconditions are limited to the fixed read-only app bootstrap and
kernel-owned int80 ABI. No new privileged instruction/device API is added.
Timeouts leave uncertain saves visible without automatic accepted-write retries.
Existing Data format, app identity, signing domain and recovery promises remain.
Document semantics and exact source tests are in expansion-app-native-v0.md.
This proposal must not be read as accepted native activation or M5 completion.

## C final-link source continuation proposal

Advance from object-only checks to a fixed Counter final link inside the same
network-disabled, image-pinned Specifications sandbox. The existing cc driver
uses its image-pinned linker with no startup objects/libraries, and a first-party
bounded converter emits the unchanged private PE layout. Existing kernel parser
conformance and two-build byte equality are required; neither executable runs.

Alternative: add a PE cross-toolchain or third-party converter. Not selected:
the fixed static image needs neither an added download nor a target dependency.
Alternative: treat object compilation as sufficient. Rejected because unresolved
helpers and final address/section layout would remain unverified.
This proposal introduces no device, VM, runtime, installed app or stable format
authority. Independent source review precedes cloud validation; actual native
activation still requires the complete boundary evidence above.

The final-link candidate additionally exercises the existing signed app envelope
on the real linked Counter, with fixed lab identity `rar.counter.v000`,
generation1/UI-only/64KiB stack. This is a test-package identity, not enrollment
or a durable anti-rollback floor. The existing RAR verifier must accept the exact
bytes and reject tamper, insufficient rights and a higher required generation.
No signer primitive, domain, public bytes or production key is changed.

## Native independent-app integration candidate

The next source candidate uses two exact signed immutable packages embedded as
kernel-only boot-image data. A Manager-only private control verifies both once
before publishing the app catalog, and re-verifies the selected package in the
launch trap. This is the same first-party app verifier/public lab key, not a new
publisher or production root. The package IDs are fixed Notes/Counter, generation1;
there is no app update/downgrade installer or writable package source in this
bounded generation1 composition.

This deliberately duplicates integrity enforcement in the kernel adapter instead
of mapping arbitrary package bytes or accepting caller-supplied executable metadata.
Alternative Manager-only verification with user-visible banks remains possible,
but would require a separate immutable-window/identity handoff. No parsing of
filesystem paths or untrusted service-selected physical addresses is introduced.
Policy remains fixed by the approved composition; untrusted apps receive neither
Manager nor this control syscall.

The shared private mapper accepts only internally constructed Modern/app handoffs.
Legacy callers retain sealed-stage checks; new app callers use verified boot-image
bytes and preserve the current supervisor-only staging permissions. Only Modern
Compositor bootstrap can receive the framebuffer. Apps use physical slots11/12,
separate SDK bootstrap, W^X image pages and guarded16/64KiB stacks. Publication
leaves a held native context; generic SEND cannot wake it. Manager starts Notes
only after authenticated Storage acknowledgement. Failure/revocation retires held
and running roots using existing whole-stride scrub.

An expected-incarnation SEND is atomic inside the trap. This closes the scheduling
race between a service querying an app binding and sending through a logical
handle after close/relaunch. No raw arbitrary destination is exposed.

Native service composition, F4/F5 launch, F6 close and the standalone Counter
fault/denial demonstration are gated by rar_applications. Existing released build
flags do not select them. Before activation require consolidated security/unsafe
review, source integration checks, independently reproducible app packages and
reviewed confined cloud build/launch evidence. This proposal alone activates none.


## Consolidated Alpha cloud candidate
The concrete expansion-alpha controller is a separate trusted-main mode and
fixed workflow. Existing released modes are not redirected. Both independent
applications are reproducibly linked in the existing pinned compiler image.
The only VM input allowance added is F4-F7 within the already closed paired
profile. No new device, ambient network, owner path or target dependency is added.
Exact first/fresh scenarios use new disposable cloud fixtures and retain full
independent evidence. Adoption/activation still requires final focused review
and passing source checks; source inclusion is not acceptance evidence.

The portable node engine is Foundation-only, bounded64-byte programs/64 steps,
eight registers/four outputs and no allocation or I/O. Its proposed boot demo
reads actual delivered timer state and executes sample+7. The4MiB Foundation
arena and256MiB firmware VM are explicitly distinct resource facts, not a claim
of MCU support. This source stage does not activate a node cloud profile.

Counter's provider-neutral local broker grants one counter increment on explicit
G input, rejects document access, then revokes. It uses a deterministic labeled
provider, not Pal inference or OS-wide agent authority. Future integrations must
not inherit ambient privileges from this test interface.
