# Deferred two-lane policy validation prototype

The production ephemeral runner retains the exact approved serial invocation
list and all 29 policy suites. Cloud run34010208569 rejected the proposed
scheduling change because existing source coverage checks require direct suite
invocations. No acceptance check is weakened to activate this optimization.

parallel-policy-tests.py is retained as an inactive prototype with cloud-only
self-tests. It is not called with --run by the production runner. Before future
activation, all source coverage checks must understand and verify the exact
fixed partition, with independent review and full cloud evidence.

Its supervisor bounds live worker groups, log bytes and deadlines. Open inherited
pipes after leader exit lead to timeout. A descendant whose leader has exited
and which closes inherited output pipes is NOT detected; disposable container
teardown, not this helper, bounds that residual. It is not a general descendant
cleanup guarantee or arbitrary-command interface. No parallel speedup is claimed.
