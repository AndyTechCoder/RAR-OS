# Bootstrap host suite

Run `tests/bootstrap/run.sh` from the repository checkout root. The suite checks the closed
command surface, early refusal of run aliases and execution-capable test modes, strict
tool-lock parsing and bounds, poisoned-PATH wrapper ordering, observed Rust/bundled-LLVM
hashes, synthetic external-pin probes, clean deterministic build-plan regeneration,
explicit evidence configuration/targets, repository-confined outputs, the empty Cargo
workspace, dependency inventory, and blocked image planning.

Only host Rust test code and approved host discovery commands run. No target linker,
emulator, firmware, target binary, image, or RAR artifact is executed.
