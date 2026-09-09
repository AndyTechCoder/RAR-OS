# Integrated Modern crypto handoff candidate

This is one integrated, initially unactivated trusted-main cloud workflow for
the M4.1 crypto prerequisite. It does not boot RAR OS, change a VM profile,
merge the runtime PR, sign a release, or establish all M4 crypto acceptance.
No Mac or SSD execution, download, build, packaging, mounting or mutation is
permitted. The workflow must receive focused independent review, pass source
checks and land on trusted main before its manual dispatch.

## Exact inputs and authority

The caller selects one nonzero forty-hex source revision in the canonical
AndyTechCoder/RAR-OS repository. It contributes only the five source paths
already fixed by source_snapshot.py. Acquisition uses a newly owned bare Git
repository, depth-one fetch and bounded raw commit/tree/blob reads: no proposal
checkout, hooks, attributes, filters, LFS resolution or submodules. Raw objects
are retained and independently hashed/root-to-leaf bound before constructing
the readonly source layer.

Controller code, driver source, construction recipe and every imported helper
remain on the workflow's exact main revision. The controller checks that checkout
and binds its Modern source bytes to main's Git blobs. Proposal files never
supply a host path, helper, recipe, entrypoint, image identity or command.

The workflow token has only contents:read and actions:read. It is sent solely
to fixed artifact/run API paths on api.github.com. A single artifact302 redirect
is validated as HTTPS GitHub artifact storage, then followed without credentials,
cookies, ambient proxies or further redirects. Exact size and the already-fixed
whole-ZIP SHA256 precede parsing. No signed URL is written to evidence or logs.
The token is removed from the environment before any child process and discarded
after acquisition. Child Docker commands receive a clean fixed environment; Git
commands additionally disable global/system config, hooks, attributes, replacement
objects and prompting, and permit only HTTPS transport to the fixed repository.

## Actual pipeline

1. Reuse compiler artifact10004371629 and reference artifact9967392023. Verify
   fixed metadata, first successful source/workflow/run identities, both retained
   image inventories and their independent inventory-parser agreement. No
   reference or compiler reconstruction merely to obtain copies already retained.
2. Bind the exact five RAR source blobs without executing any of them.
3. Load the inventoried compiler parent and verify its actual immutable daemon
   identity and ordered rootfs. Acquire only the already-pinned Rust1.95 bootstrap
   digest. Create a fresh run-owned parent tag, refusing to overwrite an existing
   tag; build the fixed driver recipe twice with no cache, pull or build networking.
4. Independently inspect both bounded driver exports: exactly the static driver
   plus complete byte-identical parent notices, no links or extra files. Both
   drivers must match. Construct the reviewed driver/source layers and derived
   compiler image, independently inspect it before loading, then verify the
   daemon's image/config/rootfs against that inspected image.
5. Invoke the existing confined compiler runner twice. It owns no host mounts,
   networking or oracle image and cannot execute its output. Require bounded
   static adapter bytes, identical independent outputs and confirmed compiler
   container removal before any adapter invocation.
6. Build and inspect two identical adapter-only scratch images with the full
   notice set and RAR source attribution; verify actual daemon identity/config.
   No compiler, proposal source, oracle or token is inside that scratch image.
7. Run the fixed288-case corpus (864 total adapter invocations). All RAR results
   are durably frozen before either independent reference is invoked. Identical
   request bytes go to RAR, libsodium and OpenSSL. There are no retries or skipped
   failures; all results must agree and satisfy the independent expected cases.

The retained reference candidates remain host-only. No third-party library is
linked into a RAR target image. The RAR attribution grants no new license and
the notice inventory is not a legal sufficiency claim.

## Evidence and budgets

The controller creates only two new fixed directories inside its disposable
hosted-job workspace. Evidence uses a private directory descriptor, fixed bounded
leaf names, O_EXCL/O_NOFOLLOW regular files, checked ownership, complete writes,
file fsync followed by directory fsync, and SHA256 acknowledgement only after
successful retention. It never replaces an earlier evidence file. Evidence is
limited to256MiB,10000files and64MiB perfile. Source bytes, exact source/main
identities, host tool hashes, daemon/bootstrap/builder identities, image and
notice inventories, driver bytes, compiler outputs and comparison wire data are
retained. Large original compiler/reference archives remain in their already
recorded immutable construction artifacts; no Mac/SSD downloads occur.

Recorded adapter execution includes request, create, before-state, output,
after-state and cleanup records. The returned container ID/name/image/ownership
must match before start; full actual stopped state is checked after attach.
Retention failure or uncertain cleanup is job-fatal. Only exact verified
invocation-owned container IDs are removed; no image deletion, cache pruning,
branch changes, raw disks, native boot or owner storage access.

The controller has a50-minute alarm and5GiB address-space ceiling, with existing
per-command/runner time/output/memory limits and a60-minute workflow ceiling.
It checks16GiB disposable workspace reserve. The actual usable hosted-runner
capacity and builder/runtime compatibility must still be measured in the first
reviewed diagnostic. Failure is retained by phase/type with no automatic retry
or fallback to broader permissions. Framework job teardown handles ambiguous
unowned objects; no cleanup by guessed name.

## Evidence scope and tests

A successful first run is labelled fixed-corpus-compared, not M4 complete or
crypto_interoperability_accepted. It demonstrates the actual end-to-end fixed
corpus and independent implementations, but challenge-driven cases, injected
confinement/timeout/crash/cleanup faults, guest block faults, final-source
regression and the remaining M4.2/M4.3 outcomes still require evidence. Adding
this controller alone proves none of those runtime outcomes.

Cloud source tests exercise default host denial before mutation, exact API and
artifact-host scope, credential-free redirect construction, HTTP size bounds,
actual fresh scratch evidence durability/exclusivity/link refusal, bounded
owned output reads, and real static-driver/export notice validation. They mock
network/process activation and do not replace a real integrated diagnostic.
The optional legacy unrecorded adapter-runner path remains for earlier fixtures;
this integrated controller always supplies durable recording and enforces the
stronger actual stopped-state check.

If the first diagnostic fails, inspect the exact retained phase and bounded
process evidence, correct the cause, review the correction and perform one
bounded retry. Never spin an unchanged job or describe a helper self-test as a
successful crypto comparison.
