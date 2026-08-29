#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

evidence=${1-}; trusted_root=${2-}; expected_decision=${3-}; expected_topology=${4-}; expected_controller=${5-}; runner=${6-}; source_tree=${7-}; build_plan=${8-}; golden=${9-}; shift 9
builder_inventory=${1-}; compiler_closure=${2-}; compiler=${3-}; build_1=${4-}; build_2=${5-}; final_binary=${6-}; receipt_1=${7-}; receipt_2=${8-}; log_1=${9-}; shift 9
log_2=${1-}; test_evidence=${2-}; case_results=${3-}; test_log=${4-}
fail() { printf 'controller helper build evidence rejected: %s\n' "$1" >&2; exit 1; }
case "$expected_decision|$expected_topology" in
    adr-0024-alternative-a\|runner-closure|adr-0024-alternative-b\|repository-binary|adr-0024-alternative-c\|controller-tool-image) ;;
    *) fail 'expected decision/topology mismatch' ;;
esac
case "$expected_controller" in *[!0-9a-f]*|'') fail 'expected controller SHA malformed' ;; esac
[ "${#expected_controller}" -eq 40 ] || fail 'expected controller SHA length invalid'
[ -d "$trusted_root" ] && [ ! -L "$trusted_root" ] || fail 'trusted root missing or symbolic'
trusted_root=$(CDPATH= cd -- "$trusted_root" && pwd -P)
safe_file() {
    file=$1
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
    parent=$(CDPATH= cd -- "$(dirname -- "$file")" && pwd -P) || fail "input parent inaccessible: $file"
    resolved=$parent/$(basename -- "$file")
    case "$resolved" in "$trusted_root"/*) ;; *) fail "input escapes trusted root: $file" ;; esac
    links=$(/usr/bin/stat -c %h "$file" 2>/dev/null || /usr/bin/stat -f %l "$file")
    owner=$(/usr/bin/stat -c %u "$file" 2>/dev/null || /usr/bin/stat -f %u "$file")
    [ "$links" = 1 ] || fail "input is hardlinked: $file"
    [ "$owner" = "$(/usr/bin/id -u)" ] || fail "input is not controller-user owned: $file"
    if find "$file" -perm -022 -print | /usr/bin/grep -q .; then fail "input is group/other writable: $file"; fi
}
for file in "$evidence" "$runner" "$source_tree" "$build_plan" "$golden" "$builder_inventory" "$compiler_closure" "$compiler" "$build_1" "$build_2" "$final_binary" "$receipt_1" "$receipt_2" "$log_1" "$log_2" "$test_evidence" "$case_results" "$test_log"; do safe_file "$file"; done
for pair in "$build_1|$build_2" "$build_1|$final_binary" "$build_2|$final_binary" "$receipt_1|$receipt_2" "$log_1|$log_2"; do
    left=${pair%%|*}; right=${pair#*|}
    [ ! "$left" -ef "$right" ] || fail "aliased independent inputs: $left and $right"
done
size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }
identity() { /usr/bin/stat -c '%d:%i:%s:%h:%u:%Y' "$1" 2>/dev/null || /usr/bin/stat -f '%d:%i:%z:%l:%u:%m' "$1"; }
sha_file() { env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
evidence_before=$(identity "$evidence")
size=$(size_of "$evidence")
[ "$size" -le 4096 ] || fail 'evidence exceeds bound'
last=$(/usr/bin/od -An -tx1 -j $((size - 1)) -N 1 "$evidence" | /usr/bin/tr -d ' \n')
[ "$last" = 0a ] || fail 'evidence lacks terminal LF'
/usr/bin/awk -F '=' '
    BEGIN { split("schema decision topology controller_sha source_tree_sha256 build_plan_sha256 golden_vector_sha256 builder_inventory_sha256 compiler_closure_manifest_sha256 compiler_sha256 build_1_sha256 build_2_sha256 final_binary_sha256 binary_bytes build_1_receipt_sha256 build_2_receipt_sha256 test_evidence_sha256 network build_count reproducible status", order, " ") }
    function reject(message) { print "controller helper build evidence rejected: " message > "/dev/stderr"; bad=1 }
    {
        if (NF != 2 || NR > 21 || $1 != order[NR] || $2 !~ /^[a-z0-9-]+$/) reject("grammar or order invalid at line " NR)
        if (++seen[$1] != 1) reject("duplicate field: " $1)
        value[$1]=$2
    }
    END {
        if (NR != 21) reject("field count invalid")
        if (value["schema"] != "rar-alpha-controller-helper-build-evidence-v0") reject("schema invalid")
        if (value["controller_sha"] !~ /^[0-9a-f]{40}$/) reject("controller SHA invalid")
        zero=sprintf("%064d", 0)
        for (i=5; i<=17; i++) if (i != 14 && (value[order[i]] !~ /^[0-9a-f]{64}$/ || value[order[i]] == zero)) reject("digest invalid: " order[i])
        if (value["binary_bytes"] !~ /^[1-9][0-9]*$/ || value["binary_bytes"] > 16777216) reject("binary byte count invalid")
        if (value["network"] != "none" || value["build_count"] != "2" || value["reproducible"] != "yes" || value["status"] != "accepted") reject("execution result invalid")
        if (value["build_1_sha256"] != value["build_2_sha256"] || value["build_1_sha256"] != value["final_binary_sha256"]) reject("build hashes differ")
        exit bad ? 1 : 0
    }
' "$evidence" || exit 1
field() { /usr/bin/sed -n "s/^$1=//p" "$evidence"; }
[ "$(field decision)" = "$expected_decision" ] && [ "$(field topology)" = "$expected_topology" ] || fail 'decision/topology context mismatch'
[ "$(field controller_sha)" = "$expected_controller" ] || fail 'controller context mismatch'
check_hash() { [ "$(field "$1")" = "$(sha_file "$2")" ] || fail "$1 context mismatch"; }
check_hash source_tree_sha256 "$source_tree"
check_hash build_plan_sha256 "$build_plan"
check_hash golden_vector_sha256 "$golden"
check_hash builder_inventory_sha256 "$builder_inventory"
check_hash compiler_closure_manifest_sha256 "$compiler_closure"
check_hash compiler_sha256 "$compiler"
check_hash build_1_sha256 "$build_1"
check_hash build_2_sha256 "$build_2"
check_hash final_binary_sha256 "$final_binary"
check_hash build_1_receipt_sha256 "$receipt_1"
check_hash build_2_receipt_sha256 "$receipt_2"
check_hash test_evidence_sha256 "$test_evidence"
[ "$(size_of "$build_1")" = "$(field binary_bytes)" ] && [ "$(size_of "$build_2")" = "$(field binary_bytes)" ] && [ "$(size_of "$final_binary")" = "$(field binary_bytes)" ] || fail 'binary size context mismatch'
directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
/bin/sh "$directory/check-controller-helper-build-receipt-v0.sh" "$receipt_1" "$trusted_root" "$expected_controller" 1 "$runner" "$source_tree" "$build_plan" "$compiler_closure" "$compiler" "$build_1" "$log_1" >/dev/null || fail 'build 1 receipt invalid'
/bin/sh "$directory/check-controller-helper-build-receipt-v0.sh" "$receipt_2" "$trusted_root" "$expected_controller" 2 "$runner" "$source_tree" "$build_plan" "$compiler_closure" "$compiler" "$build_2" "$log_2" >/dev/null || fail 'build 2 receipt invalid'
receipt_field() { /usr/bin/sed -n "s/^$2=//p" "$1"; }
[ "$(receipt_field "$receipt_1" job_nonce)" != "$(receipt_field "$receipt_2" job_nonce)" ] || fail 'build job nonces are not distinct'
[ "$(receipt_field "$receipt_1" root_nonce)" != "$(receipt_field "$receipt_2" root_nonce)" ] || fail 'build root nonces are not distinct'
/bin/sh "$directory/check-controller-helper-test-evidence-v0.sh" "$test_evidence" "$trusted_root" "$expected_controller" "$runner" "$source_tree" "$final_binary" "$golden" "$case_results" "$test_log" >/dev/null || fail 'test evidence context invalid'
test_nonce=$(/usr/bin/sed -n 's/^job_nonce=//p' "$test_evidence")
[ "$test_nonce" != "$(receipt_field "$receipt_1" job_nonce)" ] && [ "$test_nonce" != "$(receipt_field "$receipt_2" job_nonce)" ] || fail 'test job aliases a build job'
[ "$evidence_before" = "$(identity "$evidence")" ] || fail 'evidence identity changed during validation'
printf '%s\n' 'controller helper build evidence context validated: independent-receipts=2 status=accepted'
