# Bootstrap host suite

Run `tests/bootstrap/run.sh` from the repository checkout root. The suite checks the closed
command surface, early refusal of run aliases and execution-capable test modes, version-2
tool-lock parsing and bounded file loading, accepted-route poisoned-`PATH` canaries, pinned
bootstrap roots, direct Git metadata use, truthful pinned/unavailable evidence rendering,
clean deterministic build-plan regeneration, descriptor-relative exclusive output,
parent-replacement resistance, interrupted-write cleanup, the empty Cargo workspace,
dependency inventory, and blocked image planning. The suite currently contains 23 tests.

Only host Rust test code and approved pinned host roots run. No downloader, `rustup`, Git
process, target linker,
emulator, firmware, target binary, image, or RAR artifact is executed.
