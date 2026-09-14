#!/bin/sh
# Trusted compile-only second stage. Never run on the Mac or SSD.
set -eu
[ "$#" -eq 2 ]
case "$2" in faults) case_cfg="--cfg rar_network_fault_peer" ;; expiry) case_cfg="--cfg rar_network_expiry" ;; *) exit 2 ;; esac
case "$1" in a) peer_cfg="" ;; b) peer_cfg="--cfg rar_network_peer_b" ;; *) exit 2 ;; esac
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
for app in rar-notes.app rar-counter.app; do
    [ -f "/inputs/$app" ] && [ ! -L "/inputs/$app" ]
    [ ! -e "/tmp/$app" ]
    cp "/inputs/$app" "/tmp/$app"
done
for name in update bad-health bad-signature bad-abi factory; do
    input="/inputs/modern-settings-$name.layer"
    [ -f "$input" ] && [ ! -L "$input" ]
    [ ! -e "/tmp/modern-settings-$name.layer" ]
    cp "$input" "/tmp/modern-settings-$name.layer"
done
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=z -C lto=fat -C codegen-units=1 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C relocation-model=static \
  -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  -C link-arg=/base:0x400000 -C link-arg=/fixed \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_signed_updates --cfg rar_expansion --cfg rar_applications $peer_cfg /source/core/modern/main.rs -o /tmp/modern-service.efi
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=z -C lto=fat -C codegen-units=1 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C relocation-model=static \
  -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  -C link-arg=/base:0x400000 -C link-arg=/fixed \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_network_service_only --cfg rar_signed_updates --cfg rar_expansion --cfg rar_applications $peer_cfg $case_cfg /source/core/modern/main.rs -o /tmp/modern-network.efi
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=2 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_platform --cfg rar_modern --cfg rar_signed_updates --cfg rar_expansion --cfg rar_applications --cfg 'rar_profile="normal"' \
  /source/nucleus/foundation/main.rs -o /tmp/modern.efi
for name in modern.efi modern-service.efi modern-network.efi; do
    printf 'RAR-SIGNED-FILE:%s\n' "$name"
    base64 -w 0 "/tmp/$name"
    printf '\n'
done
printf 'RAR-SIGNED-BUILD:END\n'
