#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

evidence=${1-}; trusted_root=${2-}; expected_controller=${3-}; runner=${4-}; source_tree=${5-}; binary=${6-}; golden=${7-}; case_results=${8-}; log=${9-}
fail() { printf 'controller helper test evidence rejected: %s\n' "$1" >&2; exit 1; }
[ -d "$trusted_root" ] && [ ! -L "$trusted_root" ] || fail 'trusted root missing or symbolic'
trusted_root=$(CDPATH= cd -- "$trusted_root" && pwd -P)
safe_file() {
    file=$1
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
    parent=$(CDPATH= cd -- "$(dirname -- "$file")" && pwd -P) || fail "input parent inaccessible: $file"
    resolved=$parent/$(basename -- "$file")
    case "$resolved" in "$trusted_root"/*) ;; *) fail "input escapes trusted root: $file" ;; esac
    links=$(/usr/bin/stat -f %l "$file" 2>/dev/null || /usr/bin/stat -c %h "$file")
    owner=$(/usr/bin/stat -f %u "$file" 2>/dev/null || /usr/bin/stat -c %u "$file")
    [ "$links" = 1 ] || fail "input is hardlinked: $file"
    [ "$owner" = "$(/usr/bin/id -u)" ] || fail "input is not controller-user owned: $file"
    if find "$file" -perm -022 -print | /usr/bin/grep -q .; then fail "input is group/other writable: $file"; fi
}
for file in "$evidence" "$runner" "$source_tree" "$binary" "$golden" "$case_results" "$log"; do safe_file "$file"; done
case "$expected_controller" in *[!0-9a-f]*|'') fail 'expected controller SHA malformed' ;; esac
[ "${#expected_controller}" -eq 40 ] || fail 'expected controller SHA length invalid'
size_of() { /usr/bin/stat -f %z "$1" 2>/dev/null || /usr/bin/stat -c %s "$1"; }
identity() { /usr/bin/stat -f '%d:%i:%z:%l:%u:%m' "$1" 2>/dev/null || /usr/bin/stat -c '%d:%i:%s:%h:%u:%Y' "$1"; }
sha_file() { env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
evidence_before=$(identity "$evidence")
size=$(size_of "$evidence")
[ "$size" -le 2048 ] || fail 'evidence exceeds bound'
last=$(/usr/bin/od -An -tx1 -j $((size - 1)) -N 1 "$evidence" | /usr/bin/tr -d ' \n')
[ "$last" = 0a ] || fail 'evidence lacks terminal LF'
/usr/bin/awk -F '=' '
    BEGIN { split("schema producer controller_sha job_nonce runner_image_sha256 source_tree_sha256 binary_sha256 golden_vector_sha256 case_results_sha256 log_sha256 test_count failed_count network observed_exit_status status", order, " ") }
    function reject(message) { print "controller helper test evidence rejected: " message > "/dev/stderr"; bad=1 }
    {
        if (NF != 2 || NR > 15 || $1 != order[NR] || $2 !~ /^[a-z0-9-]+$/) reject("grammar or order invalid at line " NR)
        if (++seen[$1] != 1) reject("duplicate field: " $1)
        value[$1]=$2
    }
    END {
        if (NR != 15) reject("field count invalid")
        if (value["schema"] != "rar-alpha-controller-helper-test-evidence-v0" || value["producer"] != "trusted-outer-controller") reject("schema or producer invalid")
        if (value["controller_sha"] !~ /^[0-9a-f]{40}$/) reject("controller SHA invalid")
        zero=sprintf("%064d", 0)
        for (i=4; i<=10; i++) if (value[order[i]] !~ /^[0-9a-f]{64}$/ || value[order[i]] == zero) reject("identity invalid: " order[i])
        if (value["test_count"] != "11" || value["failed_count"] != "0" || value["network"] != "none" || value["observed_exit_status"] != "0" || value["status"] != "accepted") reject("test result invalid")
        exit bad ? 1 : 0
    }
' "$evidence" || exit 1
/usr/bin/awk -F '|' '
    BEGIN { split("official-short-vectors streaming-boundaries-match-one-shot round-trip-and-layout matches-language-neutral-golden-vector accepts-every-phase-role-kind-combination rejects-noncanonical-values rejects-mutated-wire-rules phase-plans-bind-ordinals-and-launch-allowlist transaction-success-hash-failure-and-cleanup-uncertainty parses-bounded-dirents-and-rejects-malformed-records enforces-linux-descriptor-authority-and-cleanup-invariants", expected, " ") }
    NR == 1 { if ($0 != "schema=rar-alpha-controller-helper-test-cases-v0") bad=1; next }
    NR > 1 { if (NR > 12 || NF != 2 || $1 != expected[NR-1] || $2 != "pass" || ++seen[$1] != 1) bad=1; count++ }
    END { if (count != 11) bad=1; exit bad ? 1 : 0 }
' "$case_results" || fail 'canonical per-case results invalid'
field() { /usr/bin/sed -n "s/^$1=//p" "$evidence"; }
[ "$(field controller_sha)" = "$expected_controller" ] || fail 'controller context mismatch'
check_hash() { [ "$(field "$1")" = "$(sha_file "$2")" ] || fail "$1 context mismatch"; }
check_hash runner_image_sha256 "$runner"
check_hash source_tree_sha256 "$source_tree"
check_hash binary_sha256 "$binary"
check_hash golden_vector_sha256 "$golden"
check_hash case_results_sha256 "$case_results"
check_hash log_sha256 "$log"
[ "$evidence_before" = "$(identity "$evidence")" ] || fail 'evidence identity changed during validation'
printf '%s\n' 'controller helper test evidence context validated: cases=11 status=accepted'
