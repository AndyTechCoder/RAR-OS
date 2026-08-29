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
spawn a process, access a filesystem, decompress data, build an image, or execute
target code. A trusted future caller must supply exact blob bytes by digest.
Unit tests compile and run only in the pinned, network-disabled cloud validation
container with read-only source and bounded tmpfs outputs.

## Accepted subset

- Uncompressed OCI layer media type semantics only.
- OCI layout version `1.0.0`, schema version 2 index/manifest documents, one
  caller-selected exact manifest digest, standard image configuration, and up
  to 256 ordered layers.
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

This slice does not access a layout directory, recursively select nested image
indexes, decompress gzip/zstd layers, or emit/validate the accepted inventory
evidence format. Consequently it does not resolve the full security finding,
activate image inventory v2, make a Lab profile ready, or authorize
provisioning, target compilation, or guest execution. Those capabilities
require reviewed follow-up slices and any owner-governed evidence-format
decisions identified in the remediation status.

Semantics are pinned to OCI Image Specification v1.1.1:

- <https://github.com/opencontainers/image-spec/blob/v1.1.1/layer.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/image-layout.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/manifest.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/descriptor.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/config.md>

Replacement requires equivalent malformed-input, traversal, whiteout-order,
  opaque-directory, special-file, and whole-path executable-discovery tests.
