# Host Policy Negative Corpus

`tools/ci/test-host-policy.sh` derives one-mutation fixtures from the canonical
configuration and rules only beneath an atomically created directory in the
dedicated pinned validation container's bounded `/tmp` tmpfs. It skips before
writing on the Mac and SSD. The shared ephemeral-root guard rejects a symbolic,
non-tmpfs, wrong-owner, wrong-mode, noncanonical, or incomplete Linux CI
scratch root. It also requires the exact clean expected source revision on a
read-only mount and a bounded `nosuid,nodev` tmpfs. Container pseudo-filesystems
are not host-backed and are not approved scratch locations.

The corpus covers commented required settings, enabled networking, conflicting
duplicates, legacy sandbox overrides, wrong and quoted sections, TOML literal
multiline decoys, weakened automatic-review policy, symbolic-link inputs,
non-forbidden rules, and missing emulator, compiler, or permission-command
rules. `test-local-sprint-preflight-policy.sh` separately proves rejection of a
dirty worktree, wrong remote, wrong workspace guard marker, missing upstream, and
unpushed commit. Deriving each case from the valid source keeps every failure
isolated to the named mutation.
