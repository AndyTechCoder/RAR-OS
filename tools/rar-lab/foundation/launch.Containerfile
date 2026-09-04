FROM debian@sha256:362e64223cc0da95422b3b13c045186fc0a81250e765d31c025fbddf257f6143
RUN set -eu; \
    printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/20260803T000000Z bookworm main\n' > /etc/apt/sources.list; \
    rm -f /etc/apt/sources.list.d/*; \
    apt-get -o Acquire::Check-Valid-Until=false update; \
    DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
      qemu-system-x86=1:7.2+dfsg-7+deb12u18+b3 ovmf=2022.11-6+deb12u2; \
    sha256sum /usr/bin/qemu-system-x86_64 /usr/share/OVMF/OVMF_CODE.fd /usr/share/OVMF/OVMF_VARS.fd > /opt/identities.sha256
COPY launch.sh /opt/rar-launch.sh
USER 65532:65532
ENTRYPOINT ["/bin/sh", "/opt/rar-launch.sh"]
