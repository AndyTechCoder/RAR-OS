# RAR Alpha cryptographic building blocks

Status: experimental SHA-512 and Ed25519 building blocks, not activated.
No production audit, encryption, signed loader or target integration is claimed.

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
no_std library compilation in disposable cloud scratch, never an OS image.
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

The first three RFC8032 section7.1 known-answer tests and initial invalid-input/
carry/inversion tests are included. This is NOT complete vector/reference/fuzz
closure. Two independently maintained host-reference comparisons, broader
malformed-point corpus, retained bounded fuzzing, resource measurements and
specialist review still gate any use in a signed loader. No key is trusted by
this primitive alone; publisher authorization belongs to a separate policy layer.
