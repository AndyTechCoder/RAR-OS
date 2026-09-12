# M4 release evidence preservation

The final acceptance gate requires successful integrated-source and exact-main
tests and cloud runtime evidence. Publication does not turn failed or partial
runs into acceptance. The separate fixed cloud preservation job copies eight
successful exact-main proof bundles to an existing draft v0.4.0-modern-alpha
prerelease. It neither extracts nor executes their contents and never downloads
them onto the owner's Mac or SSD.

Required categories are System Install faults, System Repair faults, Data faults,
signed runtime, independent crypto comparison, Foundation, Platform and Desktop.
A successful exact-main Specifications push run is also bound in the durable
release-record.json. Each artifact's ID, exact size and SHA256, run ID, repository,
workflow path, completed-success status, frozen-main source/controller SHA,
main branch identity, and exact workflow source title and exact successful attempt's artifact name are verified before upload. The two System matrix artifacts
also have distinct fixed role names. Thus another source's successful run or a
different matrix job cannot silently substitute for the required proof.

The helper reuses the reviewed cloud-only GitHub artifact reader, which separates
API authorization from validated HTTPS archive redirects. Opaque ZIP bytes are
hashed and uploaded unchanged to the one fixed repository/draft release. There
is no extraction, OS execution, source-selected command, filesystem write,
arbitrary destination, raw device or local host operation.

Only this dedicated job has contents-write and actions-read permission, using
the ephemeral job token outside target contexts. It can GET the exact draft and
POST only fixed proof names and release-record.json. Its code provides no release
publish, source edit, tag change, delete or overwrite operation. Matching existing
assets can be reused after full size/digest checks following an interrupted cloud
transfer; mismatched or unrelated assets stop without mutation. There is no
automatic retry loop or implicit cleanup of existing GitHub assets.

Eight artifacts are bounded to128MiB each and512MiB total; plan JSON to16KiB.
Network reads/uploads have per-request limits, the transfer has a20-minute
deadline checked between bounded requests, and the hosted workflow has a30-minute
cap. Successful uploads require exact returned names, sizes, SHA256 and uploaded
state. Public assets are durable GitHub release evidence, not90-day-only Actions
links. The record marks draft asset verification, not independent release approval.

After the job completes successfully, the main agent verifies the final release
gate and may publish the draft under the owner's standing release authority.
No release is published by this helper. Pure mocked tests cover exact selection,
cross-source/wrong workflow/role/failed/expired evidence refusal, fixed API methods
and URLs, no mutation before complete metadata validation, digest refusal,
idempotent reuse, and refusal to overwrite or delete existing assets.

## External publication race

The helper rechecks the exact draft/prerelease/source before every upload and at completion. These checks cannot make external publication atomic with an upload. No other actor may publish or mutate this draft during preservation; the owner agent publishes only after the workflow succeeds and the final release gate passes. An interrupted transfer is resumed by matching existing fixed-name asset digests, never overwriting or deleting assets. Tests inject failure before and after server-side creation at each of the nine uploads.

## Frozen release source versus publication controller

The OS release is pinned to the exact revision that was merged to main and exercised by all selected proofs. The preservation workflow itself runs from reviewed current main and takes that frozen `source_sha` explicitly; it does not change or rebuild the release revision. The draft target, every proof run head SHA, every artifact source and every title must match the frozen SHA, and all runs must name head branch `main` in the canonical repository. Specifications must be a successful main push; Foundation, Platform, Desktop and all Modern runtime workflows must be successful main `workflow_dispatch` runs, matching their real checked-in workflow interfaces. A proposal-branch run, old PR run or mismatched source is refused.

This permits a publication-tool-only correction after the OS source is frozen, without repeating unchanged target execution. It does not waive any exact-main runtime gate: those tests still run against the exact released main revision. The permanent record binds both `source` (released OS) and `publisher_source` (reviewed preservation controller); the release review verifies that later publication-only commits change no target or VM/campaign controller bytes. This M4 release remains at source `88cf7345a3b83d6faa80abe2eaaf57ddd2b53d6e`; no newer publication helper is relabeled as its tested OS source.
