#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

tree=${1-}
scratch=${2-}
[ -d "$tree" ] && [ ! -L "$tree" ] || exit 1
[ -d "$scratch" ] && [ ! -L "$scratch" ] || exit 1
# AppleDouble entries are removable-volume metadata, not source. Keep them out
# of the inventory only when they are regular files. Symlinks, directories, and
# special entries remain invalid even when their names use that prefix.
find "$tree" -type l -print | /usr/bin/grep -q . && exit 1
find "$tree" -name '._*' ! -type f -print | /usr/bin/grep -q . && exit 1
count=$(find "$tree" -type f ! -name '._*' -print | /usr/bin/wc -l | /usr/bin/tr -d ' ')
[ "$count" -ge 2 ] && [ "$count" -le 256 ] || exit 1

work=$(mktemp -d "$scratch/qmp-tree-hash.XXXXXX")
paths=$work/paths
manifest=$work/manifest
cleanup() { /bin/rm -rf "$work"; }
trap cleanup EXIT HUP INT TERM
find "$tree" -type f ! -name '._*' -print | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort > "$paths"
: > "$manifest"
while IFS= read -r relative; do
    case "$relative" in '' | *[!A-Za-z0-9._/-]*) exit 1 ;; esac
    file=$tree/$relative
    [ -f "$file" ] && [ ! -L "$file" ] || exit 1
    size=$(/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file") || exit 1
    [ "$size" -le 1048576 ] || exit 1
    if command -v sha256sum >/dev/null 2>&1; then digest=$(sha256sum "$file"); else digest=$(/usr/bin/shasum -a 256 "$file"); fi
    /usr/bin/printf '%s|%s|%s\n' "$relative" "$size" "${digest%% *}" >> "$manifest"
done < "$paths"
[ "$(/usr/bin/wc -l < "$manifest" | /usr/bin/tr -d ' ')" -eq "$count" ] || exit 1
if command -v sha256sum >/dev/null 2>&1; then result=$(sha256sum "$manifest"); else result=$(/usr/bin/shasum -a 256 "$manifest"); fi
printf '%s\n' "${result%% *}"
