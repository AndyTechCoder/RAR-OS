# Modern reference role v0 — candidate, not provisioned or activated

This is host-only testing/packaging infrastructure under ADR0020, not RAR target
code or an OS dependency. Historical Alpha/F transcript contracts are unchanged.
No image digest, independent build, runtime inventory, comparison or reference
closure is claimed until the reviewed controller actually produces the evidence.

## Role and construction

The candidate Containerfile uses the existing pinned Rust1.95.0 provision image
only to compile pinned OpenSSL3.0.13 and libsodium1.0.19 with their existing
image-input source hashes. Downloads must be controller-owned fixed URLs,
bounded, hash-verified and archive-inventory checked before extraction. Docker
build runs network-disabled with a minimal context containing only those two
archives, this fixed adapter and Containerfile. No target source is present.

The final FROMscratch image contains only /reference-sodium, /reference-openssl and two upstream license
texts. Each adapter links only its own independent reference. Both are static; ELF INTERP/NEEDED are forbidden. OpenSSL's default provider is statically built in and explicitly selected;
external provider/configuration material is absent. OpenSSL is built
without DSO, modules, engines, automatic config or shared libraries; runtime
initialization also forbids config loading. No compiler, source, shell, fetcher,
package manager, QEMU, firmware, certificates or reference runtime config is
copied. The final image runs as65532:65532 with read-only root, no mounts/network/
credentials, dropped capabilities, no-new-privileges and controller-enforced
process, memory, wall-time and output limits. No caller supplies argv or paths.

Both implementations reside only in this oracle role and compute in separate
processes from the same frozen input. The controller selects exactly one of the
two fixed entrypoints per invocation; no untrusted path or argv is accepted.
Their independence means independent upstream implementations, not separate
trust authorities: the trusted controller still owns comparisons. Two clean
provisions must reproduce both executables and final filesystem inventory before
a digest identity is accepted. Exact tool versions/hashes, licenses, source
archives, configure flags, image inventory and raw results must be retained.

The target compiler/test role must independently prove absence of both oracle
binaries/libraries. This Containerfile does not establish that separate role.
Nothing in the reference role may be copied into target or launch images.

## One bounded request per process

All fields are bytes/little-endian integers. stdin is exactly16 header bytes
plus payload, then EOF. No trailing bytes, textual encodings, paths or commands.

Header: bytes0..8 ASCII RARMCR00; byte8 operation1..5; bytes9..12 zero;
bytes12..16 payload length u32. Absolute payload limit4416 bytes. Unknown
operations, inconsistent lengths, reserved bits and truncation are invalid.

Operations:
1. SHA256: payload is0..4096 message bytes; output32 bytes.
2. SHA512: payload is0..4096 message bytes; output64 bytes.
3. Pure Ed25519 verify: public key32, signature64, message0..4096; no output bytes.
4. IETF ChaCha20-Poly1305 seal: key32, nonce12, AAD lengthu16, data lengthu16,
   AAD0..256, plaintext0..4096. Output ciphertext followed by full16-byte tag.
5. Open: same48-byte prefix, then tag16, AAD and ciphertext with those lengths.
   Output plaintext only on successful authentication.
Signing is deliberately absent. A separately reviewed bounded public-fixture
packager will produce signed test layers; neither oracle is a signing service.

The controller chooses/fixes requests independently, including unpredictable
public challenge bytes and case identities. Cases are immutable before
comparison; duplicate/missing/reordered/changed cases fail the overall verdict.

## Output and errors

stdout is exactly64 header bytes followed by one implementation's result.
Header bytes0..8 ASCII RARMCO00; byte8 echoes operation; byte9 status; byte10
implementation ID (1 libsodium,2 OpenSSL); bytes11..16 zero; bytes16..20 output
length u32; bytes20..24 zero; bytes24..56 SHA256 of the entire exact input
header+payload; bytes56..64 zero. Maximum output4176 bytes.

Statuses:0 successful result,1 invalid signature/tag,2 tool failure. Status1 is
allowed only for verify/open and has zero output length. Status2 has zero output
length. No unauthenticated plaintext is emitted. Each successful length must
match the operation. A controller validates framing, implementation identity,
input hash, exit code, status, length and exact output from each independent
invocation—not merely whether two arbitrary blobs match.

Exit0 means the adapter completed (possibly reporting invalid authentication);
64 malformed request;70 initialization/version/cryptographic tool failure;
74 output failure. Any nonzero exit or stderr/tool anomaly fails the comparison
gate. Partial output is never evidence of success. Version checks require
OpenSSL3.0.13 and libsodium1.0.19; build/source identity checks are additional.

Strict RAR Ed25519 subgroup/canonical rejection tests remain separate from the
shared reference acceptance corpus. Library disagreement or unusual negative
return is not normalized into a pass. Reference bytes and verdicts are
development evidence, never production trust or permission to activate code.

## Remaining evidence

Compile/runtime parser negatives, published vectors, independent target-result
comparisons, malformed-input/resource/fuzz coverage, two reproducible image
builds, inventories, compiler-role absence and trusted-controller integration
remain required. No new workflow, disk access or VM profile is enabled here.
