#!/bin/sh
# Fixed Foundation-only portable node source. Compile, never execute here.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
rustc --edition 2024 --target x86_64-unknown-uefi   -C opt-level=z -C panic=abort -C no-redzone=yes -C debuginfo=0 -C strip=symbols   -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE   --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build   --cfg rar_node --cfg 'rar_profile="normal"' /source/nucleus/foundation/main.rs -o /tmp/node.efi
printf 'RAR-NODE-FILE:node.efi\n'
base64 -w 0 /tmp/node.efi
printf '\nRAR-NODE-BUILD:END\n'
