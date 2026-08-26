#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root=$root/out
/bin/mkdir -p "$output_root"
work=$(mktemp -d "$output_root/launch-evidence.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/verify-launch-evidence.sh
protocol=$root/spec/alpha/evidence/acceptance-v1.plan

make_ppm() {
    label=$1 path=$2
    case "$label" in
        boot) pixels=aaa ;; nucleus) pixels=bbb ;; isolation) pixels=ccc ;;
        recovery) pixels=ddd ;; launcher) pixels=eee ;; pointer) pixels=fff ;;
        terminal) pixels=ggg ;; settings) pixels=hhh ;; demo-1) pixels=iii ;;
        demo-2) pixels=jjj ;; update) pixels=kkk ;; integration) pixels=lll ;;
        final) pixels=zzz ;; *) exit 1 ;;
    esac
    /usr/bin/printf 'P6\n1 1\n255\n%s' "$pixels" > "$path"
}

make_evidence() {
    base=$1 maximum=$2
    /bin/mkdir -p "$base"
    selected=$work/selected.$maximum
    /usr/bin/awk -F '|' -v maximum="$maximum" '
        function value(letter) { return index("ABCDEFG", letter) }
        /^#/ || /^schema=/ || !NF { next }
        value($1) > 0 && value($1) <= maximum { print $0 }
    ' "$protocol" > "$selected"
    : > "$base/serial.log"
    /usr/bin/printf '%s\n' schema=rar-alpha-action-transcript-v1 > "$base/actions.v1"
    sequence=0
    while IFS='|' read -r minimum input marker label capture; do
        before=$(/usr/bin/wc -c < "$base/serial.log" | /usr/bin/tr -d ' ')
        /usr/bin/printf '%s\n' "$marker" >> "$base/serial.log"
        after=$(/usr/bin/wc -c < "$base/serial.log" | /usr/bin/tr -d ' ')
        sequence=$((sequence + 1))
        /usr/bin/printf '%s|%s|%s|%s|%s|%s\n' "$sequence" "$input" "$marker" "$before" "$after" "$label" >> "$base/actions.v1"
        [ "$capture" = 0 ] || make_ppm "$label" "$base/$label.ppm"
    done < "$selected"
    make_ppm final "$base/final.ppm"
    find "$base" -name '._*' -type f -exec /bin/rm -f {} \;
}

evidence_a=$work/a
make_evidence "$evidence_a" 1
/bin/sh "$checker" "$evidence_a" milestone-a 1 >/dev/null
/bin/rm -f "$evidence_a/final.ppm"
if /bin/sh "$checker" "$evidence_a" milestone-a 1 >/dev/null 2>&1; then exit 1; fi
make_ppm final "$evidence_a/final.ppm"
/usr/bin/printf 'P3\n1 1\n255\nabc' > "$evidence_a/boot.ppm"
if /bin/sh "$checker" "$evidence_a" milestone-a 1 >/dev/null 2>&1; then exit 1; fi
make_ppm boot "$evidence_a/boot.ppm"
/bin/ln -s boot.ppm "$evidence_a/extra"
if /bin/sh "$checker" "$evidence_a" milestone-a 1 >/dev/null 2>&1; then exit 1; fi
/bin/rm -f "$evidence_a/extra"
/bin/mkdir "$evidence_a/extra"
if /bin/sh "$checker" "$evidence_a" milestone-a 1 >/dev/null 2>&1; then exit 1; fi
/bin/rmdir "$evidence_a/extra"
/usr/bin/printf '%s\n' metadata > "$evidence_a/._ignored"
if /bin/sh "$checker" "$evidence_a" milestone-a 1 >/dev/null 2>&1; then exit 1; fi
/bin/rm -f "$evidence_a/._ignored"

evidence_e=$work/e
make_evidence "$evidence_e" 5
/bin/sh "$checker" "$evidence_e" milestone-e 1 >/dev/null
/bin/rm -f "$evidence_e/pointer.ppm"
if /bin/sh "$checker" "$evidence_e" milestone-e 1 >/dev/null 2>&1; then exit 1; fi
make_ppm pointer "$evidence_e/pointer.ppm"
/usr/bin/sed -i.bak 's/surface:pointer-accepted/surface:pointer-ignored/' "$evidence_e/actions.v1"
/bin/rm -f "$evidence_e/actions.v1.bak"
if /bin/sh "$checker" "$evidence_e" milestone-e 1 >/dev/null 2>&1; then exit 1; fi
/bin/rm -rf "$evidence_e"
make_evidence "$evidence_e" 5
/bin/cp "$evidence_e/demo-1.ppm" "$evidence_e/demo-2.ppm"
if /bin/sh "$checker" "$evidence_e" milestone-e 1 >/dev/null 2>&1; then exit 1; fi

evidence_g=$work/g
make_evidence "$evidence_g" 7
/bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null
/usr/bin/sed -i.bak '/data:post-sha256:/d' "$evidence_g/serial.log"
/bin/rm -f "$evidence_g/serial.log.bak"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

overflow=$work/overflow
make_evidence "$overflow" 1
/usr/bin/awk 'BEGIN { for (i = 0; i < 600000; i++) printf "0123456789abcdef" }' > "$overflow/serial.log"
if /bin/sh "$checker" "$overflow" milestone-a 64 >/dev/null 2>&1; then exit 1; fi

printf '%s\n' 'launch evidence negative checks passed'
