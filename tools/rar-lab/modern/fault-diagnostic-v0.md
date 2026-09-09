# Retained Data-fault diagnostics v0

This read-only cloud controller inspects a selected capture from a completed,
first-attempt trusted-main modern-data-faults workflow in AndyTechCoder/RAR-OS.
Inputs are canonical run/artifact IDs and the exact source revision. GitHub
metadata binds the artifact to that run/controller/repository; the manifest
must match the supplied source. This is not independent Git source-closure
proof and never grants runtime or milestone acceptance.

The ephemeral read-only token is cleared before archive parsing. The whole ZIP
digest and size are checked before opening it. No extraction, subprocess,
VM, compiler, reference adapter, or proposal helper is invoked. Only manifest
and active capture are decompressed; flat regular allowlisted members,
resource limits, and a 128 KiB escaped JSON output bound apply. The report
selects recorded events/receipts/cut-entry/fault metadata, not image payloads.

Missing captures (for example a build failure) are reported as absent, not
passed. Event observations remain untrusted evidence; this tool does not
whitelist them. Any acceptance-validator change requires diagnosis, tests,
review and a corrected cloud campaign, never an unchanged rerun.

Initial diagnostic target: run 34352861329, artifact 10104503506, runtime source
5b8555e86b54056efa2927e094b3f1746e66d668. Its first capture failed with
"unexpected fault-time lifecycle/device event"; the exact event must be read
before deciding a fix. M4.1, M4.2 and M4.3 remain incomplete.

Pure tests use synthetic in-memory ZIPs and inert clients only. The workflow
must be reviewed and source-checked on its exact head before merging to main,
then dispatched from trusted main. No local files or owner data are accessed.
