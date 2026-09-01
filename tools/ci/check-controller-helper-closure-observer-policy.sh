#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=${1-}
[ -n "$root" ] || root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
fail() { printf 'controller-helper observer policy failed: %s\n' "$1" >&2; exit 1; }
workflow=$root/.github/workflows/controller-helper-closure-observer.yml
wrapper=$root/tools/ci/run-controller-helper-closure-observer.sh
harness=$root/tools/ci/controller-helper-closure-observer-harness.sh
observer=$root/tools/ci/observe-controller-helper-closure.sh
catalog=$root/tools/ci/fixtures/controller-helper-closure-observer/cases.v0
fixture=$root/tools/ci/fixtures/controller-helper-closure-observer/base-closure.v0
pins=$root/tools/ci/fixtures/controller-helper-closure-observer/tool-pins.v0
receipt=$root/tools/ci/fixtures/controller-helper-closure-observer/expected-observation.receipt.v0
for file in "$workflow" "$wrapper" "$harness" "$observer" "$catalog" "$fixture" "$pins" "$receipt"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "required file missing: $file"
done
/bin/sh -n "$wrapper" "$harness" "$observer" || fail 'shell syntax invalid'
for forbidden in workflow_dispatch repository_dispatch pull_request pull_request_target schedule workflow_call workflow_run; do
    ! /usr/bin/grep -Eq "^[[:space:]]*$forbidden:" "$workflow" || fail "forbidden trigger: $forbidden"
done
[ "$(/usr/bin/grep -Fxc '  push:' "$workflow")" -eq 1 ] || fail 'push trigger not exact'
[ "$(/usr/bin/grep -Fxc '      - main' "$workflow")" -eq 1 ] || fail 'main branch not exact'
[ "$(/usr/bin/grep -Fxc '  contents: read' "$workflow")" -eq 1 ] || fail 'permissions not read-only'
[ "$(/usr/bin/grep -Fxc '    runs-on: ubuntu-24.04' "$workflow")" -eq 1 ] || fail 'runner not pinned'
[ "$(/usr/bin/grep -Fc 'actions/checkout@' "$workflow")" -eq 0 ] || fail 'checkout action receives an implicit token'
[ "$(/usr/bin/grep -Fxc '        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2' "$workflow")" -eq 1 ] || fail 'upload action pin changed'
[ "$(/usr/bin/grep -Fxc '      - name: Acquire exact main into bounded storage' "$workflow")" -eq 1 ] || fail 'bounded anonymous acquisition step missing'
for required in \
    'checkout_image=rust:1.95.0@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3' \
    'identity="rar-c2b-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"' \
    'partial="$RUNNER_TEMP/controller-helper-closure-checkout-partial-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"' \
    '/usr/bin/docker volume create --driver local' '--opt type=tmpfs --opt device=tmpfs' \
    'o=size=67108864,uid=$runner_uid,gid=$runner_gid,mode=0700,noexec,nosuid,nodev' \
    '/usr/bin/docker create --name "$acquisition" --read-only --network bridge' \
    '--mount "type=volume,source=$volume,target=/checkout"' \
    'GIT_CONFIG_NOSYSTEM=1' 'GIT_TERMINAL_PROMPT=0 GIT_ASKPASS=/bin/false' \
    '/usr/bin/git init /checkout' \
    '/usr/bin/git -C /checkout remote add origin https://github.com/AndyTechCoder/RAR-OS.git' \
    '-c filter.lfs.smudge= -c filter.lfs.required=false' \
    'fetch --no-tags --depth=1 origin "$1"' \
    'checkout --detach FETCH_HEAD' \
    '/usr/bin/timeout --signal=TERM --kill-after=10s 120s /usr/bin/docker start --attach "$acquisition"' \
    '/usr/bin/timeout --kill-after=2s 10s /usr/bin/docker rm --force "$acquisition"' \
    '/usr/bin/docker create --name "$transfer" --read-only --network none' \
    '--mount "type=volume,source=$volume,target=/source,readonly"' \
    '--mount "type=bind,source=$partial,target=/destination"' \
    '/usr/bin/cp -a /source/. /destination/' \
    '/usr/bin/timeout --signal=TERM --kill-after=10s 30s /usr/bin/docker start --attach "$transfer"' \
    '/usr/bin/timeout --kill-after=2s 10s /usr/bin/docker volume rm "$volume"' \
    '/usr/bin/chmod -R a-w "$partial"' \
    '/usr/bin/rmdir "$GITHUB_WORKSPACE"' \
    '/usr/bin/mv "$partial" "$GITHUB_WORKSPACE"'; do
    /usr/bin/grep -Fq -- "$required" "$workflow" || fail "bounded acquisition boundary missing: $required"
done
[ "$(/usr/bin/grep -Fc -- '--network bridge' "$workflow")" -eq 1 ] || fail 'acquisition network authority changed'
[ "$(/usr/bin/grep -Fc -- '--network none' "$workflow")" -eq 1 ] || fail 'transfer network denial changed'
[ "$(/usr/bin/grep -Fc '[ "$bytes" -le 67108864 ] && [ "$files" -le 8192 ] && [ "$objects" -le 32768 ]' "$workflow")" -eq 2 ] || fail 'checkout ceilings changed'
[ "$(/usr/bin/grep -Fc '[ -z "$(/usr/bin/git -C /' "$workflow")" -ge 4 ] || fail 'checkout cleanliness and ref checks missing'
/usr/bin/grep -Fq '"$RUNNER_TEMP"/controller-helper-closure-checkout-partial-"$GITHUB_RUN_ID"-"$GITHUB_RUN_ATTEMPT") /usr/bin/rm -rf -- "$partial"' "$workflow" || fail 'partial cleanup is not identity-bound'
! /usr/bin/grep -Fq '/var/run/docker.sock' "$workflow" || fail 'checkout gains Docker socket'
! /usr/bin/grep -Eq '\$\{\{[[:space:]]*github\.token|GITHUB_TOKEN|ACTIONS_RUNTIME_TOKEN|PASSWORD|SECRET|CREDENTIAL' "$workflow" "$wrapper" "$harness" || fail 'credential value access present'
[ "$(/usr/bin/grep -Fxc '          actual_sha=$(git rev-parse HEAD)' "$workflow")" -eq 1 ] || fail 'exact checkout verification missing'
[ "$(/usr/bin/grep -Fxc '          [[ "$actual_sha" == "$GITHUB_SHA" ]]' "$workflow")" -eq 1 ] || fail 'checkout identity is not exact'
[ "$(/usr/bin/grep -Fxc '          retention-days: 14' "$workflow")" -eq 1 ] || fail 'retention changed'
[ "$(/usr/bin/grep -Fxc '          overwrite: false' "$workflow")" -eq 1 ] || fail 'artifact overwrite enabled'
! /usr/bin/grep -B2 -F 'uses: actions/upload-artifact@' "$workflow" | /usr/bin/grep -Fq 'if: always()' || fail 'artifact upload bypasses validation'
for required in \
    '--read-only' '--network none' '--user 65532:65532' '--cpus 1' '--memory 512m' \
    '--memory-swap 512m' '--pids-limit 64' '--security-opt no-new-privileges' \
    '--cap-drop ALL' '--tmpfs /tmp:rw,noexec,nosuid,nodev,size=64m' \
    '--tmpfs /evidence:rw,noexec,nosuid,nodev,size=4m' \
    'target=/workspace,readonly' '/usr/bin/docker create' '/usr/bin/docker start --attach' \
    '/usr/bin/docker inspect' '/usr/bin/docker rm --force' '/usr/bin/env -i'; do
    /usr/bin/grep -Fq -- "$required" "$wrapper" || fail "wrapper boundary missing: $required"
done
! /usr/bin/grep -Eq -- '--privileged|--cap-add|--network[ =](host|bridge)|docker exec|/var/run/docker.sock|--env([ =]|$)' "$wrapper" || fail 'wrapper gains forbidden container or inherited-environment authority'
! /usr/bin/grep -Fq 'docker cp' "$wrapper" "$harness" || fail 'stopped tmpfs evidence copy is forbidden'
[ "$(/usr/bin/grep -Fxc 'decode_stream() {' "$wrapper")" -eq 1 ] || fail 'stream decoder missing or duplicated'
[ "$(/usr/bin/grep -Fxc '/usr/bin/docker start --attach "$container" | decode_stream "$evidence"' "$wrapper")" -eq 1 ] || fail 'exact stream handoff changed'
[ "$(/usr/bin/grep -Fxc 'pipe_status=("${PIPESTATUS[@]}")' "$wrapper")" -eq 1 ] || fail 'stream status binding missing'
for required in \
    '[ "${#pipe_status[@]}" -eq 2 ]' \
    '[ "${pipe_status[0]}" -eq 0 ]' \
    '[ "${pipe_status[1]}" -eq 0 ]' \
    '[ "$case_bytes" -le 32768 ]' \
    '[ "$manifest_bytes" -le 1048576 ]' \
    '[ "$receipt_bytes" -le 4096 ]'; do
    /usr/bin/grep -Fq -- "$required" "$wrapper" || fail "stream bound missing: $required"
done
[ "$(/usr/bin/grep -Fxc '/usr/bin/dash "$subject" || fail '"'"'production observer failed'"'"'' "$harness")" -eq 1 ] || fail 'production observer count changed'
/usr/bin/grep -Fq 'generate_subject() {' "$harness" || fail 'bound generated-subject harness missing'
/usr/bin/grep -Fq 'done < "$subject" > "$case_subject"' "$harness" || fail 'generated subject is not derived from production source'
/usr/bin/grep -Fq 'case_base=/tmp/controller-helper-closure-observer-cases' "$harness" || fail 'bounded case root missing'
/usr/bin/grep -Fq 'unexpected inherited descriptor: 9' "$harness" || fail 'generated descriptor fault missing'
/usr/bin/grep -Fq "hardlinked=x" "$harness" || fail 'generated hardlink fault missing'
/usr/bin/grep -Fq 'injected phase mutation failure' "$harness" || fail 'generated phase fault missing'
/usr/bin/grep -Fq 'hardlinked=$(/usr/bin/find -P "$root" -type f -links +1 -printf x -quit)' "$observer" || fail 'production no-hardlink predicate missing'
/usr/bin/grep -Fq "[ -z \"\$hardlinked\" ] || fail 'closure contains a hardlinked regular file'" "$observer" || fail 'production hardlink rejection missing'
for marker in \
    'RAR-C2B-BEGIN:cases' 'RAR-C2B-END:cases' \
    'RAR-C2B-BEGIN:manifest' 'RAR-C2B-END:manifest' \
    'RAR-C2B-BEGIN:receipt' 'RAR-C2B-END:receipt'; do
    [ "$(/usr/bin/grep -Fc "$marker" "$wrapper")" -eq 1 ] || fail "stream marker changed: $marker"
done
[ "$(/usr/bin/grep -Fxc "    printf 'RAR-C2B-BEGIN:%s\\n' \"\$label\"" "$harness")" -eq 1 ] || fail 'stream begin emitter changed'
[ "$(/usr/bin/grep -Fxc "    printf 'RAR-C2B-END:%s\\n' \"\$label\"" "$harness")" -eq 1 ] || fail 'stream end emitter changed'
for emission in \
    'emit_file cases "$case_file"' \
    'emit_file manifest "$evidence/controller-helper-closure.sha256"' \
    'emit_file receipt "$evidence/controller-helper-closure.receipt"'; do
    [ "$(/usr/bin/grep -Fxc "$emission" "$harness")" -eq 1 ] || fail "stream emission changed: $emission"
done
/usr/bin/awk '
    /emit_file cases "\$case_file"/ { cases=NR }
    /emit_file manifest "\$evidence\/controller-helper-closure.sha256"/ { manifest=NR }
    /emit_file receipt "\$evidence\/controller-helper-closure.receipt"/ { receipt=NR }
    END { exit !(cases && manifest && receipt && cases < manifest && manifest < receipt) }
' "$harness" || fail 'stream emission order changed'
/usr/bin/awk '
    /docker start --attach/ { start=NR }
    /docker rm --force "\$container"/ { removed=NR }
    /record=\$evidence\/controller-helper-closure-observer-run-evidence.v0/ { record=NR }
    END { exit !(start && removed && record && start < removed && removed < record) }
' "$wrapper" || fail 'producer revocation order changed'
[ "$(/usr/bin/grep -Fxc 'case_count=21' "$catalog")" -eq 1 ] || fail 'runtime catalog count changed'
[ "$(/usr/bin/grep -Ec '^O[0-9][0-9][0-9]\|' "$catalog")" -eq 21 ] || fail 'runtime catalog incomplete'
/usr/bin/grep -Fq "'case_count=21'" "$harness" || fail 'case evidence count changed'
[ "$(/usr/bin/grep -Fxc 'effect_rule=never-touch-real-toolchain,checkout,retained-evidence,lock,inventory,profile,gate,readiness' "$fixture")" -eq 1 ] || fail 'fixture effect boundary changed'
[ "$(/usr/bin/wc -l < "$receipt" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'receipt shape line count changed'
for denial in helper_compiled=false helper_executed=false target_compiled=false readiness=false; do
    /usr/bin/grep -Fqx "$denial" "$receipt" || fail "receipt denial missing: $denial"
done
! /usr/bin/grep -Eq '^[[:space:]]*(/usr/bin/|/bin/)?(cargo|rustc|ld|qemu|curl|wget|nc|git|gh)([[:space:]]|$)' "$wrapper" "$harness" || fail 'forbidden runtime command present'
for artifact in \
    controller-helper-closure-observer.cases.v0 \
    controller-helper-closure.sha256 \
    controller-helper-closure.receipt \
    controller-helper-closure-observer-run-evidence.v0; do
    [ "$(/usr/bin/grep -Fxc "            \${{ runner.temp }}/controller-helper-closure-observer/$artifact" "$workflow")" -eq 1 ] || fail "artifact path changed: $artifact"
done
/usr/bin/grep -Fq 'Independently validate exact candidate evidence' "$workflow" || fail 'independent validation missing'
/usr/bin/grep -Fq 'Retain four validated candidate files' "$workflow" || fail 'validated retention missing'
printf '%s\n' 'controller-helper observer policy passed: anonymous exact-main checkout and isolated candidate'
