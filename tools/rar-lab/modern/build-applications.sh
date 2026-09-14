#!/bin/sh
# Fixed cloud compile/package-data stage. Never run either resulting target.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
rustc --edition 2024 --target x86_64-unknown-uefi \
  -C opt-level=z -C lto=fat -C codegen-units=1 -C panic=abort -C no-redzone=yes \
  -C debuginfo=0 -C strip=symbols -C relocation-model=static \
  -C link-arg=/timestamp:0 -C link-arg=/DEBUG:NONE \
  -C link-arg=/base:0x400000 -C link-arg=/fixed \
  --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
  /source/apps/expansion/notes/main.rs -o /tmp/rar-notes.efi
/usr/bin/cc -std=c11 -Os -Wall -Wextra -Werror -pedantic -ffreestanding -fno-builtin \
  -fno-stack-protector -fno-pie -mno-red-zone -fno-asynchronous-unwind-tables -fno-unwind-tables \
  -DRAR_APP_NATIVE -nostdlib -static -no-pie -Wl,--build-id=none \
  -Wl,-T,/source/tools/rar-lab/expansion/c_app.ld /source/apps/expansion/counter/main.c -o /tmp/counter.elf
/usr/bin/python3 -I -B /source/tools/rar-lab/expansion/c_app_pe.py --convert < /tmp/counter.elf > /tmp/rar-counter.efi
for name in rar-notes.efi rar-counter.efi; do
    printf 'RAR-APP-FILE:%s\n' "$name"
    base64 -w 0 "/tmp/$name"
    printf '\n'
done
printf 'RAR-APP-BUILD:END\n'
