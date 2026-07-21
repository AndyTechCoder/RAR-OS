#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
[ "$#" -eq 1 ] || { printf '%s\n' 'repository-snapshot:usage' >&2; exit 1; }
destination=$1
case "$destination" in /*) :;; *) printf '%s\n' 'repository-snapshot:absolute-destination' >&2; exit 1;; esac
destination_parent=$(dirname -- "$destination")
canonical_parent=$(CDPATH= cd -- "$destination_parent" && pwd -P) || exit 1
case "$canonical_parent/" in "$root/"*) printf '%s\n' 'repository-snapshot:repository-destination' >&2; exit 1;; esac
cd "$root"
find . -xdev -print | LC_ALL=C sort | while IFS= read -r state_entry; do
    if stat -c '%F|%a|%u|%g' "$state_entry" >/dev/null 2>&1; then
        state_metadata=$(stat -c '%F|%a|%u|%g' "$state_entry")
    else
        state_metadata=$(stat -f '%HT|%Lp|%u|%g' "$state_entry")
    fi
    if [ -L "$state_entry" ]; then
        state_detail="link:$(readlink "$state_entry")"
    elif [ -f "$state_entry" ]; then
        state_detail="file:$(sha256sum "$state_entry" | cut -d ' ' -f 1)"
    elif [ -d "$state_entry" ]; then
        state_detail=directory
    else
        state_detail=other
    fi
    printf '%s|%s|%s\n' "$state_entry" "$state_metadata" "$state_detail"
done > "$destination"
