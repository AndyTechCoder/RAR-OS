# Modern reference candidate provisioning v0

This is a trusted-main, manually dispatched cloud construction job under accepted
ADR0020. It produces candidate evidence only. It does not execute RAR target
code, run either reference adapter, build an OS image, start a VM, sign layers,
activate a profile, publish a release or establish Milestone4 completion.

## Inputs and confinement

The workflow checks out only its exact main commit, with persisted credentials
disabled. The controller requires the fixed repository, main ref, workflow_dispatch,
Linux x86_64 Ubuntu24 hosted-runner context, exact checkout location/revision and
a clean source tree. These are misuse/provenance checks, not a claim that
environment variables enforce a sandbox.

Acquisition accepts only the existing pinned Rust1.95.0 construction image and
the hash-pinned OpenSSL3.0.13/libsodium1.0.19 source archives. Archive acquisition
uses TLS without proxy/auth/cookie handlers, a restricted HTTPS redirect host
set, finite read/deadline/size budgets, and a mandatory SHA256 match. Before any
container extraction, source archives are inventoried as bounded data. Links,
devices, sparse entries, path escapes, duplicate names, special mode bits,
multiple roots and oversized expansion are rejected. Streaming inflation is
bounded at320MiB including archive headers, with at most256MiB declared payload;
large PAX/header declarations cannot bypass the decompressed-byte bound. The host never extracts
these archives.

Each fresh construction context contains exactly the two archives, three fixed
C/header sources and the fixed Containerfile. It contains no target checkout,
credentials or owner files. Docker builds use linux/amd64, no cache, no RUN
network, the pinned base and a fixed source epoch. No secrets, SSH forwarding,
device options, privileged builder, source-controlled build arguments or host
mounts are supplied. Construction uses the existing hosted daemon and its
BuildKit; daemon/tool versions and platform resources are recorded, not falsely
described as digest-pinned new runtime identities.

The two sequential builds have fresh contexts and tags and disable instruction
cache reuse. They share the immutable acquired base and hosted daemon. This
checks repeatability within that recorded construction environment, not
independence across machines, toolchains or suppliers. Each build has a
900-second command deadline and 8MiB output limit; the hosted job has a
45-minute deadline. Memory/disk are finite hosted-job resources, not purported
per-build hard quotas. No global cache pruning or host cleanup is performed.

## Evidence and acceptance

Evidence retains source archives and inventories, construction-source hashes,
build logs (including compiler/linker/tool versions and hashes), daemon
versions/resources, both saved reference images, both validated image
inventories and a final manifest. Docker proxy/credential configuration is not
included in the recorded platform fields.

The data-only validator checks image config identity, every layer digest,
static ELF/role contents, ownership/permissions and process configuration. Both
builds must have identical image IDs, layer digests, named file digests, sizes
and metadata. A mismatch, malformed archive or any tool failure fails the job;
there is no alternate source, fallback digest, retry or weakened validator.
The final manifest says candidate-reproduced-not-activated only after equality.

## Remaining activation gates

A candidate run is not runtime certification. Actual Docker/BuildKit archive
compatibility, reproducibility and construction-tool evidence must be reviewed.
Peak controller memory must be measured and bounded for the accepted profile.
Reference execution needs reviewed exact image identities, real lifecycle/
fault/cleanup evidence, target/compiler reference-absence proof and full
controller-owned three-way vector comparisons. The current adapter runner
is not called by this job. No Modern OS/guest-disk profile is activated here.

The existing Specifications wrapper runs only pure URL/archive/guard tests for
this controller; it never invokes its provisioning main function. The manual
workflow must first reach reviewed main before dispatch. Provisioning and
runtime evidence remain explicitly absent until real retained runs prove them.

## Evidence binding and acquisition environment

The manifest records hashes and sizes of every retained evidence file (except
itself), exact resolved Python/git/docker client binary identities, the pulled
base image ID/platform/repo digest, BuildKit/buildx versions, and controller peak
RSS. The child environment excludes user Docker/Git/proxy/TLS/plugin variables;
Git system/global configuration and fsmonitor/hooks are disabled for inspection.
A failed build may leave an ambiguous daemon operation until the hosted job is
torn down; uploading failure evidence does not prove early daemon cancellation.
Such a run never establishes candidate success and must not be automatically
retried or used for activation. Cancellation/teardown closure remains required
before accepting the later runtime profile.

## Buildx client-state compatibility fix

The first real run33956829038 stopped at Buildx inspection because the empty
Docker config path was unwritable. The controller now creates one fresh, empty,
mode0700 `modern-reference-docker-config/` directory under the disposable cloud
workspace. Buildx may write its own client metadata there. It is not the runner's
normal Docker config, receives no copied credentials/configuration, is never
mounted into construction or runtime containers, and is not retained as an
artifact. The complete child environment still comes from a fixed dictionary.
Git global/system config remains disabled. No permission elevation, host config
fallback, reference/target execution or sandbox relaxation is introduced.
The evidence manifest now labels identity/acquisition/inventory/build phases and
marks the final stage complete only on candidate success.

## Explicit reference process environment and failure evidence

Cloud run 33957914221 built the first candidate successfully, then rejected its
process configuration. The previous empty-environment assumption is unsuitable
for BuildKit, which supplies a default PATH even for a scratch image. The final
stage now explicitly sets exactly `PATH=/nonexistent`; inventory requires this
single entry, rejects duplicate or additional entries, and retains all previous
user, entrypoint, command, mount and executable restrictions. The absent search
directory grants no executable lookup authority. Adapters use fixed absolute
entrypoints. This does not activate an execution role; any future role must
validate the same exact environment before starting a container.

Each bounded candidate archive is retained before inventory validation, with the
manifest stage identifying the inventory attempt. A retained archive is not
acceptance: only successful inventories enter the builds list, and the status
remains failed unless two complete inventories reproduce. No candidate is run.
The earlier failed run did not retain its candidate config, so the injected PATH
is the source-backed explanation, not a recovered byte-for-byte config claim.
