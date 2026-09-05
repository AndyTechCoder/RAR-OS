#!/bin/sh
# Trusted cloud container only; no source-selected command or option.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
[ -f /artifact/boot.img ] && [ ! -L /artifact/boot.img ]
[ "$(stat -c %s /artifact/boot.img)" -eq 16777216 ]
sha256sum -c /opt/identities.sha256 >/dev/null
ulimit -f 65536
exec timeout --signal=TERM --kill-after=2 30 /usr/bin/python3 -I -B /opt/rar-launch.py
