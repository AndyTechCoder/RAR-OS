#!/bin/sh
set -eu

controller_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$controller_root"

probe=${1-}
case "$probe" in
    milestone-a | milestone-b | milestone-c | milestone-d | milestone-e | milestone-f | milestone-g)
        /bin/sh tools/ci/check-development-controller-v2.sh >/dev/null || {
            printf '%s\n' 'development probe blocked: v2 controller contract is invalid' >&2
            exit 73
        }
        printf '%s\n' 'development probe blocked: v2 controller is reviewed but inactive; v1 is permanently retired' >&2
        exit 73
        ;;
    *)
        echo "unsupported development probe: $probe" >&2
        exit 64
        ;;
esac
