# ADR 0034: Modern Alpha Update and Recovery Boundary

Status: Proposed — independent architecture/security review pending

## Context

Milestone3 is published with a genuine protected graphical desktop. It has fixed
boot images and volatile files; there is no runtime block driver, signed component
loader, persistent filesystem or update engine to merely activate.
The owner now directs Milestone4 under the five-milestone fast-track plan.

## Decision drivers

Deliver genuine signed replacement and recoverable persistent state while keeping
the desktop usable, target code RAR-owned, tests cloud-only and claims bounded.

## Considered options

A. A distinct Modern-v0 experimental composition reuses released Desktop
mechanisms and apps, adds a replaceable Settings executable, isolated update/
storage services, bounded persistent synthetic System/Data images and immutable
laboratory recovery input. Recommended, subject to concrete interface/profile
review before use.
B. Complete production hardware roots, general package/SDK infrastructure and a
general-purpose filesystem before any update demonstration. Too broad for Alpha.
C. Demonstrate signatures only on the host, retain all state in RAM, or change a
palette and call it an OS update. Rejected: it does not meet Milestone4.

## Proposed decision

Implement A without reinterpreting Desktop-v0 or historical Alpha bytes. The
nucleus provides checked memory/process/capability mechanisms; filesystem,
signature/update policy and application behavior remain outside the kernel.
Settings is the first real replaceable code component, not a permanently special
public app type. Runtime role identities and image budgets remain experimental.

Use ADR0019's established pure Ed25519/SHA512 plus SHA256 content identities;
retain the public laboratory key distinction. Specify canonical metadata, exact
signed bytes and validation precedence before the parser/loader is accepted.
Use established authenticated encryption for synthetic Data; choose its exact
nonce/key/storage framing in the reviewed Modern interface specification.

System and Data access must be independently revocable. A system updater or
recovery process cannot possess a Data-write handle or select the Data device by
changing an untrusted request field. Lower-level device authority must enforce
this split; calling one shared unrestricted driver "isolated" is insufficient.

The cloud-only profile extension is limited to bounded synthetic regular-file
images surviving guest restarts within the same disposable test session. All
boot/recovery inputs are immutable. New device emulation must not introduce
uncontained DMA or arbitrary port/MMIO authority. Exact model, ownership, geometry,
timeouts and fail-closed validation are unresolved until the profile contract is
reviewed. Existing profiles are unchanged and cannot run the extension.

Candidate installation, trial activation, health decision and durable commit
have distinct states. Preserve prior verified bytes/state until the new commit is
durable. Separate security rollback floors from authorized last-known-good
fallback; specify the policy explicitly rather than accidentally making recovery
impossible after a version increment. New boots validate durable state themselves.

## Consequences

This is a larger implementation than the GUI milestone because it crosses disk,
crypto and process-lifecycle boundaries. Focused stronger review is justified;
repeated authorization paperwork is not. The published desktop remains a usable
baseline while this work is incomplete.

## Security and data impact

No change to the constitution, dependency policy or real user-data promises.
No Mac/SSD mutation or target execution. No real secrets/data or guest networking.
Software-only laboratory roots are not physical immutability, anti-rollback
hardware or a Secure Enclave. Corrupt/ambiguous Data is read-only or unavailable,
never silently formatted. Public fixture encryption is not production privacy.

## Compatibility and migration

Modern-v0 receives its own explicit private bootstrap, storage and lifecycle
versions. Desktop RAM state is discarded, never migrated into owner data.
Changes to the new experimental formats require version rejection plus explicit
fixture migration/export policy. Stable RLM/RSM/RCI and production vault formats
remain separate future contracts; no silent reinterpretation.

## Validation

The milestone task lists actual runtime acceptance requirements. Focused
independent review must settle device authority, durable atomicity, signed-byte
encoding, crypto interoperability, health and endpoint generation semantics.
All source gates and real cloud tests must pass before implementation merges.
No model-only, host-forged file or screenshot fixture can count as guest proof.

## Replacement path

Keep block transport, filesystem/state, crypto verifier, manifest parser, lifecycle
manager and UI behind documented bounded interfaces. Replace each independently
with conformance and migration/failure tests, not cross-subsystem internals.
