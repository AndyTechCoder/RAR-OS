# Host-safety negative suite

Run `tests/host-safety/run.sh` from the repository checkout root. It compiles and executes only
host Rust test code. The suite never invokes an emulator, firmware, target binary, or RAR
artifact.

Coverage includes strict profile parsing, command allowlisting, raw and host paths, resource
bounds, networking, host sharing, passthrough, clipboard, elevation, acceleration, aliases,
malformed records, missing and mismatched pins, certification and authorization binding,
bounded inputs, content-addressed paths, canonical-root enforcement, real-file and symlink
refusal, streaming artifact/firmware hashing, actual Gregorian timestamp validation,
resolver-lie/nonexistent/symlink/wrong-byte cases, stable descriptor identity, same-handle
continuity for artifact, firmware, disk, and emulator into the pathless mock-spawner boundary,
post-verification pathname replacement races for all four resources, staged-descriptor atomic
output verification, parent-directory synchronization, injected write/fsync/rename/unlink
failures, cleanup-error propagation, competing-writer preservation, replace-after-commit
preservation, Darwin/Linux FFI type and mode checks, SHA-256 differential chunks (1, 7, 63,
and 65 bytes), padding boundaries, deterministic randomized short reads, injected read faults,
and resolver/spawner call counts. The suite currently contains 36 tests on macOS; the
platform-gated Linux ABI test runs on Linux.
