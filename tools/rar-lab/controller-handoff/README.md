# Controller handoff core

This dependency-free host library implements the deterministic SHA-256,
fixed 256-byte handoff-manifest codec, typed phase/output plans, and the safe
transaction policy defined under `spec/alpha/lab/`. The safe transaction core
is parameterized over descriptor operations and includes attempt-local cleanup.
Its x86-64 Linux adapter consumes only sealed, already-open controller root
descriptors, uses descriptor-relative child operations, and cannot resolve a
root path. It is not RAR OS target code, does not link into an image, and
contains no process, container, cloud, network, external/release publication,
or launch authority and cannot acquire filesystem roots autonomously.

The side-effect-free `attempt` module implements structural encoders and decoders
for the experimental active, transition, and recovery-inventory records. It
checks wire sizes, field values, local canonical ordering, reserved bytes, and
record hashes. It deliberately does not authorize a transition chain, session
takeover, cleanup, or inventory origin: those contextual policy APIs remain
absent until they have executable negative tests in the isolated lab. It
performs no journal I/O,
watchdog work, process management, descriptor transfer, root acquisition, or
activation. The future trusted controller remains responsible for durable file
operations and must not treat successful parsing as proof that a write reached
stable storage.

The Linux adapter is the sole module allowed to contain `unsafe` code. Its
documented invariants bind the x86-64 syscall ABI, pointer lifetimes, descriptor
ownership, bounded directory-record parsing, root-purpose attestation, and
exclusive controller ownership of cleanup roots. It may be connected only by
the trusted controller after that controller issues a sealed stopped-producer
token and root-specific attestations. Those tokens remain crate-internal and no
controller or executable entry point exists.

Deadline and cancellation boundaries limit user-space work but cannot interrupt
an in-progress kernel synchronization call. The eventual trusted controller
must run the helper under a bounded outer watchdog, use local non-network/non-
FUSE task storage, and recover or discard attempt-local roots from controller-
owned persistent state after forced termination. Until isolated Linux tests,
real identities, that recovery path, and independent review exist, the v2
Development Lab remains blocked and this library cannot activate a probe.

Source tests contain structural round trips and three language-neutral prehash
header skeletons for future isolated execution. The separate shell validator
continues to bind all 97 declarative attempt cases; the Rust codec does not yet
claim behavioral coverage of them,
but repository gates do not compile or execute changed Rust code on the Mac.
Test execution remains blocked until an isolated cloud host compiler identity
and closure are reviewed and pinned.
