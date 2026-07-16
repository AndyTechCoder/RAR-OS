# Host-safety negative suite

Run `tests/host-safety/run.sh` from the repository checkout root. In the pinned Linux CI
bootstrap it compiles and executes only host Rust test code. Under ADR 0012, local macOS
invocation intentionally returns 2 before compilation until a descriptor-bound launcher exists.
The suite never invokes an emulator, firmware, target binary, or RAR artifact.

Coverage includes strict profile parsing, command allowlisting, raw and host paths, resource
bounds, networking, host sharing, passthrough, clipboard, elevation, acceleration, aliases,
malformed records, missing and mismatched pins, certification and authorization binding,
atomic authorization-consumption protocol, sequential/concurrent replay refusal in hostile-state
test doubles, consumer failure, irreversible consumption after resolver and spawner failures,
bounded inputs, content-addressed paths, canonical-root enforcement, real-file and symlink
refusal, streaming artifact/firmware hashing, actual Gregorian timestamp validation,
resolver-lie/nonexistent/symlink/wrong-byte cases, stable descriptor identity, same-handle
continuity for artifact, firmware, disk, and emulator into the pathless mock-spawner boundary,
post-verification pathname replacement races for all four resources, staged-descriptor atomic
output verification, parent-directory synchronization, injected write/fsync/rename/unlink
failures, cleanup-error propagation, competing-writer preservation, replace-after-commit
preservation, nonblocking special-file refusal, Darwin/Linux FFI type and mode checks, SHA-256 differential chunks (1, 7, 63,
and 65 bytes), padding boundaries, deterministic randomized short reads, injected read faults,
and resolver/spawner call counts. Exact suite totals and platform-gated ABI results are
reported by the exact-head Linux CI job.
