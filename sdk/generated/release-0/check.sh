#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
map=$root/sdk/generated/release-0/generation-v1.map
generated=$root/sdk/generated/release-0/lib.rs

while IFS='|' read -r source token; do
    [ -n "$source" ] || continue
    [ -f "$root/$source" ] || { echo "missing generation source: $source" >&2; exit 1; }
    found=false
    while IFS= read -r line; do
        case "$line" in *"$token"*) found=true; break ;; esac
    done < "$generated"
    [ "$found" = true ] || { echo "generated token missing: $token" >&2; exit 1; }
done < "$map"

echo "R0-002 generated Rust mapping passed"
