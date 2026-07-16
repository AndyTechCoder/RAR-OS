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
grep -q '^Status: Approved$' "$root/docs/approval-record.md" || exit 2
grep -q '^Approval: approved$' "$root/docs/approval-record.md" || exit 2
grep -q '^Status: Ready — Gate 0 owner approval recorded ' "$root/docs/tasks/release-0.md" || exit 2
grep -q '^Status: Mandatory and effective immediately$' "$root/docs/host-safety.md" || exit 2
cd "$root"

for path in out out/r0 out/r0/host-tests out/r0/tmp out/r0/host-tests/bootstrap-tests; do
    if [ -L "$path" ]; then
        echo "bootstrap test output must not be a symbolic link: $path" >&2
        exit 2
    fi
done

mkdir -p out/r0/host-tests out/r0/tmp
resolved_host_tests=$(CDPATH= cd -- out/r0/host-tests && pwd -P)
[ "$resolved_host_tests" = "$root/out/r0/host-tests" ] || {
    echo "bootstrap test output escaped the repository checkout" >&2
    exit 2
}
TMPDIR="$root/out/r0/tmp"
export TMPDIR
if [ -d /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk ]; then
    SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
    export SDKROOT
fi
RAR_REPO_ROOT="$root" rustc \
    --edition 2024 \
    --test \
    --deny unsafe_code \
    tests/bootstrap/src/main.rs \
    -o out/r0/host-tests/bootstrap-tests
RAR_REPO_ROOT="$root" out/r0/host-tests/bootstrap-tests
