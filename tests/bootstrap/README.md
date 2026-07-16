# Bootstrap host suite

`tests/bootstrap/run.sh` is executable only inside the digest-pinned Linux CI root. The physical Mac verifies the proposed lock and closure, then refuses before compiler execution because no descriptor-bound Mach-O launcher is approved.

The suite covers the closed command surface; refusal routes; version-3 lock parsing; immutable lock-digest authority and shell-to-Rust handoff; bounded shell and Rust record loading; platform-specific lock selection; complete bootstrap-closure checks; read-only CI tool mounts; matching and wrong-byte tool substitutions; poisoned-`PATH` execution of every accepted route; exact Git-blob source materialization; committed-script capture; real SHA-1/SHA-256 Git object-ID validation; hidden-index rejection; commit-tree-derived source hashing; clean-worktree binding; source/lock/tool-probe mutation faults; pre-commit snapshot revalidation; complete certifiability inputs; versioned renderer contracts including image-plan v3; deterministic plan regeneration; output confinement and non-recursive cleanup; strict Class B inventory; and blocked image/evidence states.

Only host Rust test code and the approved immutable CI closure execute. No downloader, `rustup`, target linker, emulator, firmware, target binary, boot image, VM image, physical device, or RAR target artifact executes.
