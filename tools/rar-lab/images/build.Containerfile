ARG BUILD_BASE
FROM ${BUILD_BASE}

ARG RUST_MUSL_URL
ARG RUST_MUSL_SHA256
ARG RUST_NONE_URL
ARG RUST_NONE_SHA256
ARG RUST_UEFI_URL
ARG RUST_UEFI_SHA256
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH} \
    TZ=UTC \
    LC_ALL=C \
    LANG=C

RUN set -eu; \
    case "$(rustc --version)" in 'rustc 1.95.0 ('*) ;; *) exit 1 ;; esac; \
    command -v curl >/dev/null; command -v tar >/dev/null; \
    command -v make >/dev/null; command -v cc >/dev/null; command -v perl >/dev/null; \
    install -d -m 0700 /bootstrap; \
    curl --fail --silent --show-error --location "$RUST_MUSL_URL" --output /bootstrap/rust-musl.tar.xz; \
    printf '%s  %s\n' "$RUST_MUSL_SHA256" /bootstrap/rust-musl.tar.xz | sha256sum --check --strict -; \
    curl --fail --silent --show-error --location "$RUST_NONE_URL" --output /bootstrap/rust-none.tar.xz; \
    printf '%s  %s\n' "$RUST_NONE_SHA256" /bootstrap/rust-none.tar.xz | sha256sum --check --strict -; \
    curl --fail --silent --show-error --location "$RUST_UEFI_URL" --output /bootstrap/rust-uefi.tar.xz; \
    printf '%s  %s\n' "$RUST_UEFI_SHA256" /bootstrap/rust-uefi.tar.xz | sha256sum --check --strict -

RUN set -eu; \
    sysroot=$(rustc --print sysroot); \
    for component in musl none uefi; do tar -xJf "/bootstrap/rust-$component.tar.xz" -C /bootstrap; done; \
    /bootstrap/rust-std-1.95.0-x86_64-unknown-linux-musl/install.sh --prefix="$sysroot" --disable-ldconfig; \
    /bootstrap/rust-std-1.95.0-x86_64-unknown-none/install.sh --prefix="$sysroot" --disable-ldconfig; \
    /bootstrap/rust-std-1.95.0-x86_64-unknown-uefi/install.sh --prefix="$sysroot" --disable-ldconfig; \
    mkdir -p /opt/rar-toolchain; \
    cp -a "$sysroot/." /opt/rar-toolchain/; \
    test -f /opt/rar-toolchain/bin/rustc; test ! -L /opt/rar-toolchain/bin/rustc; \
    cp --dereference "$sysroot/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld" /opt/rar-toolchain/bin/ld.lld; \
    /opt/rar-toolchain/bin/rustc --version; \
    test -f /opt/rar-toolchain/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib; \
    test -d /opt/rar-toolchain/lib/rustlib/x86_64-unknown-none/lib; \
    test -d /opt/rar-toolchain/lib/rustlib/x86_64-unknown-uefi/lib

RUN set -eu; \
    rm -rf /bootstrap; \
    rm -rf /opt/rar-reference/openssl /opt/rar-reference/libsodium; \
    mkdir -p /build; chown 65532:65532 /build; chmod 0700 /build; \
    test -f /opt/rar-toolchain/bin/rustc; test ! -L /opt/rar-toolchain/bin/rustc; \
    test -f /opt/rar-toolchain/bin/ld.lld; test ! -L /opt/rar-toolchain/bin/ld.lld; \
    test ! -e /opt/rar-reference

USER 65532:65532
WORKDIR /workspace
