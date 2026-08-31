#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
subject=$root/spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields
fail() { printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-evidence-v0 source check failed: '"$1" >&2; exit 1; }
[ -f "$subject" ] && [ ! -L "$subject" ] || fail 'subject unavailable'
actual=$(env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$subject" | /usr/bin/awk '{print $1}')
[ "$actual" = 4de1353a2bc22af60e56f38f8139c925fffdcaca14e362521d570f79271ac175 ] || fail 'subject bytes escaped review'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-evidence-v0' "$subject" || fail 'schema changed'
for line in \
 'header_fields=schema,controller_sha,source_sha,subject_sha,validation_sha,dispositions_sha,templates_sha,precedence_sha,faults_sha,cases_sha,base_fixture_sha,fixture_image_sha,tool_pins_sha,run_nonce,root_identity,runtime_case_count,residual_proof_count,logical_relationship_count,failed_count,verdict' \
 'runtime_case_fields=case_id,kind,disposition_id,class_id,stage,mutation_or_fault,repair,expected_exit,observed_exit,stdout_sha256,stderr_sha256,pre_state_sha256,post_state_sha256,result' \
 'residual_proof_fields=case_id,kind,source_id,class_or_relation,stage_or_sides,disposition,reason,source_sha256,proof_sha256,result' \
 'case_count_rule=117-runtime-disposition+37-runtime-precedence+12-runtime-fault=166-runtime;30-disposition-residual+13-precedence-residual=43-residual-proofs;209-logical-relationships-total,each-required-ID-exactly-once' \
 'ordering_rule=logical-order-exactly-V001-through-V147+Q001-through-Q050+X001-through-X012;runtime+residual-projections-preserve-logical-order' \
 'residual_result_rule=pass-only-when-catalog+reason+source+proof-identities-match-reviewed-residual-contract;residual-records-have-no-expected-exit+observed-exit+stdout+stderr+pre-state+post-state+filesystem-delta' \
 'anti_replay_rule=run-nonce+root+controller+source+fixture-image+tool-pins-tuple-never-reused,no-cross-revision-evidence' \
 'no_success_effect=lock+inventory+profile+gate+readiness+workflow+GitHub+compiler-use+helper-build-unchanged'; do grep -Fqx "$line" "$subject" || fail "missing invariant: $line"; done
grep -Fq 'local_rule=' "$subject" || fail 'local execution denial missing'
if grep -R -Fq 'controller-helper-closure-verifier-evidence-v0.fields' "$root/.github/workflows"; then fail 'source-only contract is wired to a workflow'; fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
printf '%s\n' 'rar-alpha-controller-helper-closure-verifier-evidence-v0 is complete, source-only, inactive, and unwired'
