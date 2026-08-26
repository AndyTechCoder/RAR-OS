# Sprint Alpha Development Lab Controller

`development-lab-v1.env` is trusted controller data loaded only from GitHub
`main`. It deliberately remains `state=blocked`: the repository currently has
no reviewed, role-separated build and launch OCI images containing byte-pinned
target compiler/linker or QEMU/UEFI inputs. A Development Probe therefore
refuses before creating a container or executing source-branch code.

Activation requires separate build and launch image digests and a reviewed
change that replaces every `unavailable` value
with a real canonical path or SHA-256 identity, changes `state` to `ready`,
updates `machine_profile_sha256` to the exact hash of
`x86_64-q35-v1.profile`, passes `check-development-lab-profile.sh` in both its
blocked and ready fixtures plus all negative tests, receives independent
correctness and security review, and merges to `main`. No workflow may download,
install, or discover an unpinned substitute at probe time.

The machine profile is host-controller data, not a stable target interface. It
requires software emulation, bounded resources, no networking, no passthrough,
no host sharing, read-only firmware, disposable snapshot storage, serial
capture, and a Unix QMP endpoint confined to the disposable cloud build root.

The two OCI roles have distinct reviewed digests. The untrusted source checkout
is mounted only into the build container and may execute arbitrary
repository-controlled build code there. That bounded, no-network cloud phase
receives compiler/linker identities but no controller-granted QEMU, firmware,
profile, credentials, or accepted launch-evidence authority. After it stops, the controller
copies, bounds, and hashes only `/build/rar-os-alpha.img`. A second launch image
mounts the frozen artifact, writable bounded evidence, and trusted controller—
but never source. It re-hashes the artifact and uses the fixed sandboxed QEMU
argv plus trusted A–G QMP scenario harness. Source-supplied emulator activity in
the build sandbox is residual untrusted computation and cannot satisfy launch
or milestone evidence.

`alpha-crypto-references-v1.env` is a separate Class C host-only inventory for
Milestone F interoperability. It remains blocked until the named OpenSSL and
libsodium reference executables/harnesses have real reviewed paths and hashes
inside the no-network build image. They never become target dependencies.

`qmp-client-v1.env` is the reproducibility and replacement contract for the
RAR-owned acceptance harness client. Activation requires its reviewed source
tree identity, build-plan hash, binary hash, version, license, fixed verb set,
and equality with the binary hash in the Lab profile. The client is a pinned
host/lab tool and never ships in RAR OS.
