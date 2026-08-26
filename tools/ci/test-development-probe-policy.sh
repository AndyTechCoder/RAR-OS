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

for milestone in b c d e f g; do
    set +e
    tools/ci/run-development-probe.sh "milestone-$milestone" >/dev/null 2>&1
    status=$?
    set -e
    [ "$status" -eq 73 ]
done

# Source branches never receive launch authority: there is exactly one QEMU
# execution site, and it is the trusted controller launcher.
[ "$(grep -RIl '^"\$qemu" \\' tools/ci tools/sprint-alpha | wc -l | tr -d ' ')" -eq 1 ]
grep -q '^"$qemu" \\' tools/ci/launch-cloud-target.sh
! grep -q 'RAR_QEMU_' tools/ci/verify-cloud-target-tools.sh
! grep -q 'RAR_FIRMWARE_' tools/ci/verify-cloud-target-tools.sh
! grep -q 'RAR_REFERENCE_' tools/ci/run-cloud-target-probe.sh
! grep -q 'verify_file .*reference' tools/ci/verify-cloud-target-tools.sh
! grep -q 'source_root,target=/workspace' tools/ci/launch-cloud-target.sh

echo "development probe policy checks passed"
