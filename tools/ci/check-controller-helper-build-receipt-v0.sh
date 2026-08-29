#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

receipt=${1-}; trusted_root=${2-}; expected_controller=${3-}; expected_ordinal=${4-}; runner=${5-}; source_tree=${6-}; build_plan=${7-}; compiler_closure=${8-}; compiler=${9-}; shift 9
output=${1-}; log=${2-}
fail() { printf 'controller helper build receipt rejected: %s\n' "$1" >&2; exit 1; }
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
for file in "$receipt" "$runner" "$source_tree" "$build_plan" "$compiler_closure" "$compiler" "$output" "$log"; do safe_file "$file"; done
case "$expected_controller" in *[!0-9a-f]*|'') fail 'expected controller SHA malformed' ;; esac
[ "${#expected_controller}" -eq 40 ] || fail 'expected controller SHA length invalid'
case "$expected_ordinal" in 1|2) ;; *) fail 'expected ordinal invalid' ;; esac
size_of() { /usr/bin/stat -f %z "$1" 2>/dev/null || /usr/bin/stat -c %s "$1"; }
identity() { /usr/bin/stat -f '%d:%i:%z:%l:%u:%m' "$1" 2>/dev/null || /usr/bin/stat -c '%d:%i:%s:%h:%u:%Y' "$1"; }
sha_file() { env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
receipt_before=$(identity "$receipt")
size=$(size_of "$receipt")
[ "$size" -le 4096 ] || fail 'receipt exceeds bound'
last=$(/usr/bin/od -An -tx1 -j $((size - 1)) -N 1 "$receipt" | /usr/bin/tr -d ' \n')
[ "$last" = 0a ] || fail 'receipt lacks terminal LF'
/usr/bin/awk -F '=' '
    BEGIN { split("schema producer controller_sha build_ordinal job_nonce root_nonce runner_image_sha256 source_tree_sha256 build_plan_sha256 compiler_closure_manifest_sha256 compiler_sha256 output_sha256 output_bytes log_sha256 fresh_root preexisting_output network observed_exit_status status", order, " ") }
    function reject(message) { print "controller helper build receipt rejected: " message > "/dev/stderr"; bad=1 }
    {
        if (NF != 2 || NR > 19 || $1 != order[NR] || $2 !~ /^[a-z0-9-]+$/) reject("grammar or order invalid at line " NR)
        if (++seen[$1] != 1) reject("duplicate field: " $1)
        value[$1]=$2
    }
    END {
        if (NR != 19) reject("field count invalid")
        if (value["schema"] != "rar-alpha-controller-helper-build-receipt-v0" || value["producer"] != "trusted-outer-controller") reject("schema or producer invalid")
        if (value["controller_sha"] !~ /^[0-9a-f]{40}$/) reject("controller SHA invalid")
        zero=sprintf("%064d", 0)
        for (i=5; i<=12; i++) if (value[order[i]] !~ /^[0-9a-f]{64}$/ || value[order[i]] == zero) reject("identity invalid: " order[i])
        if (value["log_sha256"] !~ /^[0-9a-f]{64}$/ || value["log_sha256"] == zero) reject("log digest invalid")
        if (value["build_ordinal"] !~ /^[12]$/ || value["output_bytes"] !~ /^[1-9][0-9]*$/ || value["output_bytes"] > 16777216) reject("ordinal or byte count invalid")
        if (value["fresh_root"] != "yes" || value["preexisting_output"] != "no" || value["network"] != "none" || value["observed_exit_status"] != "0" || value["status"] != "accepted") reject("freshness or execution result invalid")
        exit bad ? 1 : 0
    }
' "$receipt" || exit 1
field() { /usr/bin/sed -n "s/^$1=//p" "$receipt"; }
[ "$(field controller_sha)" = "$expected_controller" ] || fail 'controller context mismatch'
[ "$(field build_ordinal)" = "$expected_ordinal" ] || fail 'ordinal context mismatch'
check_hash() { [ "$(field "$1")" = "$(sha_file "$2")" ] || fail "$1 context mismatch"; }
check_hash runner_image_sha256 "$runner"
check_hash source_tree_sha256 "$source_tree"
check_hash build_plan_sha256 "$build_plan"
check_hash compiler_closure_manifest_sha256 "$compiler_closure"
check_hash compiler_sha256 "$compiler"
check_hash output_sha256 "$output"
check_hash log_sha256 "$log"
[ "$(field output_bytes)" = "$(size_of "$output")" ] || fail 'output size context mismatch'
[ "$receipt_before" = "$(identity "$receipt")" ] || fail 'receipt identity changed during validation'
printf '%s\n' "controller helper build receipt context validated: ordinal=$expected_ordinal"
