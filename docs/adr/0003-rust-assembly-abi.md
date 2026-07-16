# ADR 0003: Rust, Assembly, and Stable RAR ABI

Status: Accepted — 2026-07-16

## Context

Trusted target code needs low-level hardware control, memory-safety discipline, cross-architecture portability, and language-neutral boundaries.

## Decision drivers

- Reduce avoidable memory-safety defects.
- Keep unavoidable low-level operations small and reviewable.
- Support Rust and C without exposing compiler-private layouts.
- Preserve a path for future languages.

## Considered options

- **C and assembly throughout:** rejected because it increases the trusted memory-safety burden.
- **Rust ABI as the native contract:** rejected because it is not a stable public boundary.
- **`no_std` Rust, limited assembly, and a stable RAR ABI:** selected.

## Decision

Implement trusted target code primarily in `no_std` Rust. Use small reviewed assembly modules for reset, context switching, architecture registers, and unavoidable low-level operations. Do not expose Rust ABI; use a small stable RAR ABI and RID-generated contracts. Provide Rust and C SDKs.

## Consequences

- Memory-safety defects are reduced but not eliminated.
- Unsafe Rust and assembly require explicit justification and review.
- Rust compiler/LLVM remain bootstrap tools rather than target runtime dependencies.
- C applications interoperate without defining the native architecture.

## Security and data impact

Unsafe and assembly boundaries are trusted attack surface. Public data crosses bounded, validated contracts rather than native structs or unchecked pointers.

## Compatibility and migration

ABI and RID versions evolve independently from compiler versions. Breaking changes require generated bindings, adapters or coordinated migration, and rollback support.

## Validation

- No public contract exposes Rust layout.
- Unsafe and assembly inventories identify invariants and focused tests.
- Rust and C fixtures agree on framing, sizes, ownership, and errors.
- Target artifacts contain no undeclared target-linked runtime dependency.

## Replacement path

Other languages, including a future RAR language, target the same versioned ABI and RID contracts. Implementations may be replaced when conformance and security evidence remain equivalent.
