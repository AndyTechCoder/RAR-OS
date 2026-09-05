# Provision only in the reviewed cloud controller. No target source in context.
FROM rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 AS provision
ENV SOURCE_DATE_EPOCH=1785715200 LC_ALL=C LANG=C TZ=UTC
COPY openssl.tar.gz libsodium.tar.gz /inputs/
RUN printf '%s\n' \
 '88525753f79d3bec27d2fa7c66aa0b92b3aa9498dafd93d7cfa4b3780cdae313  /inputs/openssl.tar.gz' \
 '018d79fe0a045cca07331d37bd0cb57b2e838c51bc48fd837a1472e50068bbea  /inputs/libsodium.tar.gz' \
 | sha256sum -c -
RUN mkdir -p /build/openssl /build/sodium && \
 tar -xf /inputs/openssl.tar.gz -C /build/openssl --strip-components=1 --no-same-owner && \
 tar -xf /inputs/libsodium.tar.gz -C /build/sodium --strip-components=1 --no-same-owner
WORKDIR /build/sodium
RUN ./configure --prefix=/opt/sodium --disable-shared --enable-static \
 --disable-dependency-tracking CFLAGS="-O2 -ffile-prefix-map=/build=rar-reference" && \
 make -j2 && make install
WORKDIR /build/openssl
RUN ./Configure linux-x86_64 no-shared no-dso no-module no-engine no-tests \
 no-legacy no-autoload-config no-pinshared --prefix=/opt/openssl --libdir=lib \
 --openssldir=/nonexistent -O2 -ffile-prefix-map=/build=rar-reference && \
 make -j2 && make install_sw
COPY reference-common.h reference-sodium.c reference-openssl.c /build/
RUN cc -std=c11 -O2 -Wall -Wextra -Werror -static -fno-ident \
 -ffile-prefix-map=/build=rar-reference -Wl,--build-id=none \
 -I/opt/sodium/include /build/reference-sodium.c \
 /opt/sodium/lib/libsodium.a -pthread -o /reference-sodium && \
 cc -std=c11 -O2 -Wall -Wextra -Werror -static -fno-ident \
 -ffile-prefix-map=/build=rar-reference -Wl,--build-id=none \
 -I/opt/openssl/include /build/reference-openssl.c \
 /opt/openssl/lib/libcrypto.a -pthread -ldl -o /reference-openssl && \
 strip --strip-all /reference-sodium /reference-openssl
RUN for name in reference-sodium reference-openssl; do \
 test "$(stat -c %s /$name)" -le 33554432 && \
 readelf -l /$name > /build/$name.program-headers && \
 readelf -d /$name > /build/$name.dynamic && \
 ! grep -q INTERP /build/$name.program-headers && \
 ! grep -q NEEDED /build/$name.dynamic || exit 1; \
 done
FROM scratch
COPY --from=provision /reference-sodium /reference-sodium
COPY --from=provision /reference-openssl /reference-openssl
COPY --from=provision /build/openssl/LICENSE.txt /licenses/openssl.txt
COPY --from=provision /build/sodium/LICENSE /licenses/libsodium.txt
USER 65532:65532
WORKDIR /
ENTRYPOINT ["/reference-sodium"]
