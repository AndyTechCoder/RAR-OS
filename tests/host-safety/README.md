# Host-safety negative suite

Run `tests/host-safety/run.sh` from the repository checkout root. It compiles and executes only
host Rust test code. The suite never invokes an emulator, firmware, target binary, or RAR
artifact.

Coverage includes strict profile parsing, command allowlisting, raw and host paths, resource
bounds, networking, host sharing, passthrough, clipboard, elevation, acceleration, aliases,
malformed records, missing and mismatched pins, certification and authorization binding,
bounded inputs, content-addressed paths, canonical-root enforcement, real-file and symlink
refusal, fresh artifact/firmware hashing, and resolver/spawner call counts.
