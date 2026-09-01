#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

manifest=tools/ci/policy-test-modes.v0
runner=tools/ci/run-ephemeral-policy-tests.sh
workflow=.github/workflows/specifications.yml
[ -f "$manifest" ] && [ ! -L "$manifest" ] && [ -s "$manifest" ] || exit 1
[ "$(/usr/bin/sed -n '1p' "$manifest")" = schema=rar-policy-test-modes-v0 ] || exit 1
[ "$(/usr/bin/sed -n '2p' "$manifest")" = 'path|mode' ] || exit 1
/usr/bin/awk -F '|' '
    NR <= 2 { next }
    NF != 2 || $1 !~ /^tools\/ci\/test-[a-z0-9.-]+\.sh$/ || $2 !~ /^(immutable|ephemeral)$/ || ++seen[$1] != 1 { bad=1 }
    END { if (NR != 33 || bad) exit 1 }
' "$manifest"

declared=$(/usr/bin/sed -n '3,$s/|.*//p' "$manifest" | /usr/bin/sort)
actual=$(/usr/bin/printf '%s\n' tools/ci/test-*.sh | /usr/bin/sort)
[ "$declared" = "$actual" ] || exit 1
ephemeral=$(/usr/bin/awk -F '|' '$2 == "ephemeral" { print $1 }' "$manifest")
immutable=$(/usr/bin/awk -F '|' '$2 == "immutable" { print $1 }' "$manifest")
[ "$(/usr/bin/printf '%s\n' "$ephemeral" | /usr/bin/awk 'NF { count++ } END { print count + 0 }')" -eq 26 ] || exit 1
[ "$(/usr/bin/printf '%s\n' "$immutable" | /usr/bin/awk 'NF { count++ } END { print count + 0 }')" -eq 5 ] || exit 1

printf '%s\n' "$ephemeral" | while IFS= read -r test; do
    [ -f "$test" ] && [ ! -L "$test" ] && [ -s "$test" ] || exit 1
    [ "$(/usr/bin/grep -Fc 'scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")' "$test")" -eq 1 ] || exit 1
    [ "$(/usr/bin/grep -Ec '^\[ "\$scratch" != disabled \] \|\| \{ .* exit 0; \}$' "$test")" -eq 1 ] || exit 1
    [ "$(/usr/bin/grep -Ec '^work=\$\(mktemp -d "\$scratch/[a-z0-9.-]+\.XXXXXX"\)$' "$test")" -eq 1 ] || exit 1
    [ "$(/usr/bin/grep -Ec '^scratch=' "$test")" -eq 1 ] || exit 1
    [ "$(/usr/bin/grep -Ec '^work=' "$test")" -eq 1 ] || exit 1
    ! /usr/bin/grep -Eq '\$root/out|output_root=|output=\$root/out' "$test" || exit 1
    ! /usr/bin/sed 's|/dev/null||g' "$test" | /usr/bin/grep -Eq '/(dev|proc|sys|run)/' || exit 1
    ! /usr/bin/grep -Eq "^(/bin/sh )?$test([[:space:]]|$)" tools/ci/check-sprint-static.sh || exit 1
    ! /usr/bin/grep -Eq "^[[:space:]]*/bin/sh[[:space:]]+$test([[:space:]]|$)" tools/ci/check-specs.sh || exit 1
done

local_immutable=$(/usr/bin/awk '
    NF == 2 && $1 == "/bin/sh" && $2 ~ /^tools\/ci\/test-[a-z0-9.-]+\.sh$/ { print $2; next }
    NF == 1 && $1 ~ /^tools\/ci\/test-[a-z0-9.-]+\.sh$/ { print $1 }
' tools/ci/check-sprint-static.sh | /usr/bin/sort)
[ "$local_immutable" = "$(/usr/bin/printf '%s\n' "$immutable" | /usr/bin/sort)" ] || exit 1

printf '%s\n' "$immutable" | while IFS= read -r test; do
    ! /usr/bin/grep -Fq 'require-ephemeral-policy-test-root.sh' "$test" || exit 1
    ! /usr/bin/grep -Eq 'mktemp|trap .*rm|/bin/(rm|mkdir|ln|mv|cp)|(^|[[:space:]])(touch|truncate|install|chmod|dd)[[:space:]]' "$test" || exit 1
done

runner_tests=$(/usr/bin/sed -n 's|^/bin/sh "$root/\(tools/ci/test-[a-z0-9.-]*\.sh\)"$|\1|p' "$runner")
[ "$runner_tests" = "$ephemeral" ] || exit 1
[ "$(/usr/bin/grep -Fxc "printf '%s\\n' 'Ephemeral policy tests passed: executed=26 source=read-only scratch=tmpfs'" "$runner")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fxc 'ulimit -f 131072' "$runner")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fxc '          path: primary-source' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fxc '          path: mutation-source' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--mount "type=bind,source=$GITHUB_WORKSPACE/primary-source,target=/workspace" \' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--mount "type=bind,source=$GITHUB_WORKSPACE/mutation-source,target=/workspace,readonly" \' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--env RAR_POLICY_MUTATION_TESTS=1 \' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--env RAR_QMP_SOURCE_TESTS=1 \' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--env RAR_EXPECTED_SOURCE_REVISION \' "$workflow")" -eq 2 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--tmpfs "/tmp:rw,nosuid,nodev,size=128m,uid=$host_uid,gid=$host_gid,mode=1777" \' "$workflow")" -eq 2 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--tmpfs "/build:rw,exec,nosuid,nodev,size=32m,uid=$host_uid,gid=$host_gid,mode=700" \' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc -- '--tmpfs "/evidence:rw,nosuid,nodev,noexec,size=64m,uid=$host_uid,gid=$host_gid,mode=700" \' "$workflow")" -eq 1 ] || exit 1
/usr/bin/awk '
    /^  validate:$/ {
        if (in_validate) bad=1
        in_validate=1
        validate_jobs++
        next
    }
    in_validate && /^  [A-Za-z0-9_-]+:$/ { in_validate=0 }
    in_validate && /^    timeout-minutes:/ {
        validate_timeouts++
        if ($0 == "    timeout-minutes: 30") approved_timeout++
        else bad=1
    }
    END {
        if (bad || validate_jobs != 1 || validate_timeouts != 1 || approved_timeout != 1) exit 1
    }
' "$workflow" || exit 1
/usr/bin/awk '
    BEGIN { approved = "            --cpus 2 --memory 2048m --memory-swap 2048m --pids-limit 256 \\" }
    function has_resource_option(line) {
        return line ~ /(^|[[:space:]])(--cpus([=[:space:]])|--memory([=[:space:]])|--memory-swap([=[:space:]])|--pids-limit([=[:space:]])|-m([=[:space:]]))/
    }
    /^[[:space:]]*docker run / {
        if (in_docker) bad=1
        in_docker=1
        vectors=0
        docker_runs++
    }
    has_resource_option($0) {
        if (in_docker && $0 == approved) vectors++
        else bad=1
    }
    in_docker && $0 !~ /\\[[:space:]]*$/ {
        if (vectors != 1) bad=1
        in_docker=0
    }
    END { if (bad || in_docker || docker_runs != 2) exit 1 }
' "$workflow" || exit 1
[ "$(/usr/bin/grep -Fxc -- '              tools/ci/run-ephemeral-policy-tests.sh' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fxc -- '              tools/ci/run-qmp-client-unit-tests.sh' "$workflow")" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc 'tools/ci/run-qmp-client-unit-tests.sh' tools/ci/check-sprint-static.sh)" -eq 1 ] || exit 1
[ "$(/usr/bin/grep -Fc 'tools/ci/run-qmp-client-unit-tests.sh' tools/ci/check-specs.sh)" -eq 2 ] || exit 1

printf '%s\n' 'Ephemeral policy-test confinement passed: ephemeral=26 immutable=5 source=read-only'
