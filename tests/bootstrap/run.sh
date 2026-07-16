#!/bin/sh
set -eu

case "$0" in
    */*) script_directory=${0%/*} ;;
    *)
        echo "bootstrap tests require an explicit checkout path" >&2
        exit 2
        ;;
esac
[ ! -L "$0" ] || {
    echo "bootstrap test script must not be a symbolic link" >&2
    exit 2
}
logical_script_directory=$(CDPATH= cd -- "$script_directory" && pwd -L)
physical_script_directory=$(CDPATH= cd -- "$script_directory" && pwd -P)
[ "$logical_script_directory" = "$physical_script_directory" ] || {
    echo "bootstrap tests refuse a symlink-aliased script path" >&2
    exit 2
}
logical_root=$(CDPATH= cd -- "$logical_script_directory/../.." && pwd -L)
root=$(CDPATH= cd -- "$physical_script_directory/../.." && pwd -P)
[ "$logical_root" = "$root" ] || {
    echo "bootstrap tests refuse a symlink-aliased repository root" >&2
    exit 2
}
for marker in Cargo.toml AGENTS.md docs/approval-record.md docs/host-safety.md docs/tasks/release-0.md; do
    if [ ! -f "$root/$marker" ] || [ -L "$root/$marker" ]; then
        echo "bootstrap repository marker is absent or unsafe: $marker" >&2
        exit 2
    fi
done
if [ -L "$root/.git" ] || { [ ! -d "$root/.git" ] && [ ! -f "$root/.git" ]; }; then
    echo "bootstrap tests require a regular .git directory or worktree file" >&2
    exit 2
fi
bootstrap_library=$root/tools/rarbuild/bootstrap-lib.sh
[ -f "$bootstrap_library" ] && [ ! -L "$bootstrap_library" ] || exit 2
if [ "${RAR_BOOTSTRAP_LIBRARY_ALREADY_LOADED-}" != 1 ]; then
    . "$bootstrap_library"
fi
rar_preflight_policy_records "$root" || exit 2
rar_file_has_exact_line "$root/docs/approval-record.md" 'Status: Approved' || exit 2
rar_file_has_exact_line "$root/docs/approval-record.md" 'Approval: approved' || exit 2
rar_file_has_exact_line "$root/docs/tasks/release-0.md" 'Status: Ready — Gate 0 owner approval recorded 2026-07-16' || exit 2
rar_file_has_exact_line "$root/docs/host-safety.md" 'Status: Mandatory and effective immediately' || exit 2
rar_load_selected_bootstrap_root "$root" || exit 2
rar_verify_selected_bootstrap_root || exit 2
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || exit 2
rar_verify_ci_execution_boundary || exit 2
rar_verify_ci_source_snapshot || exit 2
RAR_BOOTSTRAP_LOCK_SHA256=$bootstrap_lock_sha256
export RAR_BOOTSTRAP_LOCK_SHA256
cd "$root"
umask 077

for path in out out/r0 out/r0/host-tests out/r0/tmp; do
    if [ -L "$path" ]; then
        echo "bootstrap test output must not be a symbolic link: $path" >&2
        exit 2
    fi
done

rar_root=$root
rar_prepare_output_parent "$root/out/r0" || exit 2
rar_prepare_output_parent "$root/out/r0/host-tests" || exit 2
rar_prepare_output_parent "$root/out/r0/tmp" || exit 2
test_directory=$(rar_allocate_private_directory "$root/out/r0/host-tests" bootstrap) || exit 2
[ ! -L "$test_directory" ] || exit 2
resolved_test_directory=$(CDPATH= cd -- "$test_directory" && pwd -P)
[ "$resolved_test_directory" = "$test_directory" ] || exit 2
bootstrap_test_cleanup() {
    [ -n "${test_directory-}" ] || return 0
    rar_cleanup_private_directory \
        "$test_directory" \
        bootstrap-tests main.rs rarbuild.rs safety.rs unix_fs.rs oversized-line.lock || return 1
    test_directory=
}
trap 'bootstrap_test_cleanup' 0
trap 'exit 129' 1
trap 'exit 130' 2
trap 'exit 143' 15
RAR_REPO_ROOT=$root
export RAR_REPO_ROOT
rar_materialize_git_sources \
    "$test_directory" \
    tests/bootstrap/src/main.rs main.rs \
    tools/rarbuild/src/lib.rs rarbuild.rs \
    tools/rar-lab/safety/src/lib.rs safety.rs \
    tools/rar-lab/safety/src/unix_fs.rs unix_fs.rs \
    tests/bootstrap/fixtures/oversized-line.lock oversized-line.lock || exit 2
rar_compile_host_rust \
    "$test_directory" \
    main.rs \
    bootstrap-tests \
    --cfg rar_flat_bootstrap \
    --test
RAR_BOOTSTRAP_BOUNDARY=$bootstrap_boundary
export RAR_BOOTSTRAP_BOUNDARY
if rar_execute_generated_host_binary "$test_directory/bootstrap-tests" --test-threads=1; then
    bootstrap_test_status=0
else
    bootstrap_test_status=$?
fi
bootstrap_test_cleanup || exit 2
trap - 0 1 2 15
exit "$bootstrap_test_status"
