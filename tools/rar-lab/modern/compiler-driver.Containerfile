# Private cloud construction artifact only; never a runnable compiler role.
# The trusted controller supplies an owned local tag already inspected as exact
# parent sha256:9bb926e46f5789c5048af8dfad598b5ef9779ae0f1c572267a000f4b12eaf914.
ARG COMPILER_PARENT
FROM ${COMPILER_PARENT} AS accepted-compiler
FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 AS driver-build
ENV SOURCE_DATE_EPOCH=1785715200 LC_ALL=C LANG=C TZ=UTC
COPY --from=accepted-compiler /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl/ /usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-musl/
COPY compiler_driver.rs /driver-source/compiler_driver.rs
RUN mkdir -p /driver-output && \
    ulimit -f 8192 && ulimit -t 60 && \
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
    touch --date=@1785715200 /driver-output/rar-compile-driver
# Export only the one driver and the complete retained parent notices. The
# controller independently inventories all bytes; no artifact is executed here.
FROM scratch
COPY --from=driver-build /driver-output/rar-compile-driver /rar-compile-driver
COPY --from=accepted-compiler /licenses/ /licenses/
USER 65532:65532
ENTRYPOINT ["/nonexistent"]
