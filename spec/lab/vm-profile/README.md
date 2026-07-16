# RAR Lab VM Profile and Launch Records — Version 1

Status: Release 0 host-only contract

These records are launcher inputs, not target OS interfaces. They are deliberately
small, canonical `key=value` formats so the host safety parser can reject duplicate,
unknown, reordered, non-ASCII, or malformed data without a third-party parser.

## Canonical encoding

- UTF-8 text containing printable ASCII values only.
- One field per line, in the exact order listed by the corresponding `.fields` file.
- Lowercase field names, one `=` delimiter, no whitespace, blank lines, comments, or
  carriage returns, and one final LF.
- Unknown, duplicate, missing, extra, or reordered fields are errors.
- Numbers use unsigned decimal with no sign or leading zeroes.
- Digests are lowercase 64-character SHA-256 values. `none` is permitted only where
  the schema explicitly names it.
- VM profiles are limited to 8 KiB, certification records to 4 KiB, authorization records
  to 2 KiB, and every line to 512 bytes. Limits are checked before field parsing.
- UTC timestamps must be real Gregorian calendar dates, including correct leap-year rules.

## VM profile

`profile-v1.fields` is the normative field order. The only architecture/backend/machine
tuples are:

| Architecture | Emulator identifier | Machine | Firmware |
| --- | --- | --- | --- |
| `x86_64` | `qemu-system-x86_64` | `q35` | Required and pinned |
| `aarch64` | `qemu-system-aarch64` | `virt` | Required and pinned |
| `thumbv8m` | `qemu-system-arm` | `mps2-an505` | `none` |

All profiles require software emulation (`tcg`), a disposable QCOW2 image, no network,
host sharing, passthrough, clipboard, elevation, or graphical display, and the emulator
sandbox. CPU count is 1–8, memory is 64–4096 MiB, runtime is 1–300 seconds, and captured
output is 1 KiB–16 MiB. Firmware, artifacts, and VM images use distinct repository-relative
paths below `out/r0/`; absolute paths, traversal, raw devices, aliases, and symlinks are
not accepted.

Before a resolver may run, the launcher validates an already-canonical repository root
using repository and approval markers, then requires the firmware (when applicable), target
artifact, and disposable disk to be regular non-symlink files at the profile's exact paths.
Artifact and firmware bytes are freshly SHA-256 hashed against their bindings.

After resolver delegation, the gate independently opens the claimed emulator through
descriptor-relative no-follow traversal, streams a fresh hash, checks stable file identity,
compares the actual bytes with the immutable pin, and carries that same open descriptor to
the spawner boundary. A resolver-supplied path/hash assertion is never sufficient.

The generated command is typed. There is no profile field for emulator arguments,
environment wrappers, shell fragments, helper programs, devices, or delegation.

## Certification record

`certification-v1.fields` binds one canonical profile and generated command to the exact
tool lock, emulator, firmware, artifact, source revision, reviewer, and review time. Its
`record_sha256` is SHA-256 over every preceding canonical line. The file must be stored at:

`out/r0/evidence/certifications/<record_sha256>.cert`

The digest is content addressing, not an owner signature. A launcher accepts the record
only when a future reviewed launcher policy separately pins that exact digest.

## Owner-authorization record

`owner-authorization-v1.fields` is separate from certification and binds the certification,
profile, and artifact for exactly one launch. Its self-digest covers every preceding line,
and its required path is:

`out/r0/authorizations/<record_sha256>.auth`

It becomes effective only when a future owner-approved launcher policy pins its exact
digest. The shipped Release 0 scaffold pins neither a certification nor an authorization,
so creating or editing local record files cannot authorize execution.

## Current certification state

No profile is certified. Required QEMU executables, x86-64/ARM64 firmware, and external
LLD are unavailable and therefore have no recorded digest. No owner-authorization record
exists. Static profile parsing and command inspection do not execute an emulator, firmware,
target binary, or RAR artifact.
