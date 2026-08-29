ARG LAUNCH_BASE
FROM ${LAUNCH_BASE}

ARG DEBIAN_SNAPSHOT
ARG QEMU_VERSION
ARG OVMF_VERSION
ARG SOURCE_DATE_EPOCH
ENV SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH} \
    TZ=UTC \
    LC_ALL=C \
    LANG=C

RUN set -eu; \
    printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/%s bookworm main\n' "$DEBIAN_SNAPSHOT" > /etc/apt/sources.list; \
    rm -f /etc/apt/sources.list.d/*; \
    apt-get -o Acquire::Check-Valid-Until=false update; \
    DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
      "qemu-system-x86=$QEMU_VERSION" "ovmf=$OVMF_VERSION"; \
    mkdir -p /opt/rar-lab/bin /opt/rar-lab/firmware; \
    cp --dereference /usr/bin/qemu-system-x86_64 /opt/rar-lab/bin/qemu-system-x86_64; \
    cp --dereference /usr/share/OVMF/OVMF_CODE.fd /opt/rar-lab/firmware/OVMF_CODE.fd; \
    test -f /opt/rar-lab/bin/qemu-system-x86_64; test ! -L /opt/rar-lab/bin/qemu-system-x86_64; \
    test -f /opt/rar-lab/firmware/OVMF_CODE.fd; test ! -L /opt/rar-lab/firmware/OVMF_CODE.fd; \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

USER 65532:65532
WORKDIR /controller
