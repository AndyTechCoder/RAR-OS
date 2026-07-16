# ADR 0006: Separate System, Data, and Recovery Domains

Status: Accepted — 2026-07-16

## Context

Replaceable software, trusted recovery, and irreplaceable user data require distinct integrity, authority, and lifecycle domains.

## Decision drivers

- Repair the smallest damaged unit.
- Preserve intact user data through reinstall, update, and reconstruction.
- Make migrations transactional and reversible.
- Keep disposable caches outside durable promises.

## Considered options

- **One mutable system and data volume:** rejected because repair could destroy unrelated data.
- **Reconstruct system and data together:** rejected because it grants recovery excessive authority.
- **Separate Root, Recovery, System Store, Data Vault, and Scratch:** selected.

## Decision

Separate Root, Recovery A/B, immutable System Store, encrypted Data Vault, and disposable Scratch. Implement a custom copy-on-write, checksummed RAR filesystem and versioned state APIs. Recovery reconstructs system code without rewriting intact user data.

## Consequences

- Updates, reinstallations, and repairs can be narrow and reversible.
- Filesystem and migration correctness become security-critical.
- Data repair remains separate from system reconstruction.

## Security and data impact

Keys and write authority are separated by domain, user, and purpose. Uncertain recovery treats Data Vault as read-only; data repair is explicit and snapshot-preserving.

## Compatibility and migration

State schemas declare export, import, downgrade, and rollback. Migrations write copy-on-write and retain the prior verified state until commit.

## Validation

- Corruption and power-loss tests cover every commit boundary.
- System reconstruction leaves verified Data Vault hashes unchanged.
- Failed migration leaves the original state readable.
- Recovery A/B rollback remains available.

## Replacement path

Filesystem and state implementations may coexist behind stable block, state, and export contracts during copy-on-write migration.
