# ADR0041: retain M4 regressions in the M5 release archive

Status: Accepted by explicit owner approval on 2026-09-15.
Scope: release evidence only; no OS behavior, target execution or owner-device authority change.

## Decision
Include the five existing M4 regression bundles (signed runtime, cryptographic comparison, Data faults, System install faults, System repair faults) alongside the four existing Alpha/Foundation/Platform/Desktop bundles. Require all nine at the same frozen main source and preserve their identities in the canonical release record.

Retain the existing128MiB per ZIP and256MiB aggregate limits, exact workflow/source/attempt/artifact/digest checks, cloud-only opaque copy, fixed asset names, no extraction or execution, no deletion or overwrite, and separate final publication gate. The archive consists of nine ZIPs and one record. System install and repair may share a workflow run but must have distinct correctly named artifact identities. M4 release assets remain untouched.

## Alternatives and consequences
Keeping only historical M4 evidence does not prove current shared-code regressions. Ephemeral workflow links expire. A separate archive would fragment the release record. The chosen extension adds five fixed categories without increasing byte limits or permissions. If the unchanged total limit cannot hold the evidence, fail closed and seek a separate decision; do not silently raise it.

## Validation
Require pure missing-category, wrong-role, size/digest/source, upload interruption and idempotence tests, focused independent review, exact CI, actual successful runtime results and final release verification. Approval is not evidence of milestone completion.
