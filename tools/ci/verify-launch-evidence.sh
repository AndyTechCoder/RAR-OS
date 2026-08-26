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
protocol=$root/spec/alpha/evidence/acceptance-v1.plan
[ -f "$protocol" ] && [ ! -L "$protocol" ] || exit 1
selected=$(/usr/bin/awk -F '|' -v maximum="$rank" '
    function value(letter) { return index("ABCDEFG", letter) }
    /^#/ || /^schema=/ || !NF { next }
    NF != 5 { exit 1 }
    value($1) > 0 && value($1) <= maximum { print $0 }
' "$protocol") || exit 1
[ -n "$selected" ] || exit 1
expected='actions.v1
final.ppm
serial.log'
captures=$(printf '%s\n' "$selected" | /usr/bin/awk -F '|' '$5 == 1 { print $4 ".ppm" } $5 != 0 && $5 != 1 { exit 1 }') || exit 1
[ -z "$captures" ] || expected="$expected
$captures"

actual=$(find "$base" -mindepth 1 -print | /usr/bin/sed "s|^$base/||" | /usr/bin/sort)
[ "$actual" = "$(printf '%s\n' "$expected" | /usr/bin/sort)" ] || exit 1

serial=$base/serial.log
[ -s "$serial" ] && [ ! -L "$serial" ] || exit 1
serial_size=$(/usr/bin/stat -f %z "$serial" 2>/dev/null || /usr/bin/stat -c %s "$serial")
[ "$serial_size" -le 8388608 ] || exit 1
markers=$(printf '%s\n' "$selected" | /usr/bin/awk -F '|' '{ print $3 }')
printf '%s\n' "$markers" | while IFS= read -r marker; do /usr/bin/grep -Fq -- "$marker" "$serial" || exit 1; done

actions=$base/actions.v1
[ "$(/usr/bin/sed -n '1p' "$actions")" = schema=rar-alpha-action-transcript-v1 ] || exit 1
plan=$(printf '%s\n' "$selected" | /usr/bin/awk -F '|' '{ print NR "|" $2 "|" $3 "|" $4 }')
actual_plan=$(/usr/bin/sed -n '2,$p' "$actions" | /usr/bin/awk -F '|' '
    BEGIN { previous = -1 }
    {
        if (NF != 6 || $1 != NR || $4 !~ /^[0-9]+$/ || $5 !~ /^[0-9]+$/ || $4 < previous || $5 <= $4) exit 1
        if ($2 == "none" && $4 != previous) exit 1
        print $1 "|" $2 "|" $3 "|" $6
        previous = $5
    }
') || exit 1
[ "$actual_plan" = "$plan" ] || exit 1
/usr/bin/sed -n '2,$p' "$actions" | while IFS='|' read -r sequence chord marker before after label; do
    /usr/bin/awk -v marker="$marker" -v expected="$after" '
        { offset += length($0) + 1; if ($0 == marker && offset == expected) found = 1 }
        END { if (!found) exit 1 }
    ' "$serial" || exit 1
done

printf '%s\n' "$expected" | while IFS= read -r name; do
    case "$name" in *.ppm)
        image=$base/$name
        [ -s "$image" ] && [ ! -L "$image" ] || exit 1
        [ "$(/usr/bin/sed -n '1p' "$image")" = P6 ] || exit 1
        dimensions=$(/usr/bin/sed -n '2p' "$image")
        set -- $dimensions
        [ "$#" -eq 2 ] || exit 1
        case "$1$2" in *[!0-9]*) exit 1 ;; esac
        [ "$1" -ge 1 ] && [ "$1" -le 4096 ] && [ "$2" -ge 1 ] && [ "$2" -le 2160 ] || exit 1
        [ "$(/usr/bin/sed -n '3p' "$image")" = 255 ] || exit 1
        header_size=$(printf 'P6\n%s\n255\n' "$dimensions" | /usr/bin/wc -c | /usr/bin/tr -d ' ')
        image_size=$(/usr/bin/stat -f %z "$image" 2>/dev/null || /usr/bin/stat -c %s "$image")
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
    size=$(/usr/bin/stat -f %z "$base/$name" 2>/dev/null || /usr/bin/stat -c %s "$base/$name")
    total_bytes=$((total_bytes + size))
done
[ "$total_bytes" -le $((limit_mib * 1024 * 1024)) ] || exit 1
printf '%s\n' "$expected" | while IFS= read -r name; do
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$base/$name"; else /usr/bin/shasum -a 256 "$base/$name"; fi
done
printf 'launch evidence verified: probe=%s total_bytes=%s\n' "$probe" "$total_bytes"
