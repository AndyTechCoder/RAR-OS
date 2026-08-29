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
    /bin/cp -R "$source/lab" "$source/boot" "$work/alpha/"
    find "$work/alpha" -name '._*' -type f -exec /bin/rm -f {} \;
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
/bin/mv "$work/alpha/boot/alpha-boot-v0.fields" "$work/alpha/boot/real.fields"
/bin/ln -s real.fields "$work/alpha/boot/alpha-boot-v0.fields"
expect_rejected symbolic-contract

printf '%s\n' 'Alpha preimplementation contract negative checks passed'
