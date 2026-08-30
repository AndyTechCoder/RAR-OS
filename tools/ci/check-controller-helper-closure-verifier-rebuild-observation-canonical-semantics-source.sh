#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
semantics=$root/spec/alpha/lab/controller-helper-closure-verifier-rebuild-observation-canonical-semantics-v0
templates=$root/spec/alpha/lab/controller-helper-closure-verifier-case-templates-v0
inventory=$root/spec/alpha/lab/controller-helper-closure-verifier-operator-inventory-v0
domain=$root/spec/alpha/lab/controller-helper-closure-verifier-input-domain-v0.fields
scalar=$root/spec/alpha/lab/controller-helper-closure-verifier-scalar-semantics-v0
observation=$root/spec/alpha/lab/controller-helper-closure-observation-v0.fields
shared=$root/spec/alpha/lab/controller-helper-closure-verifier-observation-repair-semantics-v0

fail() {
    printf 'controller-helper closure verifier rebuild-observation-canonical semantics source check failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'
}

for file in "$semantics" "$templates" "$inventory" "$domain" "$scalar" "$observation" "$shared"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular source is unavailable: $file"
done
[ "$(sha_file "$semantics")" = 43a969d44d748e1a88107c6f5ac560f392162ef206e76c047b05284bc48b9ce0 ] || fail 'canonical rebuild semantics bytes escaped review'
[ "$(sha_file "$templates")" = 443d30414ec3cc8542755006ada9a40d52e0f5efe3b26de7fdc5f82dc1152be4 ] || fail 'case-template bytes escaped review'
[ "$(sha_file "$inventory")" = ea2aef334d7c6b612635ea5237926df1a459a255c61c73a9e5b998e1cc244a80 ] || fail 'operator inventory bytes escaped review'
[ "$(sha_file "$domain")" = 67555f2d565569e95b44a247dda630c9b98d293ba0773880f248d69d802ac66c ] || fail 'input domain bytes escaped review'
[ "$(sha_file "$scalar")" = fedf6b24d1b9356ebbcaf2c27f011358937a05671a06021d083c2c874ceaca10 ] || fail 'scalar semantics bytes escaped review'
[ "$(sha_file "$observation")" = 944229644f0805876403cd858d0d8c3c993d73bf00baef4c3ffcbbc7a2522836 ] || fail 'observation schema bytes escaped review'
[ "$(sha_file "$shared")" = f31acd357fc3e6889818697776155da6a2c49b0aea20a1180a8bf7089d70b4ad ] || fail 'shared observation repair semantics bytes escaped review'

for required in \
    'status=experimental-incomplete-inactive-source-only' \
    'execution_authority=none' \
    'semantic_row_count=3' \
    'repair_token_coverage=rebuild-observation-canonical' \
    'covered_template_count=3' \
    'canonical_rebuild_rule=reconstruct-exactly-23-LF-terminated-lines-in-observation_schema-order+encoding+key-spelling;copy-the-recorded-scalar-primary-post-bytes-only-into-the-one-named-field;copy-the-other-22-field-values-byte-for-byte-from-the-pinned-base-receipt;no-field-insertion+deletion+reordering+normalization+coercion+ambient-rederivation' \
    'manifest_derivation_rule=from-the-exact-unchanged-manifest-bytes-compute-lowercase-64-ASCII-hex-SHA-256+LF-octet-count-as-canonical-positive-decimal-1-through-65536+raw-octet-count-as-canonical-positive-decimal-1-through-1048576' \
    'named_mismatch_rule=the-base-named-field-must-equal-its-manifest_derivation_rule-value;the-recorded-S008-or-S011-primary-post-bytes-must-differ-from-that-derived-value+remain-in-the-declared-field-domain;failure-invalidates-before-rebuild' \
    'dependent_consistency_rule=every-untargeted-manifest_sha256+manifest_entries+manifest_bytes-field-must-equal-its-manifest_derivation_rule-value-after-rebuild;the-one-targeted-field-must-equal-only-the-recorded-primary-post-bytes;all-other-20-fields-must-equal-their-pinned-base-bytes' \
    'manifest_preservation_rule=manifest-path+device+inode+nlink+uid+gid+mode+bytes+size+mtime+ctime-remain-exactly-recorded+unchanged-through-primary+rebuild+launch;the-rebuild-never-writes+aliases+replaces+normalizes+parses+sorts+or-truncates-the-manifest' \
    'shared_receipt_safety_rule=apply-without-relaxation-the-pinned-shared-observation-repair-semantics-receipt_file_rule+receipt_stability_rule-to-the-rebuilt-receipt' \
    'independence_rule=rebuild-reads-only-the-byte-pinned-base-receipt+recorded-S008-or-S011-primary-post-bytes+recorded-unchanged-manifest-bytes+future-byte-pinned-controller-SHA-256;no-verifier-output+scratch+another-case+ambient-host-state+unreviewed-path' \
    'remaining_status=2-non-none-repair-tokens+23-primary-families+pre-start+repair-coupled-links+required-path-symlinks+raw-name+path-alias+mount+tree+manifest-specific-primary-families+exact-base+controller+runtime-precedence+fault+evidence+verdict-remain-absent' \
    'activation_rule=blocked;this-slice-cannot-create-fixtures+execute-mutations+apply-repairs+or-authorize-a-controller' \
    'consumer_rule=this-contract-does-not-authorize-fixture+mutation+repair+controller+container+compiler+helper+target+VM+emulator+workflow+wiring+gate+readiness' \
    'local_rule=text+hash+structure-check-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$semantics" || fail "required invariant is missing: $required"
done

[ "$(tail -c 1 "$semantics" | /usr/bin/od -An -tuC | /usr/bin/tr -d ' ')" = 10 ] || fail 'semantics lack one terminal LF'
if LC_ALL=C grep -n '[^ -~]' "$semantics" >/dev/null; then fail 'semantics contain a non-ASCII byte'; fi
if grep -n "$(printf '\r')" "$semantics" >/dev/null; then fail 'semantics contain CR'; fi
[ "$(grep -Ec '^B[0-9][0-9][0-9]\|C[0-9][0-9][0-9]\|[^| ]+\|[^| ]+\|[^| ]+\|[^| ]+$' "$semantics")" -eq 3 ] || fail 'semantic rows are malformed'

grep -Fqx 'B001|C075|field./evidence/controller-helper-closure.receipt.manifest_sha256|set-other-lower-sha256|exact-S011-post-bytes|manifest_entries=entries-derivation+manifest_bytes=bytes-derivation' "$semantics" || fail 'C075 rebuild semantics changed'
grep -Fqx 'B002|C076|field./evidence/controller-helper-closure.receipt.manifest_bytes|base-plus-1|exact-S008-post-bytes|manifest_sha256=digest-derivation+manifest_entries=entries-derivation' "$semantics" || fail 'C076 rebuild semantics changed'
grep -Fqx 'B003|C082|field./evidence/controller-helper-closure.receipt.manifest_entries|base-plus-1|exact-S008-post-bytes|manifest_sha256=digest-derivation+manifest_bytes=bytes-derivation' "$semantics" || fail 'C082 rebuild semantics changed'

[ "$(grep -c '|rebuild-observation-canonical|' "$templates")" -eq 3 ] || fail 'canonical rebuild template count changed'
grep -Fqx 'C075|D078|E066|observation-record|pre-start|field./evidence/controller-helper-closure.receipt.manifest_sha256=set-other-lower-sha256|rebuild-observation-canonical|E066@observation-record+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'C075 template changed'
grep -Fqx 'C076|D079|E067|observation-record|pre-start|field./evidence/controller-helper-closure.receipt.manifest_bytes=base-plus-1|rebuild-observation-canonical|E067@observation-record+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'C076 template changed'
grep -Fqx 'C082|D087|E073|candidate-manifest|pre-start|field./evidence/controller-helper-closure.receipt.manifest_entries=base-plus-1|rebuild-observation-canonical|E073@candidate-manifest+normal-exit-status-1+no-valid-final-receipt' "$templates" || fail 'C082 template changed'
grep -Fqx 'R003|rebuild-observation-canonical|opaque' "$inventory" || fail 'canonical rebuild repair token changed'
grep -Fqx 'S008|base-plus-1|receipt-manifest-count-field|manifest_bytes-base-is-mathematical-1-through-1048576-or-manifest_entries-base-is-1-through-65536+canonical-positive-decimal-bytes|compute-in-unbounded-mathematical-integer+set-canonical-decimal-to-base-plus-1;post-range-is-2-through-1048577-or-2-through-65537+truncate+wrap+native-width-coercion-are-invalid' "$scalar" || fail 'S008 scalar semantics changed'
grep -Fqx 'S011|set-other-lower-sha256|field|declared-64-byte-lowercase-hex-field|set-64-ASCII-zero-bytes-when-base-is-not-all-zero+otherwise-set-64-ASCII-one-bytes' "$scalar" || fail 'S011 scalar semantics changed'

grep -Fqx 'receipt_field_order=schema,status,controller_sha,source_sha,repository,ref,event,run_id,run_attempt,runner_os,runner_image_version,oci_image,closure_root,generator_sha256,find_sha256,sort_sha256,manifest_sha256,manifest_entries,manifest_bytes,helper_compiled,helper_executed,target_compiled,readiness' "$observation" || fail 'observation receipt field order changed'
grep -Fqx 'receipt_encoding=ascii-key-value,LF-only,no-NUL,no-blank-line,exactly-23-lines,maximum-4096-bytes' "$observation" || fail 'observation receipt encoding changed'
for inherited in receipt_file_rule receipt_stability_rule; do
    [ "$(grep -c "^${inherited}=" "$shared")" -eq 1 ] || fail "shared receipt safety invariant is missing or duplicated: $inherited"
done

if grep -R -Fq 'controller-helper-closure-verifier-rebuild-observation-canonical-semantics-v0' "$root/.github/workflows"; then
    fail 'inactive canonical rebuild semantics are wired directly to GitHub Actions'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI lock was activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory is not blocked'

printf '%s\n' 'controller-helper closure verifier canonical observation rebuild semantics cover one token and three templates, inactive and directly unwired'
