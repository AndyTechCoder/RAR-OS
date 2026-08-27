#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output=$root/out
[ ! -L "$output" ] || exit 1
/bin/mkdir -p "$output"
work=$(mktemp -d "$output/controller-helper-evidence.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
fixtures=$root/spec/alpha/lab/fixtures/controller-helper
build_checker=$root/tools/ci/check-controller-helper-build-evidence-v0.sh
test_checker=$root/tools/ci/check-controller-helper-test-evidence-v0.sh
build_source=$fixtures/build-evidence.v0
test_source=$fixtures/test-evidence.v0
controller=1111111111111111111111111111111111111111

run_build() {
    /bin/sh "$build_checker" "$1" "$root" "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$9" "${10}" "${11}" "${12}" "${13}" "${14}" "${15}" "${16}" "${17}" "${18}" "${19}" "${20}" "${21}"
}
canonical_build() {
    run_build "$1" adr-0024-alternative-a runner-closure "$controller" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/build-plan.v0" "$fixtures/golden-vector.v0" "$fixtures/builder-inventory.v0" "$fixtures/compiler-closure.v0" "$fixtures/compiler.v0" "$fixtures/helper-build-1.v0" "$fixtures/helper-build-2.v0" "$fixtures/helper-final.v0" "$fixtures/build-1-receipt.v0" "$fixtures/build-2-receipt.v0" "$fixtures/build-1.log.v0" "$fixtures/build-2.log.v0" "$fixtures/test-evidence.v0" "$fixtures/test-cases.v0" "$fixtures/test.log.v0"
}
reject_build_record() { label=$1; candidate=$2; if canonical_build "$candidate" >/dev/null 2>&1; then printf 'unsafe helper build evidence passed: %s\n' "$label" >&2; exit 1; fi; }
reject_build_call() { label=$1; shift; if run_build "$@" >/dev/null 2>&1; then printf 'unsafe helper build context passed: %s\n' "$label" >&2; exit 1; fi; }
run_test() { /bin/sh "$test_checker" "$1" "$root" "$controller" "$2" "$3" "$4" "$5" "$6" "$7"; }
reject_test_call() { label=$1; shift; if run_test "$@" >/dev/null 2>&1; then printf 'unsafe helper test context passed: %s\n' "$label" >&2; exit 1; fi; }
sha_file() { env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }

canonical_build "$build_source" >/dev/null
run_test "$test_source" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/helper-final.v0" "$fixtures/golden-vector.v0" "$fixtures/test-cases.v0" "$fixtures/test.log.v0" >/dev/null

build_candidate=$work/build.v0
for mutation in \
    'decision=adr-0024-alternative-b' \
    'topology=controller-tool-image' \
    'controller_sha=2222222222222222222222222222222222222222' \
    'build_2_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' \
    'binary_bytes=027' 'network=host' 'build_count=1' 'reproducible=no' 'status=rejected'; do
    key=${mutation%%=*}
    /usr/bin/sed "s/^$key=.*/$mutation/" "$build_source" > "$build_candidate"
    reject_build_record "$key" "$build_candidate"
done
/usr/bin/awk 'NR == 5 { saved=$0; next } NR == 6 { print; print saved; next } { print }' "$build_source" > "$build_candidate"; reject_build_record reordered "$build_candidate"
/bin/cp "$build_source" "$build_candidate"; /usr/bin/printf '%s\n' extra=value >> "$build_candidate"; reject_build_record extra "$build_candidate"
size=$(/usr/bin/stat -f %z "$build_source" 2>/dev/null || /usr/bin/stat -c %s "$build_source")
/bin/dd if="$build_source" of="$build_candidate" bs=1 count=$((size - 1)) 2>/dev/null; reject_build_record missing-lf "$build_candidate"

reject_build_call same-output-path "$build_source" adr-0024-alternative-a runner-closure "$controller" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/build-plan.v0" "$fixtures/golden-vector.v0" "$fixtures/builder-inventory.v0" "$fixtures/compiler-closure.v0" "$fixtures/compiler.v0" "$fixtures/helper-build-1.v0" "$fixtures/helper-build-1.v0" "$fixtures/helper-final.v0" "$fixtures/build-1-receipt.v0" "$fixtures/build-2-receipt.v0" "$fixtures/build-1.log.v0" "$fixtures/build-2.log.v0" "$fixtures/test-evidence.v0" "$fixtures/test-cases.v0" "$fixtures/test.log.v0"
/bin/cp "$fixtures/helper-build-1.v0" "$work/hardlink-source.v0"
if /bin/ln "$work/hardlink-source.v0" "$work/hardlink-alias.v0" 2>/dev/null; then
    reject_build_call hardlinked-output "$build_source" adr-0024-alternative-a runner-closure "$controller" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/build-plan.v0" "$fixtures/golden-vector.v0" "$fixtures/builder-inventory.v0" "$fixtures/compiler-closure.v0" "$fixtures/compiler.v0" "$work/hardlink-source.v0" "$work/hardlink-alias.v0" "$fixtures/helper-final.v0" "$fixtures/build-1-receipt.v0" "$fixtures/build-2-receipt.v0" "$fixtures/build-1.log.v0" "$fixtures/build-2.log.v0" "$fixtures/test-evidence.v0" "$fixtures/test-cases.v0" "$fixtures/test.log.v0"
fi

receipt_2=$work/receipt-2.v0
/usr/bin/sed 's/^job_nonce=.*/job_nonce=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/; s/^root_nonce=.*/root_nonce=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc/' "$fixtures/build-2-receipt.v0" > "$receipt_2"
receipt_hash=$(sha_file "$receipt_2")
/usr/bin/sed "s/^build_2_receipt_sha256=.*/build_2_receipt_sha256=$receipt_hash/" "$build_source" > "$build_candidate"
reject_build_call reused-build-root "$build_candidate" adr-0024-alternative-a runner-closure "$controller" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/build-plan.v0" "$fixtures/golden-vector.v0" "$fixtures/builder-inventory.v0" "$fixtures/compiler-closure.v0" "$fixtures/compiler.v0" "$fixtures/helper-build-1.v0" "$fixtures/helper-build-2.v0" "$fixtures/helper-final.v0" "$fixtures/build-1-receipt.v0" "$receipt_2" "$fixtures/build-1.log.v0" "$fixtures/build-2.log.v0" "$fixtures/test-evidence.v0" "$fixtures/test-cases.v0" "$fixtures/test.log.v0"

test_candidate=$work/test.v0
for mutation in \
    'controller_sha=2222222222222222222222222222222222222222' \
    'test_count=07' 'failed_count=1' 'network=host' 'observed_exit_status=1' 'status=rejected'; do
    key=${mutation%%=*}
    /usr/bin/sed "s/^$key=.*/$mutation/" "$test_source" > "$test_candidate"
    reject_test_call "$key" "$test_candidate" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/helper-final.v0" "$fixtures/golden-vector.v0" "$fixtures/test-cases.v0" "$fixtures/test.log.v0"
done
bad_cases=$work/cases.v0
/usr/bin/awk 'NR != 4' "$fixtures/test-cases.v0" > "$bad_cases"
cases_hash=$(sha_file "$bad_cases")
/usr/bin/sed "s/^case_results_sha256=.*/case_results_sha256=$cases_hash/" "$test_source" > "$test_candidate"
reject_test_call missing-case "$test_candidate" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/helper-final.v0" "$fixtures/golden-vector.v0" "$bad_cases" "$fixtures/test.log.v0"
/usr/bin/sed '4s/.*/streaming-boundaries-match-one-shot|pass/' "$fixtures/test-cases.v0" > "$bad_cases"
cases_hash=$(sha_file "$bad_cases")
/usr/bin/sed "s/^case_results_sha256=.*/case_results_sha256=$cases_hash/" "$test_source" > "$test_candidate"
reject_test_call duplicate-case "$test_candidate" "$fixtures/runner-image.v0" "$fixtures/source-tree.v0" "$fixtures/helper-final.v0" "$fixtures/golden-vector.v0" "$bad_cases" "$fixtures/test.log.v0"

printf '%s\n' 'controller helper evidence negative checks passed'
