#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
source_root=$repository_root
if [ "$#" -eq 1 ] && [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ]; then
    scratch=$(/bin/sh "$repository_root/tools/ci/require-ephemeral-policy-test-root.sh")
    [ "$scratch" = /tmp ] || exit 1
    case "$1" in "$scratch"/*) source_root=$1 ;; *) exit 1 ;; esac
elif [ "$#" -ne 0 ]; then
    exit 1
fi

[ -d "$source_root" ] && [ ! -L "$source_root" ] || exit 1
source_root=$(CDPATH= cd -- "$source_root" && pwd -P)
case "$source_root" in
    "$repository_root") ;;
    /tmp/*) [ "${RAR_POLICY_MUTATION_TESTS-}" = 1 ] || exit 1 ;;
    *) exit 1 ;;
esac
[ -d "$source_root/tools" ] && [ ! -L "$source_root/tools" ] || exit 1
[ -d "$source_root/tools/ci" ] && [ ! -L "$source_root/tools/ci" ] || exit 1
[ "$(CDPATH= cd -- "$source_root/tools/ci" && pwd -P)" = "$source_root/tools/ci" ] || exit 1

fail() {
    printf 'portable stat policy rejected: %s\n' "$1" >&2
    exit 1
}

check_file() {
    relative=$1
    expected_stat_lines=$2
    path=$source_root/$relative
    [ ! -L "$path" ] || fail "symbolic validator: $relative"
    [ -f "$path" ] && [ -s "$path" ] || fail "missing or empty validator: $relative"
    [ "$(CDPATH= cd -- "$(dirname -- "$path")" && pwd -P)" = "$source_root/tools/ci" ] ||
        fail "validator parent escaped source root: $relative"
    actual=$(/usr/bin/grep -Ec '/usr/bin/stat -[cf]' "$path")
    [ "$actual" -eq "$expected_stat_lines" ] ||
        fail "unexpected stat invocation count: $relative"
}

require_line() {
    relative=$1
    expected=$2
    [ "$(/usr/bin/grep -Fxc -- "$expected" "$source_root/$relative")" -eq 1 ] ||
        fail "portable stat line missing or duplicated: $relative"
}

check_file tools/ci/test-proposed-adr-classifier-policy.sh 1
check_file tools/ci/hash-source-tree.sh 1
check_file tools/ci/check-containerfile-static-policy.sh 1
check_file tools/ci/check-reference-verdict-v0.sh 1
check_file tools/ci/check-controller-handoff-attempt-v0.sh 1
check_file tools/ci/check-controller-helper-build-receipt-v0.sh 4
check_file tools/ci/check-reference-evidence-v0.sh 1
check_file tools/ci/verify-launch-evidence.sh 7
check_file tools/ci/verify-accepted-evidence-v0.sh 2
check_file tools/ci/test-controller-helper-evidence-v0-policy.sh 1
check_file tools/ci/check-controller-helper-build-evidence-v0.sh 4
check_file tools/ci/check-controller-helper-test-evidence-v0.sh 4
check_file tools/ci/test-reference-verdict-v0-policy.sh 1
check_file tools/ci/check-development-image-inputs.sh 1

require_line tools/ci/test-proposed-adr-classifier-policy.sh '    size=$(/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file")'
require_line tools/ci/hash-source-tree.sh '    size=$(/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file") || exit 1'
require_line tools/ci/check-containerfile-static-policy.sh '    size=$(/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file")'
require_line tools/ci/check-reference-verdict-v0.sh 'size=$(/usr/bin/stat -c %s "$verdict" 2>/dev/null || /usr/bin/stat -f %z "$verdict")'
require_line tools/ci/check-controller-handoff-attempt-v0.sh '    size=$(/usr/bin/stat -c %s "$file" 2>/dev/null || /usr/bin/stat -f %z "$file")'
require_line tools/ci/check-controller-helper-build-receipt-v0.sh '    links=$(/usr/bin/stat -c %h "$file" 2>/dev/null || /usr/bin/stat -f %l "$file")'
require_line tools/ci/check-controller-helper-build-receipt-v0.sh '    owner=$(/usr/bin/stat -c %u "$file" 2>/dev/null || /usr/bin/stat -f %u "$file")'
require_line tools/ci/check-controller-helper-build-receipt-v0.sh 'size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }'
require_line tools/ci/check-controller-helper-build-receipt-v0.sh 'identity() { /usr/bin/stat -c '\''%d:%i:%s:%h:%u:%Y'\'' "$1" 2>/dev/null || /usr/bin/stat -f '\''%d:%i:%z:%l:%u:%m'\'' "$1"; }'
require_line tools/ci/check-reference-evidence-v0.sh 'size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }'
require_line tools/ci/verify-launch-evidence.sh 'serial_size=$(/usr/bin/stat -c %s "$serial" 2>/dev/null || /usr/bin/stat -f %z "$serial")'
require_line tools/ci/verify-launch-evidence.sh 'serial_links=$(/usr/bin/stat -c %h "$serial" 2>/dev/null || /usr/bin/stat -f %l "$serial")'
require_line tools/ci/verify-launch-evidence.sh 'actions_size=$(/usr/bin/stat -c %s "$actions" 2>/dev/null || /usr/bin/stat -f %z "$actions")'
require_line tools/ci/verify-launch-evidence.sh 'actions_links=$(/usr/bin/stat -c %h "$actions" 2>/dev/null || /usr/bin/stat -f %l "$actions")'
require_line tools/ci/verify-launch-evidence.sh '        image_size=$(/usr/bin/stat -c %s "$image" 2>/dev/null || /usr/bin/stat -f %z "$image")'
require_line tools/ci/verify-launch-evidence.sh '        image_links=$(/usr/bin/stat -c %h "$image" 2>/dev/null || /usr/bin/stat -f %l "$image")'
require_line tools/ci/verify-launch-evidence.sh '    size=$(/usr/bin/stat -c %s "$base/$name" 2>/dev/null || /usr/bin/stat -f %z "$base/$name")'
require_line tools/ci/verify-accepted-evidence-v0.sh 'size=$(/usr/bin/stat -c %s "$manifest" 2>/dev/null || /usr/bin/stat -f %z "$manifest")'
require_line tools/ci/verify-accepted-evidence-v0.sh 'links=$(/usr/bin/stat -c %h "$manifest" 2>/dev/null || /usr/bin/stat -f %l "$manifest")'
require_line tools/ci/test-controller-helper-evidence-v0-policy.sh 'size=$(/usr/bin/stat -c %s "$build_source" 2>/dev/null || /usr/bin/stat -f %z "$build_source")'
require_line tools/ci/check-controller-helper-build-evidence-v0.sh '    links=$(/usr/bin/stat -c %h "$file" 2>/dev/null || /usr/bin/stat -f %l "$file")'
require_line tools/ci/check-controller-helper-build-evidence-v0.sh '    owner=$(/usr/bin/stat -c %u "$file" 2>/dev/null || /usr/bin/stat -f %u "$file")'
require_line tools/ci/check-controller-helper-build-evidence-v0.sh 'size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }'
require_line tools/ci/check-controller-helper-build-evidence-v0.sh 'identity() { /usr/bin/stat -c '\''%d:%i:%s:%h:%u:%Y'\'' "$1" 2>/dev/null || /usr/bin/stat -f '\''%d:%i:%z:%l:%u:%m'\'' "$1"; }'
require_line tools/ci/check-controller-helper-test-evidence-v0.sh '    links=$(/usr/bin/stat -c %h "$file" 2>/dev/null || /usr/bin/stat -f %l "$file")'
require_line tools/ci/check-controller-helper-test-evidence-v0.sh '    owner=$(/usr/bin/stat -c %u "$file" 2>/dev/null || /usr/bin/stat -f %u "$file")'
require_line tools/ci/check-controller-helper-test-evidence-v0.sh 'size_of() { /usr/bin/stat -c %s "$1" 2>/dev/null || /usr/bin/stat -f %z "$1"; }'
require_line tools/ci/check-controller-helper-test-evidence-v0.sh 'identity() { /usr/bin/stat -c '\''%d:%i:%s:%h:%u:%Y'\'' "$1" 2>/dev/null || /usr/bin/stat -f '\''%d:%i:%z:%l:%u:%m'\'' "$1"; }'
require_line tools/ci/test-reference-verdict-v0-policy.sh 'size=$(/usr/bin/stat -c %s "$accepted" 2>/dev/null || /usr/bin/stat -f %z "$accepted")'
require_line tools/ci/check-development-image-inputs.sh 'size=$(/usr/bin/stat -c %s "$inputs" 2>/dev/null || /usr/bin/stat -f %z "$inputs")'

printf '%s\n' 'portable stat policy passed: files=14 fallbacks=30'
