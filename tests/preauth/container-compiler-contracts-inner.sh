#!/bin/sh
set -eu

fail() {
    printf 'container compiler contract: %s\n' "$1" >&2
    exit 1
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
. "$root/tools/toolchain/preauth-build-root.sh"

[ "$(id -u)" = "${RAR_TEST_UID-}" ] || fail uid
[ "$(id -g)" = "${RAR_TEST_GID-}" ] || fail gid
[ "${RAR_PREAUTH_CONTAINER_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || fail image
[ -z "${RAR_CI_BOOTSTRAP_IMAGE-}" ] || fail host-identity-leak

umask 077
/usr/bin/mkdir -m 700 /tmp/poison /tmp/home /tmp/read-only-rustup
printf '%s\n' '#!/bin/sh' 'printf "%s\n" ambient-rustc-used >&2' 'exit 99' > /tmp/poison/rustc
/usr/bin/chmod 0555 /tmp/poison/rustc /tmp/read-only-rustup
PATH=/tmp/poison:/usr/bin:/bin
HOME=/tmp/home
RUSTUP_HOME=/tmp/read-only-rustup
export PATH HOME RUSTUP_HOME
[ "$(command -v rustc)" = /tmp/poison/rustc ] || fail poisoned-path
[ ! -w "$RUSTUP_HOME" ] || fail rustup-home-writable

rustc_path=$(preauth_build_pinned_container_rustc_path "$root") || fail accepted-boundary
[ "$rustc_path" = /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc ] || fail resolved-path
[ ! -L "$rustc_path" ] || fail resolved-symlink
[ "$(/usr/bin/sha256sum "$rustc_path")" = 'bff349e72704ff70bc08a234a3847338e797065bbedde5e556808bc87b7bf7c6  /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc' ] || fail resolved-hash

if (unset RAR_PREAUTH_CONTAINER_IMAGE; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail missing-image-accepted
fi
if (RAR_PREAUTH_CONTAINER_IMAGE=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; export RAR_PREAUTH_CONTAINER_IMAGE; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail wrong-image-accepted
fi
if (RAR_CI_BOOTSTRAP_IMAGE=$RAR_PREAUTH_CONTAINER_IMAGE; export RAR_CI_BOOTSTRAP_IMAGE; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail host-identity-accepted
fi
if (RAR_PREAUTH_CONTAINER_RUSTC_PATH=/Users/host/.rustup/toolchains/1.95.0/bin/rustc; export RAR_PREAUTH_CONTAINER_RUSTC_PATH; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail host-path-accepted
fi
if (RAR_PREAUTH_CONTAINER_RUSTC_ROOT=/tmp/writable-root; export RAR_PREAUTH_CONTAINER_RUSTC_ROOT; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail root-override-accepted
fi
if (RAR_PREAUTH_CONTAINER_RUSTC_SHA256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; export RAR_PREAUTH_CONTAINER_RUSTC_SHA256; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail hash-override-accepted
fi
if (RAR_PREAUTH_CONTAINER_RUSTC_VERSION=1.94.0; export RAR_PREAUTH_CONTAINER_RUSTC_VERSION; preauth_build_pinned_container_rustc_path "$root" >/dev/null 2>&1); then
    fail version-override-accepted
fi

/usr/bin/ln -s "$rustc_path" /tmp/poison/rustc-link
if rar_validate_absolute_file /tmp/poison/rustc-link; then
    fail symlink-accepted
fi
if rar_validate_absolute_file /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/../bin/rustc; then
    fail noncanonical-path-accepted
fi

printf '%s\n' 'container compiler contract checks passed'
