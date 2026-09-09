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
   tag; revalidate and retain its exact ID immediately before and after each build.
   Build the fixed driver recipe twice with no cache, pull or build networking.
4. Independently inspect both bounded driver exports: exactly the static driver
   plus complete byte-identical parent notices and two fixed consumed-musl
   inventory files, no links or extra files. The pinned bootstrap's explicit
   find/sort/sha256sum commands record the complete copied sysroot before and
   after rustc; pipefail and cmp require equality. The outer parser independently
   compares both exported inventories byte-for-byte with the accepted parent
   files/directories, hashes, modes and numeric ownership. Both drivers must match. Construct the reviewed driver/source layers and derived
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
retained. Large input streams are SHA256/size-recorded and sent in bounded zero-copy64KiB
chunks, avoiding repeated full-tail allocations. Large original compiler/reference archives remain in their already
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
A mocked full main() path additionally checks the864-invocation success sequence
and stops before any adapter execution on compiler-output mismatch, parent tag
substitution, or consumed-sysroot mismatch. This uses symbolic images and mocked
network/process helpers; it is orchestration coverage, not runtime proof.
The optional legacy unrecorded adapter-runner path remains for earlier fixtures;
this integrated controller always supplies durable recording and enforces the
stronger actual stopped-state check.

If the first diagnostic fails, inspect the exact retained phase and bounded
process evidence, correct the cause, review the correction and perform one
bounded retry. Never spin an unchanged job or describe a helper self-test as a
successful crypto comparison.

## Retained failure inspection and observable diagnostics

Diagnostic34332066725 used trusted main cb863b78ef914f9a1d9e2c2dddf704928e0a91f2
and source bfd8647b10e1daa38934b1e12363ba919938bbf7. It stopped during
driver-construction before independent comparison. Its exact error remains to
be read from retained artifact10096184040 (387106 bytes, SHA256
bd5ee6fe146338fa72f3a6349a26d4705b93345a2b7c48f5dc26be30a6b19451).
This is a failed diagnostic, not crypto acceptance.

The optional inspect_first_failure workflow input selects only crypto_failure.py.
It never calls the compiler/comparison entrypoint. It reads just that fixed
artifact/run through the existing credential-separated GitHub client, checks
receipt/whole-ZIP digest before parsing, reads bounded flat regular members in
memory without extracting any file, checks the complete retained manifest/hash
inventory and prints a bounded JSON rendering of its last three command records.
It neither builds nor loads images, invokes a subprocess, boots a VM, writes
workspace files, or accepts an arbitrary artifact ID, URL, path or command.
The workflow checkout remains a disposable cloud operation. No evidence is
downloaded to the Mac/SSD. CPU/address-space/time/output bounds apply; filesystem
output is denied with a zero file-size limit. Inspection logs do not establish
runtime success.

Future comparison failures print the bounded phase/type and, outside the
credential-sensitive acquisition phase, the retained validation error as one
escaped JSON line. They still retain the full evidence artifact and fail.
No guard, acceptance condition or retry rule changes.

Receipt verification uses exact types and values for every fixed field, including
both canonical repository numeric IDs and names on the run and both repository
IDs on the artifact receipt. Booleans cannot alias integer run attempts. Pure
negative tests mutate/delete every fixed leaf and exercise type aliases.

## First diagnostic cause and scoped correction

Read-only cloud inspection34336034920/job102415522709 verified the complete
fixed artifact receipt and inventory without extracting files or running target
code. Command77 was docker buildx inspect default; it exited1 with
"ERROR: mkdir /nonexistent: permission denied". Earlier pinned image acquisition,
independent inventories, source binding and compiler-parent loading reached their
gates; no driver build or crypto comparison had run. Peak controller RSS was
2444788KiB, below its configured ceiling.

The generic attached CLI transport deliberately supplies DOCKER_CONFIG=/nonexistent.
That remains unchanged for reference/compiler runner operations. The integrated
build controller now creates two fresh0700 directories under its newly owned
disposable work root, one for Docker CLI state and one for Buildx state. It passes
only fixed PATH/locale and those explicit paths through env -i, plus the explicit
Docker --config option and existing fixed daemon socket. It never imports the
runner's existing configuration, credentials, proxy settings or builder context.
Directory inode/device/ownership/mode are rechecked before every command.
Neither directory is in a build context or mounted into an adapter/VM.
No change to network policy, image validation, resource limits or acceptance.

Docker documents the CLI override at
https://docs.docker.com/reference/cli/docker/#change-the-docker-directory
and Buildx state-directory lookup at
https://docs.docker.com/build/building/variables/#buildx_config.
Tests cover exact full-pipeline command environment, fresh empty private
directories, preexisting-directory refusal and substituted inode/owner/mode/link
refusal without changing or deleting any prior fixture.

This is a diagnosed compatibility correction, not runtime acceptance. One
reviewed bounded diagnostic retry may follow exact-head source validation and
trusted-main merge. Do not run an unchanged retry.

Failed public CLI commands also print one canonical JSON record with their
sequence, argv, exit status, final 8192 stderr bytes and final 2048 stdout bytes.
Newlines and non-ASCII bytes are escaped; workflow-command text stays inert.
Artifact HTTP acquisition is not part of this command path. Tests force a
client failure before any build and verify bounded tails and escaped framing.
This avoids needing a separate artifact-reader change for ordinary tool errors.
