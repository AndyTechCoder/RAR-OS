#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
validator=$root/tools/ci/check-controller-handoff-attempt-v0.sh
contract=$root/spec/alpha/lab/controller-handoff-attempt-v0.fields
cases=$root/spec/alpha/lab/controller-handoff-attempt-cases.v0
fixtures=$root/spec/alpha/lab/fixtures/controller-handoff-attempt-policy

for file in "$fixtures/noncanonical.fields" "$fixtures/noncanonical.cases"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || exit 1
done

/bin/sh "$validator" "$contract" "$cases" >/dev/null
if /bin/sh "$validator" "$fixtures/noncanonical.fields" "$cases" >/dev/null 2>&1; then
    printf '%s\n' 'controller handoff attempt policy failed: accepted a noncanonical contract' >&2
    exit 1
fi
if /bin/sh "$validator" "$contract" "$fixtures/noncanonical.cases" >/dev/null 2>&1; then
    printf '%s\n' 'controller handoff attempt policy failed: accepted a noncanonical case table' >&2
    exit 1
fi

printf '%s\n' 'controller handoff attempt immutable-fixture checks passed'
