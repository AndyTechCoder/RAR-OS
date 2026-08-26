# Development Lab Image Provisioning

These three draft recipes describe role-separated Class B/C host images; none
is a RAR OS target image and no provisioning workflow is currently authorized.

1. `build.Containerfile` starts from an immutable official Rust base, installs
   digest-pinned Rust target components, and exposes only the reviewed
   compiler/linker paths to the untrusted build phase.
2. `launch-base.Containerfile` starts from a distinct immutable Debian base and
   installs exact QEMU/OVMF package versions from one immutable Debian snapshot.
3. `launch.Containerfile` compiles/tests the RAR QMP client twice with networking
   disabled, requires byte-identical binaries, and copies only that client into
   the launch base. Target source is never present in the final launch image.

`image-inputs-v1.env` is `decision-blocked`: researched upstream inputs are
immutable, but version 1 cannot express the separate reference-image topology
selected by accepted ADR 0020 and therefore cannot transition to ready or
authorize image output. The official amd64 Docker manifest digests were
resolved from Docker Registry v2; Rust component hashes came from the publisher
`.sha256` files; the OpenSSL checksum came from its official release; the
libsodium archive was streamed independently from both publisher and GitHub
release URLs and produced the same SHA-256; and QEMU/OVMF versions were selected
from Debian's `20260803T000000Z` bookworm amd64 snapshot index. The fixed source
epoch is that snapshot instant (`2026-08-03T00:00:00Z`).

A new reviewed input/output schema must implement the accepted topology.
Candidate provisioning remains deliberately absent until that schema and its
real identities pass independent review. The future provisioner must use a pinned isolated builder, a bounded fresh
context, two independent OCI exports, byte/digest comparison, complete
inventories and licenses, retained evidence, and no publication authority. It
must not expose reference oracles to the untrusted build role or run on the Mac.

Primary provenance endpoints:

- <https://registry-1.docker.io/v2/library/rust/manifests/1.95.0-bookworm>
- <https://registry-1.docker.io/v2/library/debian/manifests/bookworm-20260803-slim>
- <https://registry-1.docker.io/v2/moby/buildkit/manifests/v0.26.2>
- <https://static.rust-lang.org/dist/>
- <https://www.openssl.org/source/old/3.0/openssl-3.0.13.tar.gz.sha256>
- <https://download.libsodium.org/libsodium/releases/>
- <https://snapshot.debian.org/archive/debian/20260803T000000Z/dists/bookworm/main/binary-amd64/Packages.xz>

The eventual pinned build and launch image digests must differ. Probe-time images
never access a package repository: both phases use `--network none`, immutable
digests, read-only roots, dropped capabilities, and bounded tmpfs mounts.
