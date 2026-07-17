# Generated Release 0 Rust contracts

`lib.rs` is the generated `no_std`, unsafe-free semantic representation of `spec/hardware/rhd-v1.fields` and `spec/boot/handoff-v1.fields`. `generate.sh` deterministically renders every `rust-*` declaration from those schemas, and `check.sh` compares the complete rendered output byte-for-byte.

It intentionally has no `repr(C)` structures, raw pointers, parser side effects, allocator dependency, or firmware callbacks. Integer discriminants mirror the wire contract, but Rust layout is never a public format. Consumers must decode explicit little-endian bytes into these owned values only after range validation and bounded copying.

The physical Mac must not compile or execute this target-facing module. `check.sh --compile` is fail-closed to the pinned GitHub Actions Linux route and performs metadata-only host compilation; it does not load or execute RAR target code. R0-003/R0-004 later supply target decoders only after authorization. Changes to either source schema require regeneration and a major/minor compatibility decision.
