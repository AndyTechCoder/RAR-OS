# Public laboratory executable packages and virgin System fixture

This is pure host-side construction for the existing public RFC8032 laboratory
root. It is not production signing, publisher authentication, a target
dependency, guest formatting or permission to modify a real disk.

settings_packages.py accepts bounded already-built Settings PE bytes, an exact
nonzero source SHA and a 32-byte build-provenance digest. It calls the existing
bounded service PE inspector before constructing the exact Modern manifest.
The actual image budget is derived from inspected bytes. Fixed variants are
factory generation1/ABI1, update generation2/ABI1, failed-health generation3/ABI1,
bad-signature generation2/ABI1, and signed bad-ABI generation2/ABI0. The caller
must build the matching code variants; a manifest label does not prove code
behavior. The bad-signature mutation is explicit and deterministic.

The factory_system function returns a fresh 8MiB byte value containing exactly
one factory selector and its A-slot package. All unused selector/slot/reserved
bytes are zero. It has no input representing an existing disk, no file/device
API and no Data argument. Factory bytes must equal a freshly reconstructed
canonical signed factory with the same source/build identity. That comparison
is fixture consistency, not independent crypto evidence. The immutable boot
copy must remain separately bound to the same package.

Cloud source self-tests cover all fixed variants, signature/digest/provenance,
exact geometry, zero regions, deterministic output and malformed/substituted
inputs. Synthetic PE bytes are never executed. No actual Settings compilation,
package publication, cloud workflow activation, production security or M4.2
completion is established by these tests.

Before activation the trusted cloud build must compile each exact source cfg,
inspect final target bounds, sign those actual bytes, bind the five embedded
input hashes/order/logical+padded extents, independently verify final PE
read-only/non-executable and pairwise-disjoint placement, and retain its
reproduction/provenance. The runtime additionally checks all input geometry,
zero padding and pairwise physical disjointness before creating any System user
input mapping. Effective page-table and actual signed-update behavior still
require the certified disposable VM proof.
