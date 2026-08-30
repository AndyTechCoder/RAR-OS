#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
semantics=$root/spec/alpha/lab/controller-helper-closure-verifier-basic-filesystem-semantics-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
scalar=$root/spec/alpha/lab/controller-helper-closure-verifier-scalar-semantics-v0

fail() {
    printf 'controller-helper closure verifier basic-filesystem-semantics source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$semantics" "$templates" "$inventory" "$domain" "$scalar"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$semantics")" = 41cd09eacc4022ac2e0fa7ada6780165339ad009a49c0fbf740fe12e6851a311 ] || fail 'basic filesystem semantics bytes escaped review'
[ "$(sha_file "$templates")" = a5ae29ea7053dc200147901b89db67271cd895013769865910314070744835a3 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$inventory")" = 33468d5d2641b440e71817b1c0d143f56d4c7090978438f8e794a0fd616a311b ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain bytes escaped review'
[ "$(sha_file "$scalar")" = 963ece5eddc873ae1ea0bcb98c95d234ef9bf372599c79c1a43a7a53fefb54d7 ] || fail 'scalar semantics bytes escaped review'

for required in \
    'status=experimental-incomplete-inactive-source-only' \
    'execution_authority=none' \
    'semantic_row_count=5' \
    'family_coverage=4-complete-families+hex-absent-file-creation-subdomain-only' \
    'covered_template_count=10' \
    'fixture_identity=controller_uid-is-mathematical-1000;controller_gid+mount-device+parent-identity+filesystem-capabilities-come-from-the-future-byte-pinned-base-descriptor' \
    'parent_rule=parent-device+inode+uid+gid+mode-are-preserved;file-create+file-remove+file-replace-preserve-parent-nlink;directory-create-increments-parent-nlink-by-exactly-1;kernel-derived-parent-size+mtime+ctime-are-permitted-only-when-recorded-before-launch-or-ack;all-other-entries-remain-byte+identity+metadata-base-equal' \
    'failure_rule=absence+type+ownership+mode+link+device+identity+atomicity+postcondition-failure-invalidates-case-before-launch-or-ack;never-skip+fallback+cross-device-copy+truncate+coerce' \
    'remaining_status=symlink+hardlink+raw-name+path-alias+mount+tree+manifest-specific-primary-families+all-non-none-repair-semantics+exact-base+controller+runtime-precedence+fault+evidence+verdict-remain-absent' \
    'activation_rule=blocked;this-slice-cannot-create-fixtures+execute-mutations+or-authorize-a-controller' \
    'consumer_rule=this-contract-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$semantics" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$semantics" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'semantics lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$semantics" >/dev/null; then fail 'semantics contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$semantics" >/dev/null; then fail 'semantics contain CR'; fi
[ "$(grep -Ec '^F[0-9][0-9][0-9]\|[a-z0-9-]+\|[a-z0-9+-]+\|[^| ]+\|[^| ]+$' "$semantics")" -eq 5 ] || fail 'semantic rows are malformed'

number=1
while [ "$number" -le 5 ]; do
    id=$(printf 'F%03d' "$number")
    [ "$(grep -c "^$id|" "$semantics")" -eq 1 ] || fail "semantic ID is missing or duplicated: $id"
    number=$((number + 1))
done

expected_families=$(printf '%s\n' empty-directory hex remove replace-same-bytes-new-inode toggle-owner-execute | /usr/bin/sort)
actual_families=$(/usr/bin/awk -F '|' '/^F[0-9][0-9][0-9]\|/ { print $2 }' "$semantics" | /usr/bin/sort)
[ "$actual_families" = "$expected_families" ] || fail 'basic filesystem family set changed'

grep -Fqx 'C035|D035|E033|scratch-boundary|pre-start|entry./tmp/rar-controller-helper-closure-verification=empty-directory|none|E033@scratch-boundary+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'empty-directory template changed'
grep -Fqx 'C055|D058|E053|input-identity|pre-start|file./verification/controller-helper-closure-verification.receipt=hex:58|none|E053@input-identity+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'verification file-creation template changed'
grep -Fqx 'C056|D059|E054|evidence-exact-set|pre-start|file./evidence/unexpected=hex:58|none|E054@evidence-exact-set+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'evidence file-creation template changed'
grep -Fqx 'C057|D060|E055|evidence-exact-set|after-safe-inputs-before-evidence-enumeration|entry./evidence/controller-helper-closure.sha256=remove|none|E055@evidence-exact-set+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'remove template changed'
grep -Fqx 'C116|D131|E096|inter-pass-comparisons|closure-first-pass-after-identity-snapshot-before-topology-snapshot|entry.closure/a=replace-same-bytes-new-inode|none|E096@inter-pass-comparisons+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'new-inode replacement template changed'
[ "$(grep -c '=toggle-owner-execute|' "$templates")" -eq 5 ] || fail 'owner-execute template count changed'

covered=$(/usr/bin/awk -F '|' '
/^C[0-9][0-9][0-9]\|/ {
    target=$6; sub(/=.*/,"",target); rhs=$6; sub(/^[^=]*=/,"",rhs); family=rhs; sub(/:.*/,"",family)
    if (family=="empty-directory" || family=="remove" || family=="replace-same-bytes-new-inode" || family=="toggle-owner-execute") n++
    if (family=="hex" && (target=="file./verification/controller-helper-closure-verification.receipt" || target=="file./evidence/unexpected")) n++
}
END { print n+0 }' "$templates")
[ "$covered" -eq 10 ] || fail 'covered template count changed'

if grep -R -Fq 'controller-helper-closure-verifier-basic-filesystem-semantics-v0' "$root/.github/workflows"; then
    fail 'inactive basic filesystem semantics are wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier basic filesystem semantics cover five rows and ten templates, inactive and directly unwired'
