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
- **Content-addressed immutable seed plus exclusive disposable child:** selected.

## Decision

Certification binds the disk schema, format, virtual size, seed SHA-256, launch-child SHA-256, and approved repository-relative paths. The launcher opens each component descriptor-relative with no-follow traversal, verifies bytes and stable identity, creates one exclusive child, and binds its descriptor into the command. Interruption or cleanup uncertainty quarantines the child; it is never reusable. No raw or physical disk is allowed.

## Consequences

Every authorized run has a distinct bound child digest and cleanup record.

## Security and data impact

Disk bytes remain beneath `out/r0/vm/x86_64`; host disks, aliases, symlinks, shared folders, and persistence are refused.

## Compatibility and migration

Changing the disk format or creation algorithm requires a new schema and certification.

## Validation

Tests cover seed/child substitution, same-inode mutation, symlink/TOCTOU, duplicate use, interruption, quarantine, and cleanup failure.

## Replacement path

A future RAR Lab storage backend may replace the child format behind the same content, exclusivity, and cleanup contract.
