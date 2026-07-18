# ADR 0018: Release 0 Debian Snapshot and OCI Closure

Status: Accepted — 2026-07-18

## Context

R0-001 pinned the compiler root but intentionally left external LLD, QEMU, and firmware unavailable.

## Decision drivers

- Bind every output- or launch-affecting byte.
- Verify Debian signatures and package checksums before accepting downloads.
- Keep acquisition and derived images out of macOS installation state.

## Considered options

- **Use locally discovered Homebrew tools:** rejected as unpinned host mutation.
- **Trust package URLs or TLS alone:** rejected because transport is not package authenticity.
- **Pinned OCI base plus signed Debian snapshot closure:** selected.

## Decision

The closure begins at `rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. A version-2 lock binds Debian snapshot `20260630T000000Z`, signed metadata and key identity, `lld-19=1:19.1.7-3+b1`, `qemu-system-x86=1:10.0.8+ds-0+deb13u1+b2`, `ovmf=2025.02-8+deb13u1`, the complete no-recommends transitive set, each package SHA-256/license, the deterministic derived OCI digest, and extracted LLD/QEMU/OVMF byte hashes. This timestamp is after the approved QEMU and OVMF uploads and before QEMU 10.0.11 replaced 10.0.8 in proposed-updates. Acquisition is confined below `out/r0/preauth/acquisition`; symlinks, path escape, mutation, signature failure, checksum failure, version drift, or closure drift fail closed.

The derived image is exported twice without cache as Docker archives under the repository output tree. Both builds use the same checked-out-source tag, fixed `SOURCE_DATE_EPOCH`, disabled provenance, and BuildKit timestamp rewriting. The archive bytes and exporter-reported image digest must match before either archive is loaded for host-build use. Evidence selects the expected revision from the GitHub event type, then requires it to equal `git rev-parse HEAD`; a pull-request merge ref is never a certification source revision.

## Consequences

The closure is larger but independently reproducible. QEMU and firmware bytes can be hashed and inspected without execution.

## Security and data impact

Downloaded bytes are untrusted until signed metadata and checksums validate. No package is installed on macOS and no acquired emulator or firmware executes.

## Compatibility and migration

Any package, snapshot, key, firmware, or base-image change creates a new version-2 lock and certification.

## Validation

Two independent acquisitions must produce identical canonical manifests, byte-identical derived archives, and identical derived OCI digests. Negative tests cover stale metadata, substitutions, key mismatch, closure drift, symlinks, inode mutation, archive/digest divergence, push/PR head selection, malformed revisions, and checked-out-head mismatch.

## Replacement path

Another host closure may replace Debian only through an ADR with equivalent provenance, signatures, complete dependency inventory, licensing, and reproducibility.
