#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'portable stat policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/portable-stat.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-portable-stat-policy.sh
fixture=$work/source

files='tools/ci/test-proposed-adr-classifier-policy.sh
tools/ci/hash-source-tree.sh
tools/ci/check-containerfile-static-policy.sh
tools/ci/check-reference-verdict-v0.sh
tools/ci/check-controller-handoff-attempt-v0.sh
tools/ci/check-controller-helper-build-receipt-v0.sh
tools/ci/check-reference-evidence-v0.sh
tools/ci/verify-launch-evidence.sh
tools/ci/verify-accepted-evidence-v0.sh
tools/ci/test-controller-helper-evidence-v0-policy.sh
tools/ci/check-controller-helper-build-evidence-v0.sh
tools/ci/check-controller-helper-test-evidence-v0.sh
tools/ci/test-reference-verdict-v0-policy.sh
tools/ci/check-development-image-inputs.sh'

reset_fixture() {
    /bin/rm -rf "$fixture"
    /bin/mkdir -p "$fixture/tools/ci"
    printf '%s\n' "$files" | while IFS= read -r file; do
        /bin/cp "$root/$file" "$fixture/$file"
    done
}

expect_rejected() {
    label=$1
    if RAR_POLICY_MUTATION_TESTS=1 /bin/sh "$checker" "$fixture" >"$work/result" 2>&1; then
        printf 'unsafe portable stat mutation unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

reset_fixture
[ "$(RAR_POLICY_MUTATION_TESTS=1 /bin/sh "$checker" "$fixture")" = \
    'portable stat policy passed: files=14 fallbacks=30' ] || exit 1

if /usr/bin/env -u GITHUB_ACTIONS RAR_POLICY_MUTATION_TESTS=1 \
    /bin/sh "$checker" "$fixture" >/dev/null 2>&1; then
    printf '%s\n' 'portable stat checker accepted missing CI attestation' >&2
    exit 1
fi

reset_fixture
/usr/bin/sed 's#/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file"#/usr/bin/stat -f %z "$file" 2>/dev/null || /usr/bin/stat -c %s "$file"#' \
    "$fixture/tools/ci/check-controller-handoff-attempt-v0.sh" > "$work/bad"
/bin/mv "$work/bad" "$fixture/tools/ci/check-controller-handoff-attempt-v0.sh"
expect_rejected bsd-first-order

reset_fixture
replacement='identity() { /usr/bin/stat -f '\''%d:%i:%z:%l:%u:%m'\'' "$1" 2>/dev/null || /usr/bin/stat -c '\''%d:%i:%s:%h:%u:%Y'\'' "$1"; }'
/usr/bin/awk -v replacement="$replacement" \
    '/^identity\(\)/ { print replacement; next } { print }' \
    "$fixture/tools/ci/check-controller-helper-build-receipt-v0.sh" > "$work/bad"
/bin/mv "$work/bad" "$fixture/tools/ci/check-controller-helper-build-receipt-v0.sh"
expect_rejected quoted-identity-bsd-first

reset_fixture
/usr/bin/printf '%s\n' 'SECRET_PORTABLE_STAT_CANARY' > "$work/canary"
/bin/rm -f "$fixture/tools/ci/check-controller-handoff-attempt-v0.sh"
/bin/ln -s "$work/canary" "$fixture/tools/ci/check-controller-handoff-attempt-v0.sh"
expect_rejected symbolic-validator
! /usr/bin/grep -Fq SECRET_PORTABLE_STAT_CANARY "$work/result" || exit 1

reset_fixture
/usr/bin/printf '%s\n' 'extra=$(/usr/bin/stat -f %z "$file" 2>/dev/null || /usr/bin/stat -c %s "$file")' \
    >> "$fixture/tools/ci/check-controller-handoff-attempt-v0.sh"
expect_rejected extra-stat-wrapper

printf '%s\n' 'portable stat policy negative checks passed'
