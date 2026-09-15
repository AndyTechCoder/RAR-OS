# ADR 0041: Expansion regression archive

Status: Accepted — 2026-09-15

## Context
M5 changes shared Modern code. Historical M4 results alone cannot establish current-source regressions. The owner explicitly approved retaining the five M4 regression bundles in the M5 archive on 2026-09-15.

## Decision drivers
Retain complete, exact-source release proof without owner-device access, extra execution authority or larger archive byte limits.

## Considered options
Keep only historical M4 proof (insufficient for changed code); retain temporary workflow links (expiry risk); use a separate archive (fragmented provenance); include the five fixed bundles in the existing M5 record (selected).

## Decision
Include signed runtime, crypto comparison, Data faults, System install faults and System repair faults alongside Alpha, Foundation, Platform and Desktop. All nine bundles must bind to the same frozen main source. System install and repair can share a workflow run but require distinct role-named artifact IDs.

## Consequences
The archive has nine ZIPs and one canonical record. Every ZIP remains at most128MiB; aggregate ZIP bytes remain at most256MiB. Missing, oversized or mismatched proof fails closed. This decision does not establish M5 completion.

## Security and data impact
Preserve fixed asset names, complete pre-upload validation, exact digests/source/workflow/attempt checks, cloud-only opaque copying, no extraction or execution, no overwrite/deletion and the separate final publication gate. No Mac/SSD access or new network authority. Existing M4 release assets remain untouched.

## Compatibility and migration
No target, disk, SDK or OS behavior changes. This is an extension of the unreleased experimental M5 evidence record, not a migration of existing user data or published M4 assets.

## Validation
Pure tests require all nine roles, reject wrong-role artifacts, enforce unchanged byte caps, and exercise every upload interruption and digest-identical resume. Exact CI, independent review, successful frozen-main runtime proofs and final archive verification remain required.

## Replacement path
Future archive changes require a separately reviewed decision. If the fixed total cannot hold the evidence, stop rather than raise limits implicitly.
