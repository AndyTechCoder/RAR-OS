#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
generator=$root/sdk/generated/release-0/generate.sh
generated=$root/sdk/generated/release-0/lib.rs

/bin/mkdir -p "$root/out"
rendered=$(/usr/bin/mktemp "$root/out/r0-002-generated.XXXXXX")
cleanup_rendered() { /bin/rm -f "$rendered"; }
trap cleanup_rendered EXIT HUP INT TERM
"$generator" > "$rendered"
/usr/bin/cmp "$rendered" "$generated" || {
    echo "generated Rust differs from canonical schemas" >&2
    exit 1
}

case "${1-}" in
    '') ;;
    --compile)
        [ "${CI-}" = true ] && [ "${GITHUB_ACTIONS-}" = true ] || {
            echo "generated Rust compilation is restricted to the pinned CI route" >&2
            exit 1
        }
        . "$root/tools/toolchain/preauth-build-root.sh"
        rustc_path=$(preauth_build_pinned_rustc_path "$root") || {
            echo "pinned Rust 1.95 compiler unavailable" >&2
            exit 1
        }
        temporary=$(/usr/bin/mktemp -d /tmp/rar-r0-generated.XXXXXX)
        cleanup() {
            /usr/bin/rm -f "$temporary/lib.rmeta"
            /usr/bin/rmdir "$temporary"
        }
        trap 'cleanup; cleanup_rendered' EXIT HUP INT TERM
        "$rustc_path" \
            --crate-name rar_r0_contracts \
            --crate-type lib \
            --emit metadata \
            --edition 2024 \
            -o "$temporary/lib.rmeta" \
            "$generated"
        ;;
    *)
        echo "usage: check.sh [--compile]" >&2
        exit 64
        ;;
esac

echo "R0-002 generated Rust regeneration passed"
