ARG BUILD_IMAGE
ARG LAUNCH_BASE_IMAGE
FROM ${BUILD_IMAGE} AS qmp-builder

ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH} \
    TZ=UTC \
    LC_ALL=C \
    LANG=C
USER root
RUN set -eu; mkdir -p /evidence; chown 65532:65532 /evidence; chmod 0700 /evidence
USER 65532:65532
COPY --chown=65532:65532 tools/rar-lab/qmp-client/README.md /controller/tools/rar-lab/qmp-client/README.md
COPY --chown=65532:65532 tools/rar-lab/qmp-client/build-plan.v1 /controller/tools/rar-lab/qmp-client/build-plan.v1
COPY --chown=65532:65532 tools/rar-lab/qmp-client/json.rs /controller/tools/rar-lab/qmp-client/json.rs
COPY --chown=65532:65532 tools/rar-lab/qmp-client/main.rs /controller/tools/rar-lab/qmp-client/main.rs
WORKDIR /controller/tools/rar-lab/qmp-client
RUN set -eu; \
    /opt/rar-toolchain/bin/rustc --edition=2024 --test -C debuginfo=0 \
      --remap-path-prefix=/controller=. main.rs -o /build/rar-qmp-client-tests; \
    /build/rar-qmp-client-tests; \
    for output in a b; do \
      /opt/rar-toolchain/bin/rustc --edition=2024 -C opt-level=s -C debuginfo=0 \
        -C strip=symbols -C codegen-units=1 -C panic=abort \
        -C metadata=rar_qmp_client_v1 --remap-path-prefix=/controller=. \
        --target=x86_64-unknown-linux-musl main.rs -o "/build/rar-qmp-client-$output"; \
    done; \
    cmp /build/rar-qmp-client-a /build/rar-qmp-client-b; \
    cp /build/rar-qmp-client-a /build/rar-qmp-client; \
    test "$(/build/rar-qmp-client --version)" = 'rar-qmp-client 1'

FROM ${LAUNCH_BASE_IMAGE}
USER root
COPY --from=qmp-builder /build/rar-qmp-client /opt/rar-lab/bin/rar-qmp-client
RUN set -eu; \
    test -f /opt/rar-lab/bin/rar-qmp-client; \
    test ! -L /opt/rar-lab/bin/rar-qmp-client; \
    chown root:root /opt/rar-lab/bin/rar-qmp-client; \
    chmod 0555 /opt/rar-lab/bin/rar-qmp-client
USER 65532:65532
WORKDIR /controller
