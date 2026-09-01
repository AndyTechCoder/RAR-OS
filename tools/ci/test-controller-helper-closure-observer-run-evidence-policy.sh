#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'controller-helper observer run-evidence mutations skipped: external read-only-source CI required'; exit 0; }
work=$(mktemp -d "$scratch/controller-helper-observer-run-evidence.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/verify-controller-helper-closure-observer-run-evidence.sh
fixtures=$root/tools/ci/fixtures/controller-helper-closure-observer
valid=$fixtures/run-evidence-valid.v0
malformed=$fixtures/run-evidence-malformed.v0
evidence=$work/evidence
wrapper=$work/wrapper
subject=$work/subject
fixture=$work/fixture
tool_pins=$work/tool-pins
record=$evidence/controller-helper-closure-observer-run-evidence.v0

export RAR_EXPECTED_REPOSITORY=AndyTechCoder/RAR-OS
export RAR_EXPECTED_REF=refs/heads/main RAR_EXPECTED_EVENT=push
export RAR_TRUSTED_CONTROLLER_SHA=cccccccccccccccccccccccccccccccccccccccc
export RAR_EXPECTED_SOURCE_REVISION=cccccccccccccccccccccccccccccccccccccccc
export RAR_EXPECTED_RUN_ID=12345 RAR_EXPECTED_RUN_ATTEMPT=1
export RAR_CI_RUNNER_IMAGE_OS=ubuntu24 RAR_CI_RUNNER_IMAGE_VERSION=20260823.283.1
export RAR_CI_RUNNER_OS=Linux RAR_CI_RUNNER_ARCH=X64
export RAR_CI_BOOTSTRAP_IMAGE=sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
export RAR_EXPECTED_WRAPPER_SHA256=f2019e2db64239587dde889676bb202e51343713e3067b45880072803e944fb6
export RAR_EXPECTED_SUBJECT_SHA256=fba77ade0af8c98bb8b09367c201a4645aaf7f8aa55fa2d6289bf55801d7161c
export RAR_EXPECTED_FIXTURE_SHA256=b185ff95d95be586f97e28f0038382021ceb3cb5c406b9c989b5b50dd6eb8c20
export RAR_EXPECTED_TOOL_PINS_SHA256=95b0263551797fc809d29496560988cacefb639a82436c69be514ab37fa11ba1
export RAR_EXPECTED_ARTIFACT_NAME=controller-helper-closure-observer-12345-1
export RAR_EXPECTED_RECORD_NONCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

reset() {
    /bin/rm -rf "$evidence"
    /bin/mkdir -m 700 "$evidence"
    /usr/bin/printf '%s\n' wrapper-v0 > "$wrapper"
    /usr/bin/printf '%s\n' subject-v0 > "$subject"
    /usr/bin/printf '%s\n' fixture-v0 > "$fixture"
    /usr/bin/printf '%s\n' tool-pins-v0 > "$tool_pins"
    /usr/bin/printf '%s\n' case-evidence-v0 > "$evidence/controller-helper-closure-observer.cases.v0"
    /usr/bin/printf '%s\n' manifest-v0 > "$evidence/controller-helper-closure.sha256"
    /usr/bin/printf '%s\n' receipt-v0 > "$evidence/controller-helper-closure.receipt"
    /bin/cp "$valid" "$record"
}
rehash() {
    pre=$work/record.pre
    /usr/bin/sed -n '1,30p' "$record" > "$pre"
    digest=$(/usr/bin/shasum -a 256 "$pre" | /usr/bin/awk '{ print $1 }')
    /bin/mv "$pre" "$record"
    /usr/bin/printf 'record_sha256=%s\n' "$digest" >> "$record"
}
reject() {
    label=$1
    shift
    if "$@" >/dev/null 2>&1; then printf 'unsafe observer run evidence passed: %s\n' "$label" >&2; exit 1; fi
}
validate() { /bin/sh "$checker" "$evidence" "$wrapper" "$subject" "$fixture" "$tool_pins"; }

/bin/sh "$checker" "$evidence" "$wrapper" "$subject" "$fixture" "$tool_pins" >/dev/null 2>&1 && exit 1
reset
validate >/dev/null
reset
/bin/cp "$malformed" "$record"
reject malformed-fixture validate
reset
/bin/rm -f "$evidence/controller-helper-closure.receipt"
reject missing-output validate
reset
/usr/bin/printf '%s\n' extra > "$evidence/extra"
reject extra-output validate
reset
/usr/bin/awk 'NR==8 { a=$0; next } NR==9 { print; print a; next } { print }' "$record" > "$work/reordered"
/bin/mv "$work/reordered" "$record"
rehash
reject reordered-field validate
reset
/usr/bin/sed -i 's/run_id=12345/run_id=12346/' "$record"; rehash
reject stale-run validate
reset
/usr/bin/sed -i 's/source_sha=cccccccccccccccccccccccccccccccccccccccc/source_sha=dddddddddddddddddddddddddddddddddddddddd/' "$record"; rehash
reject cross-revision validate
reset
/usr/bin/printf '%s\n' wrapper-mutated > "$wrapper"
mutated=$(/usr/bin/shasum -a 256 "$wrapper" | /usr/bin/awk '{ print $1 }')
/usr/bin/sed -i "s/wrapper_sha256=.*/wrapper_sha256=$mutated/" "$record"; rehash
reject self-attested-input validate
reset
/usr/bin/sed -i 's/case_evidence_sha256=.*/case_evidence_sha256=0000000000000000000000000000000000000000000000000000000000000000/' "$record"; rehash
reject zero-digest validate
reset
/bin/rm -f "$evidence/controller-helper-closure-observer.cases.v0"
/bin/ln "$evidence/controller-helper-closure.sha256" "$evidence/controller-helper-closure-observer.cases.v0"
reject aliased-output validate
reset
/usr/bin/awk 'BEGIN { for (i=0; i<5000; i++) printf "x" }' >> "$record"
reject oversized-record validate
reset
/usr/bin/printf '%s\n' changed-manifest > "$evidence/controller-helper-closure.sha256"
reject wrong-output validate
reset
/usr/bin/sed -i 's/artifact_name=.*/artifact_name=wrong/' "$record"; rehash
reject wrong-artifact validate
reset
/usr/bin/sed -i 's/status=candidate-not-reviewed-not-ready/status=ready/;s/verdict=candidate-not-reviewed-not-ready/verdict=ready/' "$record"; rehash
reject ready-substitution validate
reset
/usr/bin/sed -i 's/output_count=4/output_count=5/' "$record"; rehash
reject wrong-output-count validate
reset
/usr/bin/sed -i 's/retention_days=14/retention_days=15/' "$record"; rehash
reject wrong-retention validate
reset
/usr/bin/sed -i 's/observed_exit=0/observed_exit=1/' "$record"; rehash
reject nonzero-exit validate
reset
/usr/bin/sed -i 's/record_nonce=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/record_nonce=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/' "$record"; rehash
reject replayed-nonce validate
reset
/bin/mv "$record" "$evidence/real-record"
/bin/ln -s real-record "$record"
reject symbolic-record validate
reset
/bin/ln "$record" "$work/hardlink"
reject hardlinked-record validate
printf '%s\n' 'controller-helper observer run-evidence policy passed: cases=20 candidate-only'
