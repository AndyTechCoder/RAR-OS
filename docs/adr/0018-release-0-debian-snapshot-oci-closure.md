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

The closure begins at `rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. A version-2 lock binds Debian snapshot `20260630T000000Z`, signed metadata and key identity, `lld-19=1:19.1.7-3+b1`, `qemu-system-x86=1:10.0.8+ds-0+deb13u1+b2`, `ovmf=2025.02-8+deb13u1`, the complete no-recommends transitive set, every binary package name/version/architecture/filename/size/hash/license hash, every signed-snapshot source package name/version, the deterministic derived OCI digest, and extracted LLD/QEMU/OVMF byte hashes. This timestamp is after the approved QEMU and OVMF uploads and before QEMU 10.0.11 replaced 10.0.8 in proposed-updates. Acquisition is confined below `out/r0/preauth/acquisition`; absolute, dot, parent, ambiguous-platform, symlink, hardlink, ancestor escape, duplicate destination, special file, inode mutation, signature failure, checksum failure, version drift, or closure drift fails closed before package use.

The acquired packages are statically extracted into a derived rootfs without running maintainer scripts, and every extracted timestamp is normalized to `SOURCE_DATE_EPOCH`. The derived image is built twice without cache and saved as canonical Docker/OCI-layout archives under the repository output tree. Each raw Docker export is fully bounded and path/type/content-digest validated before extraction. Its only permitted directory headers are the zero-length, root-owned `0755` structural parents `blobs/` and `blobs/sha256/`; they must contain the subsequently validated content-addressed files and never enter the canonical archive. The export is then projected to the explicit graph rooted at `manifest.json`, `index.json`, `oci-layout`, and `repositories`: the sole config, ordered layers, and the unique OCI image manifest reachable from those roots. Content-addressed Docker-store metadata with no inbound graph edge is omitted from canonical generation and can never enter evidence; any dangling member in the resulting canonical archive is rejected. Each outer transport archive is canonicalized with sorted members, fixed epoch, numeric root ownership, fixed `0644` file mode, and fixed GNU tar framing. Docker's required legacy `repositories` index is accepted only as one root-owned `0644` regular file of at most 512 bytes whose byte-exact canonical JSON maps `rar-preauth:<checked-out-commit>` to the verified final layer; its tag must equal the sole manifest tag, and the manifest/config/image-ID chain remains independently verified.

For the pinned Buildx Docker driver with `--load`, `containerimage.digest` is explicitly classified as `docker-config-id`: it equals `containerimage.config.digest` and the loaded Docker image ID, and it is not represented or certified as an OCI distribution-manifest digest. The separately typed selected OCI manifest is the unique index-reachable `application/vnd.oci.image.manifest.v1+json` blob. Its exact bytes, digest, size, config descriptor, ordered uncompressed layer descriptors, media types and sizes must match the Docker-save manifest, config rootfs diff IDs and archive payloads. The index must bind that exact manifest, platform `linux/amd64`, media type and size. CI hashes distinct typed nodes for the Buildx config identity, Docker config, selected OCI manifest, ordered layer-descriptor set, ordered rootfs-diff-ID set, loaded image config and canonical archive; equality is required only where this pinned transformation proves it. Metadata strings, tags and RepoDigests alone never establish the mapping.

A bounded streaming verifier rejects malformed framing, unsafe or duplicate paths, unsupported types, truncation, oversized members/totals/counts, noncanonical or substituted repository bindings, unreachable or extra content-addressed blobs, index/manifest/config/layer substitution, media-type/platform/size mismatch, diff-ID mismatch, metadata-only spoofing, and loaded-image substitution. Both builds use the same checked-out-source tag, fixed epoch, disabled provenance, and exact typed mapping. The canonical archive digest covers the `repositories` bytes together with every other member. Evidence selects the expected revision from the GitHub event type, then requires it to equal `git rev-parse HEAD`; a pull-request merge ref is never a certification source revision.

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
