# Modern compiler handoff candidate

Unactivated host-only infrastructure for M4 crypto comparisons. This is not
RAR OS linkage, guest execution, or a change to the accepted compiler baseline.

The reproduced compiler parent is
sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914
from cloud run34083184108 at main d9cf06b87449391077f18566bffa1905fb3895bb.
The derived image must have a new independently verified identity. Its exact
ordered parent layers and files remain unchanged; no retagging, whiteouts,
replacement paths, links or inherited extra authority are allowed.

## Driver layer

compiler_driver_layer.py constructs and independently inspects the one-file
canonical USTAR layer for /rar-compile-driver: root-owned0555, fixed timestamp,
at most2MiB static x86-64 ET_EXEC. It reuses the independent ELF byte parser,
requires exactly one NX stack and an entry in exactly one executable file-backed
load, rejects PT_INTERP/PT_DYNAMIC, W+X, malformed loads, and unbounded mapping.
Canonical byte equality rejects hidden concatenated entries and extra padding.
It has no filesystem write operations and refuses execution unless Python is
isolated with bytecode writes disabled. It activates no image. Construction
callers must provide read-only exact-main tool sources, not a mutable directory.

The driver has a distinct RAR path, not an upstream Rust toolchain identity.
Its fixed rustc command clears inherited environment and sets TMPDIR=/build;
there is no fallback writable /tmp or HOME. Existing source and output bounds,
nonroot/capability/NNP/seccomp guards, exact five-input inventory, and no output
execution remain. Parent-enforced isolation is still mandatory.

Source revision/source hash/recipe hash in the layer report are provenance
labels only. The future trusted-main construction must bind them to exact Git
blobs, build the driver twice with the pinned private bootstrap and network off,
verify identical outputs, and bind output hashes independently. No caller report
can substitute for that construction proof.

## Remaining integration gates

- Bind exact five source files to trusted-main Git tree/blob identities and
  immutable source-layer bytes, not arbitrary checkout content or labels.
- Inspect the full new config/rootfs: exact parent prefix, this driver layer,
  exact source layer, no other additions, exact process environment/entrypoint.
- Independently bind required Rust-std/musl and RAR notices to driver provenance.
  The final static adapter image must carry its applicable notices too; notices
  in the compiler parent do not automatically satisfy separate distribution.
- Run the driver only in a reviewed cloud compiler role: fixed nonroot identity,
  read-only root, no network/IPC/devices/host mounts, bounded noexec/nosuid/nodev
  /build tmpfs, resource limits and verified effective daemon configuration.
- Accept output only after clean process completion, bounded EOF and independent
  static-ELF inspection. Never execute it inside the compiler role.
- Reproduce adapter bytes and its new scratch image containing only the adapter
  and required inert notices. Stop the compiler container before adapter runs.
- Freeze corpus and RAR results before invoking either independent reference.
  Oracle output never enters compiler source, adapter construction or RAR input.
- Retain actual confinement/timeout/cleanup and three-way comparison evidence.

Current tests are source-level layer, framing, refusal and command construction
tests in the existing cloud Specifications sandbox, not runtime acceptance.
No compiler runtime, adapter runtime, Modern VM or disk profile is activated.

The layer report records SHA256 and Git blob identities of both this helper
and its adjacent compiler_elf.py parser. Those measured identities must match
the trusted-main tree before construction/inspection; labels alone cannot
establish provenance. The caller's read-only source mount prevents changes
between identity measurement and import. Both sources are bounded at128KiB.
Focused tests cover aggregate multi-LOAD limits, address/offset congruence,
ambiguous executable entry mappings, no-bytecode refusal and the exact2MiB limit.
