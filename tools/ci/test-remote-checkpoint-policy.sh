#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
checker=$root/tools/ci/verify-remote-checkpoint.sh
tag=sprint-alpha-0.1/A
head=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
object=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
valid=$(printf '%s\n%s\n' "$object refs/tags/$tag" "$head refs/tags/$tag^{}")

/bin/sh "$checker" "$tag" "$head" "$valid" >/dev/null

expect_rejected() {
    label=$1
    records=$2
    if /bin/sh "$checker" "$tag" "$head" "$records" >/dev/null 2>&1; then
        printf 'unsafe remote checkpoint unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

expect_rejected missing-tag ''
expect_rejected lightweight-tag "$head refs/tags/$tag"
expect_rejected missing-peeled "$object refs/tags/$tag"
expect_rejected moved-tag "$(printf '%s\n%s\n' "$object refs/tags/$tag" "cccccccccccccccccccccccccccccccccccccccc refs/tags/$tag^{}")"
expect_rejected unexpected-record "$(printf '%s\n%s\n%s\n' "$object refs/tags/$tag" "$head refs/tags/$tag^{}" "$head refs/tags/other")"
if /bin/sh "$checker" unrelated/v1 "$head" "$valid" >/dev/null 2>&1; then
    printf '%s\n' 'unsafe remote checkpoint unexpectedly passed: wrong phase tag' >&2
    exit 1
fi

printf '%s\n' 'remote checkpoint negative checks passed'
