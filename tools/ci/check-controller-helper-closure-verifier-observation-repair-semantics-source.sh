#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
semantics=$root/spec/alpha/lab/controller-helper-closure-verifier-observation-repair-semantics-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
observation=$root/spec/alpha/lab/controller-helper-closure-observation-v0.fields

fail() {
    printf 'controller-helper closure verifier observation-repair-semantics source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$semantics" "$templates" "$inventory" "$domain" "$observation"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$semantics")" = b8da645a5bcdacac0281c5adbec940f4d66f83c2088af6fc9d302d1db57ee8d2 ] || fail 'observation repair semantics bytes escaped review'
[ "$(sha_file "$templates")" = 8df14525a37c49df99d669b638efd316c8993906d69e063625445f431ab204f8 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$inventory")" = c018c7fee8e1c70145a4c6adae852ef310ab20fd9d6b0cfad320521cdd76f062 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain bytes escaped review'
[ "$(sha_file "$observation")" = 944229644f0805876403cd858d0d8c3c993d73bf00baef4c3ffcbbc7a2522836 ] || fail 'observation schema bytes escaped review'

for required in \
    'status=experimental-complete-source-only-inactive' \
    'execution_authority=none' \
    'semantic_row_count=6' \
    'repair_token_coverage=repair-observation-manifest-digest+repair-observation-manifest-digest-and-bytes+repair-observation-manifest-fields' \
    'covered_template_count=8' \
    'ordering_rule=complete+validate+record-the-manifest-primary-postcondition-first;derive-each-declared-repair-from-that-one-exact-post-primary-byte-string;apply-all-fields-of-the-one-declared-token;launch-only-after-combined-postconditions-pass' \
    'target_rule=repair-target-is-one-exact-field-in-the-declared-23-line-observation-receipt;field-pre-bytes-come-from-the-byte-pinned-base-receipt;untargeted-lines+order+LF-termination-remain-byte-identical' \
    'digest_derivation=SHA-256-over-the-exact-complete-post-primary-manifest-byte-string+lowercase-encode-as-exactly-64-ASCII-hex-bytes-with-no-prefix+separator+LF+NUL' \
    'entries_derivation=count-every-0A-octet-in-the-exact-post-primary-manifest-byte-string-in-unbounded-mathematical-integer+encode-canonical-positive-ASCII-decimal;post-count-must-be-1-through-65536' \
    'bytes_derivation=count-every-octet-in-the-exact-post-primary-manifest-byte-string-in-unbounded-mathematical-integer+encode-canonical-positive-ASCII-decimal;post-count-must-be-1-through-1048576' \
    'untargeted_consistency_rule=repair-observation-manifest-digest-and-bytes-requires-entries_derivation-equal-the-byte-pinned-base-receipt-manifest_entries;repair-observation-manifest-digest-requires-entries_derivation+bytes_derivation-equal-the-byte-pinned-base-receipt-manifest_entries+manifest_bytes;each-equality-is-a-combined-precondition+postcondition-and-the-preserved-field-bytes-remain-exact' \
    'receipt_file_rule=repair-reconstructs-exactly-23-LF-terminated-lines-in-the-declared-single-link-regular-observation-file-through-one-controller-private-pre-start-handle;exact-complete-write+flush+close-must-succeed;device+inode+nlink+uid+gid+mode-are-preserved' \
    'receipt_stability_rule=after-write-handle-close-open-exactly-one-controller-private-read-only-receipt-handle+one-read-only-recorded-parent-handle-with-no-write+replace+rename+unlink+mapping-authority;then-revoke-every-route-that-carries-write+replace+rename+unlink+mapping-authority-to-the-receipt-including-controller+fixture+namespace+process+proc+ptrace+path+descriptor+handle+mapping+backing-routes;the-two-declared-read-only-handles-are-the-only-retained-route-exceptions;immediately-before-launch-verify-through-those-handles-the-exact-path+device+inode+nlink+uid+gid+mode+size+mtime+ctime+complete-bytes+future-byte-pinned-SHA-256;close-both-read-only-handles;that-exact-state-must-remain-unchanged-through-launch' \
    'primary_preservation_rule=manifest-path+device+inode+nlink+uid+gid+mode+post-primary-bytes+size+mtime+ctime-remain-exactly-recorded+unchanged-through-repair+launch;repair-never-writes+aliases+replaces+normalizes+parses+sorts+or-truncates-the-primary-manifest' \
    'independence_rule=repair-reads-only-the-byte-pinned-base-receipt+recorded-post-primary-manifest-bytes+future-byte-pinned-controller-SHA-256;no-verifier-output+scratch+another-case+ambient-host-state+unreviewed-path' \
    'failure_rule=target+pre-bytes+ordering+derivation+bound+encoding+receipt-canonicality+identity+short-write+flush+close+route-revocation+post-close-verification+post-close-mutation+receipt-stability+primary-preservation+combined-postcondition-failure-invalidates-before-launch;never-skip+retry+wrap+truncate+coerce+fallback+repair-an-undeclared-field' \
    'remaining_status=closed-by-controller-helper-closure-verifier-cases-v0+faults-v0.fields+evidence-v0.fields;this-slice-remains-inactive' \
    'activation_rule=blocked;this-slice-cannot-create-fixtures+execute-mutations+apply-repairs+or-authorize-a-controller' \
    'consumer_rule=this-contract-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$semantics" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$semantics" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'semantics lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$semantics" >/dev/null; then fail 'semantics contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$semantics" >/dev/null; then fail 'semantics contain CR'; fi
[ "$(grep -Ec '^O[0-9][0-9][0-9]\|[^| ]+\|[^| ]+\|[^| ]+\|[^| ]+$' "$semantics")" -eq 6 ] || fail 'semantic rows are malformed'

number=1
while [ "$number" -le 6 ]; do
    id=$(printf 'O%03d' "$number")
    [ "$(grep -c "^$id|" "$semantics")" -eq 1 ] || fail "semantic ID is missing or duplicated: $id"
    number=$((number + 1))
done
[ "$(grep -c '^O[0-9][0-9][0-9]|repair-observation-manifest-fields|' "$semantics")" -eq 3 ] || fail 'manifest-fields repair row count changed'
[ "$(grep -c '^O[0-9][0-9][0-9]|repair-observation-manifest-digest-and-bytes|' "$semantics")" -eq 2 ] || fail 'digest-and-bytes repair row count changed'
[ "$(grep -c '^O[0-9][0-9][0-9]|repair-observation-manifest-digest|' "$semantics")" -eq 1 ] || fail 'digest repair row count changed'
grep -Fqx 'O004|repair-observation-manifest-digest-and-bytes|field./evidence/controller-helper-closure.receipt.manifest_sha256|pre-start+primary-postcondition-recorded+base-field-pre-bytes-recorded+entries_derivation-equals-base-manifest_entries|set-field-value-to-digest_derivation+preserve-proven-consistent-manifest_entries' "$semantics" || fail 'digest-and-bytes digest-row consistency proof changed'
grep -Fqx 'O005|repair-observation-manifest-digest-and-bytes|field./evidence/controller-helper-closure.receipt.manifest_bytes|pre-start+primary-postcondition-recorded+base-field-pre-bytes-recorded+entries_derivation-equals-base-manifest_entries|set-field-value-to-bytes_derivation+preserve-proven-consistent-manifest_entries' "$semantics" || fail 'digest-and-bytes byte-row consistency proof changed'
grep -Fqx 'O006|repair-observation-manifest-digest|field./evidence/controller-helper-closure.receipt.manifest_sha256|pre-start+primary-postcondition-recorded+base-field-pre-bytes-recorded+entries_derivation-equals-base-manifest_entries+bytes_derivation-equals-base-manifest_bytes|set-field-value-to-digest_derivation+preserve-proven-consistent-manifest_entries+manifest_bytes' "$semantics" || fail 'digest-only preserved-field consistency proof changed'

[ "$(grep -c '|repair-observation-manifest-fields|' "$templates")" -eq 6 ] || fail 'manifest-fields template count changed'
[ "$(grep -c '|repair-observation-manifest-digest-and-bytes|' "$templates")" -eq 1 ] || fail 'digest-and-bytes template count changed'
[ "$(grep -c '|repair-observation-manifest-digest|' "$templates")" -eq 1 ] || fail 'digest template count changed'

/usr/bin/awk -F '|' '
/^C[0-9][0-9][0-9]\|/ && ($7=="repair-observation-manifest-fields" || $7=="repair-observation-manifest-digest-and-bytes" || $7=="repair-observation-manifest-digest") {
    n++
    if ($5 != "pre-start") exit 20
    target=$6; sub(/=.*/, "", target)
    if (target != "file./evidence/controller-helper-closure.sha256") exit 21
}
END { if (n != 8) exit 22 }
' "$templates" || fail 'repair template phase, target, or count changed'

for id in C077 C078 C079 C080 C081 C083 C084 C113; do
    [ "$(grep -c "^$id|" "$templates")" -eq 1 ] || fail "covered repair template is missing or duplicated: $id"
done

grep -Fqx 'receipt_field_order=schema,status,controller_sha,source_sha,repository,ref,event,run_id,run_attempt,runner_os,runner_image_version,oci_image,closure_root,generator_sha256,find_sha256,sort_sha256,manifest_sha256,manifest_entries,manifest_bytes,helper_compiled,helper_executed,target_compiled,readiness' "$observation" || fail 'observation receipt field order changed'
grep -Fqx 'receipt_encoding=ascii-key-value,LF-only,no-NUL,no-blank-line,exactly-23-lines,maximum-4096-bytes' "$observation" || fail 'observation receipt encoding changed'

if grep -R -Fq 'controller-helper-closure-verifier-observation-repair-semantics-v0' "$root/.github/workflows"; then
    fail 'inactive observation repair semantics are wired directly to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier observation repair semantics cover three tokens and eight templates, inactive and directly unwired'
