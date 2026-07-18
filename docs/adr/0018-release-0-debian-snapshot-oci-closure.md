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

The closure begins at `rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3`. The version-4 input lock binds only immutable external inputs and interpretation policy: Debian snapshot `20260630T000000Z`, signed metadata and key identity, `lld-19=1:19.1.7-3+b1`, `qemu-system-x86=1:10.0.8+ds-0+deb13u1+b2`, `ovmf=2025.02-8+deb13u1`, the complete no-recommends transitive set, every binary and source package identity, licenses, grammar versions, algorithms, path policy, and atomic-publication policy. It contains no source-dependent archive, index, manifest, config, layer, artifact, disk, graph, attestation, review, authorization, or session identity. Closure v2/v3 records are rejected rather than reinterpreted. `preauth-transaction` validates the complete signed closure and bounded archive plan first, snapshots exact package bytes once into private exclusively owned content-addressed staging, validates licenses and every tool/firmware byte there, and only then constructs an unpublished bundle. Absolute, dot, parent, ambiguous-platform, link escape, collision, special file, writable alias, mutation, signature failure, checksum failure, version drift, or closure drift fails closed before package use.

`preauth-input-bundle-v1` is the untrusted byte-delivery envelope for that already approved closure. `preauth-input-producer` may use network access only to fetch the exact digest-pinned OCI base and signed snapshot origins enumerated by `preauth-input-delivery-v1.policy`. It performs download-only APT acquisition, static metadata reads and extraction without installation or maintainer scripts, places every raw signed metadata, keyring, package, copyright, base-OCI, tool and firmware byte under a content-addressed name, and emits a deterministic canonical manifest and archive. It has no source build, graph, certification, attestation, review, owner, authority, session or launch capability. Its checks are diagnostic only: delivery does not grant trust, and `preauth-transaction` must independently validate the same delivered bytes against the input lock and signatures. The committed acquisition-policy digest identifies the delivery policy; each runtime bundle digest remains external evidence and is never committed into the input lock.

The acquired packages are statically extracted into a derived rootfs without running maintainer scripts, and every extracted timestamp is normalized to `SOURCE_DATE_EPOCH`. The derived image is built twice without cache and saved as canonical Docker/OCI-layout archives under the repository output tree. Each raw Docker export is fully bounded and path/type/content-digest validated before extraction. Its only permitted directory headers are the zero-length, root-owned `0755` structural parents `blobs/` and `blobs/sha256/`; they must contain the subsequently validated content-addressed files and never enter the canonical archive. The export is then projected to the explicit graph rooted at `manifest.json`, `index.json`, `oci-layout`, and `repositories`: the sole config, ordered layers, and the unique OCI image manifest reachable from those roots. The pinned Docker export's `manifest.json` must also contain exactly one `LayerSources` entry per config `rootfs.diff_ids` entry. Each key is that exact diff ID and each value has only `mediaType`, `digest`, and `size`. For this pinned Docker `--load` representation, `mediaType` is exactly `application/vnd.oci.image.layer.v1.tar`; the descriptor digest must equal its map key, the same-position Docker-save layer digest, and the config's same-position uncompressed rootfs diff ID, while its size and digest must match the exact byte-present layer payload. This makes `LayerSources` a redundant, fail-closed binding to the already rooted uncompressed layer rather than separate compatibility authority. Gzip, Docker-variant, vendor, wildcard, missing, duplicate, extra, or cross-linked descriptors are rejected; JSON object order carries no identity or authority. Content-addressed Docker-store metadata with no inbound graph edge is omitted from canonical generation and can never enter evidence; any dangling member in the resulting canonical archive is rejected. Each outer transport archive is canonicalized with sorted members, fixed epoch, numeric root ownership, fixed `0644` file mode, and fixed GNU tar framing. Docker's required legacy `repositories` index is accepted only as one root-owned `0644` regular file of at most 512 bytes whose byte-exact canonical JSON maps `rar-preauth:<checked-out-commit>` to the verified final layer; its tag must equal the sole manifest tag, and the manifest/config/image-ID chain remains independently verified.

For the pinned Buildx Docker driver with `--load`, `containerimage.digest` is explicitly classified as `docker-config-id`: it equals `containerimage.config.digest` and the loaded Docker image ID, and it is not represented or certified as an OCI distribution-manifest digest. The separately typed selected OCI manifest is the unique index-reachable `application/vnd.oci.image.manifest.v1+json` blob. Its exact bytes, digest, size, config descriptor, ordered uncompressed layer descriptors, media types and sizes must match the Docker-save manifest, config rootfs diff IDs and archive payloads. The byte-present canonical `index.json` must contain exactly one descriptor with that manifest's digest, size and media type. The pinned Docker/containerd export omits the optional descriptor `platform`; the verifier therefore requires its canonical absence and binds `linux/amd64` independently through the validated image config and loaded-image inspection. The descriptor annotations are exactly `io.containerd.image.name=docker.io/library/rar-preauth:<checked-out-commit>` and `org.opencontainers.image.ref.name=<checked-out-commit>`, in canonical order, with no other keys. The immutable `transaction-graph-v1`, never the committed input lock, hashes distinct typed nodes for the raw archive/index, canonical archive/index, selected OCI manifest, Buildx config identity, Docker config, loaded config, ordered layer-descriptor set, and ordered rootfs-diff-ID set; equality is required only where this pinned transformation proves it. Metadata strings, tags and RepoDigests alone never establish the mapping.

`docker save` is the raw index producer. Its bounded source representation is strictly parsed and must be semantically identical to the independently derived canonical index, but its producer key order is not certified. Before the projected archive is created, the verifier emits canonical index bytes derived from the already validated selected-manifest descriptor, checked-out revision, and exact annotations. Those bytes atomically replace the private extracted `index.json`; the canonical archive digest and `canonical_oci_index_sha256` bind only the replacement bytes. The raw source digest remains diagnostic evidence and is never labelled as, substituted for, or granted the authority of the canonical index. The projected archive must load to the same exact Docker config ID before it can enter certification evidence.

A bounded streaming verifier rejects malformed framing, unsafe or duplicate paths, unsupported types, truncation, oversized members/totals/counts, noncanonical or substituted repository bindings, unreachable or extra content-addressed blobs, index/manifest/config/layer substitution, media-type/platform/size mismatch, diff-ID mismatch, metadata-only spoofing, and loaded-image substitution. Both builds use the same checked-out-source tag, fixed epoch, disabled provenance, and exact typed mapping. The canonical archive digest covers the `repositories` bytes together with every other member. Evidence selects the expected revision from the GitHub event type, then requires it to equal `git rev-parse HEAD`; a pull-request merge ref is never a certification source revision.

## Consequences

The closure is larger but independently reproducible. QEMU and firmware bytes can be hashed and inspected without execution.

## Security and data impact

Downloaded bytes are untrusted until signed metadata and checksums validate. No package is installed on macOS and no acquired emulator or firmware executes.

## Compatibility and migration

Any package, snapshot, key, firmware, base image, grammar, algorithm, or policy change creates a new input lock. Source-dependent typed OCI identities belong to a new immutable transaction graph rather than a lock revision. Closure v2/v3 locks have no migration path.

## Validation

Two independent acquisitions must produce identical canonical manifests, byte-identical derived archives, and identical derived OCI digests. Negative tests cover stale metadata, substitutions, key mismatch, closure drift, symlinks, inode mutation, archive/digest divergence, push/PR head selection, malformed revisions, and checked-out-head mismatch.

## Replacement path

Another host closure may replace Debian only through an ADR with equivalent provenance, signatures, complete dependency inventory, licensing, and reproducibility.
