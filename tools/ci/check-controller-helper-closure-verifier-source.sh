#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
verifier=$root/tools/ci/verify-controller-helper-closure-candidate.sh
contract=$root/spec/alpha/lab/controller-helper-closure-verification-v0.fields

fail() {
    printf 'controller-helper closure verifier source check failed: %s\n' "$1" >&2
    exit 1
}

[ -f "$verifier" ] && [ ! -L "$verifier" ] || fail 'verifier is unavailable'
[ -f "$contract" ] && [ ! -L "$contract" ] || fail 'verification contract is unavailable'
/bin/sh -n "$verifier" || fail 'verifier shell syntax is invalid'

verifier_sha=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$verifier" | /usr/bin/awk '{ print $1 }')
contract_sha=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$contract" | /usr/bin/awk '{ print $1 }')
[ "$verifier_sha" = 3cbeeb85abc3023980a8afe444178ea7acc31f298b3b0975d2c4d6630c82a76c ] || fail 'verifier bytes escaped review'
[ "$contract_sha" = 7b4fa610e1ceaef39432ac24493fd1307865cd2e59dae153d1d3aaa3881e6ac7 ] || fail 'verification contract bytes escaped review'

for required in \
    '#!/usr/bin/dash' \
    "[ \"\${GITHUB_EVENT_NAME-}\" = push ] || fail 'only a push event may verify a candidate'" \
    "[ \"\${GITHUB_REF-}\" = refs/heads/main ] || fail 'only the canonical main ref may verify a candidate'" \
    "[ \"\$GITHUB_SHA\" = \"\$RAR_TRUSTED_CONTROLLER_SHA\" ] || fail 'controller is not exact main'" \
    "[ \"\$GITHUB_SHA\" = \"\$RAR_EXPECTED_SOURCE_REVISION\" ] || fail 'source is not exact main'" \
    "require_nonzero_sha \"\${RAR_REVIEWED_VERIFIER_SHA256-}\" 'reviewed verifier'" \
    "[ \"\$0\" = \"\$script\" ] || fail 'executing verifier path mismatch'" \
    "[ \"\$verifier_sha\" = \"\$RAR_REVIEWED_VERIFIER_SHA256\" ] || fail 'verifier escaped controller-bound reviewed identity'" \
    'evidence=/evidence' \
    'verification=/verification' \
    'tool_pins=/trusted/controller-helper-closure-verifier-tools.v0' \
    'mountinfo=/proc/self/mountinfo' \
    '"$comparator" -s -- "$tool_pins" "$tool_pins_canonical" || fail '\''verifier tool pins are not canonical bytes'\''' \
    '"$comparator" -s -- "$observation" "$observation_canonical" || fail '\''observation receipt is not canonical bytes'\''' \
    '"$comparator" -s -- "$manifest" "$scratch/first.manifest" || fail '\''candidate manifest differs from complete closure set'\''' \
    'capture_pass second' \
    '[ "$script_identity_before" = "$(stat_identity "$script")" ] || fail '\''verifier identity changed during verification'\''' \
    '[ "$verifier_sha" = "$(sha_file "$script")" ] || fail '\''verifier bytes changed during verification'\''' \
    '[ "$script_identity_before" = "$(stat_identity "$script")" ] || fail '\''verifier identity changed before receipt publication'\''' \
    '[ "$verifier_sha" = "$(sha_file "$script")" ] || fail '\''verifier bytes changed before receipt publication'\''' \
    '"$comparator" -s -- "$scratch/first.topology.sorted" "$scratch/second.topology.sorted" || fail '\''closure topology changed between verification passes'\''' \
    '    '\''status=candidate-exact-set-verified-not-reviewed-not-ready'\'' \' \
    '    '\''helper_compiled=false'\'' \' \
    '    '\''helper_executed=false'\'' \' \
    '    '\''target_compiled=false'\'' \' \
    '    '\''readiness=false'\'' >&7 || fail '\''cannot write verification receipt'\''' \
    'exec 8> "$verification_receipt" || fail '\''cannot exclusively create verification receipt'\''' \
    '    printf '\''%s\n'\'' "$line" >&8 || fail '\''cannot copy validated verification receipt'\'''; do
    grep -Fqx "$required" "$verifier" || fail "verifier invariant is missing: $required"
done

if grep -Fq -- '-xdev' "$verifier"; then
    fail 'verifier silently prunes cross-device closure entries'
fi
for forbidden in \
    '/usr/bin/rustc' \
    '/usr/bin/cargo' \
    '/usr/bin/docker' \
    '/usr/bin/qemu' \
    '/usr/bin/curl' \
    '/usr/bin/wget' \
    '/usr/bin/gh'; do
    if grep -Fq "$forbidden" "$verifier"; then
        fail "verifier contains forbidden execution authority: $forbidden"
    fi
done
if grep -R -Fq 'verify-controller-helper-closure-candidate.sh' "$root/.github/workflows"; then
    fail 'verifier is wired to GitHub Actions before separate authorization'
fi
for caller in "$root/tools/ci/check-specs.sh" "$root/tools/ci/check-sprint-static.sh"; do
    if grep -Fq '/bin/sh tools/ci/verify-controller-helper-closure-candidate.sh' "$caller"; then
        fail "static gate executes inactive verifier: $caller"
    fi
done
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'rust_toolchain_closure_manifest_sha256=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock contains an unreviewed closure digest'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'
grep -qx 'compiler_closure_manifest_sha256=unavailable' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory contains an unreviewed closure digest'

printf '%s\n' 'controller-helper closure verifier source is inactive, unwired, and byte-bound'
