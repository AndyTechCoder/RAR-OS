# Contributing to RAR OS

RAR OS is currently a proprietary RAR project. Contribution authorization does not waive repository policies or ownership requirements.

## Workflow

1. Select an approved task packet.
2. Confirm its specifications, dependencies, and owned paths.
3. Create or reference required ADRs before changing shared contracts.
4. Implement the smallest complete vertical change.
5. Add positive, negative, fault, and conformance tests.
6. Update documentation, examples, migrations, and limitations.
7. Run `tools/ci/check-specs.sh` plus the active release test commands.
8. Submit completion evidence described in `docs/handoff.md`.

## Changes requiring special review

- Nucleus or unsafe/assembly code
- Cryptography, signing, identity, capabilities, or recovery
- Persistent formats or state migrations
- New target-linked dependencies or binary firmware
- Shared RID/ABI/device contracts
- Changes to tier or release promises

## Commit scope

Keep specification, implementation, tests, and documentation for one coherent change together. Do not mix unrelated formatting or architectural changes.
