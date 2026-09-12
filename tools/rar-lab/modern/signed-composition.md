# Signed Modern build and factory provisioning

The trusted-main compile controller first reproduces the existing kernel,
supervisor and three standalone Settings code variants. Only after exact
two-build equality does it derive canonical provenance from source SHA,
controller SHA, compiler image identity, all artifact hashes and fixed cfgs.

It generates five public-laboratory signed packages from those exact payloads.
Bad-signature and bad-ABI reuse update code; failed-health uses its separately
compiled code. Factory System media is newly constructed, never inferred from
or written over existing media. It has no Data input. Package hashes, sizes,
padding, build digest and System hash are retained.

A new exclusive signed-inputs directory in disposable cloud evidence holds only
the five fixed files. The compiler receives it and source read-only; credentials,
network, devices, privileges and existing resource bounds are unchanged.
Two fresh containers compile a matched signed supervisor/kernel using the fixed
build-signed.sh recipe. No source-selected path or shell argument is accepted.

Before acceptance the controller verifies every exact padded package occurs
uniquely in one read-only, non-executable section at page-aligned RVA, with
disjoint page extents and initialized zero padding. Both signed artifacts and
bank placements must reproduce. Outputs are signed-modern.efi,
signed-modern-service.efi, modern-system.img and signed-inputs/*.layer, with
exact metadata in manifest.json.

This does not boot an OS or establish physical page-table protection, stack
sufficiency, live updates, fallback, recovery or M4 completion. Only the later
certified VM controller may consume the signed outputs after its own review.
Legacy modern.efi remains an explicitly separate preliminary artifact and must
not be substituted for signed-modern.efi in a signed-runtime test.


### Cloud input ownership correction

Actual compile-only run34645307815 reproduced all five preliminary UEFI inputs
but failed before signed compilation because the runner-owned0700 input
directory was unreadable to compiler UID65532. Generated public laboratory
package files now receive0444 through their exclusively created open
descriptors; after all file writes finish, only their new containing directory
receives0555. Parent paths, source, System image and unrelated files are not
changed. The bind remains read-only and the compiler remains nonroot with no
network or devices. This corrects data readability, not execution authority.
A fresh exact-controller cloud build is still required to prove the fix.


### Signed composition size optimization

Run34647701939 passed input access and compiled the first actual signed pair,
but strict PE inspection rejected its mapped-image budget. No target ran, and
the budget is not raised. The signed recipe now requests size optimization and
one code-generation unit for the signed supervisor; kernel optimization and all Settings package
variants retain their existing independent recipe. Pinned compiler, fixed base,
W^X rules, layout limits and all confinement stay unchanged. Budget failures now
report the measured mapping and fixed limit to make the next result actionable.
Actual signed reproduction and later runtime timing/health remain gates.
