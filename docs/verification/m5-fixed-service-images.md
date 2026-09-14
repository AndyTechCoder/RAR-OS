# Fixed native Alpha service image separation

The cloud native build at source `9a1b0dc7c9aba5adf73fe1c50e4d10129266e7e2`
(run 34830692116) measured the combined service image at 143360 bytes against
the unchanged 131072-byte limit. It was rejected before guest launch. The earlier
932dde9 source failed the same bound; code-sharing annotations did not fix it.

The native Alpha composition now independently links a fixed network-role PE.
The kernel selects this immutable embedded payload only for the existing role 7,
physical slot 10. All other desktop roles use the existing common service.
The private constructor, PE validation, W^X mapping, incarnation checks, signed
desktop publication, capabilities, retirement and 2 MiB allocation are unchanged.
The network image cannot select a role: its entry refuses anything except role 7.
The normal native-Alpha service no longer dispatches role 7. Earlier compositions
without rar_applications retain the original build and dispatch.

Both service images retain the 128 KiB PE limit. This is essential: IMAGE is at
0x100000 and USER_STACK at 0x120000 within the private stride. Raising the limit
would overlap the stack and is not an acceptable fix.

Cloud-only build scripts transfer exactly three named images in a private
controller envelope. Each is inspected before any VM launch; the network PE
must occur exactly once in the kernel. Both independent build outputs must be
byte-identical. Fault/expiry compiler fixtures apply to the network image only.
No public application format, stable SDK, disk format or trust grant changes.

Pure framing tests cover order, truncation, duplicate/missing embedding,
invalid encoding, transfer bounds and both oversized service images. Object
checks compile the network-only variants. Actual size and guest behavior still
require the exact-source native cloud workflow; source CI is not runtime proof.

Failure receipts remain unconditional failures. Their already-bounded synthetic
diagnostics are printed in the job log (escaped and capped); this permits remote
diagnosis without downloading artifacts to the owner's Mac or SSD. Success
criteria are unchanged. No user data, credential or external network is introduced.
