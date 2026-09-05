# Modern Alpha core

Experimental, unactivated RAR-owned no_std protocol implementation.
See docs/interfaces/modern-system-v0.md for exact bytes and failure rules.

- manifest.rs: strict laboratory signature, metadata, budget, rollback minimum,
  payload hash and existing bounded PE/W^X validation. Parsed metadata is
  untrusted; verified immutable bytes still are not kernel execution authority.
- journal.rs: canonical checksummed System selection records, two-record
  selection, monotonic install high-water, and explicit authorized fallback
  planning. No disk I/O, encryption, data migration, health or durable commit.
- lib.rs: safe modules and reused RAR crypto/PE code; forbids unsafe code.
  No external target dependencies, allocation or local execution.

Focused tests run only in the existing isolated cloud Specifications container.
The same sources compile as no_std. Record faults are model tests; they do not
claim real block writes, reboot persistence, atomic lifecycle or recovery.
A positive signed-package reference fixture and end-to-end runtime gates remain
required before activation. No production trust or cryptographic audit claim.

Replacement: retain the explicit experimental contract and conformance corpus.
Never link the host-only reference implementations into these modules.
