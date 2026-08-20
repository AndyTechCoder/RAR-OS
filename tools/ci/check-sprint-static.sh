#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

tools/ci/check-specs.sh
/bin/sh -n \
    tools/ci/check-sprint-static.sh \
    tools/ci/run-development-probe.sh \
    tools/rarbuild/rarbuild \
    tools/rarbuild/bootstrap-lib.sh \
    tests/host-safety/run.sh \
    tests/bootstrap/run.sh \
    spec/fixtures/release-0/generate.sh \
    spec/fixtures/release-0/run.sh \
    sdk/generated/release-0/generate.sh \
    sdk/generated/release-0/check.sh

echo "sprint static checks passed"
