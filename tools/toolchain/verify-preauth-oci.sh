#!/bin/sh
set -eu

[ "$#" -eq 4 ] || {
    echo "usage: verify-preauth-oci.sh <archive-one> <metadata-one> <archive-two> <metadata-two>" >&2
    exit 64
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
allowed=$root/out/r0/preauth/acquisition/derived-build

resolve_input() {
    resolved=$(/usr/bin/realpath -e "$1") || exit 73
    case "$resolved" in "$allowed"/*) ;; *) echo "derived OCI evidence path escape" >&2; exit 73 ;; esac
    [ -f "$resolved" ] && [ ! -L "$1" ] || { echo "invalid derived OCI evidence input" >&2; exit 73; }
    printf '%s\n' "$resolved"
}

archive_one=$(resolve_input "$1")
metadata_one=$(resolve_input "$2")
archive_two=$(resolve_input "$3")
metadata_two=$(resolve_input "$4")

archive_one_before=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$archive_one")
archive_two_before=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$archive_two")
/usr/bin/cmp "$archive_one" "$archive_two" || {
    echo "derived OCI archives are not byte-identical" >&2
    exit 73
}

extract_digest() {
    digest=$(/usr/bin/sed -n 's/.*"containerimage.digest": "\([^"]*\)".*/\1/p' "$1")
    case "$digest" in sha256:*) ;; *) echo "missing derived OCI digest" >&2; exit 73 ;; esac
    hex=${digest#sha256:}
    case "$hex" in *[!0-9a-f]*) echo "invalid derived OCI digest" >&2; exit 73 ;; esac
    [ "${#hex}" -eq 64 ] || { echo "invalid derived OCI digest length" >&2; exit 73; }
    printf '%s\n' "$digest"
}

digest_one=$(extract_digest "$metadata_one")
digest_two=$(extract_digest "$metadata_two")
[ "$digest_one" = "$digest_two" ] || {
    echo "derived OCI metadata digests differ" >&2
    exit 73
}

archive_one_after=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$archive_one")
archive_two_after=$(/usr/bin/stat -c '%d:%i:%s:%Y' "$archive_two")
[ "$archive_one_before" = "$archive_one_after" ] && [ "$archive_two_before" = "$archive_two_after" ] || {
    echo "derived OCI archive mutated during verification" >&2
    exit 73
}

printf 'derived_oci_archive_sha256=%s\n' "$(/usr/bin/sha256sum "$archive_one" | /usr/bin/cut -d ' ' -f 1)"
printf 'derived_oci_digest=%s\n' "$digest_one"
