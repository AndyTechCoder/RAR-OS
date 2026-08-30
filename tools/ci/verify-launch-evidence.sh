#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

base=${1-} probe=${2-} limit_mib=${3-}
[ -d "$base" ] && [ ! -L "$base" ] || exit 1
case "$probe" in milestone-a) rank=1 ;; milestone-b) rank=2 ;; milestone-c) rank=3 ;; milestone-d) rank=4 ;; milestone-e) rank=5 ;; milestone-f) rank=6 ;; milestone-g) rank=7 ;; *) exit 1 ;; esac
case "$limit_mib" in '' | *[!0-9]*) exit 1 ;; esac
[ "$limit_mib" -ge 1 ] && [ "$limit_mib" -le 64 ] || exit 1

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
protocol=$root/spec/alpha/evidence/acceptance-v2.plan
protocol_sha256=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd
[ -f "$protocol" ] && [ ! -L "$protocol" ] || exit 1
if [ -x /usr/bin/sha256sum ]; then
    actual_protocol_sha256=$(/usr/bin/sha256sum "$protocol" | /usr/bin/awk '{ print $1 }')
else
    actual_protocol_sha256=$(/usr/bin/shasum -a 256 "$protocol" | /usr/bin/awk '{ print $1 }')
fi
[ "$actual_protocol_sha256" = "$protocol_sha256" ] || exit 1
[ "$(/usr/bin/sed -n '1p' "$protocol")" = schema=rar-alpha-acceptance-plan-v2 ] || exit 1
selected=$(/usr/bin/awk -F '|' -v maximum="$rank" '
    function value(letter) { return index("ABCDEFG", letter) }
    /^#/ || /^schema=/ || !NF { next }
    NF != 5 { exit 1 }
    value($1) > 0 && value($1) <= maximum { print $0 }
' "$protocol") || exit 1
[ -n "$selected" ] || exit 1
expected='actions.v2
final.ppm
serial.log'
captures=$(printf '%s\n' "$selected" | /usr/bin/awk -F '|' '$5 == 1 { print $4 ".ppm" } $5 != 0 && $5 != 1 { exit 1 }') || exit 1
[ -z "$captures" ] || expected="$expected
$captures"

actual=$(find "$base" -mindepth 1 -print | /usr/bin/sed "s|^$base/||" | /usr/bin/sort)
[ "$actual" = "$(printf '%s\n' "$expected" | /usr/bin/sort)" ] || exit 1

serial=$base/serial.log
[ -f "$serial" ] && [ -s "$serial" ] && [ ! -L "$serial" ] || exit 1
serial_size=$(/usr/bin/stat -c %s "$serial" 2>/dev/null || /usr/bin/stat -f %z "$serial")
serial_links=$(/usr/bin/stat -c %h "$serial" 2>/dev/null || /usr/bin/stat -f %l "$serial")
[ "$serial_size" -le 8388608 ] || exit 1
[ "$serial_links" -eq 1 ] || exit 1
[ "$(/usr/bin/tail -c 1 "$serial" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || exit 1
markers=$(printf '%s\n' "$selected" | /usr/bin/awk -F '|' '{ print $3 }')
printf '%s\n' "$markers" | while IFS= read -r marker; do
    [ "$(/usr/bin/grep -Fxc -- "$marker" "$serial")" -eq 1 ] || exit 1
done
/usr/bin/awk -F '|' -v maximum="$rank" '
    function value(letter) { return index("ABCDEFG", letter) }
    FNR == NR {
        if (/^#/ || /^schema=/ || !NF) next
        if (NF != 5 || ++known[$3] != 1) exit 1
        minimum[$3] = value($1)
        next
    }
    ($0 in minimum) {
        count[$0]++
        if (minimum[$0] > maximum) bad = 1
    }
    END {
        for (marker in minimum) {
            if (minimum[marker] <= maximum && count[marker] != 1) bad = 1
            if (minimum[marker] > maximum && count[marker] != 0) bad = 1
        }
        exit bad ? 1 : 0
    }
' "$protocol" "$serial" || exit 1

actions=$base/actions.v2
[ -f "$actions" ] && [ -s "$actions" ] && [ ! -L "$actions" ] || exit 1
actions_size=$(/usr/bin/stat -c %s "$actions" 2>/dev/null || /usr/bin/stat -f %z "$actions")
actions_links=$(/usr/bin/stat -c %h "$actions" 2>/dev/null || /usr/bin/stat -f %l "$actions")
[ "$actions_size" -le 1048576 ] || exit 1
[ "$actions_links" -eq 1 ] || exit 1
[ "$(/usr/bin/tail -c 1 "$actions" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || exit 1
[ "$(/usr/bin/sed -n '1p' "$actions")" = schema=rar-alpha-action-transcript-v2 ] || exit 1
[ "$(/usr/bin/sed -n '2p' "$actions")" = "protocol_sha256=$protocol_sha256" ] || exit 1
[ "$(/usr/bin/grep -c '^protocol_sha256=' "$actions")" -eq 1 ] || exit 1
plan=$(printf '%s\n' "$selected" | /usr/bin/awk -F '|' '{ print NR "|" $2 "|" $3 "|" $4 "|" $5 }')
actual_plan=$(/usr/bin/sed -n '3,$p' "$actions" | /usr/bin/awk -F '|' -v serial_size="$serial_size" '
    BEGIN { previous = -1 }
    {
        if (NF != 8 || $1 !~ /^[1-9][0-9]*$/ || $1 != NR || $4 !~ /^(0|[1-9][0-9]*)$/ || $5 !~ /^[1-9][0-9]*$/ || $4 < previous || $5 <= $4 || $5 > serial_size) exit 1
        if ($2 == "none" && $4 != previous) exit 1
        if (($7 == "0" && $8 != "none") || ($7 == "1" && $8 !~ /^[0-9a-f]{64}$/) || ($7 != "0" && $7 != "1")) exit 1
        print $1 "|" $2 "|" $3 "|" $6 "|" $7
        previous = $5
    }
') || exit 1
[ "$actual_plan" = "$plan" ] || exit 1
/usr/bin/sed -n '3,$p' "$actions" | while IFS='|' read -r sequence chord marker before after label capture capture_sha256; do
    /usr/bin/awk -v marker="$marker" -v expected_before="$before" -v expected_after="$after" '
        {
            start = offset
            offset += length($0) + 1
            if ($0 == marker && start >= expected_before && offset == expected_after) found = 1
        }
        END { if (!found) exit 1 }
    ' "$serial" || exit 1
    case "$capture" in
        0) [ "$capture_sha256" = none ] || exit 1 ;;
        1)
            image=$base/$label.ppm
            if [ -x /usr/bin/sha256sum ]; then output=$(/usr/bin/sha256sum "$image"); else output=$(/usr/bin/shasum -a 256 "$image"); fi
            [ "${output%% *}" = "$capture_sha256" ] || exit 1
            ;;
        *) exit 1 ;;
    esac
done

printf '%s\n' "$expected" | while IFS= read -r name; do
    case "$name" in *.ppm)
        image=$base/$name
        [ -s "$image" ] && [ ! -L "$image" ] || exit 1
        image_links=$(/usr/bin/stat -c %h "$image" 2>/dev/null || /usr/bin/stat -f %l "$image")
        [ "$image_links" -eq 1 ] || exit 1
        [ "$(/usr/bin/sed -n '1p' "$image")" = P6 ] || exit 1
        dimensions=$(/usr/bin/sed -n '2p' "$image")
        set -- $dimensions
        [ "$#" -eq 2 ] || exit 1
        case "$1$2" in *[!0-9]*) exit 1 ;; esac
        [ "$1" -ge 1 ] && [ "$1" -le 4096 ] && [ "$2" -ge 1 ] && [ "$2" -le 2160 ] || exit 1
        [ "$(/usr/bin/sed -n '3p' "$image")" = 255 ] || exit 1
        header_size=$(printf 'P6\n%s\n255\n' "$dimensions" | /usr/bin/wc -c | /usr/bin/tr -d ' ')
        image_size=$(/usr/bin/stat -c %s "$image" 2>/dev/null || /usr/bin/stat -f %z "$image")
        [ "$image_size" -eq $((header_size + 3 * $1 * $2)) ] || exit 1
        ;;
    esac
done

case "$probe" in milestone-e | milestone-f | milestone-g)
    digests=
    for name in launcher pointer terminal settings demo-1 demo-2; do
        if command -v sha256sum >/dev/null 2>&1; then output=$(sha256sum "$base/$name.ppm"); else output=$(/usr/bin/shasum -a 256 "$base/$name.ppm"); fi
        digests="$digests
${output%% *}"
    done
    [ "$(printf '%s\n' "$digests" | /usr/bin/awk 'NF' | /usr/bin/sort -u | /usr/bin/wc -l | /usr/bin/tr -d ' ')" -eq 6 ] || exit 1
    ;;
esac

total_bytes=0
for name in $expected; do
    size=$(/usr/bin/stat -c %s "$base/$name" 2>/dev/null || /usr/bin/stat -f %z "$base/$name")
    total_bytes=$((total_bytes + size))
done
[ "$total_bytes" -le $((limit_mib * 1024 * 1024)) ] || exit 1
printf '%s\n' "$expected" | while IFS= read -r name; do
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$base/$name"; else /usr/bin/shasum -a 256 "$base/$name"; fi
done
printf 'launch evidence verified: probe=%s total_bytes=%s\n' "$probe" "$total_bytes"
