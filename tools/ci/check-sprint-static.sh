#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

tools/ci/check-specs.sh
/bin/sh -n \
    tools/ci/check-sprint-static.sh \
    tools/ci/run-development-probe.sh \
    tools/ci/run-cloud-target-probe.sh \
    tools/ci/verify-cloud-target-tools.sh \
    tools/ci/development-probe-status.sh \
    tools/ci/test-development-probe-policy.sh \
    tools/rarbuild/rarbuild \
    tools/rarbuild/bootstrap-lib.sh \
    tests/host-safety/run.sh \
    tests/bootstrap/run.sh \
    spec/fixtures/release-0/generate.sh \
    spec/fixtures/release-0/run.sh \
    sdk/generated/release-0/generate.sh \
    sdk/generated/release-0/check.sh

tools/ci/test-development-probe-policy.sh

echo "sprint static checks passed"
