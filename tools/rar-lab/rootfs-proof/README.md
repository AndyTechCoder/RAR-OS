# RAR Lab Effective Rootfs Proof Foundation

Status: experimental, non-activating host tool

This RAR-owned Rust library implements the non-activating foundation of the
whole-rootfs image proof required by `docs/security-remediation-status.md`. It
parses bounded OCI layout, index, manifest, and configuration documents; binds
every selected descriptor to an exact SHA-256 digest and byte size; validates
the requested platform and ordered uncompressed `diff_ids`; parses bounded
POSIX ustar layer bytes; applies OCI whiteout and opaque-whiteout semantics; and
exposes every effective regular file or directory. It identifies executable-mode
files and ELF objects at any path, including paths outside role-specific `/opt`
roots, and binds every effective regular file to its SHA-256 content digest.

The implementation uses Rust `std` only, contains no `unsafe`, and does not
spawn a process, build an image, or execute target code. Its bounded in-process
gzip decoder cannot launch or activate an image. The
Linux cloud-only layout adapter confines filesystem reads beneath an opened,
non-symlink root directory handle; inspects candidates through non-activating
path-only handles before opening verified regular files; and rejects symlinks,
special files, out-of-root resolution, and size changes before content parsing.
Reads use one exact-sized allocation and never probe beyond the declared
ceiling. The sole public document resolver uses a bounded source
interface that supplies exact blob bytes by digest and receives the descriptor's
exact declared size as the ceiling before each access. A trusted future layout
consumer must enforce that ceiling before allocation or I/O.
Unit tests compile and run only in the pinned, network-disabled cloud validation
container with read-only source and bounded tmpfs outputs.

A RAR-owned decoder implements one bounded gzip member, including stored,
fixed-Huffman, and dynamic-Huffman DEFLATE blocks, header bounds, exact
end-of-stream handling, CRC-32, and uncompressed-size checks. The OCI resolver
accepts the standard gzip layer media type only after verifying the compressed
descriptor digest, and separately verifies the decoded bytes against the image
configuration's uncompressed `diff_id` before applying the layer. It is based
directly on [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951) and
[RFC 1952](https://www.rfc-editor.org/rfc/rfc1952), not third-party code.

A separate RAR-owned [RFC 8878](https://www.rfc-editor.org/rfc/rfc8878)
foundation parses one bounded Zstandard frame; decodes raw and RLE blocks; and
decodes compressed blocks whose literals are raw, RLE, or one or four RFC 8878
Huffman streams using direct or bounded FSE-compressed weights. It supports
direct and treeless literal sections, preserving the most recent Huffman table
across compressed blocks. It parses all sequence-count encodings and
predefined, RLE, FSE-compressed, or repeated
literal-length, offset, and match-length tables; decodes their interleaved
reverse bitstream; executes bounded literal and overlapping match copies; and
preserves sequence tables and repeat offsets across compressed blocks. It
verifies the standard lower-32-bit XXH64 frame content checksum using
dependency-free RAR-owned code, and rejects dictionaries, skippable frames,
concatenated frames, malformed checksums, and trailing bytes with explicit
errors.
It is not connected to OCI layer acceptance until complete compressed-block
support and its focused tests are ready.

## Accepted subset

- Uncompressed and single-member gzip OCI layer media types; all gzip blobs in
  one image are limited to 256 MiB total, and all decoded/uncompressed layer
  bytes in that image are limited to 1 GiB total.
- OCI layout version `1.0.0`, schema version 2 index/manifest documents, one
  caller-selected exact manifest digest, up to 8 nested index hops and 64 total
  index documents, standard image configuration, and up to 256 ordered layers.
- Lowercase SHA-256 descriptor grammar, exact descriptor sizes and blob hashes,
  requested configuration architecture/OS, and uncompressed `rootfs.diff_ids`.
- Strict duplicate-key-rejecting JSON with 1 MiB document, 32-level nesting,
  4096-item container, and 256 KiB decoded-string limits.
- Strict `ustar\0` / version `00` headers and valid header checksums.
- Canonical relative UTF-8 paths of at most 4096 bytes.
- Regular files and directories only.
- At most 65,536 entries, 16 MiB of aggregate path text, and 1 GiB per layer;
  at most 262,144 effective entries and 64 MiB of effective path text.
- OCI `.wh.<name>` and `.wh..wh..opq` processing against lower layers before
  same-layer additions, independent of archive order.
- Materialization of omitted parent directories, including safe lower-layer
  file-to-directory transitions required by child paths and whiteout markers.
- SHA-256 content digests for every effective regular file, with directories
  represented without a content digest.
- Rejection of links, devices, FIFOs, extensions, setuid/setgid modes,
  malformed numbers, truncation, duplicate paths/markers, and nonzero trailing
  data.

## Not yet a complete image proof

This slice does not yet accept zstd OCI layers or emit/validate the accepted
inventory evidence format. Consequently it does not
resolve the full security finding, activate image inventory v2, make a Lab
profile ready, or authorize provisioning, target compilation, or guest
execution. Those capabilities require reviewed follow-up slices and any
owner-governed evidence-format decisions identified in the remediation status.

Semantics are pinned to OCI Image Specification v1.1.1:

- <https://github.com/opencontainers/image-spec/blob/v1.1.1/layer.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/image-layout.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/manifest.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/descriptor.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/config.md>

Replacement requires equivalent malformed-input, traversal, whiteout-order,
  opaque-directory, special-file, and whole-path executable-discovery tests.
