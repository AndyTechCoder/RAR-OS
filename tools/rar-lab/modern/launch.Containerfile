FROM debian@sha256:362e64223cc0da95422b3b13c045186fc0a81250e765d31c025fbddf257f6143
RUN set -eu; \
    printf 'deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/20260803T000000Z bookworm main\n' > /opt/rar-sources.list; \
    apt-get -o Acquire::http::Pipeline-Depth=0 -o Dir::Etc::sourcelist=/opt/rar-sources.list -o Dir::Etc::sourceparts=- -o Acquire::Check-Valid-Until=false update; \
    DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::http::Pipeline-Depth=0 -o Dir::Etc::sourcelist=/opt/rar-sources.list -o Dir::Etc::sourceparts=- install --yes --no-install-recommends \
      qemu-system-x86=1:7.2+dfsg-7+deb12u18+b3 ovmf=2022.11-6+deb12u2 python3=3.11.2-1+b1; \
    printf 'trusted-modern-container-v0\n' > /opt/rar-modern-container; \
    sha256sum /usr/bin/python3.11 /usr/bin/qemu-system-x86_64 /usr/share/OVMF/OVMF_CODE.fd /usr/share/OVMF/OVMF_VARS.fd > /opt/identities.sha256
COPY system_fault_launch.py system_fault_scenario.py system_selector_fault.py signed_runtime_launch.py signed_runtime_scenario.py signed_runtime_evidence.py mounted_error_launch.py mounted_error_scenario.py unavailable_launch.py unavailable_scenario.py fault_launch.py fault_scenarios.py fault_audit.py block_disk.py block_wire.py block_process.py vm_profile.py vm_session.py persistence.py data_provision.py data_oracle.py visual_oracle.py launch.py /opt/rar-modern/
USER 65532:65532
ENTRYPOINT ["/usr/bin/python3", "-I", "-B", "/opt/rar-modern/launch.py"]
