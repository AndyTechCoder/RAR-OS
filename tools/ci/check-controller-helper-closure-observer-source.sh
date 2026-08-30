#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
observer=$root/tools/ci/observe-controller-helper-closure.sh
contract=$root/spec/alpha/lab/controller-helper-closure-observation-v0.fields

fail() {
    printf 'controller-helper closure observer source check failed: %s\n' "$1" >&2
    exit 1
}

[ -f "$observer" ] && [ ! -L "$observer" ] || fail 'observer is unavailable'
[ -f "$contract" ] && [ ! -L "$contract" ] || fail 'contract is unavailable'
/bin/sh -n "$observer" || fail 'observer shell syntax is invalid'

observer_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$observer" | /usr/bin/awk '{ print $1 }')
contract_sha=$(env LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$contract" | /usr/bin/awk '{ print $1 }')
[ "$observer_sha" = e3b4be670797d3f1bc84960d1d1207e470f87ba5a7fadc6327d86b7b61a7f320 ] || fail 'observer bytes escaped review'
[ "$contract_sha" = 944229644f0805876403cd858d0d8c3c993d73bf00baef4c3ffcbbc7a2522836 ] || fail 'contract bytes escaped review'

for required in \
    "#!/usr/bin/dash" \
    "[ \"\${GITHUB_EVENT_NAME-}\" = push ] || fail 'only a push event may observe the closure'" \
    "[ \"\${GITHUB_REF-}\" = refs/heads/main ] || fail 'only the canonical main ref may observe the closure'" \
    "[ \"\$GITHUB_SHA\" = \"\$RAR_TRUSTED_CONTROLLER_SHA\" ] || fail 'controller is not exact main'" \
    "[ \"\$GITHUB_SHA\" = \"\$RAR_EXPECTED_SOURCE_REVISION\" ] || fail 'source is not exact main'" \
    "[ \"\$0\" = \"\$script\" ] || fail 'executing observer path mismatch'" \
    "evidence=/evidence" \
    "manifest=\$evidence/controller-helper-closure.sha256" \
    "receipt=\$evidence/controller-helper-closure.receipt" \
    "exec 3> \"\$manifest\" || fail 'cannot exclusively create candidate manifest'" \
    "exec 4> \"\$receipt\" || fail 'cannot exclusively create observation receipt'" \
    "    [ \"\$manifest_bytes_expected\" -le 1048576 ] || fail 'closure manifest exceeds reviewed bounds'" \
    "[ -z \"\$unexpected\" ] || fail 'closure contains a non-directory or non-regular entry'" \
    "    'status=observed-not-reviewed-not-ready' \\" \
    "    'helper_compiled=false' \\" \
    "    'helper_executed=false' \\" \
    "    'target_compiled=false' \\" \
    "    'readiness=false' >&4 || fail 'cannot write observation receipt'"; do
    grep -Fqx "$required" "$observer" || fail "observer invariant is missing: $required"
done

if grep -Fq -- '-xdev' "$observer"; then
    fail 'observer silently prunes cross-device closure entries'
fi
if grep -R -Fq 'observe-controller-helper-closure.sh' "$root/.github/workflows"; then
    fail 'observer is wired to GitHub Actions before separate authorization'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'rust_toolchain_closure_manifest_sha256=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock contains an unreviewed closure digest'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'
grep -qx 'compiler_closure_manifest_sha256=unavailable' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory contains an unreviewed closure digest'

printf '%s\n' 'controller-helper closure observer source is inactive and byte-bound'
