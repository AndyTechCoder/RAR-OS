# Corrupt-mount unavailable proof

This adds one fixed negative scenario to the existing trusted-main Modern
persistence diagnostic, after its successful two-VM persistence proof. It uses
a separate invocation-owned disconnected container with fresh firmware, one
VM, an exact194-sector synthetic Data image and a separate8 MiB zeroed System
image. The successful persistence images are never mutated or reused here.

The controller provisions a fresh valid empty public-key Data fixture and
flips only the first magic byte in each512-byte header copy. It confirms that
the independent Data oracle rejects those bytes before boot. Device geometry,
preflight, kernel grants, launch arguments and all host confinement stay intact.
No raw disk, host/owner data, passthrough, network or alternate size is added.

The guest receives the fixed Terminal sequence LIST, WRITE NOTE DENIED, LIST,
then Files activation. A typed-command screenshot precedes each Enter, and an
Unavailable screenshot follows it, so identical stale error frames cannot prove
later processing. The two LIST requests remain allowed after the first
Unavailable response; the intervening mutation stays locked at the client.
The test does not claim that the locked WRITE reaches the storage service.

All nine actual frames, QMP input boundaries, readiness, exact preflight/argv,
event stream, terminated VM/backends and role-separated inode identities are
retained and independently rechecked. Data records must contain only the two
guest header reads plus at most one firmware sector0 read, no pending read,
no repeated mount/slot scan, and zero write or flush requests. Whole Data bytes,
zero System hash and immutable boot hash must remain unchanged after teardown.

This proves corrupt-header refusal/no autoformat and sticky unavailable
behavior in this fixed Alpha scenario. It is not System recovery, repair,
general corruption tolerance or physical power-loss safety. The Files/Terminal
device-denial boot checks need to be included in the selected target revision
before this run can also supply their actual guest evidence.

Tests are pure/inert cloud source fixtures: exact envelope, malformed and
rehash attempts, record geometry, no mutation/repeated mount, QMP/frame
causality, visual states and scenario command/cleanup integration. No actual
VM evidence is claimed until the reviewed trusted-main campaign succeeds.
