#!/bin/sh
set -eu

controller_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$controller_root"

probe=${1-}
case "$probe" in
    milestone-a)
        tools/ci/run-cloud-target-probe.sh milestone-a
        ;;
    *)
        echo "unsupported development probe: $probe" >&2
        exit 64
        ;;
esac
