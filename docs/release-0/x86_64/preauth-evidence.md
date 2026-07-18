# Prompt 7A Pre-authorization Evidence

Status: prepared records only; exact-head CI attestation and independent review pending; execution unauthorized

Prompt 7A uses a two-phase evidence model. Committed `phase=prepared` records bind immutable inputs and deliberately contain no Git commit, workflow run, or generated archive claim; this avoids an impossible commit self-reference. Each push and pull-request route checks out the explicit branch head, rejects merge-ref or event mismatch, independently builds the closure, and emits a `phase=attested` record bound to that checked-out SHA, event, run ID, archive bytes, reported image descriptor, actually selected image ID, packages, profile, artifact, disk, and closure. Stale phase, head, run, archive, or image evidence is rejected. Run URLs and final exact-head hashes are durable PR evidence, not committed authorization.

The approved closure contains exactly 36 canonical package rows. Every row binds binary name/version/architecture/filename/size/hash/license hash and signed-snapshot source package name/version. Acquisition compares the complete observed manifest and every closure evidence field to the committed lock before extracting any package into the derived image. The strict OCI verifier bounds and checks the tar, Docker's required canonical `repositories` member, manifest, config, layers, diff IDs, reported descriptor, and actually loaded image ID across two byte-identical builds. The repository index is one root-owned `0644` regular file of at most 512 bytes and must byte-exactly bind the sole `rar-preauth:<checked-out-head>` tag to the verified final layer; its bytes are covered by the canonical archive digest. The raw Docker export is validated before extraction and projected to the exact rooted graph. At the diagnosed closure, six inline content-addressed Docker-store blobs had no inbound edge and were excluded; the canonical archive contains only its four roots, sole config, six ordered layers, and unique OCI image manifest. A final archive containing any unreferenced valid-looking digest still fails closed. Diagnostics sort and cap normalized path/type/size/mode/ownership/digest/class/inbound facts and never print member content.

The selected profile SHA-256 is
`8e7bc38fa513700556b7ea493ffd42b6df6b4adcaf0a4719a0c7fe11f7eb165f`,
its typed command SHA-256 is
`7d8e5f500c35b5da4de0d3f2a6d9b667563bb6e3ff7ed6503192ee8e69e0550d`,
and its twice-built static artifact SHA-256 is
`96b7705f1dd987060c34ac049afd5a0d20fa58d8aff6586ce9090dbdf8a989ea`.
The deterministic seed and disposable child disk bytes both have SHA-256
`141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766`;
their content-bound disk record digest is
`89e160c117154dded20d7daeaf75576dd082a9d83d5ade47f9254a6e35371826`.
The prepared certification, execution-host record, and complete identity graph are canonical self-hashed records under `spec/lab/`. The graph binds every package/source/signature/license, tool, firmware, artifact, disk, profile, command, execution-host, authority-policy, resolver/spawner, prepared-certification, consumption-key, and source-tree edge. It remains `authorization_state=unissued`.

Every route records `target_execution=not-attempted`,
`qemu_execution=not-attempted`, `emulator_execution=not-attempted`,
`vm_execution=not-attempted`, and `aws_calls=not-attempted`. No review,
owner authorization, or launch authority is implied.
