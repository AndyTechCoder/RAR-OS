#!/bin/sh
set -eu
path=${1-} expected=${2-}
[ -f "$path" ] && [ ! -L "$path" ] || exit 1
resolved=$(/usr/bin/readlink -f -- "$path" 2>/dev/null || /usr/bin/realpath "$path") || exit 1
[ "$resolved" = "$path" ] || exit 1
case "$expected" in *[!0-9a-f]*) exit 1 ;; esac
[ "${#expected}" -eq 64 ] || exit 1
if command -v sha256sum >/dev/null 2>&1; then actual=$(sha256sum "$path"); else actual=$(/usr/bin/shasum -a 256 "$path"); fi
[ "${actual%% *}" = "$expected" ] || exit 1
