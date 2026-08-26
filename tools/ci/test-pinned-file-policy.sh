#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root=$root/out
/bin/mkdir -p "$output_root"
work=$(mktemp -d "$output_root/pinned-file.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
file=$work/tool
/usr/bin/printf '%s\n' original > "$file"
hash_output=$(/usr/bin/shasum -a 256 "$file")
hash=${hash_output%% *}
/bin/sh "$root/tools/ci/verify-pinned-file.sh" "$file" "$hash"
/usr/bin/printf '%s\n' mutation > "$file"
if /bin/sh "$root/tools/ci/verify-pinned-file.sh" "$file" "$hash"; then exit 1; fi
/usr/bin/printf '%s\n' original > "$file"
/bin/ln -s tool "$work/link"
if /bin/sh "$root/tools/ci/verify-pinned-file.sh" "$work/link" "$hash"; then exit 1; fi
if /bin/sh "$root/tools/ci/verify-pinned-file.sh" "$work/missing" "$hash"; then exit 1; fi
printf '%s\n' 'pinned file negative checks passed'
