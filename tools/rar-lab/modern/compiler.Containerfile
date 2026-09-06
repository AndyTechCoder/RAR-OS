# Private construction only. No RAR source in this context or output execution.
ARG SOURCE_DATE_EPOCH=1785715200
FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 AS provision
ENV SOURCE_DATE_EPOCH=1785715200 LC_ALL=C LANG=C TZ=UTC
COPY rust-musl.tar.xz /inputs/rust-musl.tar.xz
RUN printf '%s\n' \
 'aee540abf132920f791ef781489851a078d69dff493fb628d49c1d573f92bb3a  /inputs/rust-musl.tar.xz' \
 | sha256sum --check --strict -
RUN mkdir -p /build && \
 tar -xJf /inputs/rust-musl.tar.xz -C /build --no-same-owner && \
 /build/rust-std-1.95.0-x86_64-unknown-linux-musl/install.sh \
 --prefix=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu --disable-ldconfig
COPY compiler_closure.py /build/compiler_closure.py
RUN RAR_COMPILER_PROVISION=pinned-rust-1.95.0 /usr/bin/python3 -I -B /build/compiler_closure.py
RUN mkdir -p /compiler-root/evidence /compiler-root/source /compiler-root/build && \
 cp /build/compiler-closure.json /compiler-root/evidence/compiler-closure.json && \
 chmod 0444 /compiler-root/evidence/compiler-closure.json && \
 chmod 0555 /compiler-root/evidence /compiler-root/source && \
 chown 65532:65532 /compiler-root/build && chmod 0700 /compiler-root/build && \
 touch --date="@1785715200" /compiler-root /compiler-root/evidence \
 /compiler-root/evidence/compiler-closure.json /compiler-root/source /compiler-root/build
FROM scratch
COPY --from=provision /compiler-root/ /
ENV PATH=/nonexistent LD_LIBRARY_PATH=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib
USER 65532:65532
WORKDIR /source
ENTRYPOINT ["/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc"]
