# DataVault-v0 experimental laboratory candidate

Status: proposed private M4 schema and unactivated implementation. ADR0034
remains proposed; this does not authorize a device, VM profile or runtime.
It is not a production filesystem, stable Vault format or confidentiality claim.
No existing Desktop data is migrated. Unknown versions fail; never autoformat.

## Authority and assumptions

The Block trait grants no isolation. A future kernel/profile must give this
service only the distinct synthetic Data device, fixed 512-byte geometry and
exact capacity. System updater/recovery has no Data-write capability.
The descriptor is public laboratory material. Each independently writable image
requires a unique key; writable clones with the same key/prefix are forbidden
by the trusted controller. Image ID in authenticated data is not nonce uniqueness.
Flush followed by exact readback must mean durable virtual-device state across
the tested restart. This is not physical power-loss safety. Whole-image rollback
or exact erasure back to virgin bytes is not detectable without another authority.
Public keys provide neither secrecy nor hostile-forgery resistance. Production
key custody, clone safety and independent anti-rollback are later work.

All integer fields are little-endian; no implicit padding/trailing bytes.
There are exactly 2 + 3*N sectors, N from1 through64. Sectors0 and1 are identical
immutable header copies. Neither mount nor publish ever writes either header.
The fixture creator is separate; there is no target formatting entrypoint.

## Header: two identical512-byte sectors

Offsets: 0..8 ASCII RARVLT00; 8..10 version0; 10..12 length512;
12..16 flags1 (PUBLIC LAB ONLY); 16..20 sector size512; 20..24 N;
24..32 first slot sector2; 32..64 nonzero image ID;
64..96 nonzero public laboratory AEAD key; 96..100 four-byte nonce prefix;
100..480 all zero; 480..512 SHA256(bytes0..480).
Copies, exact capacity and every field/checksum must match or mounting fails
read-only/unavailable, with no repair or format. Header SHA256 identity used below
covers all512 bytes, including its checksum.

## Slots and nonce lifecycle

Physical slot i owns sectors 2+3*i (reservation), 3+3*i (authenticated payload),
and 4+3*i (commit). Virgin means ALL THREE sectors are entirely zero.
No consumed sector is ever rewritten or reclaimed. Nonce = prefix[4] followed
by i as an8-byte integer. Seal is invoked at most once per physical slot and key.

Reservation is exactly512 bytes of0xA5. First require all three sectors virgin,
write the reservation, flush, and require exact full readback BEFORE any seal.
A failed operation locks the instance read-only for the rest of the boot.

The authenticated payload is exactly512 bytes:
0..8 RARVPY00; 8..16 physical slot index; 16..24 positive revision;
24..56 SHA256 of previous committed payload sector, or all zero for revision1;
56..64 zero; 64..320 encrypted256-byte full snapshot; 320..336 AEAD tag;
336..512 zero.
AEAD is RFC8439 ChaCha20-Poly1305. Its112-byte associated data is the16 ASCII
bytes RAR-VAULT-AAD-V0, then32-byte header identity, then payload bytes0..64.
Revision must be precisely prior committed revision+1; physical slot, parent
digest and all bounds/framing must match. Revisions do not count burned slots.
The first previous digest is zero. Any overflow locks writes rather than wrapping.

Commit is exactly512 bytes of0xC3. Ciphertext, tag and authenticated chain metadata
are written together, flushed and read back exactly before this marker starts.
Then write/flush/read back the commit marker before updating RAM and acknowledging.
A publication-stage I/O error is Indeterminate: a new boot may find the update
committed despite the caller not receiving success. Other errors also lock writes.
Do not retry a failed publication in the same instance.

## Snapshot plaintext

Exactly256 bytes. 0..8 RARDAT00; byte8 count0..4; 9..16 zero.
Starting at16, each record is name_len:u8, data_len:u8, name bytes, data bytes.
Names are1..12 ASCII letters/digits/dot/underscore/hyphen, unique and sorted
lexicographically by bytes. This is a flat namespace with no path traversal.
Each data value is0..64 bytes; aggregate file data is at most128 bytes.
Unused bytes are zero. Parsing re-encodes and demands exact canonical equality,
rejecting duplicate/out-of-order names, alternate padding or malformed bounds.
Each commit replaces the complete bounded snapshot, not a replay command stream.

## Recovery classification

Mount reads both headers and EVERY slot, never seals and never writes.
The first wholly virgin slot begins the free suffix; any later nonvirgin slot
is a hole/corruption and fails read-only/unavailable.

A nonempty partial reservation containing only zero/A5 bytes is burned-before-seal
only when payload and commit are both all zero. It is permanently skipped.
Any other reservation encoding or conflicting peer bytes is corruption.
A full A5 reservation permanently consumes its slot:
- all-zero commit: incomplete/burned; payload may be torn, never reseal it;
- nonempty commit containing only zero/C3 bytes: accept only after the complete
  payload authenticates, decodes canonically and extends the exact prior chain;
- every other commit encoding or failed authentication: corruption, unavailable.
Partial commit markers can therefore recover the new complete state before an
ACK; they never accept partial ciphertext/tag, because ordered writes cannot start
the marker until authenticated payload bytes are durable.
Incomplete/burned slots may precede later valid commits, and remain consumed.
The next write uses the first wholly virgin suffix slot. Exhaustion serves the
last validated snapshot read-only. Ambiguous corruption never triggers formatting.

A later arbitrary transformation of an acknowledged commit to exact virgin bytes
cannot be distinguished from a never-started commit. This and whole-media rollback
are excluded from the detectable virtual crash model, not silently claimed solved.
Fixed marker partiality preserves the same authenticated state rather than
discarding a valid record because a variable-length tag publication was torn.

## Implementation and evidence status

services/modern/vault.rs is a safe no_std candidate over an abstract Block trait.
It owns bounded state, canonical snapshots, mount, ordered publication and sticky
write refusal. It has no native ports, device selector, kernel grants, formatting,
networking, production secrets or runtime entrypoint.

Cloud-only fake-block tests model volatile versus durable bytes, old/new outcomes
at every read/write/flush/readback, every prefix tear of all three sector writes,
sticky failure, no consumed-slot rewrite, header preservation, exhaustion,
malformed snapshots, corruption, holes and wrong-slot/chain records.
These are model tests, not guest persistence or controller-owned fault evidence.
Actual PIO/kernel integration, public-profile review, reference interoperability,
independent image oracle, fresh-VM persistence and full M4 fault proofs remain
mandatory before activation/merge/release. Replace transport, codec, crypto and
lifecycle independently; future schemas require explicit reject/export/migration,
never reinterpret old bytes or erase unknown data.
