# Controller handoff core

This dependency-free host library implements the deterministic SHA-256,
fixed 256-byte handoff-manifest codec, typed phase/output plans, and the safe
transaction policy defined under `spec/alpha/lab/`. The transaction core is
parameterized over descriptor operations and includes attempt-local cleanup;
it cannot open a file by itself. It is not RAR OS target code, does not link
into an image, and contains no container, cloud, Linux syscall, publication,
or launch authority.

The later Linux descriptor adapter must implement the reviewed stop/open/copy/
recheck contract without weakening it and may be connected only by the trusted
controller after it issues a sealed stopped-producer token. Until that adapter,
its isolated tests, real identities, and independent review exist, the v2
Development Lab remains blocked and this library cannot activate a probe.

Nine unit tests and one language-neutral golden manifest vector are present,
but repository gates do not compile or execute changed Rust code on the Mac.
Test execution remains blocked until an isolated cloud host compiler identity
and closure are reviewed and pinned.
