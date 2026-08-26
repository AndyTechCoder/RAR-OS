#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root=$root/out
[ ! -L "$output_root" ] || exit 1
/bin/mkdir -p "$output_root"
work=$(mktemp -d "$output_root/reference-evidence.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-reference-evidence-v0.sh
fixtures=$root/spec/alpha/lab/fixtures
valid=$fixtures/comparison-evidence.v0
transcript=$fixtures/comparison-transcript.v0
inventory=$fixtures/reference-inventory.v0
harness=$fixtures/reference-harness.v0
candidate=$work/evidence.v0

expect_rejected() {
    label=$1
    evidence_arg=${2-$candidate}
    transcript_arg=${3-$transcript}
    inventory_arg=${4-$inventory}
    harness_arg=${5-$harness}
    if /bin/sh "$checker" "$evidence_arg" "$transcript_arg" "$inventory_arg" "$harness_arg" >/dev/null 2>&1; then
        printf 'unsafe reference evidence unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}
mutate_byte() {
    offset=$1
    value=$2
    /bin/cp "$valid" "$candidate"
    /usr/bin/printf '%s' "$value" | /usr/bin/xxd -r -p | /bin/dd of="$candidate" bs=1 seek="$offset" conv=notrunc 2>/dev/null
}

/bin/sh "$checker" "$valid" "$transcript" "$inventory" "$harness" >/dev/null
/bin/dd if="$valid" of="$candidate" bs=1 count=383 2>/dev/null
expect_rejected truncated
/bin/cp "$valid" "$candidate"
/usr/bin/printf x >> "$candidate"
expect_rejected trailing-byte
for item in '16:81:total-size' '20:02:record-count' '24:01:header-flags' '28:01:header-reserved' '32:00:transcript-binding' '64:00:inventory-binding' '96:00:harness-binding' '128:02:case-id' '132:02:operation' '134:01:target-status' '136:01:reference-1-status' '140:01:output-size' '142:01:record-flags' '144:00:reference-output' '272:00:reference-hash' '368:01:record-reserved'; do
    old_ifs=$IFS; IFS=:
    set -- $item
    IFS=$old_ifs
    mutate_byte "$1" "$2"
    expect_rejected "$3"
done
expect_rejected wrong-transcript "$valid" "$fixtures/source-context.v0" "$inventory" "$harness"
expect_rejected wrong-inventory "$valid" "$transcript" "$fixtures/source-context.v0" "$harness"
expect_rejected wrong-harness "$valid" "$transcript" "$inventory" "$fixtures/source-context.v0"
/bin/cp "$valid" "$work/real.v0"
/bin/rm -f "$candidate"
/bin/ln -s real.v0 "$candidate"
expect_rejected symbolic-evidence

printf '%s\n' 'reference evidence negative checks passed'
