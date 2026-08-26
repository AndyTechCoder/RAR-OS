#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output=$root/out
[ ! -L "$output" ] || exit 1
/bin/mkdir -p "$output"
work=$(mktemp -d "$output/controller-helper.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
source=$root/tools/sprint-alpha/controller-helper-v0.env
checker=$root/tools/ci/check-controller-helper-inventory-v0.sh
candidate=$work/inventory.env

reject() {
    label=$1
    if /bin/sh "$checker" "$candidate" >/dev/null 2>&1; then
        printf 'unsafe controller helper inventory unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

/bin/sh "$checker" "$source" >/dev/null
/usr/bin/sed 's/^state=blocked$/state=ready/' "$source" > "$candidate"; reject ready
/usr/bin/sed 's/^decision=unavailable$/decision=adr-0024-alternative-a/' "$source" > "$candidate"; reject blocked-decision
/usr/bin/sed 's/^topology=unavailable$/topology=runner-closure/' "$source" > "$candidate"; reject blocked-topology
/usr/bin/sed 's/^compiler_sha256=unavailable$/compiler_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' "$source" > "$candidate"; reject blocked-compiler
/usr/bin/sed 's/^binary_bytes=unavailable$/binary_bytes=1/' "$source" > "$candidate"; reject blocked-binary
/usr/bin/sed 's/^dependency_count=0$/dependency_count=1/' "$source" > "$candidate"; reject dependency
/usr/bin/sed 's/^target_linked=false$/target_linked=true/' "$source" > "$candidate"; reject target-linked
/usr/bin/awk 'NR == 3 { saved=$0; next } NR == 4 { print; print saved; next } { print }' "$source" > "$candidate"; reject reordered
/usr/bin/sed '2p' "$source" > "$candidate"; reject duplicate
/bin/cp "$source" "$candidate"; /usr/bin/printf '%s\n' extra=value >> "$candidate"; reject extra
/bin/cp "$source" "$work/real.env"; /bin/rm -f "$candidate"; /bin/ln -s real.env "$candidate"; reject symbolic

printf '%s\n' 'controller helper inventory negative checks passed'
