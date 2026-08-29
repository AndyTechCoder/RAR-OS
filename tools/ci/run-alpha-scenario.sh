#!/bin/sh
set -eu

probe=${1-} client=${2-} socket=${3-} evidence=${4-}
case "$probe" in milestone-a) rank=1 ;; milestone-b) rank=2 ;; milestone-c) rank=3 ;; milestone-d) rank=4 ;; milestone-e) rank=5 ;; milestone-f) rank=6 ;; milestone-g) rank=7 ;; *) exit 1 ;; esac
[ -x "$client" ] || exit 1
[ "$evidence" = /evidence ] && [ -d "$evidence" ] && [ ! -L "$evidence" ] || exit 1
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
plan=$root/spec/alpha/evidence/acceptance-v1.plan
[ -f "$plan" ] && [ ! -L "$plan" ] || exit 1

selected=$evidence/selected-plan.v1
/usr/bin/awk -F '|' -v maximum="$rank" '
    function value(letter) { return index("ABCDEFG", letter) }
    /^#/ || /^schema=/ || !NF { next }
    NF != 5 { exit 1 }
    value($1) > 0 && value($1) <= maximum { print $0 }
' "$plan" > "$selected"
[ -s "$selected" ] || exit 1

transcript=$evidence/actions.v1
/usr/bin/printf '%s\n' 'schema=rar-alpha-action-transcript-v1' > "$transcript"
sequence=0
last_offset=0
"$client" wait-ready "$socket" 30000

while IFS='|' read -r minimum input marker label capture; do
    before=$last_offset
    case "$input" in
        continue)
            before=$("$client" serial-offset "$evidence/serial.log")
            "$client" continue "$socket"
            ;;
        key:*)
            before=$("$client" serial-offset "$evidence/serial.log")
            "$client" key-chord "$socket" "${input#key:}"
            ;;
        pointer:*)
            before=$("$client" serial-offset "$evidence/serial.log")
            coordinates=${input#pointer:}
            old_ifs=$IFS
            IFS=,
            set -- $coordinates
            IFS=$old_ifs
            [ "$#" -eq 3 ] || exit 1
            "$client" pointer "$socket" "$1" "$2" "$3"
            ;;
        none) ;;
        *) exit 1 ;;
    esac
    case "$before" in '' | *[!0-9]*) exit 1 ;; esac
    [ "$before" -ge "$last_offset" ] || exit 1
    after=$("$client" wait-trace "$socket" "$evidence/serial.log" "$marker" "$before" 60000)
    case "$after" in '' | *[!0-9]*) exit 1 ;; esac
    [ "$after" -gt "$before" ] || exit 1
    sequence=$((sequence + 1))
    /usr/bin/printf '%s|%s|%s|%s|%s|%s\n' "$sequence" "$input" "$marker" "$before" "$after" "$label" >> "$transcript"
    last_offset=$after
    case "$capture" in 0) ;; 1) "$client" capture "$socket" "$evidence/$label.ppm" ;; *) exit 1 ;; esac
done < "$selected"

/bin/rm -f "$selected"
"$client" capture "$socket" "$evidence/final.ppm"
"$client" quit "$socket"
