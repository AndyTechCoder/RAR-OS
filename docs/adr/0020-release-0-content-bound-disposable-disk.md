# ADR 0020: Release 0 Content-Bound Disposable Disk

Status: Accepted — 2026-07-18

## Context

A path-only disposable disk can change after certification or authorization and can preserve unintended state.

## Decision drivers

- Bind exact initial bytes and format.
- Prevent reuse or persistence across the one-shot launch.
- Make cleanup failure visible and fail closed.

## Considered options

- **Path-only disposable file:** rejected because content substitution is undetected.
- **Persistent reusable image:** rejected because replayed state crosses runs.
- **Content-addressed immutable seed/initial image plus supervisor-owned exclusive writable child:** selected.

## Decision

`disk-v2` binds the format, virtual size, immutable seed bytes, immutable initial-image bytes, descriptor slots, and creation policy. `preauth-transaction` creates and publishes only those immutable objects inside its transaction bundle; it cannot create a public writable child. Paths are not authority.

After review and owner authorization, `preauth-session` opens the immutable seed from the transaction bundle descriptor-relative with no-follow traversal and retains that descriptor. The execution supervisor alone may create exactly one private exclusive writable child from the held seed immediately before conditional consumption and spawn. The runtime-resource leaf binds the child descriptor identity and initial digest without adding it to or rewriting the transaction graph. Interruption or cleanup uncertainty quarantines the child; it is never reusable. No raw or physical disk is allowed.

## Consequences

Every authorized run has a distinct runtime-resource child identity and cleanup record, while deterministic transaction evidence remains immutable.

## Security and data impact

Disk bytes remain inside the private transaction/session namespaces; host disks, public writable children, aliases, symlinks, shared folders, and persistence are refused.

## Compatibility and migration

Disk v1 and its public child-path contract are rejected. Changing the disk format, immutable creation algorithm, or supervisor child policy requires a new schema and certification.

## Validation

Tests cover seed/initial substitution, same-inode mutation, symlink/TOCTOU, child singleton, duplicate use, interruption, quarantine, and cleanup failure.

## Replacement path

A future RAR Lab storage backend may replace the child format behind the same content, exclusivity, and cleanup contract.
