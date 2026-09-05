#!/bin/sh
# Fixed command list inside networkless trusted cloud tool container.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
rustc --version >&2
rustc --edition 2024 --test /source/nucleus/platform/model.rs -o /tmp/kernel-tests
rustc --edition 2024 --test /source/services/platform/model.rs -o /tmp/service-tests
/tmp/kernel-tests > /tmp/model-tests.log
/tmp/service-tests >> /tmp/model-tests.log
rustc --edition 2024 -C opt-level=2 /source/nucleus/foundation/image.rs -o /tmp/image
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=2 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C relocation-model=static \
  -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  -C link-arg=/base:0x400000 -C link-arg=/fixed \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  /source/core/platform/main.rs -o /tmp/platform-service.efi
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=2 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_platform --cfg 'rar_profile="normal"' \
  /source/nucleus/foundation/main.rs -o /tmp/platform.efi
/tmp/image /tmp/platform.efi /tmp/platform.img
for name in platform.efi platform.img platform-service.efi model-tests.log; do
    printf 'RAR-FILE:%s\n' "$name"
    base64 -w 0 "/tmp/$name"
    printf '\n'
done
printf 'RAR-BUILD:END\n'
