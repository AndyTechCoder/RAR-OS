#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
[ "$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")" = /tmp ] || exit 1
ulimit -f 860160

# Rust compilation remains serial within the existing /build budget.
/bin/sh "$root/tools/ci/check-alpha-crypto-primitives.sh"

# Exactly28 private-fixture suites, split into two bounded, disjoint lanes.
/usr/bin/python3 -I -B "$root/tools/ci/parallel-policy-tests.py" --run

# This suite uses a fixed /build output path and must remain serial.
/bin/sh "$root/tools/ci/test-release-0-reference-harness-policy.sh"

printf '%s\n' 'Ephemeral policy tests passed: executed=29 source=read-only scratch=tmpfs'
