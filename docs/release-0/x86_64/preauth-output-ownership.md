# Prompt 7A Transaction Output Ownership

Status: M1.5 cutover complete; M2 publication implementation incomplete

ADR 0022 makes `preauth-transaction` the sole preparation boundary. The former output-skeleton, acquisition, prepared-record, disk-child, and attestation paths are no longer workflow-reachable. Their executable names are unconditional effect-free refusal shims or dedicated negative fixtures.

M2 will use held directory descriptors, private exclusive content-addressed staging, file/directory/parent synchronization, and atomic no-replace publication of one immutable transaction bundle. No public writable child disk, partial output, path reopen, plain archive extraction, or separately mutable evidence file is allowed.

At M1.5 the sole entrypoint deliberately exits `73` with `preauth-transaction-m2-incomplete`; therefore it creates no output tree and publishes no graph. The cutover gate checks that refusal is side-effect-free and that no legacy workflow, parser, type export, fallback, or generated record remains production-reachable.
