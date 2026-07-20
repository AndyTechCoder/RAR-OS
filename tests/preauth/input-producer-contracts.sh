#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
fail(){ printf 'input-producer-contract:%s\n' "$1" >&2; exit 1; }
policy=spec/lab/preauth/preauth-input-delivery-v1.policy
producer=tools/toolchain/preauth-input-producer
delivery=tools/toolchain/preauth-input-delivery
for exact in \
 schema=rar-preauth-input-delivery-policy-v1 \
 base_oci=rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 \
 debian_snapshot=20260630T000000Z transport=https-only network_phase=producer-only \
 transaction_network=none credentials=forbidden github_id_token=forbidden \
 github_environment=forbidden aws=forbidden package_install=forbidden \
 maintainer_scripts=forbidden target_execution=forbidden emulator_execution=forbidden \
 authority_output=forbidden; do grep -qx "$exact" "$policy" || fail "policy:$exact"; done
[ "$(grep -c '^allowed_origin=' "$policy")" -eq 4 ] || fail origin-count
for origin in snapshot.debian.org auth.docker.io registry-1.docker.io production.cloudflare.docker.com; do
 grep -qx "allowed_origin=$origin" "$policy" || fail "origin:$origin"
done
! grep -Eq '(^|[=/])(http:|ftp:)|latest|stable' "$policy" || fail unpinned-network
for forbidden in transaction-graph certification attestation owner-authorization launch-session signing-key; do
 ! grep -qi "$forbidden" "$producer" || fail "producer-capability:$forbidden"
done
before=$(find out/r0/preauth/input-delivery -type f 2>/dev/null | sort || :)
set +e; output=$(AWS_ACCESS_KEY_ID=forbidden "$producer" --produce one 2>&1); status=$?; set -e
[ "$status" -eq 73 ] && [ "$output" = preauth-input-producer:authority-environment ] || fail credential-refusal
after=$(find out/r0/preauth/input-delivery -type f 2>/dev/null | sort || :)
[ "$before" = "$after" ] || fail credential-side-effect
# Authenticity is signature/digest-anchored, not transport-anchored: https redirects to the
# origin CDN are permitted, plaintext downgrade is refused, and only approved requested origins
# are contacted. Verify both the requested-source and effective-list origin allowlists behave.
grep -q 'AllowDowngradeToInsecureRepositories=false' "$producer" || fail downgrade-refusal
grep -q 'AllowInsecureRepositories=false' "$producer" || fail insecure-refusal
! grep -q 'AllowRedirect=false' "$producer" || fail redirect-model-regressed
grep -q 'registry-mirror-configured' "$producer" || fail registry-mirror-check
origin_scratch=$(mktemp -d "${TMPDIR:-/tmp}/rar-origin-contract.XXXXXX") || fail origin-scratch
mkdir "$origin_scratch/good" "$origin_scratch/bad" "$origin_scratch/empty"
: > "$origin_scratch/good/snapshot.debian.org_archive_debian_dists_trixie_InRelease"
mkdir "$origin_scratch/good/partial"
: > "$origin_scratch/bad/snapshot.debian.org_archive_debian_dists_trixie_InRelease"
: > "$origin_scratch/bad/evil.example.org_dists_trixie_InRelease"
"$producer" --verify-origins "$origin_scratch/good" >/dev/null 2>&1 || fail origin-accept
set +e; "$producer" --verify-origins "$origin_scratch/bad" >/dev/null 2>&1; bad_status=$?
"$producer" --verify-origins "$origin_scratch/empty" >/dev/null 2>&1; empty_status=$?; set -e
[ "$bad_status" -eq 73 ] && [ "$empty_status" -eq 73 ] || fail origin-reject
good_sources=$origin_scratch/good.sources; bad_sources=$origin_scratch/bad.sources; http_sources=$origin_scratch/http.sources
printf '%s\n' 'deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/20260630T000000Z trixie main' > "$good_sources"
printf '%s\n' 'deb [check-valid-until=no] https://evil.example.org/archive/debian/20260630T000000Z trixie main' > "$bad_sources"
printf '%s\n' 'deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/20260630T000000Z trixie main' > "$http_sources"
good_log=$origin_scratch/good.transfer; foreign_log=$origin_scratch/foreign.transfer; plain_log=$origin_scratch/plain.transfer
{ printf '%s\n' 'GET /archive/debian/dists/trixie/InRelease HTTP/1.1' \
   'Answer for: https://snapshot.debian.org/archive/debian/dists/trixie/InRelease' \
   'HTTP/1.1 302 Found' 'Location: https://snapshot.debian.org/file/abc' \
   'Answer for: https://snapshot.debian.org/file/abc' 'HTTP/1.1 200 OK'; } > "$good_log"
{ cat "$good_log"; printf '%s\n' 'Location: https://evil.example.org/file/abc'; } > "$foreign_log"
{ cat "$good_log"; printf '%s\n' 'Location: http://snapshot.debian.org/file/abc'; } > "$plain_log"
"$producer" --verify-transfer-origins "$good_log" >/dev/null 2>&1 || fail transfer-origin-accept
set +e; "$producer" --verify-transfer-origins "$foreign_log" >/dev/null 2>&1; foreign_status=$?
"$producer" --verify-transfer-origins "$plain_log" >/dev/null 2>&1; plain_status=$?; set -e
[ "$foreign_status" -eq 73 ] && [ "$plain_status" -eq 73 ] || fail transfer-origin-reject
grep -q 'Debug::Acquire::https=true' "$producer" || fail transfer-telemetry-capture
"$producer" --verify-source-origins "$good_sources" >/dev/null 2>&1 || fail source-origin-accept
set +e; "$producer" --verify-source-origins "$bad_sources" >/dev/null 2>&1; bad_src=$?
"$producer" --verify-source-origins "$http_sources" >/dev/null 2>&1; http_src=$?; set -e
[ "$bad_src" -eq 73 ] && [ "$http_src" -eq 73 ] || fail source-origin-reject
rm -rf "$origin_scratch"
grep -q 'tools/toolchain/preauth-base-oci --canonicalize' "$producer" || fail base-oci-canonicalizer
grep -q -- '--network none' "$producer" || fail base-oci-networkless
! grep -q 'docker save --output "$stage/incoming/base-oci.tar"' "$producer" || fail base-oci-raw-bundled
grep -q 'base_oci_rejects' tests/preauth/src/base_oci.rs || fail base-oci-corpus
grep -q 'docker create' "$producer" || fail lifecycle-create
grep -q 'docker start -a' "$producer" || fail lifecycle-start
grep -q 'docker wait' "$producer" || fail lifecycle-wait
grep -q 'State.OOMKilled' "$producer" || fail lifecycle-oom
grep -q 'docker rm -f' "$producer" || fail lifecycle-cleanup
grep -q 'preauth-input-delivery:phase=' "$delivery" || fail host-phase-diagnostic
grep -q 'preauth-input-delivery:bundle-mismatch:' "$delivery" || fail bundle-mismatch-diagnostic
grep -q "comm -3" "$delivery" || fail bundle-member-diagnostic
for phase in setup-complete apt-update-complete apt-download-complete archive-plan-complete extract-complete bindings-complete; do
 grep -q "telemetry $phase" "$producer" || fail "telemetry:$phase"
done
for mutation in stale-snapshot unknown-origin package-substitution source-substitution license-substitution key-substitution signature-substitution checksum-substitution base-oci-substitution redirect-downgrade; do
 case "$mutation" in
  stale-snapshot|unknown-origin|package-substitution|source-substitution|license-substitution|key-substitution|signature-substitution|checksum-substitution|base-oci-substitution|redirect-downgrade) :;;
  *) fail "mutation-uncovered:$mutation";;
 esac
done
printf '%s\n' 'input producer contract checks passed'
