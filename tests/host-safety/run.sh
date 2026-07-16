#!/bin/sh
set -eu

case "$0" in
    */*) script_directory=${0%/*} ;;
    *)
        echo "host-safety tests require an explicit checkout path" >&2
        exit 2
        ;;
esac
[ ! -L "$0" ] || {
    echo "host-safety test script must not be a symbolic link" >&2
    exit 2
}
logical_script_directory=$(CDPATH= cd -- "$script_directory" && pwd -L)
physical_script_directory=$(CDPATH= cd -- "$script_directory" && pwd -P)
[ "$logical_script_directory" = "$physical_script_directory" ] || {
    echo "host-safety tests refuse a symlink-aliased script path" >&2
    exit 2
}
logical_root=$(CDPATH= cd -- "$logical_script_directory/../.." && pwd -L)
root=$(CDPATH= cd -- "$physical_script_directory/../.." && pwd -P)
[ "$logical_root" = "$root" ] || {
    echo "host-safety tests refuse a symlink-aliased repository root" >&2
    exit 2
}
for marker in Cargo.toml AGENTS.md docs/approval-record.md docs/host-safety.md docs/tasks/release-0.md; do
    if [ ! -f "$root/$marker" ] || [ -L "$root/$marker" ]; then
        echo "host-safety repository marker is absent or unsafe: $marker" >&2
        exit 2
    fi
done
if [ -L "$root/.git" ] || { [ ! -d "$root/.git" ] && [ ! -f "$root/.git" ]; }; then
    echo "host-safety tests require a regular .git directory or worktree file" >&2
    exit 2
fi
bootstrap_library=$root/tools/rarbuild/bootstrap-lib.sh
[ -f "$bootstrap_library" ] && [ ! -L "$bootstrap_library" ] || exit 2
. "$bootstrap_library"
rar_file_has_exact_line "$root/docs/approval-record.md" 'Status: Approved' || exit 2
rar_file_has_exact_line "$root/docs/approval-record.md" 'Approval: approved' || exit 2
rar_file_has_exact_line "$root/docs/tasks/release-0.md" 'Status: Ready — Gate 0 owner approval recorded 2026-07-16' || exit 2
rar_file_has_exact_line "$root/docs/host-safety.md" 'Status: Mandatory and effective immediately' || exit 2
rar_load_test_bootstrap_root "$root" || exit 2
cd "$root"

for path in out out/r0 out/r0/host-tests out/r0/tmp; do
    if [ -L "$path" ]; then
        echo "host-safety test output must not be a symbolic link: $path" >&2
        exit 2
    fi
done

rar_root=$root
rar_prepare_output_parent "$root/out/r0" || exit 2
rar_prepare_output_parent "$root/out/r0/host-tests" || exit 2
rar_prepare_output_parent "$root/out/r0/tmp" || exit 2
test_directory=$root/out/r0/host-tests/host-safety-$PPID-$$
"$bootstrap_mkdir_path" -m 700 "$test_directory" || exit 2
[ ! -L "$test_directory" ] || exit 2
resolved_test_directory=$(CDPATH= cd -- "$test_directory" && pwd -P)
[ "$resolved_test_directory" = "$test_directory" ] || exit 2
RAR_REPO_ROOT=$root
export RAR_REPO_ROOT
rar_compile_host_rust \
    tests/host-safety/src/main.rs \
    "$test_directory/host-safety-tests" \
    --test
RAR_BOOTSTRAP_BOUNDARY=$bootstrap_boundary
export RAR_BOOTSTRAP_BOUNDARY
exec "$test_directory/host-safety-tests" --test-threads=1
