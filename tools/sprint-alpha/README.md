# Sprint Alpha Development Lab Controller

`development-lab-v1.env` is permanently non-activating legacy controller data.
Accepted ADR 0020 requires a three-role v2 topology, so the v1 validator rejects
even a syntactically complete `state=ready` profile. The repository currently has
no reviewed, role-separated build and launch OCI images containing byte-pinned
target compiler/linker or QEMU/UEFI inputs. A Development Probe therefore
refuses before creating a container or executing source-branch code.

The replacement contracts in `spec/alpha/lab/` define separate build,
reference, and launch roles and a bounded comparison transcript. They are
source-ready but do not provision or activate the Lab. A reviewed v2 controller
still must bind real image digests and replace every `unavailable` value
with a real canonical path or SHA-256 identity, changes `state` to `ready`,
updates `machine_profile_sha256` to the exact hash of the selected profile,
passes its validator and negative tests, receives independent
correctness and security review, and merges to `main`. No workflow may download,
install, or discover an unpinned substitute at probe time.

`development-lab-v2.env` is the exact non-activating instance of that shape.
`check-development-lab-profile-v2.sh` accepts only its complete blocked form;
any activating identity or `state=ready` fails until the real immutable inputs,
reproduction evidence, and reviewed controller exist. Its policy tests mutate
state, fields, bounds, grammar, and file type without contacting cloud services.

The machine profile is host-controller data, not a stable target interface. It
requires software emulation, bounded resources, no networking, no passthrough,
no host sharing, read-only firmware, disposable snapshot storage, serial
capture, and a Unix QMP endpoint confined to the disposable cloud build root.

The three OCI roles have distinct reviewed digests. The untrusted source checkout
is mounted only into the build container and may execute arbitrary
repository-controlled build code there. That bounded, no-network cloud phase
receives compiler/linker identities but no controller-granted QEMU, firmware,
reference tools, profile, credentials, or accepted launch-evidence authority.
After it stops, the controller copies and bounds only the frozen artifact and
comparison transcript. The reference role receives only the transcript and
emits comparison evidence. A separate launch image
mounts the frozen artifact, writable bounded evidence, and trusted controller—
but never source. It re-hashes the artifact and uses the fixed sandboxed QEMU
argv plus trusted A–G QMP scenario harness. Source-supplied emulator activity in
the build sandbox is residual untrusted computation and cannot satisfy launch
or milestone evidence.

`alpha-crypto-references-v1.env` is a separate Class C host-only inventory for
Milestone F interoperability. Version 1 remains unconditionally blocked: it
cannot express the isolated reference-image topology selected by accepted ADR
0020 or grant executable paths. A new reviewed inventory schema must bind that
topology and real identities before activation. The references never become
target dependencies or enter the untrusted build image.

`qmp-client-v1.env` is the reproducibility and replacement contract for the
RAR-owned acceptance harness client. `source-ready` records reviewed source and
build-plan identities while deliberately leaving the binary unavailable; it
does not activate the Lab. Activation requires a twice-reproduced cloud binary
hash, version/license/verb checks, and equality with the binary hash in the Lab
profile. The client is a pinned host/lab tool and never ships in RAR OS.

`tools/ci/report-sprint-alpha-gates.sh` is a read-only orientation report for a
task already rooted in the exact SSD workspace. It reports all local and
repository decision gates at once instead of stopping at the first failure. It
never contacts GitHub, proves the active Codex permission profile, or replaces
the strict local/remote preflight gates; those remain separate evidence.
