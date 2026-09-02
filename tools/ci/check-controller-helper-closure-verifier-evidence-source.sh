#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
contract=$root/spec/alpha/lab/controller-helper-closure-verifier-evidence-v0.fields
validator=$root/tools/ci/verify-controller-helper-closure-verifier-evidence.sh
policy=$root/tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh
valid=$root/tools/ci/fixtures/controller-helper-closure-verifier/evidence-valid.v0
malformed=$root/tools/ci/fixtures/controller-helper-closure-verifier/evidence-malformed.v0
cases=$root/tools/ci/fixtures/controller-helper-closure-verifier/evidence-cases.v0
fail() { printf 'C3VA evidence source check failed: %s\n' "$1" >&2; exit 1; }
sha_file() { env -u LC_CTYPE LC_ALL=C LANG=C /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }

for file in "$contract" "$validator" "$policy" "$valid" "$malformed" "$cases"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] ||
        fail "required source unavailable: $file"
done
[ "$(sha_file "$contract")" = d3cff3a4f9e566cb37fdea4b7aefd769e8ef6b99fece1ccef44ec30eac198448 ] ||
    fail 'evidence contract bytes escaped review'
[ "$(sha_file "$validator")" = e7ff91c5bdd2c170370453e748a0cfd87853860ecb8b613a68597a1ee03c58b0 ] ||
    fail 'evidence validator bytes escaped review'
[ "$(sha_file "$policy")" = 74153523a0d69bcb26823739eb755511e7664e5571172d7cec7f8d5b24e3b32c ] ||
    fail 'evidence policy bytes escaped review'
[ "$(sha_file "$valid")" = cc5bda22d4bba1eeb7e53d9604fcef23b0eabfc7402216826dcd124d89c6072c ] ||
    fail 'valid seed bytes escaped review'
[ "$(sha_file "$malformed")" = 847158a1a0201ffea557e78845dcdafb356519e3fe3adaa1e4302820866500e2 ] ||
    fail 'malformed seed bytes escaped review'
[ "$(sha_file "$cases")" = ac194d1c6ff7bba2c50f17d835c03ad237e3cbb5aa65e58443869f4b6b9bb3c4 ] ||
    fail 'evidence mutation catalog bytes escaped review'

for required in \
    'status=experimental-C3VA-candidate-source-only-unwired' \
    'execution_authority=none-until-complete-C3VA-review+merge+exact-main-validation' \
    'trusted_header_rule=repository+controller_sha+source_sha+run_id+run_attempt+verification_receipt_sha256-must-byte-equal-validator-invocation-context;all-other-digest-domains-must-byte-equal-separately-trusted-inputs' \
    'aggregate_rule=checked-unsigned-arithmetic;blob_count-equals-actual-B-records;chunk_count-equals-actual-C-records+sum-of-B-declared-chunk-counts;decoded_bytes-equals-sum-of-decoded-C-payload-lengths+sum-of-B-declared-decoded-lengths;normalized_count-equals-actual-N-records' \
    'chunk_payload_rule=decoded-length-1..1436;every-nonfinal-chunk-exactly-1436-decoded-bytes+canonical-one-equals-padding;final-chunk-1..1436;concatenated-decoded-length+SHA256-equal-B-header' \
    'decoded_equality_rule=consume-exact-declared-field-bytes+SHA256-must-equal-field-digest;checked-field-count+framing+raw-byte-sum-must-equal-B-decoded-length;B-SHA256-covers-complete-envelope-framing+raw-bytes+terminating-LFs' \
    'semantic_payload_rule=payload_bytes-canonical-positive-1..16777216;payload_sha256-nonzero-lowercase-SHA256;validator-consumes+hashes-exact-arbitrary-payload-and-rejects-extension' \
    'oracle_derivation_rule=observed-event+timeout-termination+residual-proof-payloads-byte-equal-exact-case-catalog-oracle;mutation-schedule+trigger+acknowledgement+residual-source-payloads-byte-equal-exact-complete-catalog-row' \
    'receipt_reconstruction_rule=verification-receipt-inputs-is-exact-31-line-canonical-receipt-bytes;validator-checks-line-order+literal+trusted-run+digest+decimal+false-domains,candidate-manifest-equals-recomputed-manifest,and-field-SHA256-equals-trusted-header-receipt-SHA256' \
    'blob_allocation=B000001-clean-success-pass-1+RUN;B000002-clean-success-pass-2+RUN;then-logical-case-order-with-four-consecutive-pre+stdout+stderr+post-blobs-per-runtime-row-or-one-residual-source-proof-blob-per-residual-row;last-B000709' \
    'raw_blob_ids_rule=nonempty-canonical-B-identities-comma-separated-no-spaces;runtime-exact-four-assigned-consecutive-IDs-in-pre+stdout+stderr+post-order;residual-exact-one-assigned-ID;globals-never-listed' \
    'base64_payload_total_max=396032120' \
    'evidence_bound=396032120+206741*128+709*256+209*2048+8192=423112696,below-440401920' \
    'anti_replay_rule=stateless-validator-proves-within-artifact-uniqueness+trusted-header-binding-only;global-historical-reuse-is-not-claimed;C3VB-one-shot-context+external-review-reject-reused-run-tuple-or-receipt' \
    'local_rule=text+hash+structure-check-only-on-Mac;validator-policy-execution-GitHub-hosted-only;never-run-verifier+controller+container+compiler+helper+target+VM+emulator-on-Mac'; do
    grep -Fqx "$required" "$contract" || fail "missing contract invariant: $required"
done

/bin/sh -n "$validator" "$policy" || fail 'validator or policy shell syntax invalid'
grep -Fqx 'schema=rar-alpha-controller-helper-closure-verifier-evidence-policy-cases-v0' "$cases" ||
    fail 'mutation catalog schema invalid'
grep -Fqx 'case_count=20' "$cases" || fail 'mutation case count declaration invalid'
[ "$(grep -Ec '^case\|EP[0-9][0-9][0-9]\|[a-zA-Z0-9-]+\|reject$' "$cases")" -eq 20 ] ||
    fail 'mutation case rows incomplete'
number=1
while [ "$number" -le 20 ]; do
    id=$(printf 'EP%03d' "$number")
    [ "$(grep -Ec "^case\\|$id\\|" "$cases")" -eq 1 ] ||
        fail "mutation case missing or duplicated: $id"
    number=$((number + 1))
done
grep -Fqx 'blob_header=B|B000001|clean-success-pass-1|RUN|1|1|ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb' "$valid" ||
    fail 'valid seed domain changed'
grep -Fqx 'chunk=C|B000001|C000001|YQ==' "$valid" ||
    fail 'valid canonical Base64 seed changed'
grep -Fqx 'chunk=C|B000001|C000001|YQ' "$malformed" ||
    fail 'malformed Base64 seed changed'
[ "$(grep -Fxc 'tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh|ephemeral' "$root/tools/ci/policy-test-modes.v0")" -eq 1 ] ||
    fail 'evidence policy registry row missing or duplicated'
grep -Fqx '/bin/sh "$root/tools/ci/test-controller-helper-closure-verifier-evidence-policy.sh"' "$root/tools/ci/run-ephemeral-policy-tests.sh" ||
    fail 'evidence policy test is not run by ephemeral runner'
if grep -R -Eq 'verify-controller-helper-closure-verifier-evidence|test-controller-helper-closure-verifier-evidence-policy' "$root/.github/workflows"; then
    fail 'C3VA source is directly wired to a workflow'
fi
if grep -Eq 'docker|rustc|cargo|qemu|/var/run/docker.sock' "$validator" "$policy"; then
    fail 'C3VA validator policy gained forbidden runtime authority'
fi
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" ||
    fail 'CI closure lock activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" ||
    fail 'helper inventory activated'
printf '%s\n' 'C3VA evidence sources are byte-bound, GitHub-policy-only, source-only, and unwired'
