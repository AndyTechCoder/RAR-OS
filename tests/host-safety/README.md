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
continuity into the mock spawner, and resolver/spawner call counts. The suite currently
contains 21 tests.
