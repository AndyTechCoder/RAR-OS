# x86-64 Prompt 7A Static Candidate

Status: non-executing certification preparation

The sole candidate profile is `r0-x86_64-preauth-v1`: x86-64, QEMU `q35`, TCG, two CPUs, 512 MiB RAM, OVMF code and disposable variables/disk, serial stdio, no display, no network, no host sharing, no passthrough, no clipboard, no elevation, emulator sandbox required, 60-second timeout, 5-second termination grace, and 1 MiB output cap. AArch64 and Tier 0 remain uncertified and unauthorized.

`preauth_entry.S` is a deliberately bounded non-executing R0-003 fragment. It fixes the R0-002 x86-64 register ABI (`RDI` address and `RSI` length), validates the pre-copy null/length/alignment envelope, and contains no source acquisition, mappings, interrupt setup, timer setup, device authority, networking, storage service, IPC, or later-release behavior. The static artifact is not evidence that boot works and is never loaded during Prompt 7A.

The assembly invariant is that `_start` is entered only in 64-bit mode with a 16-byte-aligned writable stack, interrupts disabled or immediately disabled, direction flag clear, and Root/Recovery retaining the immutable DMA-revoked entry slice. This fragment does not dereference `RDI`; the complete adapter remains subject to Prompt 7 review and R0-003 execution evidence after separate authorization.

The build uses only `/usr/bin/as` from the pinned base OCI closure and `/usr/bin/ld.lld-19` from the signed Debian snapshot closure. It emits no dynamic interpreter, no build ID, and no target-linked dependency. CI builds it twice in distinct clean output directories and compares the ELF bytes before static metadata inspection.

The host closure is separately exported twice with fixed timestamps and no cache. CI requires byte-identical Docker archives and matching image digests before loading the derived image. Push and pull-request workflows bind every evidence record to the explicitly checked-out commit; event merge refs and mismatches fail before acquisition. These operations execute host compilers, static inspectors, and test doubles only. They never execute the candidate, QEMU, firmware, an emulator, a VM, a guest, a device, or AWS authority.
