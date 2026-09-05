FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
RUN set -eu; \
    curl --fail --location --silent --show-error \
      https://static.rust-lang.org/dist/rust-std-1.95.0-x86_64-unknown-uefi.tar.xz -o /tmp/uefi.tar.xz; \
    printf '%s  %s\n' 4cc55629480aa8ab5b39eb6b7458433b48461d6626fdea0330fb88e23af818ea /tmp/uefi.tar.xz | sha256sum -c -; \
    tar -xJf /tmp/uefi.tar.xz -C /tmp; \
    /tmp/rust-std-1.95.0-x86_64-unknown-uefi/install.sh --prefix="$(rustc --print sysroot)" --disable-ldconfig; \
    cp -a "$(rustc --print sysroot)" /opt/rar-toolchain
COPY build.sh /opt/rar-build.sh
ENV PATH=/opt/rar-toolchain/bin:/usr/bin:/bin LC_ALL=C SOURCE_DATE_EPOCH=1785715200
USER 65532:65532
WORKDIR /source
ENTRYPOINT ["/bin/sh", "/opt/rar-build.sh"]
