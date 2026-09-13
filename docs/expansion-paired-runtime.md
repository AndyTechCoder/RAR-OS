# Expansion paired cloud runtime

Base runtime: reviewed and activated through PR #205; actual run 34772050322
passed. Capture extension described below is pending its own source review/CI
and actual proof. This is not M5 closure.

## Concrete journey

The fixed Expansion workflow checks out one exact source SHA and the trusted
main controller separately. Existing container inspection runs before any code:
non-root, read-only root, network none, no capabilities, devices, credentials or
host sharing, fixed CPU/memory/process/output limits and disposable tmpfs.
The new build recipe retains the pinned Rust toolchain and adds only a fixed
first-party script. It composes the existing public-lab signed Settings bank,
builds peer A and B UEFI binaries independently twice, verifies exact equality,
packages each boot image with the existing checked packager and retains hashes.

The separate launch recipe uses the same pinned QEMU/OVMF/Python inputs.
The already reviewed Pair owns two ISA NE2000 guests and one unnamed UNIX
datagram socketpair. Neither endpoint is a host IP listener or Internet route.
Both machines pass paused topology checks before either runs. Existing M4
entrypoints, default profiles and build flags remain unchanged.

After both desktops are observed, the controller generates two distinct public
128-bit challenges. It types each challenge only into its originating guest.
Each receiving guest is given only NET RECV, not the expected payload. An
independent visual oracle checks all pixels, including the actual received
value. A closes its channel, a subsequent send remains unavailable, and Files
still opens. Exactly eleven scene boundaries and the complete fixed QMP/input
transcripts are retained. Screen polling is bounded and never reissues a command.

Both guests are killed and all six block backends joined before retained Data
is read. The independent checker verifies distinct storage identities, unchanged
virgin Data, no storage mutation requests, matching System and boot hashes,
paused NIC/port inventory, exact launch arguments, lifecycle events, process
reaping, input causality and actual pixels. Corrupt copies of the successful
evidence must be refused. All source/target work remains cloud-only.

## Bounds and failure behavior

Two peers only; no workflow-selected model, port, path, backend or shell command.
Per-pair lifetime is 120 seconds after Pair construction. Complete scenario is
bounded to 180 seconds, outer launch to 240 seconds. Each command is sent once.
Screenshots allow at most 24 observations per planned boundary. Output is at
most 64 MiB; each guest's existing serial, command and block limits remain.
Any failure terminates the pair; partial or failed cleanup never permits a
success/frozen-image claim. The outer controller retains failure metadata and
cleans up only its exact owned disposable cloud containers.

Pure orchestration tests use fake paths and executors, never actual files,
sockets, Docker or guests. They cover each of eight execution failures,
reproducibility mismatch and rejected evidence. Visual and command-oracle tests
cover malformed scenes, unexpected input and missing capture boundaries.

## Explicit remaining acceptance

A passing run proves only the described paired networking journey. Runtime
malformed/flood/drop/expiry/revocation and peer-death campaigns, captured-wire
comparison, general independently installed SDK apps, private documents,
agent broker, portable node, profile transitions, integrated update/recovery
regressions and release are still required by the M5 packet. Existing pure
codec/driver/grant tests are not substitutes for these guest proofs.
The deterministic public lab network is not encrypted or authenticated.

## Independent captured-wire extension

Only the Expansion profile adds a transparent QEMU filter-dump on its existing
private socket netdev. Two literal PCAP paths live in the fresh exclusive cloud
tmpfs root. They are not host mounts, input paths or guest-selected destinations.
QEMU filter type, file, netdev, queue, status, position, insert and 554-byte cap
are checked while stopped, alongside the complete existing paused preflight.
Legacy M4 launch arguments and network-none behavior remain unchanged.

After both guests and backends are reaped, bounded no-follow regular-file reads
retain both PCAPs in the canonical pair-v1 evidence. The independent first-party
wire oracle requires exactly two complete packets in each capture, exact order,
addresses, ports, IPv4 header, IP/UDP checksums, packet IDs and fresh challenge
bytes. Missing, extra, clipped, reordered or mutated bytes fail. In particular
the post-close send must create no additional packet. Capture timestamps are
framing-checked and monotonic, not treated as guest authenticity proof.

The parser has byte mutation/truncation tests and a checksum known answer. Its
expected bytes are composed independently from the target codec. The existing
positive runtime proof remains tied to its old immutable controller and v0
format; it is not relabeled as captured-wire evidence. QEMU is an already
approved host tool, not code linked into RAR OS. This extension does not add a
host network endpoint or change the private socketpair.
