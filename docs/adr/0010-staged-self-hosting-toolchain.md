# ADR 0010: Staged Self-Hosting Toolchain

Status: Accepted — 2026-07-16

## Context

Initial RAR OS development needs mature host compilers, assemblers, linkers, inspection tools, and emulators. Requiring a new compiler before OS foundations would block evidence, while permanent host dependence would prevent the Release 6 self-hosting goal.

## Decision drivers

- Begin reproducible architecture work without importing host tools into target images.
- Pin provenance, versions, hashes, and licenses.
- Preserve a path to independent Tier 3 development.
- Keep lower tiers free of unnecessary development tooling.

## Considered options

- **Build a complete RAR toolchain first:** rejected because it delays foundational OS evidence.
- **Depend permanently on unpinned host tools:** rejected because builds would not be reproducible or self-hosted.
- **Use pinned Class B host tools, then self-host in Release 6:** selected.

## Decision

Bootstrap with documented, pinned host tools that never become undeclared target runtime dependencies. Release 6 provides the Tier 3 compiler, assembler, linker, RAR-native build orchestrator, package, debug, signing, and documentation environment needed for reproducible Stage 1 and Stage 2 self-builds of the approved tiers, SDKs, tools, system applications, recovery images, packages, and system images. Self-hosting is a development-tier capability, not a requirement for lower-tier devices and not a claim that external compiler source became RAR-owned.

## Consequences

- Early releases can use mature host tooling while retaining target ownership.
- Tool versions and output-affecting inputs become release evidence.
- Release 6 carries explicit porting and reproducibility work.
- A future RAR language remains a separate, evidence-driven program.

## Security and data impact

Host tools remain outside the shipped trusted runtime. Builds record provenance and dependency inventories, and target execution remains subject to the certified VM and owner-authorization boundary.

## Compatibility and migration

Build descriptions and public target contracts remain independent from one compiler implementation. Stage transitions compare outputs and preserve the last reproducible toolchain until the replacement is accepted.

The exact bootstrap graph, stage semantics, signed-output treatment, and key custody remain Release 6 task-packet or follow-up ADR decisions.

## Validation

- Clean identical inputs produce identical unsigned target artifacts.
- Evidence records source revision, tools, hashes, targets, and configuration.
- Dependency reports show no undeclared target-linked third-party code.
- Release 6 Stage 1 and Stage 2 outputs satisfy the approved reproducibility policy.

## Replacement path

Compilers, linkers, build orchestrators, and emulators may be replaced independently when pinned provenance and reproducible conformance evidence remain available.
