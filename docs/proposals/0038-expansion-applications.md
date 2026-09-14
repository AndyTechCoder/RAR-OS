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
