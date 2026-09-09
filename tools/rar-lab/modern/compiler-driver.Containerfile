# Private cloud construction artifact only; never a runnable compiler role.
# The trusted controller supplies an owned local tag already inspected as exact
# parent sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914.
ARG COMPILER_PARENT
FROM ${COMPILER_PARENT} AS accepted-compiler
FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 AS driver-build
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
ENV SOURCE_DATE_EPOCH=1785715200 LC_ALL=C LANG=C TZ=UTC
COPY --from=accepted-compiler /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl/ /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl/
COPY compiler_driver.rs /driver-source/compiler_driver.rs
RUN mkdir -p /driver-output && \
    ulimit -f 8192 && ulimit -t 60 && \
    chmod 0555 /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl && \
    /usr/bin/find /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl -printf '%y %m %U %G %p\n' | /usr/bin/sort > /driver-output/consumed-musl.tree && \
    /usr/bin/find /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl -type f -print0 | /usr/bin/sort -z | /usr/bin/xargs -0 -r /usr/bin/sha256sum -- > /driver-output/consumed-musl.sha256 && \
    /usr/bin/env -i PATH=/nonexistent LC_ALL=C LANG=C \
    TMPDIR=/driver-output SOURCE_DATE_EPOCH=1785715200 \
    LD_LIBRARY_PATH=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib \
    /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 --crate-name=rar_compile_driver --crate-type=bin -D warnings \
    --target=x86_64-unknown-linux-musl \
    --sysroot=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu \
    -C opt-level=2 -C panic=abort -C codegen-units=1 -C strip=symbols -C debuginfo=0 \
    -C target-cpu=x86-64 -C target-feature=+crt-static -C link-self-contained=yes \
    -C relocation-model=static -C metadata=rar-modern-compiler-driver-v0 \
    -C linker=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld \
    --remap-path-prefix=/driver-source=/rar-driver-source \
    /driver-source/compiler_driver.rs -o /driver-output/rar-compile-driver && \
    test "$(stat -c %s /driver-output/rar-compile-driver)" -le 2097152 && \
    chmod 0555 /driver-output/rar-compile-driver && \
    touch --date=@1785715200 /driver-output/rar-compile-driver && \
    /usr/bin/find /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl -printf '%y %m %U %G %p\n' | /usr/bin/sort > /driver-output/post-musl.tree && \
    /usr/bin/find /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl -type f -print0 | /usr/bin/sort -z | /usr/bin/xargs -0 -r /usr/bin/sha256sum -- > /driver-output/post-musl.sha256 && \
    /usr/bin/cmp /driver-output/consumed-musl.tree /driver-output/post-musl.tree && \
    /usr/bin/cmp /driver-output/consumed-musl.sha256 /driver-output/post-musl.sha256 && \
    chmod 0444 /driver-output/consumed-musl.tree /driver-output/consumed-musl.sha256 && \
    touch --date=@1785715200 /driver-output/consumed-musl.tree /driver-output/consumed-musl.sha256
# Export the driver, complete notices and two pre/post-equal consumed-sysroot
# records. The trusted outer controller compares those records byte-for-byte
# with its independent pinned-parent inventory before any driver execution.
FROM scratch
COPY --from=driver-build /driver-output/rar-compile-driver /rar-compile-driver
COPY --from=driver-build /driver-output/consumed-musl.tree /consumed-musl.tree
COPY --from=driver-build /driver-output/consumed-musl.sha256 /consumed-musl.sha256
COPY --from=accepted-compiler /licenses/ /licenses/
USER 65532:65532
ENTRYPOINT ["/nonexistent"]
