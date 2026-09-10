# Bounded crypto adapter failure probes

Source candidate only. No activation, fault coverage or M4 acceptance is claimed.
The trusted cloud handoff may invoke this only after focused review, source checks
and a trusted-main merge. The existing inventoried target/reference images are
reused; no executable, library, image, mount, network or compiler role is added.

Nine probes cover each existing adapter identity with three failure modes:
- Start an interactive adapter without supplying input, verify it is running,
  and require a real docker-wait transport deadline at 250 ms. Verify the adapter
  is still running afterward, then remove only its verified invocation-owned ID.
- Start the same bounded process, verify running state, send SIGKILL only to its
  verified ID, and require the daemon's exit137/stopped/no-OOM/no-live-PID record.
- Send a valid SHA256(empty) request through the actual attached transport with
  a one-byte stdout ceiling and require its bounded output-limit failure.

These intentionally tighter probe limits do not change production limits.
Expected failures are explicit test outcomes, never crypto comparison results.
Any unexpected result, ambiguous creation, unconfirmed cleanup or evidence
failure aborts the disposable job. There are no retries or cleanup by name.
Ownership must be established before start, signal or removal. Effective
confinement and the fixed user, entrypoint, environment and working directory
are verified before start; observations retain exact before/after configuration.

The timeout probe depends on Docker preserving interactive stdin for a detached
start. Running-state checks make incompatible behavior a test failure, not a
false timeout pass. Source mocks do not prove that daemon behavior. Actual cloud
execution after review is needed. Cleanup affects only created disposable test
containers, never source, images, caches, owner files, the Mac or SSD.

The caller must retain each probe's create, configuration, observed failure and
cleanup records plus the complete campaign result. This is a focused process
failure test, not a sandbox escape assessment, a complete fault matrix or a
production cryptographic-security claim.

## Controller integration

The handoff runs the nine probes once after all 972 comparison calls, reusing
exact inventoried images and durable evidence retention. A probe failure fails
the job in `crypto-failure-probes`; it cannot yield comparison completion.
The source suite covers ordering, image selection, retained evidence and failure
propagation as well as the individual probe state/cleanup negatives. No actual
probe coverage is claimed until the trusted-main cloud run succeeds.
