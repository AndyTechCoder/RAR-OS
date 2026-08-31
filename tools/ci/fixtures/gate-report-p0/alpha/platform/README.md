# Experimental Alpha Platform Contract Set v0

Status: pending independent review; no implementation, build, image, launch, or
execution authority

This directory defines the private Alpha boundary from Recovery to the first
Core loader and the two initial state services. It implements accepted ADRs
0026 Alternative C, 0028 Alternative A, and 0029 Alternative B without changing
R0-002.

Recovery validates only the outer platform entry and its fixed source roles.
Nucleus validates and maps one fixed Core bootstrap, retains exactly two
identity-bound state slots, and gives Core only one component-source capability
plus two opaque selectors. Core alone parses the component bundle. Each state
service alone parses its matching state image. Core never receives readable
state bytes or a redeem token.

All executable identity is derived from versioned, role-separated preimages
over exact immutable bytes. Executable mappings transition from `RW+NX` to
non-writable `RX` with a TLB flush and no writable alias. Immutable source and
mutable destination regions never alias.

These formats are experimental and replaceable. They provide no persistent
storage, package, update, physical-hardware, production-signing, or owner-data
promise. Milestone C owns component loading and state-slot redemption; Milestone
D owns state import and reconstruction. Neither may reinterpret these bytes or
reopen the boot boundary.

`alpha-validation-v0.fields` is the sole first-failure precedence authority.
Every rejection has an empty externally visible effect log. The final
`contract-set-v0.manifest` binds the reviewed byte set and is not ready until
architecture, correctness, security, mutation, merge, and exact-main gates pass.
