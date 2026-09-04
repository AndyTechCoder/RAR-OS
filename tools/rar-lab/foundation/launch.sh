#!/bin/sh
# Sole emulator entry point for Foundation. No user-selectable arguments.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
[ -f /artifact/boot.img ] && [ ! -L /artifact/boot.img ]
[ "$(stat -c %s /artifact/boot.img)" -eq 16777216 ]
sha256sum -c /opt/identities.sha256 >&2
ulimit -f 512
exec timeout --signal=TERM --kill-after=2 25 \
  /usr/bin/qemu-system-x86_64 \
  -machine q35,accel=tcg -cpu qemu64 -smp 1 -m 256M \
  -nodefaults -no-user-config -display none -monitor none -serial stdio \
  -nic none -no-reboot -no-shutdown \
  -sandbox on,obsolete=deny,elevateprivileges=deny,spawn=deny,resourcecontrol=deny \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd \
  -drive if=ide,format=raw,readonly=on,file=/artifact/boot.img
