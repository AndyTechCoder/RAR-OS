#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'alpha contract policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/alpha-contracts.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-alpha-preimplementation-contracts.sh
source=$root/spec/alpha

require_checker_line() {
    [ "$(/usr/bin/grep -Fxc -- "$1" "$checker")" -eq 1 ] || {
        printf 'Alpha contract policy test blocked: missing or duplicate checker binding: %s\n' "$1" >&2
        exit 1
    }
}

require_checker_line '    boot_cases_digest=1a59e0d9135b018d46fbb70318f53ba79a876c580b8ef1ce174f0b5eeb7c7222'
require_checker_line '    boot_case_count=50'
require_checker_line '    boot_cases_digest=370f829f791681cb4c1fb96dbf850f9535751a7a64295534562ea47a9f84bee3'
require_checker_line '    boot_case_count=41'
require_checker_line 'require_digest "$boot/cases.v0" "$boot_cases_digest"'
require_checker_line 'validate_case_file "$boot/cases.v0" '\''schema=rar-alpha-boot-cases-v0'\'' "$boot_case_count"'

expect_rejected() {
    label=$1
    if /bin/sh "$checker" "$work/alpha" >/dev/null 2>&1; then
        printf 'unsafe Alpha contract mutation unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

reset_fixture() {
    /bin/rm -rf "$work/alpha"
    /bin/mkdir -p "$work/alpha"
    /bin/cp -R "$source/lab" "$source/boot" "$source/evidence" "$work/alpha/"
    if [ -e "$source/platform" ] || [ -L "$source/platform" ]; then
        /bin/cp -R "$source/platform" "$work/alpha/"
    fi
    find "$work/alpha" -name '._*' -type f -exec /bin/rm -f {} \;
    /bin/sh "$checker" "$work/alpha" >/dev/null || {
        printf '%s\n' 'Alpha contract policy test blocked: reset fixture is not a valid baseline' >&2
        exit 1
    }
}

/bin/sh "$checker" >/dev/null

reset_fixture
/usr/bin/sed '/^role_rule|reference|/d' "$work/alpha/lab/development-lab-profile-v2.fields" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/lab/development-lab-profile-v2.fields"
expect_rejected missing-reference-isolation

reset_fixture
/usr/bin/sed 's/controller,machine-profile/reference-binary,machine-profile/' "$work/alpha/lab/development-lab-profile-v2.fields" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/lab/development-lab-profile-v2.fields"
expect_rejected weakened-build-runtime-boundary

reset_fixture
/usr/bin/sed 's/^maximum_total_bytes=1048576$/maximum_total_bytes=unbounded/' "$work/alpha/lab/comparison-transcript-v0.fields" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/lab/comparison-transcript-v0.fields"
expect_rejected unbounded-transcript

reset_fixture
/usr/bin/sed 's/^r0_source_producer=recovery-only$/r0_source_producer=root-or-recovery/' "$work/alpha/boot/alpha-boot-v0.fields" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/boot/alpha-boot-v0.fields"
expect_rejected mixed-r0-producer

reset_fixture
/usr/bin/sed 's/^r0_handoff_contract_sha256=./r0_handoff_contract_sha256=f/' "$work/alpha/boot/alpha-boot-v0.fields" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/boot/alpha-boot-v0.fields"
expect_rejected stale-r0-binding

reset_fixture
/usr/bin/sed '$d' "$work/alpha/boot/cases.v0" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/boot/cases.v0"
expect_rejected missing-negative-case

reset_fixture
/usr/bin/sed 's/^missing-nucleus-path|/missing-recovery-path|/' "$work/alpha/boot/cases.v0" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/boot/cases.v0"
expect_rejected duplicate-case-id

reset_fixture
/usr/bin/sed 's/^unknown-firmware-type|recovery-map|reject$/unknown-firmware-type|recovery-map|reject|extra/' "$work/alpha/boot/cases.v0" > "$work/bad"
/bin/mv "$work/bad" "$work/alpha/boot/cases.v0"
expect_rejected malformed-case-row

reset_fixture
if [ -e "$work/alpha/platform/contract-set-v0.manifest" ] ||
    [ -L "$work/alpha/platform/contract-set-v0.manifest" ]; then
    /bin/mv "$work/alpha/platform/contract-set-v0.manifest" "$work/removed-p0-manifest"
else
    /bin/mkdir -p "$work/alpha/platform"
    /usr/bin/printf '%s\n' partial-p0 > "$work/alpha/platform/partial.fixture"
fi
expect_rejected partial-p0-without-manifest

reset_fixture
/bin/mv "$work/alpha/boot/alpha-boot-v0.fields" "$work/alpha/boot/real.fields"
/bin/ln -s real.fields "$work/alpha/boot/alpha-boot-v0.fields"
expect_rejected symbolic-contract

printf '%s\n' 'Alpha preimplementation contract negative checks passed'
