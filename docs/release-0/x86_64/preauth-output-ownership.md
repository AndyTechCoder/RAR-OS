# Prompt 7A repository-output ownership

Status: host-only construction control; no target or emulator execution authorized

The exact-head workflow creates the complete allowlisted `out/r0` directory skeleton on the GitHub runner before the first container. `prepare-preauth-output.sh` rejects absolute/test-root escapes, dot or parent ambiguity, symlinked components, unallocated pre-existing nodes, non-directories, foreign ownership, and modes other than `0755`. Its diagnostics contain repository-relative paths only.

The first drift at head `44ea907e267bda7d52ca61991b574e58b9372f9f` was the root acquisition container creating `out`, `out/r0`, and `out/r0/preauth` through a writable workspace mount. Its final recursive ownership correction covered only `out/r0/preauth/acquisition`, leaving those ancestors owned by root. The later non-root build therefore failed before compilation at `out/r0/preauth/build`.

The corrected ownership map is:

| Output | Creator | Effective identity | Mode/type | Successor |
| --- | --- | --- | --- | --- |
| complete allowlisted directory skeleton | runner preparation guard | invoking UID/GID | `0755` directory | all stages |
| APT state/cache, package bytes, licenses, derived rootfs | acquisition container | invoking UID/GID | package-defined regular files/directories, repository-confined | Buildx and closure verifier |
| OCI verifier and record validator | pinned Rust compiler container | invoking UID/GID | regular executable host tools | host OCI/certification steps |
| Buildx metadata, Docker-save exports, canonical archives | runner Docker client/engine | invoking runner output paths | strict verified regular files | OCI verifier and evidence |
| static ELF, disposable disk bytes, host-test evidence | verified derived build container | invoking UID/GID | regular files below pre-created destinations | attestation validator |

No repository output is bind-mounted writable by container root. Download-only APT, apt-secure signature validation, `dpkg-query`, and `dpkg-deb` extraction use repository-local writable state and do not require root. Every repository-writing container therefore runs with the invoking UID/GID, a read-only container root, and an explicit private `HOME` and `/tmp`. Because there is no root-producing stage, no privileged export/import path exists; the staged-tar attack surface and partial privileged-import cleanup state are absent rather than accepted.

`tests/preauth/output-ownership.sh` exercises two independent deterministic preparations, foreign-owner and mode refusal, unallocated pre-existing content, symlink and special-file refusal, destination collision, and successful write probes for every later output destination. Exact-head CI is the executable proof that acquisition, compilation, canonical OCI construction, static artifact production, and evidence emission all retain runner ownership.

All routes continue to state `target_execution=not-attempted`, `qemu_execution=not-attempted`, `emulator_execution=not-attempted`, and `vm_execution=not-attempted`.
