# ADR 0022: Release 0 Two-Boundary Preauthorization

Status: Accepted — 2026-07-18

## Context

Prompt 7A previously mixed source-independent closure inputs with source-dependent
outputs, rewrote a prepared identity graph during CI, reopened validated paths, and
maintained parallel authority and lifecycle models.  That design could not prove
that the bytes reviewed, attested, authorized, and eventually supplied to an
execution supervisor were the same bytes.

The owner approved a narrow redesign with two replaceable tools and one existing
execution-supervisor boundary.  This ADR freezes their responsibilities and the
temporal evidence graph.  It grants no execution authority.

## Decision drivers

- Eliminate commit self-reference and mutable prepared output claims.
- Keep construction entirely separate from authorization and launch control.
- Retain one owned byte snapshot from validation through consumption.
- Publish outputs and their graph exactly once or expose no trusted result.
- Make review and owner authorization mandatory predecessors of any launch session.

## Considered options

- **Patch the existing prepared graph and shell pipeline:** rejected because it
  preserves graph mutation, path reopening, and parallel authority models.
- **Combine building and authorization in one executable:** rejected because a
  construction tool must never receive launch or authority capability.
- **Use two tools with one immutable bundle handoff:** selected.

## Decision

## Temporal evidence DAG

`closure-input-lock-v4` contains only immutable external inputs, grammar and tool
schema versions, algorithms, and approved policies.  It contains no source-dependent
archive, index, manifest, config, artifact, disk, graph, attestation, review, or
authorization digest.

`preauth-transaction` accepts that lock and a checked-out source revision.  One
bounded transaction owns immutable private snapshots of every input from validation
through use.  It emits one atomically published `transaction-bundle-v1` and one
canonical `transaction-graph-v1`.  The graph binds the source revision, input-lock
digest, complete signed package/source/license closure, tool and firmware inputs,
all typed OCI identities, twice-built artifact, immutable disk seed and initial
image, profile, descriptor-form command, and execution-host/supervisor/resolver/
spawner/wrapper/resource-controller identities.  It is serialized and hashed once;
no later phase may rewrite it.  This tool has no authority, network, credential,
session, resolver, spawner, or launch capability.

Independent push and pull-request runs create distinct `ci-attestation-v3` leaves.
Each leaf binds the same source revision and transaction graph plus its exact event,
run, attempt, workflow, and job-workflow reference.  Pull-request leaves are always
non-authorizing.  `attestation-set-v1` retains both event-specific leaves and proves
their deterministic graph equality.

`review-certificate-v1` may be created only after clean independent architecture,
correctness, and security reviews and binds their verdict evidence to the exact
source, graph, and attestation set.  Prompt 7A does not fabricate this record.

`owner-authorization-v2` may be created only after a later Prompt 7 owner approval.
It binds the exact graph, attestation set, review certificate, profile, artifact,
command, execution host, scope, `max_launches=1`, nonce, trusted time, and version.
Prompt 7A neither creates nor simulates a production owner authorization.

`signed-launch-session-v2` is the final root.  It cannot validate or be consumed
without both a clean review certificate and an authenticated owner authorization.
Prompt 7A proves refusal when either is absent.

## Boundary 1: preauth-transaction

The transaction owns each untrusted source exactly once: no-follow open, private
exclusive snapshot, complete bounded parse and merged plan, then consumption only
from the owned snapshot.  A public path is never reopened after validation.  An
open descriptor is insufficient if an attacker retains a writable alias; the
snapshot must be private and exclusively owned or equivalently sealed.

OCI tar/JSON and Debian ar/data inputs are planned completely before effects,
including member, compressed, expanded, aggregate, ratio, depth, component, link,
collision, type, ownership, mode, xattr, overwrite, and arithmetic bounds.  The
transaction builds a private content-addressed tree, validates final held bytes,
fsyncs every file and directory, and publishes one versioned bundle using atomic
descriptor-relative no-replace rename followed by parent fsync.  Failure exposes no
partially trusted public output.  The disk output is immutable; no writable launch
child exists at this boundary.

The version-1 transaction bounds are part of the public host contract: at most 64
input objects; 4 GiB aggregate compressed/input bytes; 8 GiB aggregate expanded and
published bytes; 512 MiB per Debian package; 1 GiB per archive member; 4,096 archive
members; expansion ratio at most 64:1 per member and in aggregate; path length 512
bytes, 64 components, and component length 255 bytes; JSON length 1 MiB, depth 64,
4,096 object keys, 4,096 array items, and 64 KiB strings; and canonical record length
1 MiB.  All additions use checked `u64` arithmetic.  Exceeding any bound rejects the
entire transaction before publication.

The transaction process may read the repository source and pinned input snapshots
and may write only its exclusive private staging directory plus one descriptor-
relative final bundle name.  It receives no network socket, AWS credential, signing
key, owner/review record, emulator-launch permission, device descriptor, arbitrary
output path, or child-process capability other than the separately pinned static
compiler/linker operations approved by Prompt 7A.  Its deterministic encoding is
canonical ASCII key/value records plus the already specified canonical JSON and
canonical archive encodings; unknown, missing, duplicate, or reordered fields fail.

## Boundary 2: preauth-session and execution supervisor

`preauth-session` accepts only an immutable transaction bundle/graph, both CI
attestations and their set, a clean review certificate, authenticated
`owner-authorization-v2`, a real signed authority/session envelope, and already
opened required descriptors.  It has no build, extraction, graph-rewrite, or path
fallback capability.

The signature envelope carries actual signature bytes over one domain-separated
canonical payload.  The payload binds the KMS key version, algorithm, and context
digest.  KMS Sign is not described as enforcing encryption context.  Exact OIDC,
conditional ledger, trusted-time, predecessor-audit, and current-event audit-closure
evidence are required.  Production AWS adapters remain unavailable and refusing
until separately provisioned and authorized.

The exact parsed session object is conditionally consumed once and passed, without
reconstruction, to the execution supervisor as
`ConsumedLaunchSession<OpenDescriptors>`.  The supervisor has no build or authority
rewrite function.  It alone owns lifecycle, resource enforcement, an exclusive
writable child created from the held immutable disk immediately before consume/
spawn, timeout, termination, exit observation, cleanup, and quarantine.  Prompt 7A
uses synthetic backends only and never spawns a process.

## Compatibility and migration

The accepted formats are `closure-input-lock-v4`, `transaction-bundle-v1`,
`transaction-graph-v1`, `disk-v2`, `execution-host-v2`, descriptor-form profile and
command v2, `ci-attestation-v3`, `attestation-set-v1`, `review-certificate-v1`,
`owner-authorization-v2`, and `signed-launch-session-v2`.

Closure v2/v3, identity graph v2, prepared certification v1, disk v1,
execution-host v1, CI attestation v2, authority v1, owner authorization v1,
consumption-key v1, and legacy VM certification are never reinterpreted.  The new
transaction and session entry points hard-refuse them.  Stale committed prepared
output records are removed rather than migrated.

## Consequences

The host implementation is larger, but trust is split into auditable least-
privilege boundaries and every generated identity has one temporal owner.  CI output
is external evidence rather than a self-referential committed claim.

## Security and data impact

No target, QEMU, firmware, emulator, VM, guest, device, AWS authority, credential,
or real spawner executes or is accessed.  Repository-local static construction and
synthetic refusal tests remain the only permitted effects.

## Validation

Contract tests reject old versions, missing/extra/duplicate/reordered or cross-typed
fields, output identities in the input lock, graph omissions, and authority/session
construction without review and owner predecessors.  Transaction tests cover
mutation, aliasing, archive bombs, collisions, failure at every publication boundary,
concurrent no-replace publication, fsync failure, and deterministic bundle bytes.
Session tests cover signature/context/audit/time substitution, replay and uncertain
transitions, descriptor loss/substitution, lifecycle legality, child singleton,
supervisor loss, timeout/kill/cleanup, and terminal quarantine.

## Replacement path

Either tool may be independently replaced by an implementation that consumes and
emits the same versioned canonical contracts and passes the complete conformance
corpus.  Combining the two trust domains requires a new owner-approved ADR.
