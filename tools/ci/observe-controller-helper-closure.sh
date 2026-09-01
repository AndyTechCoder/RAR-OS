#!/usr/bin/dash
set -eu

PATH=/usr/bin:/bin
LC_ALL=C
LANG=C
umask 077
export PATH LC_ALL LANG

fail() {
    printf 'controller-helper closure observation failed: %s\n' "$1" >&2
    exit 1
}

hash_file() {
    output=$(/usr/bin/sha256sum -- "$1") || fail "cannot hash $1"
    digest=${output%% *}
    case "$digest" in
        '' | *[!0-9a-f]*) fail "non-canonical digest for $1" ;;
    esac
    [ "${#digest}" -eq 64 ] || fail "wrong digest length for $1"
    printf '%s\n' "$digest"
}

require_positive_decimal() {
    value=$1
    label=$2
    case "$value" in
        '' | 0 | 0* | *[!0-9]*) fail "$label is not canonical positive decimal" ;;
    esac
    [ "${#value}" -le 20 ] || fail "$label exceeds 20 digits"
}

[ "${RAR_CONTROLLER_HELPER_CLOSURE_DISCOVERY-}" = 1 ] || fail 'explicit discovery mode is required'
[ "${GITHUB_ACTIONS-}" = true ] || fail 'GitHub Actions boundary is absent'
[ "${CI-}" = true ] || fail 'CI boundary is absent'
[ "${GITHUB_EVENT_NAME-}" = push ] || fail 'only a push event may observe the closure'
[ "${GITHUB_REF-}" = refs/heads/main ] || fail 'only the canonical main ref may observe the closure'
[ "${GITHUB_REPOSITORY-}" = AndyTechCoder/RAR-OS ] || fail 'canonical repository mismatch'

for revision in "${GITHUB_SHA-}" "${RAR_TRUSTED_CONTROLLER_SHA-}" "${RAR_EXPECTED_SOURCE_REVISION-}"; do
    case "$revision" in
        '' | *[!0-9a-f]*) fail 'revision is malformed' ;;
    esac
    [ "${#revision}" -eq 40 ] || fail 'revision length is invalid'
done
[ "$GITHUB_SHA" = "$RAR_TRUSTED_CONTROLLER_SHA" ] || fail 'controller is not exact main'
[ "$GITHUB_SHA" = "$RAR_EXPECTED_SOURCE_REVISION" ] || fail 'source is not exact main'

require_positive_decimal "${GITHUB_RUN_ID-}" 'run id'
require_positive_decimal "${GITHUB_RUN_ATTEMPT-}" 'run attempt'
[ "${RAR_CI_RUNNER_IMAGE_OS-}" = ubuntu24 ] || fail 'runner OS evidence mismatch'
case "${RAR_CI_RUNNER_IMAGE_VERSION-}" in
    '' | *[!0-9.]* | .* | *. | *..*) fail 'runner image version is malformed' ;;
esac
[ "${#RAR_CI_RUNNER_IMAGE_VERSION}" -le 64 ] || fail 'runner image version is oversized'
[ "${RAR_CI_RUNNER_OS-}" = Linux ] || fail 'runner operating system mismatch'
[ "${RAR_CI_RUNNER_ARCH-}" = X64 ] || fail 'runner architecture mismatch'

image=sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = "$image" ] || fail 'bootstrap image identity mismatch'

shell=/usr/bin/dash
hasher=/usr/bin/sha256sum
finder=/usr/bin/find
sorter=/usr/bin/sort
counter=/usr/bin/wc
matcher=/usr/bin/grep
directory=/usr/bin/mkdir
root=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu
rustc=$root/bin/rustc
script=/workspace/tools/ci/observe-controller-helper-closure.sh

[ "$0" = "$script" ] || fail 'executing observer path mismatch'
for file in "$shell" "$hasher" "$finder" "$sorter" "$counter" "$matcher" "$directory" "$rustc" "$script"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required regular file is unavailable: $file"
done
[ -d "$root" ] && [ ! -L "$root" ] || fail 'toolchain root is unavailable or symbolic'
[ "$(hash_file "$shell")" = a6f559e00b69a4aa4d8cb607be18d9386c5aee55c509e2c075549dcf00e00fc7 ] || fail 'shell identity mismatch'
[ "$(hash_file "$hasher")" = 89f8c1d1ba3c76138f3771e1a91e2796ade6180b1c1e4258c04698ff32787c97 ] || fail 'hasher identity mismatch'
[ "$(hash_file "$counter")" = e8fe45a85ebdb0dade6dabf96f21dfd686c6414ff2a4a8980727076a5981d2af ] || fail 'counter identity mismatch'
[ "$(hash_file "$matcher")" = bd6686bf7a650a9717fd7e73fdb07dc63b70547a1da41bce093c56df937a66eb ] || fail 'matcher identity mismatch'
[ "$(hash_file "$directory")" = bd3d9b36a1cc1c63b8dca7967003f4e5b0bb8556c87cbb7c1916aa358bf3f053 ] || fail 'directory creator identity mismatch'
[ "$(hash_file "$rustc")" = bff349e72704ff70bc08a234a3847338e797065bbedde5e556808bc87b7bf7c6 ] || fail 'rustc identity mismatch'

scratch=/tmp/rar-controller-helper-closure
[ ! -e "$scratch" ] && [ ! -L "$scratch" ] || fail 'private scratch path already exists'
/usr/bin/mkdir -- "$scratch" || fail 'cannot create private scratch directory'
device_unsorted=$scratch/devices.unsorted
devices=$scratch/devices
unsorted=$scratch/paths.unsorted
paths=$scratch/paths

/usr/bin/find -P "$root" -printf '%D\n' > "$device_unsorted" || fail 'cannot inspect closure devices'
/usr/bin/sort -u "$device_unsorted" > "$devices" || fail 'cannot sort closure devices'
device_count=$(/usr/bin/wc -l < "$devices") || fail 'cannot count closure devices'
set -f
set -- $device_count
set +f
[ "$#" -eq 1 ] && [ "$1" -eq 1 ] || fail 'closure crosses a device boundary'

unexpected=$(/usr/bin/find -P "$root" ! \( -type d -o -type f \) -printf x -quit) || fail 'cannot inspect closure topology'
[ -z "$unexpected" ] || fail 'closure contains a non-directory or non-regular entry'
hardlinked=$(/usr/bin/find -P "$root" -type f -links +1 -printf x -quit) || fail 'cannot inspect closure hardlinks'
[ -z "$hardlinked" ] || fail 'closure contains a hardlinked regular file'

evidence=/evidence
[ -d "$evidence" ] && [ ! -L "$evidence" ] || fail 'controller evidence boundary is unavailable'
manifest=$evidence/controller-helper-closure.sha256
receipt=$evidence/controller-helper-closure.receipt
for output in "$manifest" "$receipt"; do
    [ ! -e "$output" ] && [ ! -L "$output" ] || fail "observation output already exists: $output"
done

/usr/bin/find -P "$root" -type f -printf '%P\n' > "$unsorted" || fail 'cannot enumerate regular closure files'
/usr/bin/sort "$unsorted" > "$paths" || fail 'cannot sort closure paths'
[ -s "$paths" ] || fail 'closure path set is empty'

set -C
exec 3> "$manifest" || fail 'cannot exclusively create candidate manifest'
set +C
previous=
count=0
manifest_bytes_expected=0
while IFS= read -r relative; do
    case "$relative" in
        '' | /* | . | .. | ./* | ../* | *//* | */./* | */../* | */. | */.. | *[!A-Za-z0-9._/+:-]*)
            fail "unsafe closure path: $relative"
            ;;
    esac
    [ "${#relative}" -le 384 ] || fail "oversized closure path: $relative"
    [ "$relative" != "$previous" ] || fail "duplicate closure path: $relative"
    file=$root/$relative
    [ -f "$file" ] && [ ! -L "$file" ] || fail "closure entry changed type: $relative"
    digest=$(hash_file "$file")
    record_bytes=$((67 + ${#relative}))
    manifest_bytes_expected=$((manifest_bytes_expected + record_bytes))
    [ "$manifest_bytes_expected" -le 1048576 ] || fail 'closure manifest exceeds reviewed bounds'
    printf '%s  %s\n' "$digest" "$relative" >&3 || fail 'cannot write candidate manifest'
    previous=$relative
    count=$((count + 1))
done < "$paths"
exec 3>&- || fail 'cannot close candidate manifest'

[ "$count" -gt 0 ] || fail 'closure manifest is empty'
recorded_count=$(/usr/bin/wc -l < "$manifest") || fail 'cannot count closure records'
set -f
set -- $recorded_count
set +f
[ "$#" -eq 1 ] && [ "$1" -eq "$count" ] || fail 'closure record count mismatch'
manifest_bytes=$(/usr/bin/wc -c < "$manifest") || fail 'cannot size closure manifest'
set -f
set -- $manifest_bytes
set +f
[ "$#" -eq 1 ] || fail 'closure manifest size is malformed'
manifest_bytes=$1
[ "$manifest_bytes" -gt 0 ] && [ "$manifest_bytes" -le 1048576 ] || fail 'closure manifest exceeds reviewed bounds'
[ "$manifest_bytes" -eq "$manifest_bytes_expected" ] || fail 'closure manifest byte count mismatch'

set -C
exec 4> "$receipt" || fail 'cannot exclusively create observation receipt'
set +C
printf '%s\n' \
    'schema=rar-alpha-controller-helper-closure-observation-v0' \
    'status=observed-not-reviewed-not-ready' \
    "controller_sha=$GITHUB_SHA" \
    "source_sha=$RAR_EXPECTED_SOURCE_REVISION" \
    "repository=$GITHUB_REPOSITORY" \
    "ref=$GITHUB_REF" \
    "event=$GITHUB_EVENT_NAME" \
    "run_id=$GITHUB_RUN_ID" \
    "run_attempt=$GITHUB_RUN_ATTEMPT" \
    "runner_os=$RAR_CI_RUNNER_IMAGE_OS" \
    "runner_image_version=$RAR_CI_RUNNER_IMAGE_VERSION" \
    "oci_image=$image" \
    "closure_root=$root" \
    "generator_sha256=$(hash_file "$script")" \
    "find_sha256=$(hash_file "$finder")" \
    "sort_sha256=$(hash_file "$sorter")" \
    "manifest_sha256=$(hash_file "$manifest")" \
    "manifest_entries=$count" \
    "manifest_bytes=$manifest_bytes" \
    'helper_compiled=false' \
    'helper_executed=false' \
    'target_compiled=false' \
    'readiness=false' >&4 || fail 'cannot write observation receipt'
exec 4>&- || fail 'cannot close observation receipt'

receipt_lines=$(/usr/bin/wc -l < "$receipt") || fail 'cannot count receipt lines'
set -f
set -- $receipt_lines
set +f
[ "$#" -eq 1 ] && [ "$1" -eq 23 ] || fail 'receipt line count mismatch'
receipt_bytes=$(/usr/bin/wc -c < "$receipt") || fail 'cannot size observation receipt'
set -f
set -- $receipt_bytes
set +f
[ "$#" -eq 1 ] && [ "$1" -gt 0 ] && [ "$1" -le 4096 ] || fail 'receipt exceeds reviewed bounds'
