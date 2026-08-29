# ADR 0019: Alpha Layer Signing Profile

Status: Accepted — 2026-08-26

Approval basis: the owner-approved Sprint Alpha contract requires a signed-layer
activation, tamper rejection, and rollback demonstration. This ADR selects the
smallest established profile needed to implement that approved behavior; it
does not establish a production RAR trust root or stable package format.

## Context

ADR 0007 permits RAR implementations of established cryptography but requires a
follow-up protocol decision. Milestone F cannot safely invent an algorithm, key
model, signed bytes, or validation order in implementation code.

## Decision drivers

- Use a publicly reviewed deterministic signature standard.
- Keep target-linked third-party code at zero.
- Make signed bytes canonical and bounded.
- Demonstrate rejection and rollback without claiming production security.
- Preserve replacement and algorithm agility.

## Considered options

- **Custom RAR signature mathematics:** rejected by ADR 0007.
- **ECDSA P-256:** interoperable, but deterministic nonce handling and encoding
  add Alpha surface area.
- **Ed25519 as specified by RFC 8032:** selected for deterministic signatures,
  fixed-size keys/signatures, official vectors, and broad independent reference
  availability.
- **Unsigned content hashes only:** rejected because hashes do not authenticate
  a publisher and cannot satisfy the signed-layer requirement.

## Decision

Experimental algorithm identifier `rar.alpha.ed25519.v0` means pure Ed25519
from RFC 8032, not Ed25519ph or Ed25519ctx. SHA-512 is used internally exactly
as required by Ed25519. Layer content uses SHA-256 content identities.

The signed message is the exact byte concatenation of:

1. ASCII `RAR-LAYER-ALPHA-V0`, followed by one zero byte; and
2. the 32-byte SHA-256 digest of the bounded canonical experimental layer
   manifest with both its `manifest_digest` and `signature` fields absent.

The Alpha manifest contains explicit version, algorithm identifier, 32-byte
public-key identifier, 32-byte manifest digest, 64-byte signature, rollback
generation, payload identities, resource bounds, and health-check identity.
The recorded manifest digest is the value computed by the exclusion rule above;
the verifier recomputes it before signature verification. Neither excluded
field is encoded as a zero placeholder in the signed preimage.
The public-key identifier is SHA-256 over the exact 32-byte Ed25519 public key.
All lengths and validation precedence are specified under `spec/alpha/` before
implementation.

Alpha uses one repository-fixture laboratory signing key and one embedded
laboratory verification key. The private fixture is public test data, is never
a production secret, is never accepted outside the Alpha lab profile, and is
visibly labeled. Unknown key identifiers, algorithms, noncanonical manifests,
digest mismatches, invalid signatures, and lower rollback generations fail
closed before payload mapping or execution.

## Consequences

- RAR must implement the selected standard from normative specifications.
- The Alpha can demonstrate authenticity semantics without third-party target
  code.
- A single test root has no production operational security or revocation
  value; it must be replaced before broader use.
- Production key hierarchy, threshold roots, revocation, transparency, key
  rotation, and additional algorithms remain later ADR work.

## Security and data impact

Secret-dependent paths require documented constant-time invariants and review.
Malformed inputs allocate no unbounded memory and grant no authority. Signature
success alone never bypasses capability, rollback, health, or isolation policy.

## Validation

- All applicable RFC 8032 Ed25519 vectors, including invalid cases.
- SHA-256 and SHA-512 official vectors and boundary cases.
- Identical results against at least two independently maintained, digest-pinned
  host-only reference implementations in the Development Lab.
- Parser, canonicalization, point/scalar rejection, signature, rollback, and
  state-machine fuzzing with retained seeds and bounded runs.
- Focused timing/constant-time analysis, unsafe-code review, cryptography
  specialist review, and negative tests for one-byte changes in every signed
  field and payload.
- No production-security claim until an independent audit.

## Compatibility and migration

Algorithm and key identifiers are explicit and versioned. A later accepted ADR
defines production roots, coexistence, migration, revocation, and downgrade
refusal. Alpha identifiers can be removed without silently reinterpreting old
bytes.

## Replacement path

Replace the laboratory root and experimental identifier through a later
accepted production-signing ADR, versioned verifier coexistence, explicit key
migration, and downgrade-refusal tests. Never reinterpret Alpha signatures as a
production identity.
