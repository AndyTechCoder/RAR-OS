# Release 0 contract fixture corpus

`cases.v1` is the canonical decoded-field fixture transport for R0-002. Each row is an independent fixture. Numeric values are the exact unsigned wire values a decoder observes after bounded little-endian reads; no fixture is a native Rust structure and no address is dereferenced.

The corpus contains valid x86-64 and AArch64 descriptions with a shared semantic identity, a maximum-size boundary case, and the required truncated, oversized, misaligned, overlapping, unknown-critical, invalid-pointer, and architecture-inconsistent failures. `run.sh` is a host-only conformance oracle using only POSIX shell builtins. It performs integer and range validation without compiling or executing RAR target code.

The compact decoded-field transport is provisional to Release 0. R0-009 may add raw byte corpora after independent decoders exist, but must preserve these cases and expected stable failure classes.
