#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
fixture_root=$root/spec/fixtures/release-0
temporary=$root/out/r0-002-fixture-check.$$
reference_binary=

/bin/mkdir -p "$root/out"
/bin/mkdir "$temporary"
cleanup() {
    if [ -n "$reference_binary" ]; then
        /bin/rm -f "$reference_binary"
    fi
    while IFS='|' read -r id expected file; do
        case "$id" in schema=*|id|'') continue ;; esac
        /bin/rm -f "$temporary/$file"
    done < "$fixture_root/cases.v1"
    /bin/rmdir "$temporary"
}
trap cleanup EXIT HUP INT TERM

"$fixture_root/generate.sh" "$temporary"
count=0
while IFS='|' read -r id expected file; do
    case "$id" in schema=*|id|'') continue ;; esac
    /usr/bin/cmp "$temporary/$file" "$fixture_root/bin/$file" || {
        echo "binary fixture differs from deterministic generator: $file" >&2
        exit 1
    }
    count=$((count + 1))
done < "$fixture_root/cases.v1"
[ "$count" -eq 23 ] || { echo "binary fixture corpus is incomplete" >&2; exit 1; }

case "${1-}" in
    '') ;;
    --ci)
        [ "${CI-}" = true ] && [ "${GITHUB_ACTIONS-}" = true ] || {
            echo "reference decoder execution is restricted to the pinned CI route" >&2
            exit 1
        }
        . "$root/tools/toolchain/preauth-build-root.sh"
        rustc_path=$(preauth_build_pinned_rustc_path "$root") || {
            echo "pinned Rust 1.95 compiler unavailable" >&2
            exit 1
        }
        reference_binary=$root/out/r0-002-reference.$$
        "$rustc_path" \
            --edition 2024 \
            -D warnings \
            -o "$reference_binary" \
            "$fixture_root/reference.rs"
        "$reference_binary" "$fixture_root"
        ;;
    *) echo "usage: run.sh [--ci]" >&2; exit 64 ;;
esac

echo "R0-002 binary fixture regeneration passed: $count fixtures"
