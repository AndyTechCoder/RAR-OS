# Public laboratory manifest signer

`lab_signer.py` is RAR-owned host-only fixture tooling for the existing Modern-v0
public RFC8032 TEST1 root. It neither changes the accepted laboratory key nor
adds any target dependency or target signing API.

The only public signing entry accepts one immutable nonzero 32-byte manifest
digest. It supplies the exact existing 19-byte domain internally and returns
64 PureEd25519 signature bytes. The internal empty-message path exists only
for the published RFC known-answer test. There is no caller-selected seed,
secret input, key generation, filesystem/network access, signing CLI, package
publication or runtime launch. Python integer arithmetic is variable-time and
must never be repurposed for real private keys.

The mathematical signing procedure and public known-answer data follow
[RFC8032 sections5.1 and7.1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1).
Point arithmetic was written for this helper using the same public extended
Edwards formulas as the RAR verifier; no upstream signing implementation was
copied or linked. Python standard-library SHA512 is host tooling only and not
part of the RAR target or an independent acceptance oracle.

The helper signs a supplied digest, not a validated package. The kernel still
must independently authenticate exact manifest/payload bytes, enforce policy,
seal executable memory and apply reduced trial authority. Anyone can produce
these public-laboratory signatures: they do not authenticate production RAR
releases or protect against a malicious publisher.

The cloud Specifications hook runs the RFC empty-message exact signature,
base/order identity, determinism, canonical scalar and bounded input refusal
tests. No helper tests execute on the Mac/SSD. Actual signed-layer generation,
RAR/OpenSSL/libsodium verification, real Settings loading and update/failure
evidence remain pending. This helper alone closes no M4 acceptance criterion.
