# Modern kernel mechanisms

lib.rs/model.rs implement and test the unactivated Modern lifecycle model:
logical endpoints, caller-local capabilities, bounded stamped queues, restricted
trial health, checked incarnations, atomic model cutover, revocation and fault
recovery. See docs/interfaces/modern-lifecycle-v0.md.

This is not a kernel runtime, dynamic loader, sealed-memory implementation,
PIO driver, storage controller or boot proof. It has no unsafe code, allocation,
external runtime dependency or OS execution entrypoint. The real trap/loader
integration and independent runtime evidence must enforce the documented model.
Tests/no_std compilation run only in the cloud Specifications sandbox.

Staging records are private and currently populated only by test fixtures.
Production verification/sealing integration is still absent. Fault/timer events
are incarnation-bound; manager failure cancels pending work and requests
controlled recovery without disrupting the active Settings instance.
