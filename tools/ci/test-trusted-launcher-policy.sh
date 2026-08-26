#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
output_root=$root/out
/bin/mkdir -p "$output_root"
work=$(mktemp -d "$output_root/trusted-launcher.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-trusted-launcher-policy.sh
canonical=$root/tools/ci/launch-cloud-target.sh
/bin/sh "$checker" "$canonical" >/dev/null

expect_rejected() {
    label=$1
    if /bin/sh "$checker" "$work/launcher" >/dev/null 2>&1; then
        printf 'unsafe launcher unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}
/usr/bin/sed '/-sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny/d' "$canonical" > "$work/launcher"
expect_rejected missing-sandbox
/usr/bin/sed 's/-nic none/-nic user/' "$canonical" > "$work/launcher"
expect_rejected user-network
/usr/bin/sed 's/-snapshot/-enable-kvm -snapshot/' "$canonical" > "$work/launcher"
expect_rejected host-acceleration
/usr/bin/sed '/wait-for-launch-release.sh/d' "$canonical" > "$work/launcher"
expect_rejected missing-release-budget
printf '%s\n' 'trusted launcher negative checks passed'
