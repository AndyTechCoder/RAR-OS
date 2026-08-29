#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

launcher=${1-}
[ -f "$launcher" ] && [ ! -L "$launcher" ] || exit 1
grep -Fq -- '-sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny' "$launcher" || exit 1
for required in '-nodefaults' '-S' '-machine q35,accel=tcg' '-nic none' '-no-reboot' '-snapshot' '-display none' '-monitor none'; do
    grep -Fq -- "$required" "$launcher" || exit 1
done
for forbidden in '-enable-kvm' '-accel kvm' '-nic user' '-net user' '-netdev user' '-netdev tap' '-virtfs' '-fsdev' '-usb-host' '-device vfio' '-drive file=/dev/' '-kernel ' '-append '; do
    ! grep -Fq -- "$forbidden" "$launcher" || exit 1
done
[ "$(grep -c '^"\$qemu" \\' "$launcher")" -eq 1 ] || exit 1
grep -Fq '/controller/tools/ci/wait-for-launch-release.sh /control/to-launch "$RAR_LAUNCH_TIMEOUT_SECONDS"' "$launcher" || exit 1
printf '%s\n' 'trusted launcher policy passed'
