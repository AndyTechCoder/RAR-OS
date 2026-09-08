# Frozen DataVault oracle candidate

This host-only checker independently interprets the experimental DataVault-v0
contract. It accepts only immutable bytes: no filenames, device handles, guest
success records, expected file value, target codec or target crypto imports.
It does not format, repair, seal runtime records, write any disk, or start a
process. Its CLI supports only pure self-tests. Actual source binding and
reading of a frozen regular cloud image belong to the separately reviewed
Modern controller; this module alone cannot prove where input bytes originated.

It checks both complete descriptor copies, schema/flags/geometry/exact image
capacity, image/key nonzero constraints, reserved bytes and header checksum.
All slots are inspected: free-suffix holes, conflicting reservations, bad
commit markers, wrong physical indices, skipped logical revisions, forked
parent hashes, bad authenticated framing/tags and noncanonical snapshots fail
closed. Burned slots never count as logical commits. A partial commit marker
is accepted only when the entire record authenticates; every earlier commit
also has to validate. Exhaustion is explicit read-only state.

The small Python ChaCha20-Poly1305 reader is separately written from the RAR
target implementation against [RFC8439](https://www.rfc-editor.org/rfc/rfc8439.html).
It verifies authentication before decrypting and returning plaintext. This
bounded offline reader handles public laboratory keys ONLY; Python integer
arithmetic is not constant-time and is not suitable for production key custody.
It is not a substitute for the two pinned independent reference libraries
required by M4. Neither it nor a reference library is linked into the OS.

Pure checks include RFC block/AEAD vectors, tag/key/nonce/AAD/ciphertext rejection,
all reservation and commit prefix lengths, both-header semantic mutations,
canonical and malformed file snapshots, holes/conflicts/forks, consumed-slot
continuation, image truncation/size bounds and exhaustion. The fixtures are
in-memory only; their repeated fixed keys must never provision writable images.
Source tests and generated fixtures are not actual frozen-guest-disk evidence.

Before M4.1 acceptance, the trusted controller must bind this reviewed oracle,
the exact frozen image hash, actual VM destruction/fresh firmware, boot1-only
unpredictable input and boot2 GUI evidence. It must read the actual retained
synthetic disk, never reconstruct it from guest text or expected values.
Host-tool/container pinning and both independent crypto comparisons remain
prerequisites. No Modern VM activation or Mac/SSD operation is authorized here.
