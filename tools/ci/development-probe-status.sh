#!/bin/sh
set -eu

probe_status=${1-}
log_status=${2-}
for value in "$probe_status" "$log_status"; do
    case "$value" in '' | *[!0-9]*) exit 64 ;; esac
    [ "$value" -le 255 ] || exit 64
done

if [ "$log_status" -ne 0 ]; then
    printf '%s\n' '74|log-capture-failure'
else
    printf '%s\n' "$probe_status|probe"
fi
