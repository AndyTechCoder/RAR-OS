#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

verdict=${1-}
expected_probe=${2-}
controller=${3-}
source=${4-}
transcript=${5-}
inventory=${6-}
evidence=${7-}
harness=${8-}
fail() { printf 'reference verdict rejected: %s\n' "$1" >&2; exit 1; }

case "$expected_probe" in milestone-[abcdefg]) ;; *) fail 'expected probe is invalid' ;; esac
for file in "$verdict" "$controller" "$source" "$transcript"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty context: $file"
done
size=$(/usr/bin/stat -f %z "$verdict" 2>/dev/null || /usr/bin/stat -c %s "$verdict")
[ "$size" -le 2048 ] || fail 'file exceeds bound'
last_hex=$(/usr/bin/od -An -tx1 -j $((size - 1)) -N 1 "$verdict" | /usr/bin/tr -d ' \n')
[ "$last_hex" = 0a ] || fail 'verdict lacks exactly one terminal LF'
if /usr/bin/grep -Ev '^[a-z0-9_]+=[a-z0-9-]+$' "$verdict" | /usr/bin/grep -q .; then fail 'grammar is invalid'; fi

/usr/bin/awk -F '=' '
    BEGIN {
        order[1]="schema"; order[2]="status"; order[3]="probe"; order[4]="controller_sha256"; order[5]="source_sha256"; order[6]="transcript_sha256"; order[7]="reference_inventory_sha256"; order[8]="comparison_evidence_sha256"; order[9]="record_count"; order[10]="reference_1_result"; order[11]="reference_2_result"; order[12]="target_result"; order[13]="reason"
        zero="0000000000000000000000000000000000000000000000000000000000000000"
    }
    function nonzero_sha(value) { return value ~ /^[0-9a-f]{64}$/ && value != zero }
    function reject(message) { print "reference verdict rejected: " message > "/dev/stderr"; bad=1 }
    {
        if (NF != 2 || NR > 13 || $1 != order[NR]) reject("field order or count is invalid")
        value[$1]=$2
    }
    END {
        if (NR != 13) reject("field count is invalid")
        if (value["schema"] != "rar-alpha-reference-verdict-v0") reject("schema is invalid")
        for (i=4; i<=8; i++) if (value[order[i]] !~ /^[0-9a-f]{64}$/) reject("digest is malformed: " order[i])
        if (value["record_count"] !~ /^(0|[1-9][0-9]*)$/) reject("record count is not canonical decimal")
        if (value["status"] == "accepted") {
            if (value["probe"] !~ /^milestone-[fg]$/) reject("accepted probe is invalid")
            for (i=4; i<=8; i++) if (!nonzero_sha(value[order[i]])) reject("accepted digest is zero or malformed: " order[i])
            if (value["record_count"] < 1 || value["record_count"] > 512) reject("accepted record count is invalid")
            if (value["reference_1_result"] != "match" || value["reference_2_result"] != "match" || value["target_result"] != "match" || value["reason"] != "all-three-match") reject("accepted result is inconsistent")
        } else if (value["status"] == "not-required") {
            if (value["probe"] !~ /^milestone-[abcde]$/) reject("not-required probe is invalid")
            for (i=4; i<=6; i++) if (!nonzero_sha(value[order[i]])) reject("not-required identity is zero or malformed: " order[i])
            if (value["reference_inventory_sha256"] != zero || value["comparison_evidence_sha256"] != zero) reject("not-required reference digest is nonzero")
            if (value["record_count"] != 0 || value["reference_1_result"] != "not-run" || value["reference_2_result"] != "not-run" || value["target_result"] != "not-evaluated" || value["reason"] != "probe-does-not-require-reference") reject("not-required result is inconsistent")
        } else reject("status is invalid")
        exit bad ? 1 : 0
    }
' "$verdict" || exit 1

field() { /usr/bin/sed -n "s/^$1=//p" "$verdict"; }
sha_file() { /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
probe=$(field probe)
status=$(field status)
[ "$probe" = "$expected_probe" ] || fail 'verdict probe does not equal expected probe'
[ "$(field controller_sha256)" = "$(sha_file "$controller")" ] || fail 'controller binding mismatch'
[ "$(field source_sha256)" = "$(sha_file "$source")" ] || fail 'source binding mismatch'
[ "$(field transcript_sha256)" = "$(sha_file "$transcript")" ] || fail 'transcript binding mismatch'

if [ "$status" = accepted ]; then
    for file in "$inventory" "$evidence" "$harness"; do
        [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "accepted verdict lacks real context: $file"
    done
    [ "$(field reference_inventory_sha256)" = "$(sha_file "$inventory")" ] || fail 'inventory binding mismatch'
    [ "$(field comparison_evidence_sha256)" = "$(sha_file "$evidence")" ] || fail 'evidence binding mismatch'
    checker=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)/check-reference-evidence-v0.sh
    validated=$(/bin/sh "$checker" "$evidence" "$transcript" "$inventory" "$harness") || exit 1
    validated_count=${validated##*=}
    [ "$validated_count" = "$(field record_count)" ] || fail 'verdict/evidence record count mismatch'
else
    [ "$inventory" = none ] && [ "$evidence" = none ] && [ "$harness" = none ] || fail 'not-required verdict received reference context'
fi

printf 'reference verdict context validated: status=%s probe=%s\n' "$status" "$probe"
