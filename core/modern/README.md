# Modern Alpha core

Experimental, unactivated RAR-owned no_std protocol implementation.
See docs/interfaces/modern-system-v0.md for exact bytes and failure rules.

- manifest.rs: strict laboratory signature, metadata, budget, rollback minimum,
  payload hash and existing bounded PE/W^X validation. Parsed metadata is
  untrusted; verified immutable bytes still are not kernel execution authority.
- journal.rs: canonical checksummed System selection records, two-record
  selection, monotonic install high-water, and explicit authorized fallback
  planning. Journal adds bounded alternate-record publication over SelectorIo:
  pre-read, write, flush, exact readback and acknowledgement. No native disk
  driver, encryption, data migration, health or execution authority.
- lib.rs: safe modules and reused RAR crypto/PE code; forbids unsafe code.
  No external target dependencies, allocation or local execution.

Focused tests run only in the existing isolated cloud Specifications container.
The same sources compile as no_std. Record and I/O faults are model tests;
they do not claim real device flush ordering, reboot persistence, atomic
lifecycle or recovery. A positive signed-package reference fixture and
end-to-end runtime gates remain required before activation.
No production trust or cryptographic audit claim.

Replacement: retain the explicit experimental contract and conformance corpus.
Never link the host-only reference implementations into these modules.


## Initial ring3 composition — source candidate

main.rs is the distinct Modern service entry. It consumes the368-byte read-only
bootstrap, uses152-byte full-incarnation int80 envelopes and wires the durable
Data service, keyboard/compositor, Files, Settings and Terminal. It includes
only RAR-owned modules and the existing RAR memory intrinsics. Desktop-v0 is not
used as a storage backend and remains unchanged.

The service loops are in services/modern/runtime.rs; durable app/session/UI
logic is in apps/modern and the existing tested Modern transport/session.
Manager/System entrypoints reserve their roles but do not implement live
updates: System only IDENTIFYs its own controller once, and both then await
future protocol work. No signed-update/recovery completion is implied.

Cloud Specifications compiles this actual ring3 entry to a Linux relocatable
object, never links or executes it. A pinned x86-64 UEFI build, bounded PE/link
inspection, certified Modern VM and actual persistent GUI demonstration remain
required. No Modern profile is activated by the source composition.
