#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
semantics=$root/spec/alpha/lab/controller-helper-closure-verifier-synchronized-link-semantics-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
filesystem=$root/spec/alpha/lab/controller-helper-closure-verifier-basic-filesystem-semantics-v0

fail() {
    printf 'controller-helper closure verifier synchronized-link-semantics source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$semantics" "$templates" "$inventory" "$domain" "$filesystem"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$semantics")" = 8e409f4715de06bb600381942e4e67ac7f7793fbb160fed97902731a19480e7f ] || fail 'synchronized link semantics bytes escaped review'
[ "$(sha_file "$templates")" = 8df14525a37c49df99d669b638efd316c8993906d69e063625445f431ab204f8 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$inventory")" = c018c7fee8e1c70145a4c6adae852ef310ab20fd9d6b0cfad320521cdd76f062 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain bytes escaped review'
[ "$(sha_file "$filesystem")" = 5f524c4077d99547466fb46ca4b80bff7944bbd9bcf5c85ed636a08aab397854 ] || fail 'basic filesystem semantics bytes escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'semantic_row_count=2' \
    'family_coverage=add-hardlink-synchronized-no-repair-subdomain+replace-with-symlink-to-synchronized-no-repair-subdomain' \
    'covered_template_count=6' \
    'controller_view=controller-retains-only-one-private-bounded-writable-handle-to-nonscratch-closure-fixture-backing;no-post-start-path+FD+namespace+proc+ptrace+mutation-access-to-/tmp-or-verifier-scratch' \
    'trigger_rule=mutation-begins-only-after-exact-named-verifier-trigger;controller-ack-is-released-only-after-atomic-operation+all-postcondition+confinement+metadata-recording-complete;timeout+partial+early+late+second-mutation-invalidates-case' \
    'symlink_rule=source-is-declared-single-link-regular-closure/a;atomic-same-filesystem-rename-replaces-it-with-one-symlink-whose-link-payload-is-exact-ASCII-a;old-regular-entry-has-no-visible-alias;new-symlink-device-equals-base-closure-device+inode-is-fresh+distinct-from-replaced-inode+every-other-live-entry+nlink-is-1+Linux-permission-mode-is-0777+uid-is-1000+gid-is-base-controller_gid+size-is-1-byte;mtime+ctime+complete-post-state-are-recorded+validated-before-ack' \
    'failure_rule=trigger+atomicity+source+destination+device+inode+link-count+payload+staging+parent+confinement+postcondition-failure-invalidates-case-before-ack;never-skip+retry+fallback+cross-device-copy' \
    'remaining_status=pre-start+repair-coupled-link-cases+required-path-symlinks+raw-name+path-alias+mount+tree+manifest-specific-primary-families+all-non-none-repair-semantics+exact-base+controller+runtime-precedence+fault+evidence+verdict-remain-absent' \
    'activation_rule=blocked;this-slice-cannot-create-fixtures+execute-mutations+or-authorize-a-controller' \
    'consumer_rule=this-contract-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$semantics" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$semantics" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'semantics lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$semantics" >/dev/null; then fail 'semantics contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$semantics" >/dev/null; then fail 'semantics contain CR'; fi
[ "$(grep -Ec '^L[0-9][0-9][0-9]\|[a-z0-9-]+\|[^| ]+\|[^| ]+\|[^| ]+$' "$semantics")" -eq 2 ] || fail 'semantic rows are malformed'
grep -Fqx 'L001|add-hardlink|entry.closure/a-to-closure/hidden-a|named-synchronized-phase+repair-none|hardlink_rule+parent_rule+record-post-state-before-ack' "$semantics" || fail 'hardlink semantic row changed'
grep -Fqx 'L002|replace-with-symlink-to|entry.closure/a-to-relative-a|named-synchronized-phase+repair-none|symlink_rule+parent_rule+record-post-state-before-ack' "$semantics" || fail 'symlink semantic row changed'

selected=$(/usr/bin/awk -F '|' '/^C[0-9][0-9][0-9]\|/ && $7 == "none" && ($6 == "entry.closure/a=add-hardlink:hidden-a" || $6 == "entry.closure/a=replace-with-symlink-to:a") { print }' "$templates")
[ "$(printf '%s\n' "$selected" | /usr/bin/awk 'NF { n++ } END { print n+0 }')" -eq 6 ] || fail 'selected template count changed'
expected=$(printf '%s\n' C099 C100 C107 C108 C109 C110 | /usr/bin/sort)
actual=$(printf '%s\n' "$selected" | cut -d '|' -f1 | /usr/bin/sort)
[ "$actual" = "$expected" ] || fail 'selected synchronized link case set changed'
if printf '%s\n' "$selected" | grep -F '|pre-start|' >/dev/null; then fail 'selected link case is not synchronized'; fi

if grep -R -Fq 'controller-helper-closure-verifier-synchronized-link-semantics-v0' "$root/.github/workflows"; then
    fail 'inactive synchronized link semantics are wired to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier synchronized link semantics cover six no-repair templates, inactive and directly unwired'
