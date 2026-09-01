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
[ "$(/usr/bin/grep -Fxc '        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1' "$workflow")" -eq 1 ] || fail 'checkout action pin changed'
[ "$(/usr/bin/grep -Fxc '        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2' "$workflow")" -eq 1 ] || fail 'upload action pin changed'
[ "$(/usr/bin/grep -Fxc '          persist-credentials: false' "$workflow")" -eq 1 ] || fail 'checkout credentials persist'
[ "$(/usr/bin/grep -Fxc '          actual_sha=$(git rev-parse HEAD)' "$workflow")" -eq 1 ] || fail 'exact checkout verification missing'
[ "$(/usr/bin/grep -Fxc '          [[ "$actual_sha" == "$GITHUB_SHA" ]]' "$workflow")" -eq 1 ] || fail 'checkout identity is not exact'
[ "$(/usr/bin/grep -Fxc '          retention-days: 14' "$workflow")" -eq 1 ] || fail 'retention changed'
[ "$(/usr/bin/grep -Fxc '          overwrite: false' "$workflow")" -eq 1 ] || fail 'artifact overwrite enabled'
! /usr/bin/grep -B2 -F 'uses: actions/upload-artifact@' "$workflow" | /usr/bin/grep -Fq 'if: always()' || fail 'artifact upload bypasses validation'
for required in     '--read-only' '--network none' '--user 65532:65532' '--cpus 1' '--memory 512m'     '--memory-swap 512m' '--pids-limit 64' '--security-opt no-new-privileges'     '--cap-drop ALL' '--tmpfs /tmp:rw,noexec,nosuid,nodev,size=64m'     '--tmpfs /evidence:rw,noexec,nosuid,nodev,size=4m'     'target=/workspace,readonly' '/usr/bin/docker create' '/usr/bin/docker start --attach'     '/usr/bin/docker inspect' '/usr/bin/docker cp' '/usr/bin/docker rm --force' '/usr/bin/env -i'; do
    /usr/bin/grep -Fq -- "$required" "$wrapper" || fail "wrapper boundary missing: $required"
done
! /usr/bin/grep -Eq -- '--privileged|--cap-add|--network[ =](host|bridge)|docker exec|/var/run/docker.sock|--env([ =]|$)' "$wrapper" || fail 'wrapper gains forbidden container or inherited-environment authority'
[ "$(/usr/bin/grep -Fc '/usr/bin/dash "$subject"' "$harness")" -eq 1 ] || fail 'production observer count changed'
/usr/bin/grep -Fq 'generate_subject() {' "$harness" || fail 'bound generated-subject harness missing'
/usr/bin/grep -Fq 'done < "$subject" > "$case_subject"' "$harness" || fail 'generated subject is not derived from production source'
/usr/bin/grep -Fq 'case_base=/tmp/controller-helper-closure-observer-cases' "$harness" || fail 'bounded case root missing'
/usr/bin/grep -Fq 'unexpected inherited descriptor' "$observer" || fail 'inherited descriptor rejection missing'
/usr/bin/grep -Fq 'closure contains an aliased regular file' "$observer" || fail 'alias rejection missing'
/usr/bin/grep -Fq 'closure path set changed during observation' "$observer" || fail 'phase mutation rejection missing'
[ "$(/usr/bin/grep -Fxc 'case_count=21' "$catalog")" -eq 1 ] || fail 'runtime catalog count changed'
[ "$(/usr/bin/grep -Ec '^O[0-9][0-9][0-9]\|' "$catalog")" -eq 21 ] || fail 'runtime catalog incomplete'
/usr/bin/grep -Fq "'case_count=21'" "$harness" || fail 'case evidence count changed'
[ "$(/usr/bin/grep -Fxc 'effect_rule=never-touch-real-toolchain,checkout,retained-evidence,lock,inventory,profile,gate,readiness' "$fixture")" -eq 1 ] || fail 'fixture effect boundary changed'
[ "$(/usr/bin/wc -l < "$receipt" | /usr/bin/tr -d ' ')" -eq 23 ] || fail 'receipt shape line count changed'
for denial in helper_compiled=false helper_executed=false target_compiled=false readiness=false; do
    /usr/bin/grep -Fqx "$denial" "$receipt" || fail "receipt denial missing: $denial"
done
! /usr/bin/grep -Eq '^[[:space:]]*(/usr/bin/|/bin/)?(cargo|rustc|ld|qemu|curl|wget|nc|git|gh)([[:space:]]|$)' "$wrapper" "$harness" || fail 'forbidden runtime command present'
! /usr/bin/grep -Eq '\$(\{|)(GITHUB_TOKEN|ACTIONS_RUNTIME_TOKEN|PASSWORD|SECRET|CREDENTIAL)' "$wrapper" "$harness" || fail 'credential value access present'
for artifact in \
    controller-helper-closure-observer.cases.v0 \
    controller-helper-closure.sha256 \
    controller-helper-closure.receipt \
    controller-helper-closure-observer-run-evidence.v0; do
    [ "$(/usr/bin/grep -Fxc "            ${{ runner.temp }}/controller-helper-closure-observer/$artifact" "$workflow")" -eq 1 ] || fail "artifact path changed: $artifact"
done
/usr/bin/grep -Fq 'Independently validate exact candidate evidence' "$workflow" || fail 'independent validation missing'
/usr/bin/grep -Fq 'Retain four validated candidate files' "$workflow" || fail 'validated retention missing'
printf '%s\n' 'controller-helper observer policy passed: main-only isolated candidate'
