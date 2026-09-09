# Modern cloud persistence diagnostic integration

This change installs reviewed host-only tooling for the owner-directed M4.1
persistence work. It does not merge the Modern target implementation, publish
v0.4, or satisfy a milestone by itself. The runtime remains in draft PR158.

## Scope

The fixed manual Modern cloud persistence workflow uses the canonical main
controller and an exact canonical source commit. It builds that source twice,
checks target images, packages the immutable boot image with the existing RAR
host packager and independently checks every boot-image byte. The isolated
runtime container owns fresh synthetic System/Data regular files. No owner
files, local machine, SSD, credentials, raw device or guest network is attached.

The controller implements the bounded Modern profile documented in
tools/rar-lab/modern/vm-profile-v0.md. Its initial permitted purpose is diagnostic
cloud validation following independent source/confinement review and source CI.
QEMU remains paused until the controller verifies its actual device graph.
A profile mismatch fails closed; it does not permit arbitrary launch arguments
or bypasses. Runtime compatibility is evidence to obtain, not a prior claim.

## Ownership and cleanup

All created cloud containers disable restart and daemon logging. Operations
and cleanup use only full IDs after exact ID/name/image/invocation-label
verification. An ambiguous create or identity mismatch is not resolved by name;
the job fails and final disposable hosted-runner teardown handles unknown
objects. Source, boot inputs and artifacts are never removed by cleanup.
The Mac and SSD receive no changes or execution.

## Acceptance still required

The first diagnostic run must retain actual two-VM serial, pixel, block and
frozen-disk evidence. A real positive envelope and systematic negative mutations
must pass the independent retained-evidence checker before persistence is
accepted. All crypto/reference, signed replacement, isolation/revocation,
rollback, System-only recovery, interrupted-write and reproduction requirements
in the active M4 contract remain binding. Tests of host helpers and inert
lifecycle fixtures are not evidence that RAR target behavior works.

The tools are experimental laboratory infrastructure; public test keys do not
provide production confidentiality. No change to stable OS formats, dependency
policy, tier meanings or persistent user-data promises is made here.

## Bounded snapshot transport diagnostic

Run 34315008347 attempts 1 and 2 stopped before VM startup while APT fetched the
pinned OVMF package: the remote server closed the connection. The exact package
URL still returned 200 to separate header-only checks, which do not prove a
complete body transfer from the cloud runner. No runtime acceptance resulted.

The launcher construction recipe disables HTTP pipelining on its two existing
APT calls using Acquire::http::Pipeline-Depth=0. This is a bounded diagnostic for
server/proxy request scheduling, not a confirmed root-cause claim. Debian's
[Bookworm APT transport documentation](https://manpages.debian.org/bookworm/apt/apt-transport-http.1.en.html)
defines zero for this purpose. The immutable base image, dated snapshot, exact
package versions, signed repository metadata, package checksums and final
firmware/executable identity checks are unchanged. No authentication bypass,
extra dependency, alternative mirror, timeout change or retry loop is added.
The existing outer build/job deadlines remain in force.

After source review and CI, one corrected cloud diagnostic may determine whether
this setting permits the pinned package transfer and the full persistence
checker. Another transfer failure remains terminal; do not accumulate blind
reruns or silently change package identities. Even a successful transfer is not
proof that the previous failures were caused by pipelining.

## Actual positive-envelope refusal matrix

After the complete positive capture validates, the trusted controller applies a
fixed 60-case refusal matrix to fresh in-memory copies of those actual bytes.
Each case changes a named envelope, frame, command, event, receipt, QOM inventory,
process cut, backend authority/capacity or retained-inode property. It also
changes Data-header bytes and a frame pixel while recomputing their outer hashes,
so an outer-hash comparison alone cannot satisfy those cases. A real positive
capture is checked again after the matrix. A float -9.0 is not an integer QEMU
SIGKILL return code; the typed cut check and focused source tests enforce this.

The matrix records the immutable base hash, each changed-input hash, case name
and rejection-message hash in refusals.json and the controller manifest. Only
ValueError from the independent checker counts as a refusal. Unexpected parser
or programming exceptions, unchanged mutations, accepted alterations, duplicate
case names or the 300-second matrix deadline fail the job. The raw positive
capture remains unchanged and retained even if a refusal case fails. No mutated
copy is supplied to a VM, disk, compiler or proposal code.

These are retained-evidence negative tests, not actual interrupted block writes,
recovery tests or independent cryptographic interoperability. They cannot finish
M4.1 or M4 on their own. Source tests validate matrix shape, mutation operations
and integer cut semantics; only an actual cloud run validates all 60 mutations
against a real positive capture. No new runtime launch authority or profile is
introduced. The next corrected run still uses the reviewed fixed cloud profile.
