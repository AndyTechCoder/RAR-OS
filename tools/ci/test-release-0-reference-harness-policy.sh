#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'Release 0 reference-harness mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/release-0-reference-harness.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
rustc=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc
binary=/build/release-0-reference-harness

"$rustc" --edition 2024 -D warnings -o "$binary" "$root/spec/fixtures/release-0/reference.rs"

reset_fixture() {
    /bin/rm -rf "$work/spec"
    /bin/mkdir -p "$work/spec/fixtures" "$work/spec/boot"
    /bin/cp -R "$root/spec/fixtures/release-0" "$work/spec/fixtures/release-0"
    /bin/cp "$root/spec/boot/handoff-v1.fields" "$work/spec/boot/handoff-v1.fields"
}

expect_rejected() {
    label=$1
    fixture_root=${2-$work/spec/fixtures/release-0}
    if "$binary" "$fixture_root" >/dev/null 2>&1; then
        printf 'unsafe Release 0 reference fixture unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

reset_fixture
"$binary" "$work/spec/fixtures/release-0" >/dev/null

reset_fixture
/usr/bin/sed '3s|[^|]*$|../../boot/handoff-v1.fields|' "$work/spec/fixtures/release-0/cases.v1" > "$work/cases.v1"
/bin/mv "$work/cases.v1" "$work/spec/fixtures/release-0/cases.v1"
expect_rejected case-path-traversal

reset_fixture
/usr/bin/sed '3s|[^|]*$|/etc/passwd|' "$work/spec/fixtures/release-0/cases.v1" > "$work/cases.v1"
/bin/mv "$work/cases.v1" "$work/spec/fixtures/release-0/cases.v1"
expect_rejected absolute-case-path

reset_fixture
/usr/bin/sed '3s/|x86_64|/|..\/..\/boot\/handoff-v1.fields|/' "$work/spec/fixtures/release-0/conformance-scenarios.v1" > "$work/scenarios.v1"
/bin/mv "$work/scenarios.v1" "$work/spec/fixtures/release-0/conformance-scenarios.v1"
expect_rejected architecture-path-traversal

reset_fixture
/bin/ln -s "$work/spec/fixtures/release-0" "$work/root-link"
expect_rejected symbolic-root "$work/root-link"

reset_fixture
/bin/mv "$work/spec/fixtures/release-0/cases.v1" "$work/real-cases.v1"
/bin/ln -s "$work/real-cases.v1" "$work/spec/fixtures/release-0/cases.v1"
expect_rejected symbolic-manifest

reset_fixture
first=$(/usr/bin/sed -n '3s/.*|//p' "$work/spec/fixtures/release-0/cases.v1")
/bin/mv "$work/spec/fixtures/release-0/bin/$first" "$work/real-binary"
/bin/ln -s "$work/real-binary" "$work/spec/fixtures/release-0/bin/$first"
expect_rejected symbolic-binary

reset_fixture
first=$(/usr/bin/sed -n '3s/.*|//p' "$work/spec/fixtures/release-0/cases.v1")
/bin/rm -f "$work/spec/fixtures/release-0/bin/$first"
/usr/bin/mkfifo "$work/spec/fixtures/release-0/bin/$first"
expect_rejected fifo-binary

reset_fixture
first=$(/usr/bin/sed -n '3s/.*|//p' "$work/spec/fixtures/release-0/cases.v1")
/usr/bin/awk 'BEGIN { for (i = 0; i < 81985; i++) printf "a" }' > "$work/spec/fixtures/release-0/bin/$first"
expect_rejected oversized-binary

reset_fixture
/usr/bin/awk 'BEGIN { for (i = 0; i < 65537; i++) printf "a" }' > "$work/spec/fixtures/release-0/cases.v1"
expect_rejected oversized-text

reset_fixture
/usr/bin/awk 'BEGIN { for (i=0; i<1025; i++) printf "a"; printf "\n" }' > "$work/spec/fixtures/release-0/cases.v1"
expect_rejected oversized-line

reset_fixture
/usr/bin/awk 'BEGIN { for (i=0; i<513; i++) print "a" }' > "$work/spec/fixtures/release-0/cases.v1"
expect_rejected excessive-lines

printf '%s\n' 'Release 0 reference-harness path and read-bound checks passed'
