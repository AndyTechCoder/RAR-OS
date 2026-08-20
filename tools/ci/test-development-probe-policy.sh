#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

[ "$(tools/ci/development-probe-status.sh 0 0)" = '0|probe' ]
[ "$(tools/ci/development-probe-status.sh 73 0)" = '73|probe' ]
[ "$(tools/ci/development-probe-status.sh 0 1)" = '74|log-capture-failure' ]
[ "$(tools/ci/development-probe-status.sh 73 1)" = '74|log-capture-failure' ]

set +e
tools/ci/run-development-probe.sh milestone-a >/dev/null 2>&1
status=$?
set -e
[ "$status" -eq 73 ]

echo "development probe policy checks passed"
