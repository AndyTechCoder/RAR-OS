# Alpha Host-Only Crypto References

`libsodium-reference.c` is the small RAR-owned command harness around the
digest-pinned libsodium 1.0.19 Class C reference implementation. It supports
only `--version`, deterministic public-key derivation, detached signing, and
detached verification over bounded hexadecimal inputs. It is not target code,
never enters a RAR image, and is built only while provisioning the pinned cloud
reference role selected by an accepted ADR. It must never be built into or
exposed to the untrusted target build image.

The OpenSSL 3.0.13 reference uses its own CLI from the independently maintained
OpenSSL source release. The two reference implementations remain independent;
the RAR harness does not share target implementation code or decide acceptance
semantics. ADR 0020 is still Proposed, so the version-1 crypto inventory is
unconditionally blocked and cannot express ready executable authority. An
accepted topology requires a new reviewed inventory schema with exact sources,
licenses, role identity, paths, and executable hashes.
