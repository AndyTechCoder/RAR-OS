# Bounded two-lane policy validation

The existing isolated Specifications container, attested runner, source binding,
2CPU/2GiB resource limits, read-only checkout and ephemeral fixture rules are
unchanged. This changes scheduling, not coverage or acceptance criteria.

The primitive suite runs serially first. The28 policy scripts listed literally
in parallel-policy-tests.py then run in two fixed alternating lanes, sequentially
inside each lane with set-eu. Review of04cb08fd found their scratch work independent:
each creates a uniquely named ephemeral fixture copy; none consumes a predecessor's
output or mutates the checkout. The release-0 reference harness stays serial
afterward because it uses the fixed /build/release-0-reference-harness output.
The29-suite success attestation remains after all work succeeds.

The supervisor verifies the28-name count, uniqueness, fixed name syntax and
disjoint complete partition. It collects both lane statuses, so a failure in
one cannot hide the other result or print success. Up to8MiB per lane is retained
in memory and replayed in fixed lane order. No new log files/caches are written;
the existing per-file/scratch limits remain inherited by the suites. A20-minute
lane deadline, output overflow, interruption or start/cleanup failure fails the
step. Both live owned worker process groups receive termination and bounded wait,
then force termination if needed. No unknown PID or shared root is cleaned.

A dead-leader/background-pipe ambiguity cannot become success: it reaches the
deadline, fails the wrapper, and the existing disposable CI container teardown
owns any remaining namespace processes. The supervisor is not a general host
process manager. Tests are fixed trusted validation code in the existing source/
controller authority arrangement, not a new arbitrary-command interface.

Cloud-only supervisor fixtures cover both success logs, each/both lane failures,
deadline, output bound, interruption, process reaping and child-start failure.
The production lane list never accepts filenames or commands from arguments.
Both full source validation and post-merge real lane completion are required;
a timing improvement is not claimed until measured on the actual workflow.
No checks are skipped, no target/VM profile is changed and nothing runs locally.

Focused fixtures also start worker0 before worker1 fails, and verify a nested
child receives the process-group SIGTERM and is reaped by its leader (rather
than being manually killed as a fallback). Two tiny fixture files remain only
in disposable tmpfs. A literal expected-name test preserves the prior28-suite
sequence independently of runtime count/partition checks.
