# Controller handoff core

This dependency-free host library implements only the deterministic SHA-256 and
fixed 256-byte handoff-manifest codec defined under `spec/alpha/lab/`. It is not
RAR OS target code, does not link into an image, and contains no container,
cloud, filesystem-copy, publication, or launch authority.

The later Linux descriptor layer must implement the reviewed stop/open/copy/
recheck contract without weakening it. Until that layer, its tests, real image
identities, and independent review exist, the v2 Development Lab remains
blocked and this library cannot activate a probe.

Seven unit tests and one language-neutral golden manifest vector are present,
but repository gates do not compile or execute changed Rust code on the Mac.
Test execution remains blocked until an isolated cloud host compiler identity
and closure are reviewed and pinned.
