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

The closure begins at `rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. The version-3 lock binds Debian snapshot `20260630T000000Z`, signed metadata and key identity, `lld-19=1:19.1.7-3+b1`, `qemu-system-x86=1:10.0.8+ds-0+deb13u1+b2`, `ovmf=2025.02-8+deb13u1`, the complete no-recommends transitive set, every binary and source package identity, and licenses. It gives distinct mandatory types to the canonical OCI archive, byte-present OCI index, selected image manifest, Docker config/image ID, Buildx config ID, loaded config ID, ordered compressed-layer descriptors, and ordered rootfs diff IDs. Version-2 records are ambiguous and rejected rather than reinterpreted. Acquisition validates the complete signed closure and bounded archive plan first, copies exact package bytes once into private content-addressed staging, validates licenses and every tool/firmware byte there, and only then atomically imports derived output. Absolute, dot, parent, ambiguous-platform, link escape, collision, special file, mutation, signature failure, checksum failure, version drift, or closure drift fails closed before package use.

The acquired packages are statically extracted into a derived rootfs without running maintainer scripts, and every extracted timestamp is normalized to `SOURCE_DATE_EPOCH`. The derived image is built twice without cache and saved as canonical Docker/OCI-layout archives under the repository output tree. Each raw Docker export is fully bounded and path/type/content-digest validated before extraction. Its only permitted directory headers are the zero-length, root-owned `0755` structural parents `blobs/` and `blobs/sha256/`; they must contain the subsequently validated content-addressed files and never enter the canonical archive. The export is then projected to the explicit graph rooted at `manifest.json`, `index.json`, `oci-layout`, and `repositories`: the sole config, ordered layers, and the unique OCI image manifest reachable from those roots. The pinned Docker export's `manifest.json` must also contain exactly one `LayerSources` entry per config `rootfs.diff_ids` entry. Each key is that exact diff ID and each value is an exact `mediaType`, `digest`, and `size` descriptor for the corresponding gzip source blob. The verifier checks the descriptor type, bounds, uniqueness, identity separation and any byte-present raw payload, but treats the map only as compatibility metadata: referenced compressed Docker-store blobs are omitted from the canonical graph and receive no certification or authority. Content-addressed Docker-store metadata with no inbound graph edge is omitted from canonical generation and can never enter evidence; any dangling member in the resulting canonical archive is rejected. Each outer transport archive is canonicalized with sorted members, fixed epoch, numeric root ownership, fixed `0644` file mode, and fixed GNU tar framing. Docker's required legacy `repositories` index is accepted only as one root-owned `0644` regular file of at most 512 bytes whose byte-exact canonical JSON maps `rar-preauth:<checked-out-commit>` to the verified final layer; its tag must equal the sole manifest tag, and the manifest/config/image-ID chain remains independently verified.

For the pinned Buildx Docker driver with `--load`, `containerimage.digest` is explicitly classified as `docker-config-id`: it equals `containerimage.config.digest` and the loaded Docker image ID, and it is not represented or certified as an OCI distribution-manifest digest. The separately typed selected OCI manifest is the unique index-reachable `application/vnd.oci.image.manifest.v1+json` blob. Its exact bytes, digest, size, config descriptor, ordered uncompressed layer descriptors, media types and sizes must match the Docker-save manifest, config rootfs diff IDs and archive payloads. The byte-present canonical `index.json` must contain exactly one descriptor with that manifest's digest, size and media type. The pinned Docker/containerd export omits the optional descriptor `platform`; the verifier therefore requires its canonical absence and binds `linux/amd64` independently through the validated image config and loaded-image inspection. The descriptor annotations are exactly `io.containerd.image.name=docker.io/library/rar-preauth:<checked-out-commit>` and `org.opencontainers.image.ref.name=<checked-out-commit>`, in canonical order, with no other keys. CI hashes distinct typed nodes for the Buildx config identity, Docker config, canonical OCI index bytes, selected OCI manifest, ordered layer-descriptor set, ordered rootfs-diff-ID set, loaded image config and canonical archive; equality is required only where this pinned transformation proves it. Metadata strings, tags and RepoDigests alone never establish the mapping.

A bounded streaming verifier rejects malformed framing, unsafe or duplicate paths, unsupported types, truncation, oversized members/totals/counts, noncanonical or substituted repository bindings, unreachable or extra content-addressed blobs, index/manifest/config/layer substitution, media-type/platform/size mismatch, diff-ID mismatch, metadata-only spoofing, and loaded-image substitution. Both builds use the same checked-out-source tag, fixed epoch, disabled provenance, and exact typed mapping. The canonical archive digest covers the `repositories` bytes together with every other member. Evidence selects the expected revision from the GitHub event type, then requires it to equal `git rev-parse HEAD`; a pull-request merge ref is never a certification source revision.

## Consequences

The closure is larger but independently reproducible. QEMU and firmware bytes can be hashed and inspected without execution.

## Security and data impact

Downloaded bytes are untrusted until signed metadata and checksums validate. No package is installed on macOS and no acquired emulator or firmware executes.

## Compatibility and migration

Any package, snapshot, key, firmware, base image, or typed OCI identity change creates a new lock schema and certification. Version-2 locks have no migration path.

## Validation

Two independent acquisitions must produce identical canonical manifests, byte-identical derived archives, and identical derived OCI digests. Negative tests cover stale metadata, substitutions, key mismatch, closure drift, symlinks, inode mutation, archive/digest divergence, push/PR head selection, malformed revisions, and checked-out-head mismatch.

## Replacement path

Another host closure may replace Debian only through an ADR with equivalent provenance, signatures, complete dependency inventory, licensing, and reproducibility.
