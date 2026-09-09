# Fixed M4 retained checkpoint inspection

This narrow trusted-main cloud workflow reads only the two immutable artifacts
pinned in `retained_checkpoint.py`: successful persistence34319265996 and
fixed-corpus crypto34340985436. It does not build or run RAR OS, QEMU, a compiler
or any reference adapter. No archive member is extracted or executed, no
artifact is downloaded to the Mac/SSD, and no source or data image is changed.

Before ZIP parsing it checks exact repository/run/workflow/controller identity,
first attempt, terminal success, artifact size/hash, expiration and whole ZIP
digest. Artifact acquisition uses the existing token-separated HTTPS client.
Both fixed downloads complete and the client token is cleared before archive
parsing. Flat regular members, duplicate names/JSON keys, compression methods,
member/aggregate sizes, resource limits and a ten-minute deadline are enforced.
The workflow has read-only GitHub permissions, no user inputs and no retries.
Output is bounded canonical JSON, never raw workflow-command text.

Persistence inspection revalidates the exact saved envelope, built binary/boot
hashes, tool-bound firmware geometry, GUI/disk agreement and VM lifecycle.
All four hash/path lines and both firmware sizes must use the exact producer
framing. Refusal JSON must preserve the historical ordered, indented encoding,
exact fields, sixty ordered case names and lowercase digest framing.
It prints the actual retained Data request/event records so the predicted
fault-campaign geometry can be compared with observed guest I/O. The retained
refusal record is reported, not newly generated or represented as device faults.

Crypto inspection verifies the complete retained member inventory and feeds
recorded immutable response bytes through the existing fixed-corpus comparison
logic. It requires byte-identical regenerated frozen/three-way JSON and the
same864 recorded invocations. The source commit/tree/blob closure is independently
rehash-validated against the pinned target revision; its reconstructed canonical
source layer and source-binding JSON must equal the retained bytes and manifest. This is response validation, not rerunning
cryptographic adapters or establishing new timing/confinement evidence.
It does not grant complete crypto acceptance, M4.1 completion or a release.

Independent review and exact-head source validation precede merging this
controller. Only its reviewed trusted-main revision may be dispatched.
The runtime fault candidate remains separate and unactivated.

Pure source tests cover baseline wiring with an inert envelope-validator mock,
canonical tool/refusal failures, Git-object/layer/report corruption, rehashed
comparison mutations, ZIP bounds and the two-download/token-clear/parser ordering.
The mocked lifecycle includes acquisition, receipt, ZIP and parser failures; no
unit test downloads artifacts, applies resource limits or executes a target.
