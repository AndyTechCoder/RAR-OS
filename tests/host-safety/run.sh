#!/bin/sh
set -eu
case "$0" in
    */*) script_directory=${0%/*} ;;
    *)
        printf '%s\n' 'host-safety:explicit-script-path-required' >&2
        exit 2
        ;;
esac
root=$(CDPATH= cd -- "$script_directory/../.." && pwd -P)
cd "$root"
. tools/rarbuild/bootstrap-lib.sh
rar_preflight_policy_records "$root"
rar_load_selected_bootstrap_root "$root"
rar_verify_selected_bootstrap_root
rar_verify_ci_execution_boundary
rar_verify_ci_source_snapshot
require_test_helper_root() {
    if [ "${rar_root-}" != "$root" ]; then
        printf '%s\n' 'host-safety:missing-rar-root-context' >&2
        return 2
    fi
}
set +e
missing_root_output=$(unset rar_root; require_test_helper_root 2>&1)
missing_root_status=$?
set -e
if [ "$missing_root_status" -ne 2 ] || [ "$missing_root_output" != host-safety:missing-rar-root-context ]; then
    printf '%s\n' 'host-safety:missing-rar-root-negative-test-failed' >&2
    exit 2
fi
rar_root=$root
require_test_helper_root
rar_prepare_output_parent "$root/out/r0"
rar_prepare_output_parent "$root/out/r0/host-tests"
directory=$(rar_allocate_private_directory "$root/out/r0/host-tests" host-cutover)
cleanup(){ [ -z "${directory-}" ] || rar_cleanup_private_directory "$directory" host-cutover-tests main.rs safety.rs; }
trap 'cleanup' 0 1 2 15
rar_materialize_git_sources "$directory" \
  tests/host-safety/src/main.rs main.rs \
  tools/rar-lab/safety/src/lib.rs safety.rs
rar_compile_host_rust "$directory" main.rs host-cutover-tests --cfg rar_flat_bootstrap --test
RAR_BOOTSTRAP_BOUNDARY=$bootstrap_boundary
export RAR_BOOTSTRAP_BOUNDARY
rar_execute_generated_host_binary "$directory/host-cutover-tests" --test-threads=1
cleanup
directory=
trap - 0 1 2 15
