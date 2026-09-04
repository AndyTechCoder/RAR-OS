#!/bin/sh
set -eu

PATH=/usr/bin:/bin
LC_ALL=C
LANG=C
export PATH LC_ALL LANG

fail() {
    printf 'Specifications authority check failed: %s\n' "$1" >&2
    exit 1
}

[ "$#" -eq 3 ] || fail 'trusted root, source root, and output path are required'
trusted_root=$1
source_root=$2
output=$3

[ -d "$trusted_root" ] && [ ! -L "$trusted_root" ] || fail 'trusted root is invalid'
[ -d "$source_root" ] && [ ! -L "$source_root" ] || fail 'source root is invalid'
[ -n "$output" ] || fail 'output path is empty'

trusted_root=$(CDPATH= cd -- "$trusted_root" && pwd -P)
source_root=$(CDPATH= cd -- "$source_root" && pwd -P)
[ "$trusted_root" != "$source_root" ] || fail 'trusted and source roots are not independent'

case "${RAR_TRUSTED_CONTROLLER_SHA-}" in *[!0-9a-f]*|'') fail 'trusted controller SHA is malformed' ;; esac
case "${RAR_EXPECTED_SOURCE_REVISION-}" in *[!0-9a-f]*|'') fail 'source SHA is malformed' ;; esac
[ "${#RAR_TRUSTED_CONTROLLER_SHA}" -eq 40 ] || fail 'trusted controller SHA length is invalid'
[ "${#RAR_EXPECTED_SOURCE_REVISION}" -eq 40 ] || fail 'source SHA length is invalid'
[ "${RAR_EXPECTED_SOURCE_REPOSITORY-}" = "${RAR_CANONICAL_REPOSITORY-}" ] ||
    fail 'only canonical-repository pull requests are accepted'

git_read() {
    repository=$1
    shift
    /usr/bin/env -i \
        HOME=/nonexistent-rar-specifications-controller \
        PATH=/usr/bin:/bin \
        LC_ALL=C \
        LANG=C \
        GIT_CONFIG_NOSYSTEM=1 \
        GIT_OPTIONAL_LOCKS=0 \
        /usr/bin/git \
        -c core.hooksPath=/dev/null \
        -c core.fsmonitor=false \
        -c core.untrackedCache=false \
        -C "$repository" "$@"
}

[ "$(git_read "$trusted_root" rev-parse HEAD)" = "$RAR_TRUSTED_CONTROLLER_SHA" ] ||
    fail 'trusted checkout does not match the controller SHA'
[ "$(git_read "$source_root" rev-parse HEAD)" = "$RAR_EXPECTED_SOURCE_REVISION" ] ||
    fail 'source checkout does not match the requested SHA'

trusted_status=$(git_read "$trusted_root" status --porcelain=v1 --untracked-files=all --ignored=matching) ||
    fail 'cannot verify trusted checkout state'
source_status=$(git_read "$source_root" status --porcelain=v1 --untracked-files=all --ignored=matching) ||
    fail 'cannot verify source checkout state'
[ -z "$trusted_status" ] || fail 'trusted checkout is not exact and clean'
[ -z "$source_status" ] || fail 'source checkout is not exact and clean'

set -- \
    .github/workflows/specifications.yml \
    tools \
    spec/alpha/evidence \
    spec/alpha/lab/fixtures \
    spec/fixtures/release-0 \
    sdk/generated/release-0 \
    tests \
    Cargo.toml \
    rust-toolchain.toml \
    rustfmt.toml

trusted_index=$(git_read "$trusted_root" ls-files -s -- "$@") ||
    fail 'cannot enumerate trusted authority closure'
source_index=$(git_read "$source_root" ls-files -s -- "$@") ||
    fail 'cannot enumerate source authority closure'
[ -n "$trusted_index" ] || fail 'trusted authority closure is empty'

if /usr/bin/printf '%s\n' "$trusted_index" | /usr/bin/awk '$1 !~ /^100(644|755)$/ { exit 1 }'; then
    :
else
    fail 'trusted authority closure contains a symlink or submodule'
fi
if /usr/bin/printf '%s\n' "$source_index" | /usr/bin/awk '$1 !~ /^100(644|755)$/ { exit 1 }'; then
    :
else
    fail 'source authority closure contains a symlink or submodule'
fi

if [ "$trusted_index" = "$source_index" ]; then
    /usr/bin/printf '%s\n' 'execution=full' >> "$output"
    /usr/bin/printf '%s\n' 'Specifications authority: trusted closure is byte-identical; executable validation enabled'
else
    /usr/bin/printf '%s\n' 'execution=isolated-proposal' >> "$output"
    /usr/bin/printf '%s\n' 'Specifications authority: controller change detected; trusted workflow permits only isolated proposal validation'
fi
