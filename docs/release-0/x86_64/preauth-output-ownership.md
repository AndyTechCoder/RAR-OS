# Prompt 7A Transaction Output Ownership

Status: M1.6 untrusted delivery complete; M2 transaction publication incomplete

ADR 0022 makes `preauth-transaction` the sole preparation boundary. The former output-skeleton, acquisition, prepared-record, disk-child, and attestation paths are no longer workflow-reachable. Their executable names are unconditional effect-free refusal shims or dedicated negative fixtures.

M2 will use held directory descriptors, private exclusive content-addressed staging, file/directory/parent synchronization, and atomic no-replace publication of one immutable transaction bundle. No public writable child disk, partial output, path reopen, plain archive extraction, or separately mutable evidence file is allowed.

M1.6 permits `preauth-input-producer` to publish one untrusted, canonical, read-only input bundle per acquisition slot beneath `out/r0/preauth/input-delivery`. It uses repository-local private staging, fixed archive metadata and atomic no-replace publication as the invoking host user. The producer cannot publish a transaction graph or any authority-bearing record. A separate networkless transaction process parses that bundle and exits `73` with `preauth-transaction:m2-incomplete`; no transaction output or graph exists yet. The cutover gate distinguishes untrusted delivery from the sole preparation entrypoint and continues to reject every legacy path and fallback.
