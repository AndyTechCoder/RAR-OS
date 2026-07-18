#!/bin/sh
set -eu

[ "$#" -eq 6 ] || {
    echo "usage: verify-preauth-oci.sh <archive-one> <metadata-one> <image-id-one> <archive-two> <metadata-two> <image-id-two>" >&2
    exit 64
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
verifier=$root/out/r0/preauth/host-tools/preauth-verify-oci
[ -x "$verifier" ] && [ ! -L "$verifier" ] || {
    echo "strict OCI verifier has not been built by the pinned host compiler" >&2
    exit 73
}

exec "$verifier" "$@"
