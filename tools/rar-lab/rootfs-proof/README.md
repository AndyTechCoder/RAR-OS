# RAR Lab Effective Rootfs Proof Foundation

Status: experimental, non-activating host tool

This RAR-owned Rust library is the first implementation slice of the whole-rootfs
image proof required by `docs/security-remediation-status.md`. It parses bounded,
uncompressed POSIX ustar layer bytes, applies OCI whiteout and opaque-whiteout
semantics across ordered layers, and exposes every effective regular file or
directory. It identifies executable-mode files and ELF objects at any path,
including paths outside the role-specific `/opt` roots.

The implementation uses Rust `std` only, contains no `unsafe`, does not spawn a
process, access a filesystem, decompress data, build an image, or execute target
code. Unit tests compile and run only in the pinned, network-disabled cloud
validation container with read-only source and bounded tmpfs outputs.

## Accepted subset

- Uncompressed OCI layer media type semantics only.
- Strict `ustar\0` / version `00` headers and valid header checksums.
- Canonical relative UTF-8 paths of at most 4096 bytes.
- Regular files and directories only.
- At most 65,536 entries, 16 MiB of aggregate path text, and 1 GiB per layer;
  at most 262,144 effective entries and 64 MiB of effective path text.
- OCI `.wh.<name>` and `.wh..wh..opq` processing against lower layers before
  same-layer additions, independent of archive order.
- Rejection of links, devices, FIFOs, extensions, setuid/setgid modes,
  malformed numbers, truncation, duplicate paths/markers, and nonzero trailing
  data.

## Not yet a complete image proof

This slice does not parse an OCI image layout, select a manifest, verify
descriptor digests/sizes, decompress gzip/zstd layers, hash effective file
contents, or emit/validate the accepted inventory evidence format. Consequently
it does not resolve the full security finding, activate image inventory v2, make
a Lab profile ready, or authorize provisioning, target compilation, or guest
execution. Those capabilities require reviewed follow-up slices and any
owner-governed evidence-format decisions identified in the remediation status.

Semantics are pinned to OCI Image Specification v1.1.1:

- <https://github.com/opencontainers/image-spec/blob/v1.1.1/layer.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/image-layout.md>
- <https://github.com/opencontainers/image-spec/blob/v1.1.1/manifest.md>

Replacement requires equivalent malformed-input, traversal, whiteout-order,
  opaque-directory, special-file, and whole-path executable-discovery tests.
