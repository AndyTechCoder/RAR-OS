#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

evidence_dir=${1-}
wrapper=${2-}
subject=${3-}
fixture=${4-}
tool_pins=${5-}
fail() { printf 'controller-helper observer run evidence rejected: %s\n' "$1" >&2; exit 1; }

[ -d "$evidence_dir" ] && [ ! -L "$evidence_dir" ] || fail 'evidence directory missing or symbolic'
for file in "$wrapper" "$subject" "$fixture" "$tool_pins"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "trusted input missing, symbolic, or empty: $file"
done
case_evidence=$evidence_dir/controller-helper-closure-observer.cases.v0
manifest=$evidence_dir/controller-helper-closure.sha256
receipt=$evidence_dir/controller-helper-closure.receipt
record=$evidence_dir/controller-helper-closure-observer-run-evidence.v0
expected_set='controller-helper-closure-observer-run-evidence.v0|f
controller-helper-closure-observer.cases.v0|f
controller-helper-closure.receipt|f
controller-helper-closure.sha256|f'
actual_set=$(/usr/bin/find -P "$evidence_dir" -mindepth 1 -maxdepth 1 -printf '%f|%y\n' | /usr/bin/sort)
[ "$actual_set" = "$expected_set" ] || fail 'output set is missing, extra, symbolic, or non-regular'

size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }
links_of() { /usr/bin/stat -c %h "$1" 2>/dev/null || /usr/bin/stat -f %l "$1"; }
sha_file() { /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
for file in "$case_evidence" "$manifest" "$receipt" "$record"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "output missing, symbolic, or empty: $file"
    [ "$(links_of "$file")" -eq 1 ] || fail "output is aliased: $file"
done
case_bytes=$(size_of "$case_evidence")
manifest_bytes=$(size_of "$manifest")
receipt_bytes=$(size_of "$receipt")
record_bytes=$(size_of "$record")
[ "$case_bytes" -ge 1 ] && [ "$case_bytes" -le 32768 ] || fail 'case evidence size invalid'
[ "$manifest_bytes" -ge 1 ] && [ "$manifest_bytes" -le 1048576 ] || fail 'manifest size invalid'
[ "$receipt_bytes" -ge 1 ] && [ "$receipt_bytes" -le 4096 ] || fail 'receipt size invalid'
[ "$record_bytes" -ge 1 ] && [ "$record_bytes" -le 4096 ] || fail 'record size invalid'

[ "$(/usr/bin/tail -c 1 "$record" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'record lacks terminal LF'
if /usr/bin/od -An -tx1 "$record" | /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)'; then fail 'record contains NUL or CR'; fi
if /usr/bin/grep -n '[^ -~]' "$record" >/dev/null; then fail 'record contains non-printable ASCII'; fi
if /usr/bin/grep -n '^$' "$record" >/dev/null; then fail 'record contains blank line'; fi
[ "$(/usr/bin/wc -l < "$record" | /usr/bin/tr -d ' ')" -eq 31 ] || fail 'record line count invalid'

for name in RAR_EXPECTED_REPOSITORY RAR_EXPECTED_REF RAR_EXPECTED_EVENT \
    RAR_TRUSTED_CONTROLLER_SHA RAR_EXPECTED_SOURCE_REVISION RAR_EXPECTED_RUN_ID \
    RAR_EXPECTED_RUN_ATTEMPT RAR_CI_RUNNER_IMAGE_OS RAR_CI_RUNNER_IMAGE_VERSION \
    RAR_CI_RUNNER_OS RAR_CI_RUNNER_ARCH RAR_CI_BOOTSTRAP_IMAGE \
    RAR_EXPECTED_WRAPPER_SHA256 RAR_EXPECTED_SUBJECT_SHA256 \
    RAR_EXPECTED_FIXTURE_SHA256 RAR_EXPECTED_TOOL_PINS_SHA256 \
    RAR_EXPECTED_ARTIFACT_NAME RAR_EXPECTED_RECORD_NONCE; do
    eval "value=\${$name-}"
    [ -n "$value" ] || fail "missing trusted expectation: $name"
done
[ "$RAR_EXPECTED_REPOSITORY" = AndyTechCoder/RAR-OS ] || fail 'repository expectation invalid'
[ "$RAR_EXPECTED_REF" = refs/heads/main ] || fail 'ref expectation invalid'
[ "$RAR_EXPECTED_EVENT" = push ] || fail 'event expectation invalid'
for revision in "$RAR_TRUSTED_CONTROLLER_SHA" "$RAR_EXPECTED_SOURCE_REVISION"; do
    case "$revision" in '' | *[!0-9a-f]*) fail 'revision malformed' ;; esac
    [ "${#revision}" -eq 40 ] && [ "$revision" != 0000000000000000000000000000000000000000 ] || fail 'revision invalid'
done
[ "$RAR_TRUSTED_CONTROLLER_SHA" = "$RAR_EXPECTED_SOURCE_REVISION" ] || fail 'controller/source revision mismatch'
for value in "$RAR_EXPECTED_RUN_ID" "$RAR_EXPECTED_RUN_ATTEMPT"; do
    case "$value" in '' | 0 | 0* | *[!0-9]*) fail 'run identity is not canonical positive decimal' ;; esac
    [ "${#value}" -le 20 ] || fail 'run identity oversized'
done
[ "$RAR_CI_RUNNER_IMAGE_OS" = ubuntu24 ] && [ "$RAR_CI_RUNNER_OS" = Linux ] && [ "$RAR_CI_RUNNER_ARCH" = X64 ] || fail 'runner identity invalid'
case "$RAR_CI_RUNNER_IMAGE_VERSION" in '' | *[!0-9.]* | .* | *. | *..*) fail 'runner image version malformed' ;; esac
[ "${#RAR_CI_RUNNER_IMAGE_VERSION}" -le 64 ] || fail 'runner image version oversized'
[ "$RAR_CI_BOOTSTRAP_IMAGE" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || fail 'OCI image identity invalid'
zero=0000000000000000000000000000000000000000000000000000000000000000
for value in "$RAR_EXPECTED_WRAPPER_SHA256" "$RAR_EXPECTED_SUBJECT_SHA256" \
    "$RAR_EXPECTED_FIXTURE_SHA256" "$RAR_EXPECTED_TOOL_PINS_SHA256" "$RAR_EXPECTED_RECORD_NONCE"; do
    case "$value" in '' | *[!0-9a-f]*) fail 'trusted digest malformed' ;; esac
    [ "${#value}" -eq 64 ] && [ "$value" != "$zero" ] || fail 'trusted digest invalid'
done
[ "$(sha_file "$wrapper")" = "$RAR_EXPECTED_WRAPPER_SHA256" ] || fail 'wrapper differs from trusted identity'
[ "$(sha_file "$subject")" = "$RAR_EXPECTED_SUBJECT_SHA256" ] || fail 'subject differs from trusted identity'
[ "$(sha_file "$fixture")" = "$RAR_EXPECTED_FIXTURE_SHA256" ] || fail 'fixture differs from trusted identity'
[ "$(sha_file "$tool_pins")" = "$RAR_EXPECTED_TOOL_PINS_SHA256" ] || fail 'tool pins differ from trusted identity'
[ "$RAR_EXPECTED_ARTIFACT_NAME" = "controller-helper-closure-observer-$RAR_EXPECTED_RUN_ID-$RAR_EXPECTED_RUN_ATTEMPT" ] || fail 'artifact expectation invalid'

[ "$(/usr/bin/tail -c 1 "$case_evidence" | /usr/bin/od -An -tx1 | /usr/bin/tr -d '[:space:]')" = 0a ] || fail 'case evidence lacks terminal LF'
if /usr/bin/od -An -tx1 "$case_evidence" | /usr/bin/grep -Eq '(^| )00( |$)|(^| )0d( |$)'; then fail 'case evidence contains NUL or CR'; fi
if /usr/bin/grep -n '[^ -~]' "$case_evidence" >/dev/null; then fail 'case evidence contains non-printable ASCII'; fi
if /usr/bin/grep -n '^$' "$case_evidence" >/dev/null; then fail 'case evidence contains blank line'; fi
[ "$(/usr/bin/wc -l < "$case_evidence" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'case evidence line count invalid'
/usr/bin/awk -F '|' \
    -v controller="$RAR_TRUSTED_CONTROLLER_SHA" \
    -v source="$RAR_EXPECTED_SOURCE_REVISION" \
    -v subject="$RAR_EXPECTED_SUBJECT_SHA256" \
    -v fixture="$RAR_EXPECTED_FIXTURE_SHA256" \
    -v pins="$RAR_EXPECTED_TOOL_PINS_SHA256" '
function hexn(v,n) { return length(v)==n && v !~ /[^0-9a-f]/ && v !~ /^0+$/ }
NR==1 { if ($0!="schema=rar-alpha-controller-helper-closure-observer-case-evidence-v0") bad=1; next }
NR==2 { if ($0!="case_count=21") bad=1; next }
NR>2 {
    ordinal=NR-2
    id=sprintf("O%03d",ordinal)
    wanted_exit=(ordinal==1 ? "0" : "1")
    wanted_result=(ordinal==1 ? "pass" : "expected-rejection")
    wanted_verdict=(ordinal==1 ? "observed-not-reviewed-not-ready" : "normalized-not-ready")
    if (NF!=14 || $1!="case" || $2!=id || $3!=controller || $4!=source || $3!=$4 ||
        $5!=subject || $6!=fixture || $7!=pins || !hexn($8,64) || !hexn($9,64) ||
        $8==$9 || seen_nonce[$8]++ || seen_root[$9]++ || $10!=wanted_exit ||
        !hexn($11,64) || !hexn($12,64) || $13!=wanted_result || $14!=wanted_verdict) bad=1
    rows++
}
END { if (rows!=21 || bad) exit 1 }
' "$case_evidence" || fail 'case evidence grammar, identity, order, uniqueness, exit, result, or verdict invalid'

case_evidence_sha=$(sha_file "$case_evidence")
manifest_sha=$(sha_file "$manifest")
receipt_sha=$(sha_file "$receipt")
for derived_digest in "$case_evidence_sha" "$manifest_sha" "$receipt_sha"; do
    case "$derived_digest" in '' | *[!0-9a-f]*) fail 'derived output digest malformed' ;; esac
    [ "${#derived_digest}" -eq 64 ] && [ "$derived_digest" != "$zero" ] || fail 'derived output digest invalid or zero'
done

expected="schema=rar-alpha-controller-helper-closure-observer-run-evidence-v0
status=candidate-not-reviewed-not-ready
repository=$RAR_EXPECTED_REPOSITORY
ref=$RAR_EXPECTED_REF
event=$RAR_EXPECTED_EVENT
controller_sha=$RAR_TRUSTED_CONTROLLER_SHA
source_sha=$RAR_EXPECTED_SOURCE_REVISION
run_id=$RAR_EXPECTED_RUN_ID
run_attempt=$RAR_EXPECTED_RUN_ATTEMPT
runner_image_os=$RAR_CI_RUNNER_IMAGE_OS
runner_image_version=$RAR_CI_RUNNER_IMAGE_VERSION
runner_os=$RAR_CI_RUNNER_OS
runner_arch=$RAR_CI_RUNNER_ARCH
oci_image=$RAR_CI_BOOTSTRAP_IMAGE
wrapper_sha256=$RAR_EXPECTED_WRAPPER_SHA256
subject_sha256=$RAR_EXPECTED_SUBJECT_SHA256
fixture_sha256=$RAR_EXPECTED_FIXTURE_SHA256
tool_pins_sha256=$RAR_EXPECTED_TOOL_PINS_SHA256
case_evidence_sha256=$case_evidence_sha
manifest_sha256=$manifest_sha
receipt_sha256=$receipt_sha
artifact_name=$RAR_EXPECTED_ARTIFACT_NAME
retention_days=14
observed_exit=0
case_evidence_bytes=$case_bytes
manifest_bytes=$manifest_bytes
receipt_bytes=$receipt_bytes
output_count=4
verdict=candidate-not-reviewed-not-ready
record_nonce=$RAR_EXPECTED_RECORD_NONCE"
[ "$(/usr/bin/sed -n '1,30p' "$record")" = "$expected" ] || fail 'record binding, order, field set, or trusted context mismatch'
record_sha=$(/usr/bin/sed -n '1,30p' "$record" | /usr/bin/shasum -a 256 | /usr/bin/awk '{ print $1 }')
case "$record_sha" in '' | *[!0-9a-f]*) fail 'record digest computation malformed' ;; esac
[ "${#record_sha}" -eq 64 ] && [ "$record_sha" != "$zero" ] || fail 'record digest computation invalid or zero'
[ "$(/usr/bin/sed -n '31p' "$record")" = "record_sha256=$record_sha" ] || fail 'record digest mismatch'
printf '%s\n' 'controller-helper observer run evidence validated: exact-main candidate-not-reviewed-not-ready'
