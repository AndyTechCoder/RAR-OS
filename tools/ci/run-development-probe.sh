#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

probe=${1-}
case "$probe" in
    host-static)
        tools/ci/check-sprint-static.sh
        ;;
    milestone-a)
        tools/ci/run-cloud-target-probe.sh milestone-a
        ;;
    *)
        echo "unsupported development probe: $probe" >&2
        exit 64
        ;;
esac
