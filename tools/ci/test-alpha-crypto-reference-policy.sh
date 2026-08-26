#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root=$root/out
/bin/mkdir -p "$output_root"
work=$(mktemp -d "$output_root/alpha-crypto.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
fixture=$work/references.env
checker=$root/tools/ci/check-alpha-crypto-references.sh
/bin/sh "$checker" >/dev/null
/usr/bin/sed \
    -e 's/^state=blocked$/state=ready/' \
    -e 's|^path_1=unavailable$|path_1=/opt/rar-reference/bin/openssl|' \
    -e 's/^sha256_1=unavailable$/sha256_1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' \
    -e 's|^path_2=unavailable$|path_2=/opt/rar-reference/bin/libsodium-reference|' \
    -e 's/^sha256_2=unavailable$/sha256_2=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/' \
    "$root/tools/sprint-alpha/alpha-crypto-references-v1.env" > "$fixture"
/bin/sh "$checker" "$fixture" >/dev/null
/usr/bin/sed 's|^path_2=.*$|path_2=/opt/rar-reference/bin/openssl|' "$fixture" > "$work/bad"
if /bin/sh "$checker" "$work/bad" >/dev/null 2>&1; then exit 1; fi
/usr/bin/sed 's/^sha256_2=.*$/sha256_2=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' "$fixture" > "$work/bad"
if /bin/sh "$checker" "$work/bad" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'Alpha crypto reference negative checks passed'
