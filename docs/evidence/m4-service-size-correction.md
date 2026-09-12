# M4.3 signed service size correction

The first repair campaign34691053018 against native source
00c6b5e93f99200eb36f799d25753510a0a9baed and controller
ff21d6b0f7e1a28cadd5df1d7f6f93c577c42619 stopped at signed build inspection:
service mapped size135168 bytes exceeded the unchanged131072-byte limit.
No VM scenario ran, and this is not repair acceptance.

The controller's service compile switches from size optimization s to z and
enables whole-program fat LTO, retaining one codegen unit. The compiler, target,
input packages, panic-abort policy, no-redzone setting, fixed service base,
relocation policy, deterministic remapping and all resource/inspection bounds
remain unchanged. Kernel build flags remain unchanged. No target code, dependency,
host path, device or execution authority is added.

The compiled output is still required to pass the existing128KiB service gate,
the same signed-package/image checks, two independent reproducible builds and
the complete actual signed-runtime campaign. A smaller image is not behavioral
acceptance. If compilation or a runtime check fails, retain and diagnose the
failure; do not widen the size bound or retry without a reason.

This narrow build correction requires independent review and exact cloud checks
before trusted-main use. All repository mutations are GitHub API operations;
no Mac/SSD files, builds, downloads, mounts or RAR execution.
