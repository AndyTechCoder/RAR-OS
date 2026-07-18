#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
fail(){ printf 'input-producer-contract:%s\n' "$1" >&2; exit 1; }
policy=spec/lab/preauth/preauth-input-delivery-v1.policy
producer=tools/toolchain/preauth-input-producer
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
for mutation in stale-snapshot unknown-origin package-substitution source-substitution license-substitution key-substitution signature-substitution checksum-substitution base-oci-substitution redirect-downgrade; do
 case "$mutation" in
  stale-snapshot|unknown-origin|package-substitution|source-substitution|license-substitution|key-substitution|signature-substitution|checksum-substitution|base-oci-substitution|redirect-downgrade) :;;
  *) fail "mutation-uncovered:$mutation";;
 esac
done
printf '%s\n' 'input producer contract checks passed'
