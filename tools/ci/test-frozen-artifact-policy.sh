#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'frozen artifact mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/frozen-artifact.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
artifact=$work/artifact
/usr/bin/printf '%s\n' original > "$artifact"
hash_output=$(/usr/bin/shasum -a 256 "$artifact")
hash=${hash_output%% *}
/bin/sh "$root/tools/ci/verify-frozen-artifact.sh" "$artifact" "$hash" >/dev/null
/usr/bin/printf '%s\n' mutated > "$artifact"
if /bin/sh "$root/tools/ci/verify-frozen-artifact.sh" "$artifact" "$hash" >/dev/null 2>&1; then
    printf '%s\n' 'mutated frozen artifact unexpectedly passed' >&2
    exit 1
fi
printf '%s\n' 'frozen artifact negative checks passed'
