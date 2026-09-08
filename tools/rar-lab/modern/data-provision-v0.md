# Empty Data image provisioning — Modern laboratory only

The bytes-only `data_provision.py` helper creates the exact 194-sector/64-slot
DataVault-v0 image expected by the proposed Modern hardware profile. It has no
file, device, subprocess, VM or formatting entrypoint. It cannot accept an
existing image, file contents, a challenge, or a recovery request.

The trusted disposable-session controller supplies a fresh 68-byte OS-random
sample for each independently writable image. One Provisioner instance spans
the entire session and all fault cases: it consumes every image identity and
key, rejects either reuse (even with another nonce prefix), and admits at most
4096 images. Zero identities/keys and wrong byte shapes fail. An allocation
failure ends the session; no retry or allocator reset may recover that identity.

These checks do not prove entropy quality or uniqueness across independent
sessions. The future reviewed controller must generate the randomness itself,
prohibit writable clones, own exact private image paths/inodes, create files
exclusively, and retain image/header hashes. Reboot reuses the same existing disk,
not this helper. Read-only frozen copies are oracle evidence, never writable
forks. The key is intentionally public in the header: no production secrecy.

The helper emits only two identical immutable canonical headers and zeroed
reservation/payload/commit sectors. It does not seal any record or encode a
snapshot. An unpredictable persistence challenge must be created separately,
checked absent from the initial image, and typed only into boot1. It cannot be
placed in initial Data by this API.

The isolated CLI self-test reads its exact trusted-checkout sibling oracle source;
it performs no image or provisioning-path I/O.
Focused cloud self-tests invoke the separate frozen-image oracle on two fresh
empty images, reject 11 invalid/reused/exhausted cases, and exercise the real
inclusive 4096-image session budget. Deterministic test entropy is test-only;
it must never supply independently writable runtime disks.

This is a provisioning source prerequisite, not an activated controller/profile,
a completed crypto interoperability gate, or fresh-VM persistence evidence.
No Mac/SSD writes, target execution or new storage authority are authorized.
