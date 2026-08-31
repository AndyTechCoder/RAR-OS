# Sprint Alpha Compact PCI BDF Integration Task Packet

Status: Non-authoritative preparation — ADR 0031 owner decision required

This packet prepares the bounded work needed if the owner approves ADR 0031
Alternative A. It is not an ADR, approval record, public-format authority,
implementation authorization, readiness claim, or permission to build, image,
boot, launch, or execute RAR OS. The current P0 compact-BDF candidate remains
opaque and PR #79 remains blocked until the complete sequence below passes.

## Objective

After exact owner approval, make the disabled-function BDF encoding total,
language-neutral, independently reconstructable by Root and Recovery, and
covered by trusted positive, negative, mutation, and exact-main evidence.

The intended Alternative A formula is recorded only as proposal content until
approval:

`bdf_u16 = (bus << 8) | (device << 3) | function`

with checked inputs `bus <= 255`, `device <= 31`, `function <= 7`, serialized
as little-endian `u16`. This packet cannot select that formula.

## Exact owner-decision gate

No integration step may begin until the owner provides this exact sentence:

`I approve ADR 0031 Alternative A for experimental Alpha compact PCI BDF encoding under the documented safety limits.`

Approval closes only the architecture choice. It does not authorize a writer,
merge, target source, build, VM, device, or execution. Generic approval,
recommendation text, passing CI, existing candidate bytes, or this packet never
substitutes for the exact decision.

## Ordered ownership and trust boundaries

Only one writer operates at a time.

### D0 — Canonical decision integration

The architecture/governance writer owns only:

- `docs/adr/0031-alpha-compact-pci-bdf-encoding.md`;
- `docs/proposals/0031-alpha-compact-pci-bdf-encoding.md`;
- the exact ADR 0031 rows in `docs/approval-record.md`;
- `docs/README.md`;
- `docs/sprint-alpha-dashboard.md`;
- `SPRINT_STATUS.md`;
- `docs/tasks/sprint-alpha-compact-bdf-integration.md` plus its exact status/hash assertions in
  `tools/ci/check-specs.sh`; and
- narrow classifier enforcement in `tools/ci/check-specs.sh`.

D0 must preserve the exact approved sentence, Alternative A, approval date,
approver identity, and experimental scope. It receives architecture,
correctness, and security review, merges separately, and passes exact-main
validation before any contract writer starts.

### P0-A — Contract and canonical fixture integration

After D0 exact-main validation, the P0 contract writer owns only:

- `spec/alpha/boot/alpha-machine-closure-v0.fields`;
- `spec/alpha/boot/cases.v0`;
- `spec/alpha/platform/alpha-validation-v0.fields`;
- `spec/alpha/platform/cases.v0`;
- `spec/alpha/platform/precedence.v0`;
- `spec/alpha/platform/fixtures/v0/closure-record.hex`;
- `spec/alpha/platform/fixtures/v0/closure-record.fixture`;
- `spec/alpha/platform/fixtures/v0/compact-bdf-vectors.fixture`;
- `spec/alpha/platform/fixtures/v0/wire-authority.fixture`;
- `spec/alpha/platform/fixtures/manifest.v0`;
- `spec/alpha/platform/contract-set-v0.manifest`;
- `docs/sprint-alpha-dashboard.md`; and
- `SPRINT_STATUS.md`.

The P0 contract writer must:

1. add the total compact formula, checked input ranges, little-endian encoding,
   and rejection behavior to `alpha-machine-closure-v0.fields`;
2. state that the compact `u16` is private Alpha v0 closure framing and is not
   a general PCI identifier or permission to access PCI configuration space;
3. reconstruct the ten disabled-function records in the already declared
   bus-master-disable order and require the fixed AHCI value `0x00fa`;
4. independently recompute the complete 136-byte disabled-vector preimage and
   its SHA-256 instead of trusting the existing candidate digest;
5. bind the resulting candidate closure bytes, fixture manifest, contract
   manifest, identities, and every affected size/digest exactly; and
6. keep machine activation blocked on the retained firmware/q35/AHCI evidence
   and all existing P0 gates.

P0-A may not change record sizes, field offsets, function order, PCI inventory
encoding, AHCI authority, tier meaning, persistent data, or any accepted ADR.
Any such need stops for a new ADR.

### T0 — Trusted checker bootstrap

The trusted-controller writer owns only:

- `tools/ci/check-alpha-wire-fixtures.sh`;
- `tools/ci/check-alpha-boot-platform-contracts.sh`; and
- `tools/ci/test-alpha-boot-platform-contract-policy.sh`.

It must:

- replace the opaque 32-byte comparison with independent preimage construction;
- derive each compact value from explicit bus/device/function literals using
  checked arithmetic, never from a truncated `u32` inventory encoding;
- require exact count, order, uniqueness, AHCI binding, reserved zeros, digest,
  and complete closure framing;
- retain bounded, no-follow, trusted-read-only source handling;
- add direct guarded-ephemeral tests that reach the compact parser; and
- remain dormant on `main` when the exact P0-A topology is absent; activate only
  for the exact pinned P0-A fixture-manifest/contract-set pair; and reject every
  partial, unknown, drifted, or cross-paired topology.

The two independent reconstruction paths are:

1. the trusted production-side fixture reconstruction in
   `tools/ci/check-alpha-wire-fixtures.sh`; and
2. a separately implemented test oracle in
   `tools/ci/test-alpha-boot-platform-contract-policy.sh`.

They must share no reconstruction function or generated preimage. Each reads
the canonical cases from `compact-bdf-vectors.fixture`, independently produces
the exact compact values, serialized bytes, 136-byte disabled-vector preimage,
and digest, and the test requires byte-for-byte and digest agreement.

Before T0 starts, P0-A must publish one immutable checkpoint commit containing
the exact candidate contract, fixture, manifest, case, and digest bytes, with
clean preliminary architecture, correctness, and security review. The handoff
records that commit SHA plus every affected contract,
closure, fixture-manifest, and contract-manifest digest. T0 consumes only that
checkpoint; it may not regenerate or alter candidate contract/fixture bytes.
T0 interprets them only under the accepted ADR and reviewed contract, binds its
trusted checks to those exact reviewed values, and merges before P0-B merges the
resulting trusted `main` back into P0. This ordering breaks the source/controller
pinning cycle. A green deferral with skipped validation or mutation steps is not
evidence.

T0 itself requires sequential independent architecture, correctness, and
security review, guarded mutation success, merge, and exact-main validation
before P0-B may consume it. These preliminary/T0 reviews do not replace the
final integrated P0-B review sequence.

### P0-B — Exact-main sync and final merge gate

After D0 and T0 reach trusted `main`, P0 must merge that exact main revision,
rerun the approved local source-only gate, push a fresh PR event, and require a
guarded run in which both pinned validation and read-only-source mutation steps
show `completed/success`, never `skipped`.

Final reviews run sequentially:

1. architecture;
2. correctness; and
3. security.

Every accepted finding is fixed in one bounded remediation, followed by clean
re-review and a fresh exact-head run. P0 remains draft until all gates pass.

## Required positive vectors

The canonical vector set must prove:

These vectors live as public candidate cases in
`spec/alpha/platform/fixtures/v0/compact-bdf-vectors.fixture`, are registered in
the fixture manifest, and are cross-referenced from `spec/alpha/boot/cases.v0`.

- minimum `00:00.0` encodes as `0x0000`;
- maximum `ff:1f.7` encodes as `0xffff`;
- bus basis `01:00.0` encodes as `0x0100` and bytes `00 01`;
- device basis `00:01.0` encodes as `0x0008` and bytes `08 00`;
- function basis `00:00.1` encodes as `0x0001` and bytes `01 00`;
- fixed AHCI `00:1f.2` encodes as `0x00fa`;
- all ten declared bus-master-capable functions encode to distinct values;
- the declared order has exact `u16` values `0x0008,0x00d0,0x00d1,0x00d2,
  0x00d7,0x00e8,0x00e9,0x00ea,0x00ef,0x00fa` and exact little-endian byte pairs
  `08 00,d0 00,d1 00,d2 00,d7 00,e8 00,e9 00,ea 00,ef 00,fa 00`;
- little-endian bytes round-trip through two independent implementations; and
- independent preimage construction produces the exact canonical digest.

## Required negative and mutation matrix

Each case must reject before authority transfer and preserve an empty effect
log:

- bus `256`, device `32`, function `8`, or any negative numeric field;
- checked-shift or combination overflow;
- shifted-`u32` truncation, byte swapping, big-endian serialization, or use of
  the inventory BDF formula;
- missing, extra, duplicate, reordered, or substituted function;
- duplicate compact output from distinct inputs;
- wrong fixed AHCI value or AHCI appearing outside its declared position;
- count, record-size, header-size, version, domain, reserved-byte, or total-size
  drift;
- any enabled bus-master bit, nonzero port state, or stale disabled-vector hash;
- candidate digest retained while its preimage changes;
- accepted contract with an opaque-only checker path still present;
- checker reconstruction with contract rule removed or altered;
- manifest/contract digest cross-pair, stale pin, partial topology, symlink,
  nonregular input, truncation, uppercase/odd/malformed/oversized hex; and
- approval absent, duplicated, mismatched, wrong alternative, wrong approver,
  wrong date, or proposal still used as canonical authority.

Mutation coverage must demonstrate that each public field and each authority-
sensitive rule is reached beyond outer manifest checks.

## Evidence ledger

The completion report records:

- exact D0, T0, P0 head and merge SHAs;
- exact trusted-main SHA used by each guarded run;
- local approved gate output;
- guarded run IDs and step dispositions for pinned validation and mutation;
- canonical contract, fixture-manifest, contract-manifest, closure, preimage,
  and disabled-vector digests;
- architecture, correctness, and security review dispositions;
- unsafe/assembly impact (`none` unless separately reviewed); and
- remaining retained-cloud machine-activation blockers.

No target build or execution result is expected or authorized by this packet.

## Completion definition

ADR 0031 integration is complete only when:

1. exact owner approval is canonical and machine-validated;
2. the public contract contains a total encoding and rejection rule;
3. two independent paths reconstruct identical canonical bytes and digest;
4. every positive, negative, and mutation case passes in guarded CI;
5. all three independent reviews are clean;
6. P0 is conflict-free, exact-main, green, merged, and separately validated on
   the resulting `main`; and
7. no build, launch, hardware, persistence, or production authority was added.

Until then, the honest state is `owner-decision-required`, compact bytes are
non-authoritative, PR #79 remains draft, and Milestone A cannot begin.
