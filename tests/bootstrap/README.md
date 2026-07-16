# Bootstrap host suite

`tests/bootstrap/run.sh` is executable only inside the digest-pinned Linux CI root. The physical Mac verifies the proposed lock and closure, then refuses before compiler execution because no descriptor-bound Mach-O launcher is approved.

The suite covers the closed command surface; refusal routes; version-3 lock parsing; bounded shell and Rust record loading; platform-specific lock selection; complete bootstrap-closure checks; wrong-byte compiler/linker/driver/stdlib canaries; poisoned-`PATH` execution of every accepted route, including recursive-safe `test`; captured-script pathname replacement; real Git commit/tree and clean-worktree binding; source/lock mutation faults; versioned renderer contracts; deterministic plan regeneration; output confinement and cleanup; dependency inventory; and blocked image/evidence states.

Only host Rust test code and the approved immutable CI closure execute. No downloader, `rustup`, target linker, emulator, firmware, target binary, boot image, VM image, physical device, or RAR target artifact executes.
