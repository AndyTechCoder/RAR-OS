# RAR Alpha cryptographic building blocks

Status: initial SHA-512 implementation; not a complete signing verifier.
No production audit, Ed25519, encryption or target integration is claimed.

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
