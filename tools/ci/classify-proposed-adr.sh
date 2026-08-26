#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

record=${1-}
[ -f "$record" ] && [ ! -L "$record" ] || exit 1
/usr/bin/sed -n '1p' "$record" | /usr/bin/grep -Eq '^# ADR [0-9]{4}: [A-Za-z0-9 -]+$' || exit 1
[ "$(/usr/bin/grep -c '^Status:' "$record")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -c '^Decision:' "$record")" -eq 1 ] || exit 1
status=$(/usr/bin/sed -n '3p' "$record")
decision=$(/usr/bin/sed -n '4p' "$record")
if [ "$status" = 'Status: Proposed — owner decision required' ] && [ "$decision" = 'Decision: Undecided' ]; then
    printf '%s\n' owner-decision-required
    exit 0
fi
date=${status#'Status: Accepted — '}
[ "$date" != "$status" ] || exit 1
case "$date" in [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;; *) exit 1 ;; esac
year=${date%%-*}
rest=${date#*-}
month=${rest%%-*}
day=${rest#*-}
year_value=${year#0}
month_value=${month#0}
day_value=${day#0}
[ -n "$year_value" ] && [ -n "$month_value" ] && [ -n "$day_value" ] || exit 1
[ "$year_value" -ge 1 ] || exit 1
case "$month" in
    01 | 03 | 05 | 07 | 08 | 10 | 12) maximum_day=31 ;;
    04 | 06 | 09 | 11) maximum_day=30 ;;
    02)
        maximum_day=28
        if [ $((year_value % 400)) -eq 0 ] || { [ $((year_value % 4)) -eq 0 ] && [ $((year_value % 100)) -ne 0 ]; }; then
            maximum_day=29
        fi
        ;;
    *) exit 1 ;;
esac
[ "$day_value" -ge 1 ] && [ "$day_value" -le "$maximum_day" ] || exit 1
case "$decision" in 'Decision: Alternative '[ABC]) ;; *) exit 1 ;; esac
printf '%s\n' accepted
