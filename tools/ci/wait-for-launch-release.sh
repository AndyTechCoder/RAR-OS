#!/bin/sh
set -eu

control=${1-} timeout_seconds=${2-}
[ -d "$control" ] && [ ! -L "$control" ] || exit 1
case "$timeout_seconds" in '' | *[!0-9]*) exit 1 ;; esac
[ "$timeout_seconds" -ge 1 ] && [ "$timeout_seconds" -le 3600 ] || exit 1
counter=0
while [ ! -f "$control/release" ]; do
    counter=$((counter + 1))
    [ "$counter" -le "$timeout_seconds" ] || exit 1
    /bin/sleep 1
done
[ -f "$control/release" ] && [ ! -L "$control/release" ] || exit 1
[ "$(/usr/bin/sed -n '1p' "$control/release")" = release ] || exit 1
