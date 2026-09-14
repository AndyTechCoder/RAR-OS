#!/bin/sh
# Same fixed serial-only Foundation machine, in a new owned cloud container.
# No selectable arguments, guest network, passthrough or owner storage.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
[ -f /artifact/boot.img ] && [ ! -L /artifact/boot.img ]
[ "$(stat -c %s /artifact/boot.img)" -eq 16777216 ]
sha256sum -c /opt/identities.sha256 >&2
ulimit -f 65536
mkdir /tmp/rar-snapshot
export TMPDIR=/tmp/rar-snapshot
cp /usr/share/OVMF/OVMF_VARS.fd /tmp/OVMF_VARS.fd
set +e
timeout --signal=TERM --kill-after=2 25   /usr/bin/qemu-system-x86_64   -machine q35,accel=tcg -cpu qemu64 -smp 1 -m 256M   -nodefaults -no-user-config -display none -monitor none -serial stdio   -nic none -no-reboot -no-shutdown   -sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny   -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd   -drive if=pflash,format=raw,file=/tmp/OVMF_VARS.fd   -drive if=ide,format=raw,snapshot=on,file=/artifact/boot.img
result=$?
set -e
# Only the exact expected timeout of a successfully halted Foundation guest.
# The independent outer checker still requires all node result markers.
[ "$result" -eq 124 ]
printf 'RAR-NODE:CLOUD-STOPPED=124\n'
