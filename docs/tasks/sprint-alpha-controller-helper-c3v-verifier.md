# Sprint Alpha - C3V exact-set verification child packet

Status: authoritative source-only child packet; implementation is phase-gated.

Parent: `docs/tasks/sprint-alpha-controller-helper-integration.md`

## D0 - registration only

This pull request changes exactly:

- `docs/tasks/sprint-alpha-controller-helper-c3v-verifier.md`
- `docs/README.md`
- `tools/ci/check-specs.sh`

D0 only registers and byte-binds this packet. D0 grants no C3VA, C3VB, C3VR, workflow, container, verifier, compiler, helper, target, acceptance, or readiness authority. D0 must receive exact-head architecture, correctness, and security review, merge unchanged, and pass a distinct exact-main Specifications run including the complete mutation suite before C3VA starts.

The C2 prerequisite is exact main `70a683dfb6dbde03f0f884ddc16ac2a2680a4f4f`, passing Specifications run `33608694457`, and passing Observer run `33608694456`. Those receipts establish the prerequisite only. C3V must create a fresh observation at its own exact merge revision.

## Required sequence

After D0, C3V has three separately reviewed and merged subphases:

1. **C3VA - contract and evidence activation.** Close the repeatability/evidence gap and activate only validators and static policy tests. C3VA adds or activates no workflow, container, or verifier runtime and never executes the existing dormant verifier source.
2. **C3VB - one-shot trusted-main verification.** After C3VA exact-main validation, add one exact-revision workflow and bounded controller/harness.
3. **C3VR - mandatory retirement.** After C3VB exact-main evidence and independent review, make the workflow inert without deleting any file, then pass a distinct exact-main gate before C3A or unrelated main work.

A later subphase must not absorb an earlier one. Every subphase starts from then-current GitHub `main`, uses a fresh branch, exact-head architecture/correctness/security review, exact-head required checks, an unchanged exact-head merge, and distinct exact-main validation.

## C3VA exact owned paths

C3VA may change only these literal paths:

- `spec/alpha/lab/controller-helper-closure-verification-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-test-plan-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-validation-v0.fields`
- `spec/alpha/lab/controller-helper-closure-verifier-cases-v0`
- `spec/alpha/lab/controller-helper-closure-verifier-faults-v0.fields`
- `tools/ci/check-controller-helper-closure-verifier-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-test-plan-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-evidence-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-validation-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-cases-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-faults-source.sh`
- `tools/ci/verify-controller-helper-closure-verifier-evidence.sh`
- `tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh`
- `tools/ci/fixtures/controller-helper-closure-verifier/evidence-valid.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/evidence-malformed.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/evidence-cases.v0`
- `tools/ci/policy-test-modes.v0`
- `tools/ci/run-ephemeral-policy-tests.sh`
- `tools/ci/check-ephemeral-policy-test-confinement.sh`
- `tools/ci/check-alpha-preimplementation-contracts.sh`
- `tools/ci/check-sprint-static.sh`
- `tools/ci/check-specs.sh`
- `spec/alpha/lab/README.md`
- `docs/sprint-alpha-dashboard.md`
- `SPRINT_STATUS.md`

No directory or descriptive ownership is granted.

## C3VA recovery erratum (2026-09-03)

Owner authorization for this bounded recovery was recorded on 2026-09-03 under the delegated safe-direction authority. This erratum supersedes only the earlier C3VA clauses that say C3VA makes no workflow or container change and that no other runner behavior or confinement exception changes. All other requirements remain authoritative.

C3VA may additionally change the literal path `.github/workflows/specifications.yml` only to increase the existing Specifications mutation-policy container's `/tmp` tmpfs from exactly `128m` to exactly `512m`. The primary validation container remains exactly `128m`; the mutation container remains read-only, networkless, unprivileged, capability-free, PID-bounded, and memory-bounded at 2048 MiB, and every other workflow trigger, job, command, mount, resource control, permission, checkout, and authority gate remains byte-unchanged.

The existing `tools/ci/run-ephemeral-policy-tests.sh` file-size ceiling becomes exactly `ulimit -f 860160`. Under the pinned Linux `dash` 512-byte unit this is exactly 440401920 bytes, equal to the evidence-file maximum and no larger. The 512 MiB tmpfs is 536870912 bytes, leaving 96468992 bytes beyond one maximum boundary fixture while remaining inside the existing 2048 MiB container memory limit.

Only `tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh` may use `/dev/zero`, and only through its three already reviewed, read-only `dd` input lines that construct the two exact 440401920-byte physical-line boundary fixtures. The confinement checker must require each complete physical line exactly once, require exactly three total `/dev/zero` occurrences, and retain the generic ban on every other `/dev`, `/proc`, `/sys`, and `/run` path. It must bind the primary 128 MiB and mutation 512 MiB tmpfs lines to their respective Docker spans, require exactly one file-limit row with the exact value, and exercise canonical-pass, wrong-mutation-size, and swapped-association inputs through the same inline POSIX-AWK state machine.

This erratum creates no new workflow, trigger, job, container, verifier execution, target execution, compiler use, helper use, network access, credential, device write, Mac/SSD execution, public format, acceptance, signing, release, or readiness authority. The packet-only amendment must merge after exact-head architecture, correctness, and security review. Its resulting-main run may retain only the already identified confinement failure from run `33801818662`; the three-file implementation must follow immediately with no unrelated main work and must pass a distinct exact-main Specifications run including the complete mutation suite.


## C3VA hosted-runner rollout recovery erratum (2026-09-03)

Owner authorization for this bounded recovery is recorded on 2026-09-03 under the delegated safe-direction authority after exact-main runs proved that GitHub concurrently schedules two released images behind the `ubuntu-24.04` label. Run `33806231004` used image `20260831.293.1`; run `33815290752` used image `20260823.283.1`. Retrying until one image happens to pass is prohibited and is not acceptance evidence.

This erratum preserves the existing GitHub-hosted, external-attested, noncertifying Class-B orchestration boundary and the immutable OCI validation root. It authorizes a closed rollout allowlist containing exactly `20260823.283.1` and `20260831.293.1`; empty, malformed, unlisted, older, third, or reordered/duplicated inventory identities fail closed. The runtime boundary enforces exact `ImageOS=ubuntu24`, `RUNNER_OS=Linux`, `RUNNER_ARCH=X64`, and membership in that two-value image-version set. Runner binary and Hosted Compute Agent identities are setup-log attestations retained in inventory and review evidence; repository code cannot read them independently and must not claim to prove them at runtime.

The two Class-B inventory identities are unique and their complete reviewed rows are exactly:

`github-hosted-runner-20260823.283.1|github-actions|ubuntu-24.04-20260823.283.1-runner-2.337.0|hosted-compute-agent-20260819.586-commit-3cc4a88dfa507ef76119ad1bb3eccc6378bb2b76-release-ubuntu24-20260823.283|GitHub-Actions-service-terms-plus-runner-images-MIT|https://github.com/actions/runner-images/releases/tag/ubuntu24%2F20260823.283|runs-on-ubuntu-24.04-and-assert-ImageOS-ImageVersion-membership|external-attested-noncertifying`

`github-hosted-runner-20260831.293.1|github-actions|ubuntu-24.04-20260831.293.1-runner-2.337.0|hosted-compute-agent-20260828.587-commit-abac92662cab4cc7352de4f9f9d2e2419aad9c29-release-ubuntu24-20260831.293|GitHub-Actions-service-terms-plus-runner-images-MIT|https://github.com/actions/runner-images/releases/tag/ubuntu24%2F20260831.293|runs-on-ubuntu-24.04-and-assert-ImageOS-ImageVersion-membership|external-attested-noncertifying`

After this packet-only amendment merges, C3VA may additionally change only these literal live paths:

- `tools/rarbuild/bootstrap-lib.sh`
- `tests/bootstrap/run.sh`
- `tests/bootstrap/src/main.rs`
- `tools/ci/check-specs.sh`
- `tools/toolchain/class-b-host-tools.v1`
- `tools/toolchain/dependencies.r0`
- `tools/toolchain/host-tools.manifest`

No directory or descriptive ownership is granted. The implementation must positively exercise both allowed versions and reject empty, malformed, older, and third versions; bind exactly two complete unique inventory rows, exact allowlist cardinality and order, the recomputed Class-B digest, and the unchanged OCI digest, CI lock, checkout/upload pins, and noncertifying status. Historical observer evidence fixtures remain byte-unchanged.

This erratum creates no new workflow, trigger, job, network access, credential, self-hosted runner, container authority, verifier execution, target execution, compiler use, device write, Mac/SSD execution, public format, acceptance, signing, release, or readiness authority. The packet-only amendment may retain only the known singleton runner mismatch. The seven-path implementation must follow with no unrelated main work and must pass independent exact-head review plus a distinct exact-main Specifications run in which primary validation and the complete mutation suite both execute and pass.


C3VA adds exactly `tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh|ephemeral` to `tools/ci/policy-test-modes.v0`. Ephemeral rows change 28 to 29; immutable rows remain exactly 5; total test rows change 33 to 34. The runner invokes all 29 ephemeral tests exactly once in registry order and reports exactly `Ephemeral policy tests passed: executed=29 source=read-only scratch=tmpfs`. The confinement checker recognizes exactly that new path and mode. No other registry row, order, mode, summary shape, runner behavior, or confinement exception changes.

C3VA must replace the digest-only evidence design with one lossless, canonical, ASCII/LF framing that retains content-addressed typed raw preimages for every runtime and residual result. It must include exact lengths, SHA-256 digests, domain tuples, canonical zero-length representation, nonaliasing identities, and deterministic ordering for:

- stdout and stderr bytes;
- pre- and post-mount inventories;
- output inventory and output bytes;
- topology snapshots and mount identities;
- mutation/fault schedule, trigger, acknowledgement, and observed event bytes;
- resource usage and timeout/termination observations;
- residual source and proof bytes;
- the raw-to-normalized projection and normalized comparison fields.

Arbitrary bytes use the RFC 4648 standard Base64 alphabet with required canonical padding, no whitespace, and no alternate spelling. The retained decoded payload maximum is exactly 296816640 bytes. Because every chunk is padded independently, the Base64 payload total is `sum_i(4*ceil(chunk_i_decoded_bytes/3))` and the exact accepted payload cap is 396032120 bytes, including independent final-chunk padding at all 709 blob boundaries, not `4*ceil(total_decoded_bytes/3)`. Each chunk carries at most 1436 decoded bytes and one Base64 payload, so its complete framing record is at most 2048 bytes. At most 206697 maximally filled chunks are needed for the decoded total; the evidence retains the conservative absolute limit of 206741 chunks.

There are exactly 709 typed blobs: four per runtime case (166*4=664), one per residual proof (43), and two run-global clean-success pass blobs, for `664+43+2=709`. Each runtime case has one pre/input/topology envelope, one stdout blob, one stderr blob, and one post/output/event/resource envelope. Each residual proof has one lossless source/proof envelope. The two run-global blobs are respectively the clean-success pass-1 and pass-2 lossless envelopes; between them they bind the domain/header/tool/fixture inputs and each carries its pass topology, canonical manifest, mount identities, output, event, and resource bytes. The validator independently recomputes pass equality and every verification-receipt binding from those two retained preimages. No blob exceeds 16777216 decoded bytes; the total decoded cap still governs. Every blob has one header of at most 256 bytes and exact type, case/domain identity, decoded length, chunk count, and SHA-256. The evidence has one header of at most 8192 bytes, exactly 209 normalized records of at most 2048 bytes, and at most 207660 logical records total: one header, 709 blob headers, 206741 chunk records, and 209 normalized records.

### Canonical decoded envelopes and projection

Every decoded blob is one canonical length-framed envelope: ASCII/LF framing surrounds arbitrary raw field bytes, and the envelope itself is encoded only once by the outer canonical Base64 chunk records. It begins with the exact lines `rar-c3v-envelope-v0`, `kind=<kind>`, `case_id=<case-id>`, and `field_count=<unsigned-canonical-decimal>`, in that order. It then contains exactly `field_count` fields in the order required below. Field ordinal `NN` is two canonical decimal digits starting at `01`; each field has the exact ASCII/LF prefix `field.NN.name=<name>`, `field.NN.bytes=<unsigned-canonical-decimal>`, `field.NN.sha256=<64-lowercase-hex>`, and `field.NN.data`, followed by exactly the declared number of arbitrary raw bytes and one terminating LF. Decimal zero is written only as `0`; other decimals have no leading zero. A zero-length field is exactly `bytes=0`, the SHA-256 of the empty string `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`, `field.NN.data` plus LF, zero data bytes, and the terminating LF. For every field, the validator must consume exactly the declared byte count, require SHA-256 of those exact raw bytes to equal the declared digest, and use checked arithmetic to prove that the actual field count and the sum of all framing and field bytes exactly equal the containing blob's declared decoded length. The blob SHA-256 is over the complete decoded envelope, including framing, raw field bytes, and terminating LFs. No unlisted line, field, kind, case identity, framing byte, CR, or alternate spelling is valid.

The only kinds and their exact ordered field names are:

- `clean-success-pass-1` and `clean-success-pass-2`, each with `domain-header`, `tool-inventory`, `fixture-inventory`, `topology`, `canonical-manifest`, `mount-identities`, `output-inventory`, `output-bytes`, `event-bytes`, `resource-bytes`, and `verification-receipt-inputs`;
- `runtime-pre-input-topology` with `domain-header`, `input-bytes`, `pre-mount-inventory`, `pre-topology`, `mount-identities`, and `mutation-schedule`;
- `runtime-stdout` with `stdout-bytes`;
- `runtime-stderr` with `stderr-bytes`;
- `runtime-post-output-event-resource` with `post-mount-inventory`, `post-topology`, `output-inventory`, `output-bytes`, `mutation-trigger`, `mutation-acknowledgement`, `observed-event`, `resource-usage`, and `timeout-termination`;
- `residual-source-proof` with `residual-source` and `residual-proof`.

The two clean-success envelopes use case identity `RUN`. Case envelopes use exactly one catalog identity in canonical logical order `V001..V147,Q001..Q050,X001..X012`. Blob identities are exactly `B000001..B000709`. `B000001` and `B000002` are clean-success-pass-1 and clean-success-pass-2. Then, walking the 209 catalog rows in canonical logical order, each runtime row receives the next four identities in kind order `runtime-pre-input-topology`, `runtime-stdout`, `runtime-stderr`, `runtime-post-output-event-resource`; each residual row receives the next one identity of kind `residual-source-proof`. No allocation choice or dynamic discovery is permitted.

A normalized record's `raw_blob_ids` is a nonempty comma-separated list of canonical blob identities with no spaces. A runtime row lists exactly its four consecutive blob identities in the kind order above; a residual row lists exactly its one blob identity. Neither run-global blob appears in `raw_blob_ids`; both are bound through the header and verification-receipt projection. For each case, the validator constructs this exact LF-terminated digest preimage: `rar-c3v-normalized-v0`, then `repository=<repository>`, `controller_sha=<controller-sha>`, `source_sha=<source-sha>`, `run_id=<run-id>`, `run_attempt=<run-attempt>`, `verification_receipt_sha256=<digest>`, `case_id=<case-id>`, `result=pass`, `raw_blob_ids=<ids>`, followed for each referenced identity by `blob=<blob-id>:<kind>:<decoded-bytes>:<sha256>` in listed order. The normalized digest is SHA-256 over those exact ASCII/LF bytes. Every value is derived or compared to trusted invocation context; a supplied normalized value never substitutes for reconstruction.

### Header consistency and replay scope

The evidence header has one exact canonical value for `repository`, `controller_sha`, `source_sha`, `run_id`, `run_attempt`, `verification_receipt_sha256`, `artifact_nonce`, `blob_count`, `chunk_count`, `decoded_bytes`, and `normalized_count`. C3VA's contract fixes their exact order and lexical grammar. The validator receives the first six identity values as trusted expected inputs and requires byte equality. Counts are unsigned canonical decimal and are accumulated with checked arithmetic. Header `blob_count` equals the 709 actual blob headers, `chunk_count` equals both the actual chunk-record count and the checked sum of all blob-declared chunk counts, `decoded_bytes` equals both the checked sum of chunk decoded lengths and the checked sum of blob-declared decoded lengths, and `normalized_count` equals the 209 actual normalized records.

The stateless validator proves `artifact_nonce` is canonical and appears exactly once, that all blob/case/domain identities are unique in the artifact, and that no record can be substituted across the header tuple. It does not claim global historical nonce uniqueness. C3VB supplies trusted `repository`, `controller_sha`, `source_sha`, `run_id`, `run_attempt`, and `verification_receipt_sha256` from the one-shot workflow context; its external evidence review rejects any reused run tuple or artifact receipt. Replay resistance is therefore within-artifact cryptographic binding plus C3VB's trusted one-shot run identity, not a self-asserted stateless promise.

The validator parses as a bounded forward-only stream, never buffers more than one 1436-byte decoded chunk plus fixed parser state, decodes losslessly, checks every declared length/count/hash, derives every normalized record and digest from retained bytes, and rejects self-asserted normalized values. Empty, alias, replay, truncation, extension, reorder, duplicate, wrong-domain, wrong-length, wrong-hash, malformed-Base64, blob-count, chunk-count, line-count, decoded-total, per-blob, oversize, projection-cycle, and cross-case substitution mutations fail closed.

The evidence file maximum is 440401920 bytes. Its conservative proof is `396032120 + 206741*128 + 709*256 + 209*2048 + 8192 = 423112696`. The exact six-file artifact maximum is 441473024 bytes, below the existing 536870912-byte output budget. Raw stdout plus stderr remains at most 65536 decoded bytes per runtime case.

C3VA must remove the test plan's undefined repeatability state and replace it with a closed, validator-enforced definition. The cases and faults catalogs must bind the added alias, replay, truncation, oversize, projection, and raw/normalized mismatch mutations. C3VA is incomplete while any named repeatability, retention, reconstruction, nonaliasing, or normalization gap remains.

The existing verifier executable source, observer production files, workflows, container controller, closure candidate, locks, inventories, profiles, gate reports, and readiness files are read-only during C3VA.

## C3VA completion gate

C3VA completes only when all changed paths are in its exact list; the frozen contract set is decision-complete; validators reconstruct normalized evidence solely from retained raw bytes; positive, malformed, mutation, size, replay, alias, and confinement tests pass; all three exact-head reviews pass; the exact reviewed head merges unchanged; and exact-main Specifications plus every mutation suite pass.

At that point, record SHA-256 digests for every C3VA-owned semantic contract, checker, validator, policy test, and fixture. C3VB must treat those bytes as read-only oracle inputs.

## C3VB exact owned paths

C3VB may change only these literal paths:

- `.github/workflows/controller-helper-closure-verifier.yml`
- `spec/alpha/lab/controller-helper-closure-verifier-activation-v0.fields`
- `tools/ci/run-controller-helper-closure-verifier.sh`
- `tools/ci/controller-helper-closure-verifier-harness.sh`
- `tools/ci/check-controller-helper-closure-verifier-workflow-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-policy.sh`
- `tools/ci/test-controller-helper-closure-verifier-policy.sh`
- `tools/ci/fixtures/controller-helper-closure-verifier/tool-pins.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/base-closure.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/cases.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/fixture-plan.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/residual-proofs.v0`
- `tools/ci/fixtures/controller-helper-closure-verifier/expected-verification.receipt.v0`
- `tools/ci/policy-test-modes.v0`
- `tools/ci/run-ephemeral-policy-tests.sh`
- `tools/ci/check-ephemeral-policy-test-confinement.sh`
- `tools/ci/check-alpha-preimplementation-contracts.sh`
- `tools/ci/check-sprint-static.sh`
- `tools/ci/check-specs.sh`
- `spec/alpha/lab/README.md`
- `docs/sprint-alpha-dashboard.md`
- `SPRINT_STATUS.md`

No directory or descriptive ownership is granted.

Every C3VA semantic contract, source checker, evidence validator, evidence policy test, and evidence fixture is frozen in C3VB by its exact C3VA-main SHA-256 digest. C3VB may consume but must not modify those files. The existing `tools/ci/verify-controller-helper-closure-candidate.sh`, direct production observer `tools/ci/observe-controller-helper-closure.sh`, and `spec/alpha/lab/controller-helper-closure-observation-v0.fields` are also frozen by exact reviewed digest and remain read-only.

C3VB adds exactly `tools/ci/test-controller-helper-closure-verifier-policy.sh|ephemeral` to the 34-row registry. Ephemeral rows change 29 to 30; immutable rows remain exactly 5; total test rows change 34 to 35. The runner invokes all 30 ephemeral tests exactly once in registry order and reports exactly `Ephemeral policy tests passed: executed=30 source=read-only scratch=tmpfs`. The confinement checker recognizes only that one added path and mode. No existing row, mode, ordering, summary shape, or C3VA test changes.

## One-shot trusted-main workflow

C3VB adds exactly one verifier workflow at `.github/workflows/controller-helper-closure-verifier.yml`. It has `contents: read`, no write permission, no pull-request trigger, no cache, and no secret or credential input. All actions use full commit SHAs.

Its only automatic trigger is `push` to `main` with the literal path filter `spec/alpha/lab/controller-helper-closure-verifier-activation-v0.fields`. The activation record is created once in C3VB and is immutable afterward. Before any repository code runs, the job proves canonical repository, push event, main ref, Linux/X64 runner, `github.event.after == github.event.head_commit.id == GITHUB_SHA`, `github.event.before` equals the literal exact C3VA-main SHA frozen during C3VB, and `github.run_attempt == 1`.

After immutable network-free transfer, pinned Git proves `HEAD == GITHUB_SHA`, exactly two ordered parents, parent 1 equals both `github.event.before` and the literal exact C3VA-main SHA, parent 2 is a distinct canonical 40-hex commit, and the merge tree equals parent 2's tree. It rejects missing, extra, or reordered parents and every identity/tree mismatch. The first-parent diff must contain only C3VB-owned paths and the activation record must have the exact reviewed schema and one-shot state.

The evidence records parent 1, parent 2, both parent/tree identities, merge SHA/tree, event before/after, run ID/attempt, and controller/source identities. The external merge gate constructs the merge with only the exact C3VB PR head that passed all three reviews and checks as parent 2. Independent post-run evidence review must compare the recorded parent 2 to that established reviewed head before C3VB completes; the workflow proves topology and byte identity while the external gate proves review authority without a circular self-hash.

A future unrelated main push does not match the path filter. A future activation-record change fails the pinned first-parent and immutable-record checks. The workflow uses `concurrency: controller-helper-closure-verifier-main`, `cancel-in-progress: false`, exactly run attempt 1, no retry loop, and a 20-minute job timeout.

Repository acquisition uses the accepted C2 bounded immutable pattern and image `rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3` for `linux/amd64`. Exactly two isolated, bounded pre-verification network phases are permitted: anonymous provisioning of that exact platform/image digest only when inventory proves it absent, and anonymous exact-revision depth-bounded source acquisition. Each phase has a distinct container identity, empty mode-0700 Docker configuration, lifetime, output bound, and cleanup/absence proof. Both containers and their configurations are terminated and removed before observation or verification. No other network is allowed; every subsequent container uses `--network none` and `--pull=never`.

C3VB invokes the frozen production observer `tools/ci/observe-controller-helper-closure.sh` directly in its own fresh isolated container to produce exactly the fresh manifest and 23-line observation receipt for the exact C3VB merge revision. It does not invoke the C2 wrapper, the O001-O021 observer harness, C2 outer validation, or any prior artifact. The controller independently validates and freezes those two files against the frozen observation contract before verification.

## Verifier confinement

`tools/ci/run-controller-helper-closure-verifier.sh` is the sole host-side Docker controller. No other changed file may invoke Docker or access `unix:///var/run/docker.sock`. The socket is never mounted into a container. Ambient `DOCKER_HOST`, `DOCKER_CONTEXT`, `DOCKER_TLS_VERIFY`, `DOCKER_CERT_PATH`, and `DOCKER_CONFIG` must be absent. The controller uses a fresh empty mode-0700 Docker config and the exact `unix:///var/run/docker.sock` endpoint.

Every observer/verifier case uses the exact image above, fixed user `65532:65532`, `--read-only`, `--network none`, `--cap-drop ALL`, `--security-opt no-new-privileges`, `--pids-limit 256`, `--cpus 2`, `--memory 2g`, `--memory-swap 2g`, no device, privileged, host, IPC, PID, user, or cgroup namespace grant, and no credential, service, agent, SSH, cloud, GitHub, or Docker socket mount.

The environment is built with `/usr/bin/env -i`, exact allowlisted variables, `PATH=/usr/bin:/bin`, `LC_ALL=C`, and `LANG=C`. Compiler, package-manager, network-client, cloud-client, helper, target, firmware, QEMU, and guest commands are absent from the allowlist and forbidden by source/policy mutations.

`/workspace`, `/trusted`, and `/evidence` are distinct read-only, nonaliased bind mounts. `/verification` is a distinct controller-owned bounded output mount. `/tmp` is a private empty 32 MiB `noexec,nosuid,nodev` tmpfs. Each case has a fresh controller-owned fixture root of at most 64 MiB, exposed read-only at the contract closure root. Only the controller may perform the one exact scheduled host-side mutation against that case fixture after a bound trigger; the container never receives fixture-write authority.

Before each case, the controller proves root/device/inode/owner/mode/link-count identities, emptiness where required, no symlink, hardlink, mount, or path alias, and no unexpected entry. It records the same identities and complete inventories after termination. One verifier process runs for at most 30 seconds. The complete fixed-order matrix runs for at most 20 minutes.

For every case, producer termination and container removal are proven before evidence transfer. Normal cleanup order is verifier container, observer container if present, case mounts/scratch, controller scratch, then Docker config. Failure cleanup uses the same ownership/identity guards and ends by proving every created container, mount, volume, scratch path, and config absent. Cleanup and absence must succeed before independent validation or upload. Policy mutations cover endpoint override, socket propagation, writable/root/source mount, alias, identity replacement, capability, privilege, namespace, environment, credential, tool, resource, timeout, output, ordering, and cleanup failures.

## Exact verification and retained evidence

The fixed logical order is `V001..V147,Q001..Q050,X001..X012`. C3VB executes exactly 117 disposition runtime cases, 37 precedence runtime cases, 12 fault runtime cases, 166 runtime cases total, and 43 residual proofs consisting of 30 disposition and 13 precedence proofs. All 209 relationships appear exactly once. Sampling, skipping, retrying, reordering, synthesizing, or dynamic discovery fails closed.

The canonical verification receipt is exactly 31 lines with status `candidate-exact-set-verified-not-reviewed-not-ready`. The normalized verdict is exactly `mechanically-verified-not-reviewed-not-ready` or `normalized-not-ready`. Missing, extra, duplicate, reordered, malformed, oversized, zero, stale, self-attested, mutable, aliased, replayed, truncated, or contradictory evidence fails closed.

The final artifact contains exactly these six files:

1. `controller-helper-closure.sha256`
2. `controller-helper-closure.receipt`
3. `controller-helper-closure-verifier-tools.v0`
4. `controller-helper-closure-verification.receipt`
5. `controller-helper-closure-verifier-evidence.v0`
6. `controller-helper-closure-verifier-verdict.v0`

The artifact name is exactly `controller-helper-closure-verification-${{ github.run_id }}-${{ github.run_attempt }}`. A single final upload uses `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`, `retention-days: 14`, `overwrite: false`, and `include-hidden-files: false`. It occurs only after producer termination, cleanup/absence proof, and an independent validator has reconstructed every normalized result from the retained raw preimages and byte-validated the exact six-file set. Nothing else is retained.

## C3VB completion gate

C3VB completes only when C3VA exact-main is green; all changed paths are in the exact C3VB list; frozen C3VA and observer/verifier digests match; both stable topology/manifest passes and all 209 relationships pass; every confinement/resource/evidence mutation passes; all three exact-head reviews pass; the exact reviewed head merges unchanged; and exact-main Specifications, the complete mutation suite, the one-shot verifier workflow, and independent six-file evidence review all pass.

This result is mechanically verified but not reviewed, not accepted, and not ready.

## C3VR exact owned paths and retirement gate

C3VR may change only:

- `.github/workflows/controller-helper-closure-verifier.yml`
- `tools/ci/check-controller-helper-closure-verifier-workflow-source.sh`
- `tools/ci/check-controller-helper-closure-verifier-policy.sh`
- `tools/ci/test-controller-helper-closure-verifier-policy.sh`
- `tools/ci/check-alpha-preimplementation-contracts.sh`
- `tools/ci/check-sprint-static.sh`
- `tools/ci/check-specs.sh`
- `spec/alpha/lab/README.md`
- `docs/sprint-alpha-dashboard.md`
- `SPRINT_STATUS.md`

C3VR does not delete any file. It replaces the workflow with a byte-pinned inert archival workflow having no automatic trigger and one manual-only job whose job-level condition is literal false, so no runner or repository code can execute. Its source/policy checks reject any active automatic trigger, true job condition, verifier/controller invocation, artifact upload, credential, permission expansion, or network action. The registry remains exactly 35 test rows: 30 ephemeral and 5 immutable. Every C3VA/C3VB evidence/runtime byte remains unchanged.

C3VR requires exact-head architecture/correctness/security review, exact-head checks, unchanged merge, and distinct exact-main Specifications plus complete mutation validation. C3A and unrelated main work remain blocked until C3VR is merged and green. No deletion is authorized.

## Permanent denied authority

C3V must not contain or change a closure-acceptance instance, controller/helper source, target source, observer production bytes, locks, inventories, approved profiles, release gates, compiler policy, boot policy, or readiness state except for the literal status-document paths explicitly named above. It must not compile or execute any helper, target, firmware, guest, kernel, bootloader, QEMU workload, or RAR OS artifact. It grants no compiler-use, target-execution, acceptance, signing, release, or readiness authority.

No C3V output may claim `reviewed`, `accepted`, `ready`, `bootable`, or an equivalent state. Only the later C3A acceptance gate may accept this closure or authorize the next capability.

## Host and storage safety

All mutations are repository-confined and GitHub-hosted. RAR OS target code must not be built, run, booted, mounted, or executed on the Mac or SSD. No local or SSD file may be created, changed, moved, or deleted. The SSD may be used only as separately authorized source/worktree storage; this packet grants no such authorization.
