#!/bin/sh
# Runs only inside the trusted cloud build image, with no network or credentials.
set -eu
[ "$#" -eq 0 ]
[ "$(id -u)" -eq 65532 ]
[ "$(uname -s)" = Linux ]
ulimit -f 131072
export TMPDIR=/tmp
cd /tmp
rustc --version >&2
rustc --edition 2024 --test /source/nucleus/foundation/model.rs -o /tmp/model-tests
/tmp/model-tests > /tmp/model-tests.log
rustc --edition 2024 -C opt-level=2 /source/nucleus/foundation/image.rs -o /tmp/image
for profile in normal panic exception; do
    rustc --edition 2024 --target x86_64-unknown-uefi \
      -C opt-level=2 -C panic=abort -C no-redzone=yes \
      -C debuginfo=0 -C strip=symbols -C link-arg=/timestamp:0 \
      --remap-path-prefix=/source=rar-source --remap-path-prefix=/tmp=rar-build \
      --cfg "rar_profile=\"$profile\"" \
      /source/nucleus/foundation/main.rs -o "/tmp/$profile.efi"
    /tmp/image "/tmp/$profile.efi" "/tmp/$profile.img"
done
# Fixed flat transfer framing. The trusted controller rejects duplicates,
# unexpected names, noncanonical Base64, truncation and oversized output.
for profile in normal panic exception; do
    for extension in efi img; do
        printf 'RAR-FILE:%s.%s\n' "$profile" "$extension"
        base64 -w 0 "/tmp/$profile.$extension"
        printf '\n'
    done
done
printf 'RAR-FILE:model-tests.log\n'
base64 -w 0 /tmp/model-tests.log
printf '\n'
printf 'RAR-BUILD:END\n'
