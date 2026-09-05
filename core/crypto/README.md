# RAR Alpha cryptographic building blocks

Status: experimental hash, Ed25519 and AEAD building blocks, not activated.
No production audit, active encryption, signed loader or target integration is claimed.

The source is RAR-authored from the established FIPS 180-4 SHA-512 algorithm.
It allocates no memory, has no unsafe code or external crate dependency, and
exposes an incremental bounded-counter hash plus a one-shot helper. Counter
overflow is an explicit error with unchanged state. Rust core/compiler_builtins
remain the previously approved bootstrap inputs when this code is linked later.

Normative source: https://doi.org/10.6028/NIST.FIPS.180-4
The initial tests cover published empty/abc/long-message values, streaming and
padding boundaries, empty updates and overflow behavior. They do not substitute
for the required independent reference, fuzz, specialist or runtime gates.

Validation is cloud-only through the existing isolated Specifications controller.
tools/ci/check-alpha-crypto-primitives.sh runs focused host model tests and a
no_std library compilation in the existing executable /build cloud tmpfs, never an OS image.
It refuses non-CI/non-Linux invocation. Do not run it on the owner's Mac or SSD.

Replacement boundary: keep algorithm code independent of key custody, manifest
encoding, rollback policy and lifecycle authority. This SHA-512 function alone
does not authenticate a layer. It must not be used as a home-grown signature,
password hash, keyed hash or production cryptographic module.

## Ed25519 candidate verifier

An experimental public-input verifier now accompanies SHA-512. It is RAR-authored
from RFC8032 sections5.1.1-5.1.7, not copied reference implementation source.
It implements pure Ed25519 only, with messages bounded to4096bytes, canonical
scalar S<L, canonical point decoding, nonidentity prime-subgroup publisher key
and prime-subgroup R. This deliberately strict acceptance profile must be
reviewed and bound to the future manifest contract before activation.
It does not accept Ed25519ph/ctx or expose signing/private-key operations.

Field arithmetic uses five canonical51-bit limbs and u128 intermediates modulo
2^255-19. Three carry sweeps plus final conditional subtraction maintain canonical
representation; multiplication coefficients remain below2^110. Extended Edwards
addition follows the established complete formulas. Scalars are public; the
implementation intentionally does not claim constant-time secret-key operations.
Do not reuse it for signing, key derivation or secret scalar multiplication.

All five pure-Ed25519 RFC8032 section7.1 known-answer tests and initial invalid-input/
carry/inversion tests are included. This is NOT complete vector/reference/fuzz
closure. Two independently maintained host-reference comparisons, broader
malformed-point corpus, retained bounded fuzzing, resource measurements and
specialist review still gate any use in a signed loader. No key is trusted by
this primitive alone; publisher authorization belongs to a separate policy layer.

## SHA256 content identities

core/crypto/sha256.rs adapts the existing RAR-owned host SHA256 implementation
for checked, non-mutating counter-overflow errors and no_std use. The historical
host module remains unchanged. Official vectors, streaming/padding boundaries
and overflow tests accompany it. Modern candidate metadata and journal models
reuse it; no new OS image links it yet. All independent-reference and runtime
gates above remain applicable.

## ChaCha20-Poly1305 candidate

RAR-owned RFC8439 implementation, unactivated. seal/open accept 32-byte keys,
12-byte nonces, at most4096 data bytes and256 associated-data bytes. Full16-byte
tags are checked before any decryption; all errors leave input unchanged.
No raw stream-cipher or standalone Poly1305 public API is exposed.

Poly1305 uses five26-bit limbs, bounded u64 multiplication, three fixed carry
sweeps and branchless canonical reduction. ChaCha uses fixed rounds and public
indices. This source-level design is not proof of compiled constant-time
behavior. No secret-memory zeroization, key custody, production privacy or
side-channel certification is claimed. Normative source:
https://www.rfc-editor.org/rfc/rfc8439.html

Tests include the RFC quarter-round, block, key-generation, MAC and AEAD vectors,
all11 AppendixA.3 carry/reduction vectors, every single-byte corruption of the
AEAD fixture's ciphertext/tag/AAD/nonce/key, and bounded padding/length cases.
Two pinned independent reference comparisons, retained fuzzing, resource and
compiled-code review are still mandatory before activation.

Nonce uniqueness is a caller obligation across abandoned writes, crashes and
rollback. The future Data protocol must durably reserve counters before use,
refuse ambiguous/exhausted state, and never derive uniqueness from a rewound
filesystem generation. Whole-volume rollback cannot be prevented by metadata
on that same volume. No Data writer or recovery integration exists yet.
