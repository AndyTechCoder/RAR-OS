#!/bin/sh
# Trusted compile-only second stage. Never run on the Mac or SSD.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
for name in update bad-health bad-signature bad-abi factory; do
    input="/inputs/modern-settings-$name.layer"
    [ -f "$input" ] && [ ! -L "$input" ]
    [ ! -e "/tmp/modern-settings-$name.layer" ]
    cp "$input" "/tmp/modern-settings-$name.layer"
done
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=s -C codegen-units=1 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C relocation-model=static \
  -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  -C link-arg=/base:0x400000 -C link-arg=/fixed \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_signed_updates /source/core/modern/main.rs -o /tmp/modern-service.efi
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=2 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_platform --cfg rar_modern --cfg rar_signed_updates --cfg 'rar_profile="normal"' \
  /source/nucleus/foundation/main.rs -o /tmp/modern.efi
for name in modern.efi modern-service.efi; do
    printf 'RAR-SIGNED-FILE:%s\n' "$name"
    base64 -w 0 "/tmp/$name"
    printf '\n'
done
printf 'RAR-SIGNED-BUILD:END\n'
