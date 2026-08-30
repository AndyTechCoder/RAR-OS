#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'launch evidence mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/launch-evidence.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/verify-launch-evidence.sh
protocol=$root/spec/alpha/evidence/acceptance-v2.plan
protocol_sha256=ffdb07b584abc94122b14a416593916cf18df439de042c97ff83fda9e4444ccd

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
    /usr/bin/printf '%s\n' \
        schema=rar-alpha-action-transcript-v2 \
        "protocol_sha256=$protocol_sha256" > "$base/actions.v2"
    sequence=0
    while IFS='|' read -r minimum input marker label capture; do
        before=$(/usr/bin/wc -c < "$base/serial.log" | /usr/bin/tr -d ' ')
        /usr/bin/printf '%s\n' "$marker" >> "$base/serial.log"
        after=$(/usr/bin/wc -c < "$base/serial.log" | /usr/bin/tr -d ' ')
        sequence=$((sequence + 1))
        if [ "$capture" = 0 ]; then
            capture_sha256=none
        else
            make_ppm "$label" "$base/$label.ppm"
            capture_sha256=$(/usr/bin/shasum -a 256 "$base/$label.ppm" | /usr/bin/awk '{ print $1 }')
        fi
        /usr/bin/printf '%s|%s|%s|%s|%s|%s|%s|%s\n' \
            "$sequence" "$input" "$marker" "$before" "$after" "$label" "$capture" "$capture_sha256" >> "$base/actions.v2"
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

for milestone in b c d f; do
    case "$milestone" in b) rank=2 ;; c) rank=3 ;; d) rank=4 ;; f) rank=6 ;; esac
    evidence_mid=$work/$milestone
    make_evidence "$evidence_mid" "$rank"
    /bin/sh "$checker" "$evidence_mid" "milestone-$milestone" 1 >/dev/null
done

evidence_e=$work/e
make_evidence "$evidence_e" 5
/bin/sh "$checker" "$evidence_e" milestone-e 1 >/dev/null
/bin/rm -f "$evidence_e/pointer.ppm"
if /bin/sh "$checker" "$evidence_e" milestone-e 1 >/dev/null 2>&1; then exit 1; fi
make_ppm pointer "$evidence_e/pointer.ppm"
/usr/bin/sed -i.bak 's/surface:pointer-accepted/surface:pointer-ignored/' "$evidence_e/actions.v2"
/bin/rm -f "$evidence_e/actions.v2.bak"
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

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/sed -i.bak '2s/.*/protocol_sha256=f7e66d58200272fc239283c42d16389584e5d647362e8623ac439b71d728ec1e/' "$evidence_g/actions.v2"
/bin/rm -f "$evidence_g/actions.v2.bak"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/sed -i.bak '1s/v2/v1/' "$evidence_g/actions.v2"
/bin/rm -f "$evidence_g/actions.v2.bak"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/sed -i.bak '2p' "$evidence_g/actions.v2"
/bin/rm -f "$evidence_g/actions.v2.bak"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/printf '%s\n' 'component:gui-responsive' >> "$evidence_g/serial.log"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/sed -i.bak '3d' "$evidence_g/actions.v2"
/bin/rm -f "$evidence_g/actions.v2.bak"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/sed -i.bak '3p' "$evidence_g/actions.v2"
/bin/rm -f "$evidence_g/actions.v2.bak"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk 'NR==3 { first=$0; next } NR==4 { print; print first; next } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk -F '|' 'BEGIN { OFS="|" } NR==3 { $4=$5-1 } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk -F '|' 'BEGIN { OFS="|" } NR==3 { $4="00" } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk -F '|' 'BEGIN { OFS="|" } NR==8 { $4=$4+1 } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk -F '|' 'BEGIN { OFS="|" } NR==8 { $2="key:unauthorized" } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk -F '|' 'BEGIN { OFS="|" } $7=="1" && !changed++ { $8="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/awk -F '|' 'BEGIN { OFS="|" } NR==3 { $7="1"; $8="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" } { print }' \
    "$evidence_g/actions.v2" > "$evidence_g/actions.v2.mut"
/bin/mv "$evidence_g/actions.v2.mut" "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/usr/bin/printf 'component:gui-responsive' > "$evidence_g/serial.log"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/bin/mv "$evidence_g/actions.v2" "$evidence_g/actions.real"
/bin/ln -s actions.real "$evidence_g/actions.v2"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

/bin/rm -rf "$evidence_g"
make_evidence "$evidence_g" 7
/bin/ln "$evidence_g/actions.v2" "$work/actions-hardlink"
if /bin/sh "$checker" "$evidence_g" milestone-g 1 >/dev/null 2>&1; then exit 1; fi

overflow=$work/overflow
make_evidence "$overflow" 1
/usr/bin/awk 'BEGIN { for (i = 0; i < 600000; i++) printf "0123456789abcdef" }' > "$overflow/serial.log"
if /bin/sh "$checker" "$overflow" milestone-a 64 >/dev/null 2>&1; then exit 1; fi

printf '%s\n' 'launch evidence negative checks passed'
