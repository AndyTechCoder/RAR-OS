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
        driver=tools/sprint-alpha/probe-milestone-a.sh
        if [ ! -x "$driver" ]; then
            printf '%s\n' \
                'probe=milestone-a' \
                'result=refused' \
                'reason=milestone-a-driver-not-implemented' \
                'target_execution=not-attempted'
            exit 73
        fi
        "$driver"
        ;;
    *)
        echo "unsupported development probe: $probe" >&2
        exit 64
        ;;
esac
