#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'launch handshake mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/launch-handshake.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
/bin/mkdir "$work/to-host" "$work/to-launch"
# Permission preparation is Linux-CI-only. The Mac test reads the fixed policy
# and models the distinct-UID bits without attempting any SSD mode change.
/usr/bin/grep -Fqx '[ "$(/usr/bin/uname -s)" = Linux ] || exit 1' "$root/tools/ci/prepare-launch-control.sh" || exit 1
/usr/bin/grep -Fqx '[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] || exit 1' "$root/tools/ci/prepare-launch-control.sh" || exit 1
/usr/bin/grep -Fqx '/bin/chmod 0711 "$control"' "$root/tools/ci/prepare-launch-control.sh" || exit 1
/usr/bin/grep -Fqx '/bin/chmod 0733 "$control/to-host"' "$root/tools/ci/prepare-launch-control.sh" || exit 1
/usr/bin/grep -Fqx '/bin/chmod 0755 "$control/to-launch"' "$root/tools/ci/prepare-launch-control.sh" || exit 1
# A distinct container UID receives write+search only on to-host and cannot
# modify the host-owned release channel; these exact other-mode bits encode it.
[ "$((0733 & 0003))" -eq 3 ] && [ "$((0733 & 0004))" -eq 0 ] || exit 1
[ "$((0755 & 0002))" -eq 0 ] && [ "$((0755 & 0005))" -eq 5 ] || exit 1
(
    /bin/sleep 1
    /usr/bin/printf '%s\n' ready > "$work/to-host/evidence-ready"
    /usr/bin/printf '%s\n' release > "$work/to-launch/release"
) &
writer=$!
/bin/sh "$root/tools/ci/wait-for-launch-release.sh" "$work/to-launch" 3
wait "$writer"
/bin/rm -f "$work/to-launch/release"
if /bin/sh "$root/tools/ci/wait-for-launch-release.sh" "$work/to-launch" 1 >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' wrong > "$work/to-launch/release"
if /bin/sh "$root/tools/ci/wait-for-launch-release.sh" "$work/to-launch" 1 >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'launch handshake negative checks passed'
