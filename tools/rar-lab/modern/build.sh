#!/bin/sh
# Trusted fixed cloud build commands. Never execute this script on the Mac.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
rustc --version >&2
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=2 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C relocation-model=static \
  -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  -C link-arg=/base:0x400000 -C link-arg=/fixed \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  /source/core/modern/main.rs -o /tmp/modern-service.efi
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=2 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  --cfg rar_platform --cfg rar_modern --cfg 'rar_profile="normal"' \
  /source/nucleus/foundation/main.rs -o /tmp/modern.efi
# Three real standalone code variants, compiled inside this same fixed sandbox.
# Negative signature/ABI packages later reuse update code with altered metadata;
# failed health is a genuinely different executable, not a manifest label.
build_settings() {
    name="$1"
    shift
    rustc --edition 2024 --target x86_64-unknown-uefi \
      -C opt-level=2 -C panic=abort -C no-redzone=yes \
      -C debuginfo=0 -C strip=symbols -C relocation-model=static \
      -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
      -C link-arg=/base:0x400000 -C link-arg=/fixed \
      --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
      --cfg rar_settings_only "$@" \
      /source/core/modern/main.rs -o "/tmp/$name"
}
build_settings modern-settings-factory.efi
build_settings modern-settings-update.efi --cfg rar_settings_v2
build_settings modern-settings-bad-health.efi --cfg rar_settings_v2 --cfg rar_settings_fail_health
for name in modern.efi modern-service.efi modern-settings-factory.efi modern-settings-update.efi modern-settings-bad-health.efi; do
    printf 'RAR-FILE:%s\n' "$name"
    base64 -w 0 "/tmp/$name"
    printf '\n'
done
printf 'RAR-BUILD:END\n'
