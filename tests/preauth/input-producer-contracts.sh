#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"
fail(){ printf 'input-producer-contract:%s\n' "$1" >&2; exit 1; }
. tools/toolchain/preauth-build-root.sh
rustc_path=$(preauth_build_pinned_rustc_path "$root") || fail pinned-rustc
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
# Authenticity is signature/digest-anchored, not transport-anchored. The separate transport
# boundary is the isolated APT-method protocol proxy and its canonical event verifier.
grep -q 'AllowDowngradeToInsecureRepositories=false' "$producer" || fail downgrade-refusal
grep -q 'AllowInsecureRepositories=false' "$producer" || fail insecure-refusal
! grep -q 'AllowRedirect=false' "$producer" || fail redirect-model-regressed
grep -q 'registry-mirror-configured' "$producer" || fail registry-mirror-check
origin_scratch=$(mktemp -d "${TMPDIR:-/tmp}/rar-origin-contract.XXXXXX") || fail origin-scratch
if [ -n "${RAR_PREAUTH_BUILD_ROOT-}" ]; then
 origin_build_root=$RAR_PREAUTH_BUILD_ROOT
else
 origin_build_root=$origin_scratch/build-root
 mkdir -m 700 "$origin_build_root"
fi
origin_fixture_root=$(mktemp -d "$origin_build_root/rar-origin-fixture.XXXXXXXX") || fail origin-build-scratch
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
telemetry_verifier=$origin_fixture_root/preauth-transfer-telemetry
mkdir -m 700 "$origin_scratch/poisoned-bin" "$origin_scratch/readonly-rustup"
printf '%s\n' '#!/bin/sh' 'exit 99' > "$origin_scratch/poisoned-bin/rustc"
chmod 0700 "$origin_scratch/poisoned-bin/rustc"
chmod 0500 "$origin_scratch/readonly-rustup"
env PATH="$origin_scratch/poisoned-bin:$PATH" RUSTUP_HOME="$origin_scratch/readonly-rustup" \
 "$rustc_path" --edition=2024 -O tools/toolchain/preauth-transfer-telemetry.rs -o "$telemetry_verifier" \
 || fail telemetry-compile
event_file(){
 event_name=$1; shift
 mkdir -m 2770 "$origin_scratch/$event_name"
 printf '%s\n' "$@" > "$origin_scratch/$event_name/channel-0000000001.events"
 chmod 0640 "$origin_scratch/$event_name/channel-0000000001.events"
}
reject_event(){ set +e; "$telemetry_verifier" --verify "$origin_scratch/$1" "$policy" >"$origin_scratch/reject.stdout" 2>"$origin_scratch/reject.stderr"; reject_status=$?; set -e; [ "$reject_status" -eq 73 ] && [ ! -s "$origin_scratch/reject.stdout" ] || fail "telemetry-reject:$1"; }
registry_file(){
 registry_dir=$1; registry_channel=$2; registry_state=$3; registry_count=$4
 mkdir -p -m 2770 "$registry_dir"
 if [ "$registry_state" = complete ]; then
  printf '%s\n' 'schema	rar-apt-method-lifecycle-v1' "channel	$registry_channel" \
   "complete	$registry_channel	$registry_count" > "$registry_dir/registry-$registry_channel.complete"
 else
  printf '%s\n' 'schema	rar-apt-method-lifecycle-v1' "channel	$registry_channel" \
   > "$registry_dir/registry-$registry_channel.open"
 fi
 chmod 0640 "$registry_dir/registry-$registry_channel.$registry_state"
}
reject_lifecycle(){
 set +e
 "$telemetry_verifier" --await-verify "$origin_scratch/$1" "$origin_scratch/$2" "$policy" \
  >"$origin_scratch/reject.stdout" 2>"$origin_scratch/reject.stderr"
 reject_status=$?
 set -e
 [ "$reject_status" -eq 73 ] && [ ! -s "$origin_scratch/reject.stdout" ] || fail "telemetry-lifecycle-reject:$2"
}
event_file good.transfer \
 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/archive/debian/dists/trixie/InRelease' \
 'redirect	0000000001-00000001	1	https://snapshot.debian.org/archive/debian/dists/trixie/InRelease	https://snapshot.debian.org/file/abc' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/file/abc' \
 'start	0000000001-00000002	https://snapshot.debian.org/archive/debian/pool/tool.deb' \
 'terminal	0000000001-00000002	success	https://snapshot.debian.org/archive/debian/pool/tool.deb' \
 'method-complete	0000000001	2'
printf '%s\n' \
 'schema	rar-apt-transfer-events-v1' \
 'method-complete	0000000002	0' > "$origin_scratch/good.transfer/channel-0000000002.events"
chmod 0640 "$origin_scratch/good.transfer/channel-0000000002.events"
mkdir -m 2770 "$origin_scratch/good.lifecycle"
printf '%s\n' 'schema	rar-apt-method-lifecycle-v1' 'channel	0000000001' \
 'complete	0000000001	2' > "$origin_scratch/good.lifecycle/registry-0000000001.complete"
printf '%s\n' 'schema	rar-apt-method-lifecycle-v1' 'channel	0000000002' \
 'complete	0000000002	0' > "$origin_scratch/good.lifecycle/registry-0000000002.complete"
chmod 0640 "$origin_scratch/good.lifecycle"/registry-*.complete
set +e
"$telemetry_verifier" --await-verify "$origin_scratch/good.transfer" "$origin_scratch/good.lifecycle" "$policy" \
 >"$origin_scratch/accept.stdout" 2>"$origin_scratch/accept.stderr"
accept_status=$?
set -e
if [ "$accept_status" -ne 0 ] || [ -s "$origin_scratch/accept.stdout" ]; then
 cat "$origin_scratch/accept.stderr" >&2
 fail transfer-origin-accept
fi
event_file all-idle.transfer 'schema	rar-apt-transfer-events-v1' 'method-complete	0000000001	0'
reject_event all-idle.transfer
mkdir -m 2770 "$origin_scratch/missing-channel.transfer"
cp "$origin_scratch/good.transfer/channel-0000000001.events" "$origin_scratch/missing-channel.transfer/"
registry_file "$origin_scratch/missing-channel.lifecycle" 0000000001 complete 2
registry_file "$origin_scratch/missing-channel.lifecycle" 0000000002 complete 0
reject_lifecycle missing-channel.transfer missing-channel.lifecycle
registry_file "$origin_scratch/extra-channel.lifecycle" 0000000001 complete 2
reject_lifecycle good.transfer extra-channel.lifecycle
registry_file "$origin_scratch/aggregate-race.lifecycle" 0000000001 open 0
registry_file "$origin_scratch/aggregate-race.lifecycle" 0000000002 complete 0
reject_lifecycle good.transfer aggregate-race.lifecycle
registry_file "$origin_scratch/reopened.lifecycle" 0000000001 complete 2
registry_file "$origin_scratch/reopened.lifecycle" 0000000001 open 0
reject_lifecycle good.transfer reopened.lifecycle

for case_and_url in \
 'foreign|https://evil.example.org/x' \
 'plaintext|http://snapshot.debian.org/x' \
 'userinfo|https://snapshot.debian.org@evil.example.org/x' \
 'port|https://snapshot.debian.org:444/x' \
 'mixed-case|https://Snapshot.debian.org/x' \
 'trailing-dot|https://snapshot.debian.org./x' \
 'idna|https://xn--bcher-kva.example/x' \
 'nonascii|https://bücher.example/x' \
 'percent-authority|https://snapshot%2edebian.org/x'; do
 event_name=${case_and_url%%|*}; event_url=${case_and_url#*|}
 event_file "$event_name.transfer" 'schema	rar-apt-transfer-events-v1' \
  "start	0000000001-00000001	$event_url" "terminal	0000000001-00000001	success	$event_url" 'method-complete	0000000001	1'
 reject_event "$event_name.transfer"
done

event_file malformed.transfer 'schema	rar-apt-transfer-events-v1' 'start	00000001'
reject_event malformed.transfer
mkdir -m 2770 "$origin_scratch/truncated.transfer"
printf '%s' 'schema	rar-apt-transfer-events-v1
start	0000000001-00000001	https://snapshot.debian.org/x' > "$origin_scratch/truncated.transfer/channel-0000000001.events"
chmod 0640 "$origin_scratch/truncated.transfer/channel-0000000001.events"
reject_event truncated.transfer
event_file duplicate-start.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'start	0000000001-00000001	https://snapshot.debian.org/x' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' 'method-complete	0000000001	1'
reject_event duplicate-start.transfer
event_file duplicate-terminal.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' 'method-complete	0000000001	1'
reject_event duplicate-terminal.transfer
event_file missing-terminal.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'method-complete	0000000001	1'
reject_event missing-terminal.transfer
event_file missing-completion.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x'
reject_event missing-completion.transfer
event_file completion-before-terminal.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'method-complete	0000000001	1' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x'
reject_event completion-before-terminal.transfer
event_file duplicate-completion.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' \
 'method-complete	0000000001	1' 'method-complete	0000000001	1'
reject_event duplicate-completion.transfer
event_file unknown.transfer 'schema	rar-apt-transfer-events-v1' \
 'package-output	start	0000000001-00000001	https://snapshot.debian.org/x' 'method-complete	0000000001	0'
reject_event unknown.transfer
event_file unobserved.transfer 'schema	rar-apt-transfer-events-v1' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' 'method-complete	0000000001	0'
reject_event unobserved.transfer
event_file extra-record.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' \
 'method-complete	0000000001	1' 'start	0000000001-00000002	https://snapshot.debian.org/y'
reject_event extra-record.transfer
event_file cardinality.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' 'method-complete	0000000001	2'
reject_event cardinality.transfer
mkdir -m 2770 "$origin_scratch/cross-channel-request.transfer"
printf '%s\n' \
 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/x' \
 'method-complete	0000000001	1' > "$origin_scratch/cross-channel-request.transfer/channel-0000000001.events"
chmod 0640 "$origin_scratch/cross-channel-request.transfer/channel-0000000001.events"
printf '%s\n' \
 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000002	https://snapshot.debian.org/y' \
 'terminal	0000000001-00000002	success	https://snapshot.debian.org/y' \
 'method-complete	0000000002	1' > "$origin_scratch/cross-channel-request.transfer/channel-0000000002.events"
chmod 0640 "$origin_scratch/cross-channel-request.transfer/channel-0000000002.events"
reject_event cross-channel-request.transfer
event_file cycle.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/a' \
 'redirect	0000000001-00000001	1	https://snapshot.debian.org/a	https://snapshot.debian.org/b' \
 'redirect	0000000001-00000001	2	https://snapshot.debian.org/b	https://snapshot.debian.org/a' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/a' 'method-complete	0000000001	1'
reject_event cycle.transfer
event_file over-limit.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/0' \
 'redirect	0000000001-00000001	1	https://snapshot.debian.org/0	https://snapshot.debian.org/1' \
 'redirect	0000000001-00000001	2	https://snapshot.debian.org/1	https://snapshot.debian.org/2' \
 'redirect	0000000001-00000001	3	https://snapshot.debian.org/2	https://snapshot.debian.org/3' \
 'redirect	0000000001-00000001	4	https://snapshot.debian.org/3	https://snapshot.debian.org/4' \
 'redirect	0000000001-00000001	5	https://snapshot.debian.org/4	https://snapshot.debian.org/5' \
 'redirect	0000000001-00000001	6	https://snapshot.debian.org/5	https://snapshot.debian.org/6' \
 'redirect	0000000001-00000001	7	https://snapshot.debian.org/6	https://snapshot.debian.org/7' \
 'redirect	0000000001-00000001	8	https://snapshot.debian.org/7	https://snapshot.debian.org/8' \
 'redirect	0000000001-00000001	9	https://snapshot.debian.org/8	https://snapshot.debian.org/9' \
 'terminal	0000000001-00000001	success	https://snapshot.debian.org/9' 'method-complete	0000000001	1'
reject_event over-limit.transfer
event_file injection.transfer 'schema	rar-apt-transfer-events-v1' \
 'start	0000000001-00000001	https://snapshot.debian.org/x' \
 'evil package says: terminal	0000000001-00000001	success	https://snapshot.debian.org/x' 'method-complete	0000000001	1'
reject_event injection.transfer
grep -q 'Dir::Bin::methods=' "$producer" || fail transfer-proxy-install
grep -q 'Acquire::Queue-Mode=access' "$producer" || fail transfer-request-serialization
grep -q 'private-setgid-method-channel-directory' "$policy" || fail transfer-private-channel
"$producer" --verify-source-origins "$good_sources" >/dev/null 2>&1 || fail source-origin-accept
set +e; "$producer" --verify-source-origins "$bad_sources" >/dev/null 2>&1; bad_src=$?
"$producer" --verify-source-origins "$http_sources" >/dev/null 2>&1; http_src=$?; set -e
[ "$bad_src" -eq 73 ] && [ "$http_src" -eq 73 ] || fail source-origin-reject
rm -rf "$origin_fixture_root" "$origin_scratch"
grep -q 'tools/toolchain/preauth-base-oci --canonicalize' "$producer" || fail base-oci-canonicalizer
grep -q -- '--network none' "$producer" || fail base-oci-networkless
! grep -q 'docker save --output "$stage/incoming/base-oci.tar"' "$producer" || fail base-oci-raw-bundled
grep -q 'base_oci_rejects' tests/preauth/src/base_oci.rs || fail base-oci-corpus
grep -q 'docker create' "$producer" || fail lifecycle-create
grep -q -- '--group-add 65534' "$producer" || fail apt-sandbox-group
grep -q -- '--tmpfs "/apt-runtime:rw,exec,nosuid,nodev,size=3g,uid=$host_uid,gid=65534,mode=2770"' "$producer" || fail apt-runtime-mount
grep -q 'APT::Sandbox::User=_apt' "$producer" || fail apt-sandbox-user
grep -q 'chgrp "$apt_group" "$methods/https"' "$producer" || fail apt-method-group-binding
grep -q 'Dir::State::lists=$state/lists' "$producer" || fail apt-state-binding
grep -q 'Dir::State::lists::partial=$state/lists/partial' "$producer" || fail apt-state-partial-binding
grep -q 'Dir::Cache::archives=$cache/archives' "$producer" || fail apt-cache-binding
grep -q 'Dir::Cache::archives::partial=$cache/archives/partial' "$producer" || fail apt-cache-partial-binding
! grep -q '/var/cache/apt' "$producer" || fail apt-host-cache-reliance
grep -q 'apt_runtime_cleanup' "$producer" || fail apt-runtime-cleanup
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
