#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
fail(){ printf 'preauth-cutover:%s\n' "$1" >&2; exit 1; }
manifest=spec/lab/preauth/cutover-v1.manifest
[ -f "$manifest" ] && [ ! -L "$manifest" ] || fail manifest-missing
[ "$(grep -c '^schema=rar-preauth-cutover-v1$' "$manifest")" -eq 1 ] || fail manifest-schema
[ "$(grep -c '^production_entrypoint=tools/toolchain/preauth-transaction$' "$manifest")" -eq 1 ] || fail sole-entrypoint
[ "$(grep -c '^status=m1.5-cutover-m2-incomplete$' "$manifest")" -eq 1 ] || fail completeness-status

for required in tools/toolchain/preauth-transaction tools/rar-lab/preauth/src/lib.rs \
 spec/lab/preauth/locks/r0-x86_64-preauth-input-v4.lock \
 spec/lab/preauth/transaction-graph-v1.fields spec/lab/preauth/transaction-bundle-v1.fields; do
 [ -f "$required" ] && [ ! -L "$required" ] || fail "unsafe-required:$required"
done

for removed in spec/lab/preauth/locks/r0-x86_64-preauth-v3.lock \
 spec/lab/preauth/prepared spec/lab/vm-profile/prepared; do
 [ ! -e "$removed" ] || fail "legacy-production-record:$removed"
done

for shim in tools/toolchain/acquire-preauth-closure.sh tools/toolchain/prepare-preauth-output.sh \
 tools/toolchain/bind-preauth-head.sh tools/toolchain/verify-preauth-oci.sh \
 nucleus/arch/x86_64/build-preauth.sh; do
 set +e; before=$(find out/r0 -type f 2>/dev/null | sort || :); output=$($shim 2>&1); status=$?; after=$(find out/r0 -type f 2>/dev/null | sort || :); set -e
 [ "$status" -eq 73 ] && [ "$output" = legacy-preauth-version-refused ] && [ "$before" = "$after" ] || fail "refusal-shim:$shim"
done

workflow=.github/workflows/specifications.yml
[ "$(grep -c 'tools/toolchain/preauth-transaction --prepare' "$workflow")" -eq 1 ] || fail workflow-entrypoint
! grep -Eq 'eval|source[[:space:]]|^[[:space:]]*\.[[:space:]]|RAR_ALLOW_.*LEGACY|fallback' "$workflow" || fail workflow-indirection

for root_path in .github/workflows tools/toolchain tools/rar-lab/preauth/src tools/rar-lab/safety/src tools/rarbuild/src tests/preauth/src tests/host-safety/src; do
 if rg -n -i 'rar-preauth-closure-v3|rar-preauth-identity-graph-v2|rar-preauth-ci-attestation-v2|rar-preauth-prepared-certification-v1|rar-execution-host-v1|rar-disposable-disk-v1|rar-vm-profile-v1|rar-vm-certification-v1|rar-vm-owner-authorization-v1|AuthorizationRecord|AuthorizationConsumptionKey|DescriptorBinding' "$root_path"; then
  fail "production-legacy-token:$root_path"
 fi
done

matches=$(rg -l -i 'rar-preauth-closure-v3|rar-preauth-identity-graph-v2|rar-preauth-ci-attestation-v2|rar-preauth-prepared-certification-v1|rar-external-authorization-v1|rar-execution-host-v1|rar-disposable-disk-v1|rar-vm-profile-v1|rar-vm-certification-v1|rar-vm-owner-authorization-v1|AuthorizationRecord|AuthorizationConsumptionKey|DescriptorBinding|prepare-preauth-output.sh|acquire-preauth-closure.sh|preauth-validate-record' . --glob '!out/**' --glob '!.git/**' || :)
printf '%s\n' "$matches" | while IFS= read -r path; do
 [ -n "$path" ] || continue
 path=${path#./}
 case "$path" in
  tests/preauth/fixtures/legacy-rejection/*|tests/preauth/fixtures/cutover-mutations.v1|docs/adr/*|docs/release-0/*|docs/tasks/*|tools/ci/check-preauth-cutover.sh|tools/ci/check-specs.sh|tests/preauth/transaction-contracts.sh|spec/lab/preauth/cutover-v1.manifest) ;;
  *) fail "unallowlisted-legacy-reference:$path";;
 esac
done

mkdir -p out/r0/cutover
snippet=out/r0/cutover/removed-api.rs
printf '%s\n' '#[path="../../../tools/rar-lab/preauth/src/lib.rs"] mod preauth;' 'use preauth::{AuthorizationRecord,AuthorizationConsumptionKey,DescriptorBinding,IdentityGraph,ClosureLock,PreparedCertification,ExecutionHostRecord,StrictAuthorityRecord};' 'fn main(){}' > "$snippet"
if rustc --edition=2024 "$snippet" -o out/r0/cutover/removed-api 2>out/r0/cutover/removed-api.stderr; then fail old-api-exported; fi
for removed_type in AuthorizationRecord AuthorizationConsumptionKey DescriptorBinding IdentityGraph ClosureLock PreparedCertification ExecutionHostRecord StrictAuthorityRecord; do
 grep -q "$removed_type" out/r0/cutover/removed-api.stderr || fail "old-api-negative-compile:$removed_type"
done
rustc --edition=2024 tools/toolchain/preauth-validate-record.rs -o out/r0/cutover/record-refusal
rustc --edition=2024 tools/toolchain/preauth-verify-oci.rs -o out/r0/cutover/oci-refusal
rustc --edition=2024 tools/rar-lab/preauth/src/disk.rs -o out/r0/cutover/disk-refusal
for executable in out/r0/cutover/record-refusal out/r0/cutover/oci-refusal out/r0/cutover/disk-refusal; do
 set +e; output=$($executable 2>&1); status=$?; set -e
 [ "$status" -eq 73 ] && [ "$output" = legacy-preauth-version-refused ] || fail "compiled-refusal:$executable"
done

reject_mutation(){
 case "$1" in
  *acquire-preauth-closure.sh*|*prepare-preauth-output.sh*|*AuthorizationRecord*|*rar-preauth-closure-v3*|*rar-preauth-prepared-certification-v1*|*RAR_ALLOW_LEGACY_PREAUTH*) return 0;;
  *) return 1;;
 esac
}
if [ "${1-}" = --self-test ]; then
 count=0
 while IFS='|' read -r kind payload; do
  case "$kind" in schema=*) continue;; esac
  [ -n "$kind" ] && reject_mutation "$payload" || fail "mutation-not-rejected:$kind"
  count=$((count+1))
 done < tests/preauth/fixtures/cutover-mutations.v1
 [ "$count" -eq 6 ] || fail mutation-count
fi
printf '%s\n' 'preauth cutover checks passed'
