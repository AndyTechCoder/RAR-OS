# Expansion application package v0 (candidate)

Status: experimental source candidate; NOT a stable ABI, installed app, runtime
grant or guest acceptance proof. Native integration requires focused review.
Related: ADR0038, M5.2. Public laboratory signing key only; no production trust.

## Envelope

Exactly 512 manifest bytes followed by the declared payload; no trailing bytes.
All integers are unsigned little-endian. No Rust/C layout is a wire contract.

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 8 | ASCII RARAPKG0 |
| 8 | 4 | Format version 0 |
| 12 | 4 | Manifest size 512 |
| 16 | 16 | Nonzero application ID |
| 32 | 32 | SHA256 of enrolled public lab Ed25519 key |
| 64 | 32 | SHA256 of exact payload bytes |
| 96 | 8 | Nonzero generation, at least trusted minimum |
| 104 | 4 | Payload byte length, 512..131072 |
| 108 | 4 | Image budget, 4096-aligned, 4096..131072 |
| 112 | 4 | Stack budget, exactly 16384 or 65536 |
| 116 | 4 | Rights: UI=1, private document=2, network=4, agent=8 |
| 120 | 4 | Experimental application ABI version 0 |
| 124 | 4 | Architecture 0x8664 |
| 128 | 288 | Reserved, all zero |
| 416 | 32 | SHA256 of manifest bytes 0..416 |
| 448 | 64 | Ed25519 signature described below |

Signature message: the exact 17-byte ASCII domain RAR-APP-ALPHA-V0 including
one trailing NUL, followed by the 32-byte manifest digest. This is distinct
from the existing Settings layer domain. Algorithms and lab key are unchanged.
The lab private seed is public RFC8032 test data, never a production secret.

Validation order: exact framing/reserved/ABI/architecture; publisher, digest
and signature; trusted nonzero generation floor and allowed rights ceiling;
payload length/hash; existing bounded PE parser; declared image budget.
UI is mandatory and unknown rights are refused. Permissions in a signed
manifest are requests, not grants. No verifier success installs or executes code.

The payload is the existing restricted x86-64 PE subset: fixed base 0x400000,
128 KiB image limit, 4 KiB pages, read-only headers, W^X, executable entry,
no imports/dynamic relocation/TLS/delayed imports. The package is a new envelope,
not a claim that the private kernel bootstrap is a public application SDK.

Errors: Framing, Compatibility, Publisher, Digest, Signature, Rollback, Budget,
Executable. Nothing is partially installed on an error. Validation does not
itself persist a generation floor; the future trusted installer must do so.

## Private document ownership (candidate integration policy)

Owner ID is SHA256 of the exact 17-byte domain RAR-APP-OWNER-V0 including
one trailing NUL, publisher digest (32 bytes), application ID (16 bytes).
Version changes retain ownership; another publisher or ID cannot alias it.

A candidate uses the unchanged DataVault-v0 snapshot encoding with exactly two
reserved records: _rar.app0 (32 nonzero owner bytes) and _rar.doc0 (0..64 bytes).
Both must exist or neither. Partial, unknown reserved, or malformed state fails
closed and must not be formatted, repaired, projected or overwritten.

Only one document-owning app fits this bounded candidate. Existing quotas
remain four records, 64 bytes per record and 128 aggregate value bytes.
Metadata plus a full 64-byte private document leave 32 shared value bytes and
two shared record slots. Installation fails safely if space is unavailable.
This is a deliberately small Alpha limit, not the future filesystem design.

Installation makes one complete candidate snapshot, retaining every existing
shared record. Its publisher must use one existing Vault transaction, not publish
metadata and document separately. Mount never installs, writes or autoformats.
Repeated install of the same owner is idempotent; a different owner is denied.

A grant binds owner ID to full kernel-stamped principal/incarnation. Principal
IDs 10..13 are reserved for proposed app/broker roles; zero incarnation is refused.
A numeric grant is internal trusted policy, not an unforgeable kernel capability.
Do not deserialize one from app bytes. The future service authenticates kernel
envelopes and revocation before calling this policy; this source does not do that.

Read/write APIs accept no user-selected path or owner. Writes create a copy;
quota/refusal leaves the original untouched. The shared Files projection excludes
reserved records only after validating the entire snapshot. Native Storage must
route all shared reads/listings through that projection before activation.

## Compatibility and replacement

No Vault header, cipher, nonce, commit or snapshot encoding changes. Existing
M4 readers reject the underscore namespace rather than expose it; whole-OS
downgrade after private installation is unsupported and must refuse without
touching Data. Settings component rollback inside an integrated M5 OS is distinct
and must retain the new Storage implementation. No existing owner Data is migrated
by this candidate. Activation needs explicit installation and rollback evidence.

A future multi-app or larger-document format needs a separately reviewed
transactional migration/rollback design. Do not silently reinterpret these keys.
Removal/deletion is not implemented. Software repair never authorizes Data repair.

## Tests and remaining work

The existing confined Specifications workflow runs package framing/policy tests,
actual RAR crypto verification against independently constructed Python fixtures,
two valid signed generations, invalid ABI/W^X/rights/publisher cases, signature
and every-payload-byte tampering; no_std compile with warnings denied.
Private-document tests cover shared preservation, atomic candidate creation,
ownership, full incarnation, quotas and malformed namespaces.

The fixture PE contains synthetic bytes and is never executed. No new host tool,
runtime dependency, syscall, device, VM profile or cloud authority is added.
Still required: reviewed native installation/loader, Rust and C application SDK,
GUI launch, useful Notes, fresh-boot persistence, runtime isolation/contained
failure, revocation, rollback/recovery and integrated acceptance.
