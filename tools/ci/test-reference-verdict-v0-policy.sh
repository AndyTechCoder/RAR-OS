#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'reference verdict mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/reference-verdict.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-reference-verdict-v0.sh
fixtures=$root/spec/alpha/lab/fixtures
accepted=$root/spec/alpha/lab/fixtures/reference-verdict-accepted.v0
not_required=$root/spec/alpha/lab/fixtures/reference-verdict-not-required.v0
controller=$fixtures/controller-context.v0
source=$fixtures/source-context.v0
transcript=$fixtures/comparison-transcript.v0
inventory=$fixtures/reference-inventory.v0
evidence=$fixtures/comparison-evidence.v0
harness=$fixtures/reference-harness.v0
candidate=$work/verdict.v0

expect_rejected() {
    label=$1
    shift
    if /bin/sh "$checker" "$candidate" "$@" >/dev/null 2>&1; then
        printf 'unsafe reference verdict unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

/bin/sh "$checker" "$accepted" milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness" >/dev/null
/bin/sh "$checker" "$not_required" milestone-a "$controller" "$source" "$transcript" none none none >/dev/null

/usr/bin/sed 's/^probe=milestone-f$/probe=milestone-e/' "$accepted" > "$candidate"
expect_rejected accepted-wrong-probe milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/usr/bin/sed 's/^reference_inventory_sha256=.*/reference_inventory_sha256=0000000000000000000000000000000000000000000000000000000000000000/' "$accepted" > "$candidate"
expect_rejected accepted-zero-inventory milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/usr/bin/sed 's/^record_count=1$/record_count=0/' "$accepted" > "$candidate"
expect_rejected accepted-zero-records milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/usr/bin/sed 's/^record_count=1$/record_count=01/' "$accepted" > "$candidate"
expect_rejected accepted-leading-zero milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/usr/bin/sed 's/^reference_2_result=match$/reference_2_result=mismatch/' "$accepted" > "$candidate"
expect_rejected reference-disagreement milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/usr/bin/sed 's/^probe=milestone-a$/probe=milestone-f/' "$not_required" > "$candidate"
expect_rejected skipped-required-reference milestone-f "$controller" "$source" "$transcript" none none none
/usr/bin/sed 's/^comparison_evidence_sha256=0.*/comparison_evidence_sha256=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee/' "$not_required" > "$candidate"
expect_rejected not-required-evidence-present milestone-a "$controller" "$source" "$transcript" none none none
/usr/bin/awk 'NR == 10 { first=$0; next } NR == 11 { print; print first; next } { print }' "$accepted" > "$candidate"
expect_rejected reordered-fields milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/bin/cp "$accepted" "$candidate"
expect_rejected expected-probe-context-mismatch milestone-g "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
expect_rejected wrong-controller-context milestone-f "$source" "$source" "$transcript" "$inventory" "$evidence" "$harness"
expect_rejected wrong-source-context milestone-f "$controller" "$controller" "$transcript" "$inventory" "$evidence" "$harness"
expect_rejected wrong-transcript-context milestone-f "$controller" "$source" "$source" "$inventory" "$evidence" "$harness"
expect_rejected wrong-inventory-context milestone-f "$controller" "$source" "$transcript" "$source" "$evidence" "$harness"
expect_rejected wrong-evidence-context milestone-f "$controller" "$source" "$transcript" "$inventory" "$source" "$harness"
/usr/bin/sed 's/^controller_sha256=.*/controller_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' "$accepted" > "$candidate"
expect_rejected fabricated-digest milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
size=$(/usr/bin/stat -c %s "$accepted" 2>/dev/null || /usr/bin/stat -f %z "$accepted")
/bin/dd if="$accepted" of="$candidate" bs=1 count=$((size - 1)) 2>/dev/null
expect_rejected missing-terminal-lf milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"
/bin/cp "$accepted" "$work/real.v0"
/bin/rm -f "$candidate"
/bin/ln -s real.v0 "$candidate"
expect_rejected symbolic-verdict milestone-f "$controller" "$source" "$transcript" "$inventory" "$evidence" "$harness"

printf '%s\n' 'reference verdict negative checks passed'
