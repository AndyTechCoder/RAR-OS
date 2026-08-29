#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'lab profile v2 mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/lab-profile-v2.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
source=$root/tools/sprint-alpha/development-lab-v2.env
checker=$root/tools/ci/check-development-lab-profile-v2.sh
profile=$work/lab-v2.env

expect_rejected() {
    label=$1
    if /bin/sh "$checker" "$profile" >/dev/null 2>&1; then
        printf 'unsafe Development Lab v2 profile unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

/bin/sh "$checker" "$source" >/dev/null

/usr/bin/sed 's/^build_oci_image=unavailable$/build_oci_image=registry.invalid\/build@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' "$source" > "$profile"
expect_rejected activating-blocked-profile

/usr/bin/sed 's/^state=blocked$/state=ready/' "$source" > "$profile"
expect_rejected unreviewed-ready-profile

/usr/bin/sed 's/^state=blocked$/state=unknown/' "$source" > "$profile"
expect_rejected unknown-state

/usr/bin/sed '/^reference_oci_image=/d' "$source" > "$profile"
expect_rejected missing-role-image

/bin/cp "$source" "$profile"
/usr/bin/printf '%s\n' 'state=blocked' >> "$profile"
expect_rejected duplicate-key

/usr/bin/sed 's/^timeout_seconds=1200$/timeout_seconds=1201/' "$source" > "$profile"
expect_rejected widened-timeout

/usr/bin/sed 's/^transcript_mib=1$/transcript_mib=2/' "$source" > "$profile"
expect_rejected widened-transcript

/usr/bin/sed 's/^schema=.*$/schema=$(id)/' "$source" > "$profile"
expect_rejected shell-syntax

/bin/mv "$profile" "$work/real.env"
/bin/ln -s real.env "$profile"
expect_rejected symbolic-profile

printf '%s\n' 'Development Lab v2 profile negative checks passed'
