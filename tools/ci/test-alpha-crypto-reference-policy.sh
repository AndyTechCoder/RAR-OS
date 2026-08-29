#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'alpha crypto policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/alpha-crypto.XXXXXX")
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
if /bin/sh "$checker" "$fixture" >/dev/null 2>&1; then exit 1; fi
if /bin/sh "$checker" "$root/tools/sprint-alpha/alpha-crypto-references-v1.env" --require-ready >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'Alpha crypto reference negative checks passed'
