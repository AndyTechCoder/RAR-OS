# Generated Release 0 Rust contracts

`lib.rs` is the generated `no_std`, unsafe-free semantic representation of `spec/hardware/rhd-v1.fields` and `spec/boot/handoff-v1.fields`.

It intentionally has no `repr(C)` structures, raw pointers, parser side effects, allocator dependency, or firmware callbacks. Integer discriminants mirror the wire contract, but Rust layout is never a public format. Consumers must decode explicit little-endian bytes into these owned values only after range validation and bounded copying.

The physical Mac must not compile or execute this target-facing module. Compile validation belongs to the pinned Linux host route; R0-003/R0-004 later supply decoders after authorization. Changes to either source manifest require regeneration and a major/minor compatibility decision.
