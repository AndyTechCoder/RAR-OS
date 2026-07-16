# Host Policy Negative Corpus

`tools/ci/test-host-policy.sh` derives one-mutation fixtures from the canonical configuration and rules in an atomically created, ignored `out/host-policy-tests.XXXXXX/` directory. The test refuses a symbolic-link or non-directory `out/` root before writing.

The corpus covers commented required settings, enabled networking, conflicting duplicates, wrong and quoted sections, TOML literal multiline decoys, weakened automatic-review policy, symbolic-link inputs, non-forbidden rules, and missing emulator rules. Deriving each case from the valid source keeps every failure isolated to the named mutation.
