#!/usr/bin/dash
set -eu

PATH=/usr/bin:/bin
LC_ALL=C
LANG=C
umask 077
export PATH LC_ALL LANG

fail() {
    printf 'controller-helper closure candidate verification failed: %s\n' "$1" >&2
    exit 1
}

sha_file() {
    output=$(/usr/bin/sha256sum -- "$1") || fail "cannot hash $1"
    digest=${output%% *}
    case "$digest" in '' | *[!0-9a-f]*) fail "non-canonical digest for $1" ;; esac
    [ "${#digest}" -eq 64 ] || fail "wrong digest length for $1"
    printf '%s\n' "$digest"
}

require_sha() {
    case "$1" in '' | *[!0-9a-f]*) fail "$2 is not lowercase SHA-256" ;; esac
    [ "${#1}" -eq 64 ] || fail "$2 SHA-256 length is invalid"
}

require_nonzero_sha() {
    require_sha "$1" "$2"
    [ "$1" != 0000000000000000000000000000000000000000000000000000000000000000 ] || fail "$2 SHA-256 is zero"
}

require_revision() {
    case "$1" in '' | *[!0-9a-f]*) fail "$2 revision is malformed" ;; esac
    [ "${#1}" -eq 40 ] || fail "$2 revision length is invalid"
}

require_positive_decimal() {
    case "$1" in '' | 0 | 0* | *[!0-9]*) fail "$2 is not canonical positive decimal" ;; esac
    [ "${#1}" -le 20 ] || fail "$2 exceeds 20 digits"
}

[ "${RAR_CONTROLLER_HELPER_CLOSURE_VERIFICATION-}" = 1 ] || fail 'explicit verification mode is required'
[ "${GITHUB_ACTIONS-}" = true ] || fail 'GitHub Actions boundary is absent'
[ "${CI-}" = true ] || fail 'CI boundary is absent'
[ "${GITHUB_EVENT_NAME-}" = push ] || fail 'only a push event may verify a candidate'
[ "${GITHUB_REF-}" = refs/heads/main ] || fail 'only the canonical main ref may verify a candidate'
[ "${GITHUB_REPOSITORY-}" = AndyTechCoder/RAR-OS ] || fail 'canonical repository mismatch'
require_revision "${GITHUB_SHA-}" 'GitHub'
require_revision "${RAR_TRUSTED_CONTROLLER_SHA-}" 'controller'
require_revision "${RAR_EXPECTED_SOURCE_REVISION-}" 'source'
[ "$GITHUB_SHA" = "$RAR_TRUSTED_CONTROLLER_SHA" ] || fail 'controller is not exact main'
[ "$GITHUB_SHA" = "$RAR_EXPECTED_SOURCE_REVISION" ] || fail 'source is not exact main'
require_positive_decimal "${GITHUB_RUN_ID-}" 'run id'
require_positive_decimal "${GITHUB_RUN_ATTEMPT-}" 'run attempt'
[ "${RAR_CI_RUNNER_IMAGE_OS-}" = ubuntu24 ] || fail 'runner OS evidence mismatch'
case "${RAR_CI_RUNNER_IMAGE_VERSION-}" in '' | *[!0-9.]* | .* | *. | *..*) fail 'runner image version is malformed' ;; esac
[ "${#RAR_CI_RUNNER_IMAGE_VERSION}" -le 64 ] || fail 'runner image version is oversized'
[ "${RAR_CI_RUNNER_OS-}" = Linux ] || fail 'runner operating system mismatch'
[ "${RAR_CI_RUNNER_ARCH-}" = X64 ] || fail 'runner architecture mismatch'
require_positive_decimal "${RAR_CONTROLLER_UID-}" 'controller uid'
require_nonzero_sha "${RAR_REVIEWED_VERIFIER_SHA256-}" 'reviewed verifier'
require_nonzero_sha "${RAR_REVIEWED_VERIFIER_TOOLS_SHA256-}" 'reviewed verifier tools'

image=sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
[ "${RAR_CI_BOOTSTRAP_IMAGE-}" = "$image" ] || fail 'bootstrap image identity mismatch'

shell=/usr/bin/dash
hasher=/usr/bin/sha256sum
directory=/usr/bin/mkdir
finder=/usr/bin/find
sorter=/usr/bin/sort
counter=/usr/bin/wc
metadata=/usr/bin/stat
comparator=/usr/bin/cmp
identity_tool=/usr/bin/id
root=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu
script=/workspace/tools/ci/verify-controller-helper-closure-candidate.sh
observer=/workspace/tools/ci/observe-controller-helper-closure.sh
evidence=/evidence
manifest=$evidence/controller-helper-closure.sha256
observation=$evidence/controller-helper-closure.receipt
tool_pins=/trusted/controller-helper-closure-verifier-tools.v0
verification=/verification
verification_receipt=$verification/controller-helper-closure-verification.receipt
scratch=/tmp/rar-controller-helper-closure-verification
mountinfo=/proc/self/mountinfo

[ "$0" = "$script" ] || fail 'executing verifier path mismatch'
for file in "$shell" "$hasher" "$directory" "$script" "$observer" "$tool_pins"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "required trusted regular file is unavailable: $file"
done
[ -d "$root" ] && [ ! -L "$root" ] || fail 'closure root is unavailable or symbolic'
[ -d "$evidence" ] && [ ! -L "$evidence" ] || fail 'evidence input boundary is unavailable'
[ -d "$verification" ] && [ ! -L "$verification" ] || fail 'verification output boundary is unavailable'
[ -f "$mountinfo" ] && [ ! -L "$mountinfo" ] || fail 'mount topology evidence is unavailable'
[ "$(sha_file "$shell")" = a6f559e00b69a4aa4d8cb607be18d9386c5aee55c509e2c075549dcf00e00fc7 ] || fail 'shell identity mismatch'
[ "$(sha_file "$hasher")" = 89f8c1d1ba3c76138f3771e1a91e2796ade6180b1c1e4258c04698ff32787c97 ] || fail 'hasher identity mismatch'
[ "$(sha_file "$directory")" = bd3d9b36a1cc1c63b8dca7967003f4e5b0bb8556c87cbb7c1916aa358bf3f053 ] || fail 'directory creator identity mismatch'
verifier_sha=$(sha_file "$script")
[ "$verifier_sha" = "$RAR_REVIEWED_VERIFIER_SHA256" ] || fail 'verifier escaped controller-bound reviewed identity'
[ "$(sha_file "$tool_pins")" = "$RAR_REVIEWED_VERIFIER_TOOLS_SHA256" ] || fail 'verifier tool pins escaped reviewed identity'
[ ! -e "$scratch" ] && [ ! -L "$scratch" ] || fail 'private scratch path already exists'
"$directory" -- "$scratch" || fail 'cannot create private scratch directory'

pin_line=0
find_sha=
sort_sha=
wc_sha=
stat_sha=
cmp_sha=
id_sha=
while IFS= read -r line; do
    pin_line=$((pin_line + 1))
    case "$pin_line:$line" in
        1:schema=rar-alpha-controller-helper-closure-verifier-tools-v0) ;;
        2:find_sha256=*) find_sha=${line#find_sha256=} ;;
        3:sort_sha256=*) sort_sha=${line#sort_sha256=} ;;
        4:wc_sha256=*) wc_sha=${line#wc_sha256=} ;;
        5:stat_sha256=*) stat_sha=${line#stat_sha256=} ;;
        6:cmp_sha256=*) cmp_sha=${line#cmp_sha256=} ;;
        7:id_sha256=*) id_sha=${line#id_sha256=} ;;
        8:status=reviewed-for-candidate-verification-only) ;;
        *) fail "tool pin grammar or order invalid at line $pin_line" ;;
    esac
done < "$tool_pins"
[ "$pin_line" -eq 8 ] || fail 'tool pin field count is invalid'
require_nonzero_sha "$find_sha" 'find tool'
require_nonzero_sha "$sort_sha" 'sort tool'
require_nonzero_sha "$wc_sha" 'wc tool'
require_nonzero_sha "$stat_sha" 'stat tool'
require_nonzero_sha "$cmp_sha" 'cmp tool'
require_nonzero_sha "$id_sha" 'id tool'
for file in "$finder" "$sorter" "$counter" "$metadata" "$comparator" "$identity_tool"; do
    [ -f "$file" ] && [ ! -L "$file" ] || fail "verification tool is unavailable: $file"
done
[ "$(sha_file "$finder")" = "$find_sha" ] || fail 'find identity mismatch'
[ "$(sha_file "$sorter")" = "$sort_sha" ] || fail 'sort identity mismatch'
[ "$(sha_file "$counter")" = "$wc_sha" ] || fail 'wc identity mismatch'
[ "$(sha_file "$metadata")" = "$stat_sha" ] || fail 'stat identity mismatch'
[ "$(sha_file "$comparator")" = "$cmp_sha" ] || fail 'cmp identity mismatch'
[ "$(sha_file "$identity_tool")" = "$id_sha" ] || fail 'id identity mismatch'
tool_pins_canonical=$scratch/tool-pins.canonical
printf '%s\n' \
    'schema=rar-alpha-controller-helper-closure-verifier-tools-v0' \
    "find_sha256=$find_sha" \
    "sort_sha256=$sort_sha" \
    "wc_sha256=$wc_sha" \
    "stat_sha256=$stat_sha" \
    "cmp_sha256=$cmp_sha" \
    "id_sha256=$id_sha" \
    'status=reviewed-for-candidate-verification-only' > "$tool_pins_canonical" || fail 'cannot reconstruct verifier tool pins'
"$comparator" -s -- "$tool_pins" "$tool_pins_canonical" || fail 'verifier tool pins are not canonical bytes'
tool_pin_bytes=$("$counter" -c < "$tool_pins") || fail 'cannot size verifier tool pins'
set -- $tool_pin_bytes
[ "$#" -eq 1 ] && [ "$1" -gt 0 ] && [ "$1" -le 2048 ] || fail 'verifier tool pins exceed reviewed bounds'
process_uid=$("$identity_tool" -u) || fail 'cannot observe verifier uid'
[ "$process_uid" = "$RAR_CONTROLLER_UID" ] || fail 'verifier uid differs from controller uid'
[ "$("$metadata" -c %u -- "$evidence")" = "$RAR_CONTROLLER_UID" ] || fail 'evidence boundary is not controller-owned'
[ "$("$metadata" -c %u -- "$verification")" = "$RAR_CONTROLLER_UID" ] || fail 'verification boundary is not controller-owned'

check_nested_mounts() {
    while IFS=' ' read -r mount_id parent_id device_id mount_root mount_point remainder; do
        case "$mount_point" in "$root" | "$root"/*) fail "closure contains a nested or replacement mount: $mount_point" ;; esac
    done < "$mountinfo"
}

stat_identity() {
    "$metadata" -c '%d:%i:%h:%u:%s:%f:%Y:%Z' -- "$1" || fail "cannot stat $1"
}

script_identity_before=$(stat_identity "$script")

safe_input() {
    file=$1
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "missing, symbolic, or empty input: $file"
    set -- $("$metadata" -c '%h %u' -- "$file") || fail "cannot inspect input ownership: $file"
    [ "$#" -eq 2 ] || fail "input metadata is malformed: $file"
    [ "$1" -eq 1 ] || fail "input is hardlinked: $file"
    [ "$2" = "$RAR_CONTROLLER_UID" ] || fail "input is not controller-owned: $file"
}

safe_input "$manifest"
safe_input "$observation"
[ ! "$manifest" -ef "$observation" ] || fail 'candidate inputs alias'
[ ! -e "$verification_receipt" ] && [ ! -L "$verification_receipt" ] || fail 'verification receipt already exists'

evidence_names=$scratch/evidence.names
"$finder" -P "$evidence" -mindepth 1 -maxdepth 1 -printf '%f\n' > "$evidence_names" || fail 'cannot enumerate evidence boundary'
"$sorter" "$evidence_names" > "$scratch/evidence.names.sorted" || fail 'cannot sort evidence names'
name_line=0
while IFS= read -r name; do
    name_line=$((name_line + 1))
    case "$name_line:$name" in
        1:controller-helper-closure.receipt | 2:controller-helper-closure.sha256) ;;
        *) fail 'evidence boundary contains an unexpected entry' ;;
    esac
done < "$scratch/evidence.names.sorted"
[ "$name_line" -eq 2 ] || fail 'evidence boundary does not contain exactly two inputs'

manifest_identity_before=$(stat_identity "$manifest")
observation_identity_before=$(stat_identity "$observation")
mountinfo_before=$(sha_file "$mountinfo")
check_nested_mounts
manifest_sha=$(sha_file "$manifest")
observation_sha=$(sha_file "$observation")
manifest_bytes=$("$counter" -c < "$manifest") || fail 'cannot size candidate manifest'
observation_bytes=$("$counter" -c < "$observation") || fail 'cannot size observation receipt'
set -- $manifest_bytes
[ "$#" -eq 1 ] || fail 'candidate manifest size is malformed'
manifest_bytes=$1
set -- $observation_bytes
[ "$#" -eq 1 ] || fail 'observation receipt size is malformed'
observation_bytes=$1
[ "$manifest_bytes" -gt 0 ] && [ "$manifest_bytes" -le 1048576 ] || fail 'candidate manifest exceeds reviewed bounds'
[ "$observation_bytes" -gt 0 ] && [ "$observation_bytes" -le 4096 ] || fail 'observation receipt exceeds reviewed bounds'

receipt_line=0
controller_sha=
source_sha=
receipt_run_id=
receipt_run_attempt=
receipt_runner_version=
receipt_generator_sha=
receipt_find_sha=
receipt_sort_sha=
receipt_manifest_sha=
receipt_entries=
receipt_bytes=
while IFS= read -r line; do
    receipt_line=$((receipt_line + 1))
    case "$receipt_line:$line" in
        1:schema=rar-alpha-controller-helper-closure-observation-v0) ;;
        2:status=observed-not-reviewed-not-ready) ;;
        3:controller_sha=*) controller_sha=${line#controller_sha=} ;;
        4:source_sha=*) source_sha=${line#source_sha=} ;;
        5:repository=AndyTechCoder/RAR-OS) ;;
        6:ref=refs/heads/main) ;;
        7:event=push) ;;
        8:run_id=*) receipt_run_id=${line#run_id=} ;;
        9:run_attempt=*) receipt_run_attempt=${line#run_attempt=} ;;
        10:runner_os=ubuntu24) ;;
        11:runner_image_version=*) receipt_runner_version=${line#runner_image_version=} ;;
        12:oci_image="$image") ;;
        13:closure_root="$root") ;;
        14:generator_sha256=*) receipt_generator_sha=${line#generator_sha256=} ;;
        15:find_sha256=*) receipt_find_sha=${line#find_sha256=} ;;
        16:sort_sha256=*) receipt_sort_sha=${line#sort_sha256=} ;;
        17:manifest_sha256=*) receipt_manifest_sha=${line#manifest_sha256=} ;;
        18:manifest_entries=*) receipt_entries=${line#manifest_entries=} ;;
        19:manifest_bytes=*) receipt_bytes=${line#manifest_bytes=} ;;
        20:helper_compiled=false | 21:helper_executed=false | 22:target_compiled=false | 23:readiness=false) ;;
        *) fail "observation receipt grammar or order invalid at line $receipt_line" ;;
    esac
done < "$observation"
[ "$receipt_line" -eq 23 ] || fail 'observation receipt field count is invalid'
require_revision "$controller_sha" 'receipt controller'
require_revision "$source_sha" 'receipt source'
[ "$controller_sha" = "$GITHUB_SHA" ] && [ "$source_sha" = "$GITHUB_SHA" ] || fail 'receipt revision context mismatch'
require_positive_decimal "$receipt_run_id" 'receipt run id'
require_positive_decimal "$receipt_run_attempt" 'receipt run attempt'
case "$receipt_runner_version" in '' | *[!0-9.]* | .* | *. | *..*) fail 'receipt runner image version is malformed' ;; esac
[ "${#receipt_runner_version}" -le 64 ] || fail 'receipt runner image version is oversized'
require_sha "$receipt_generator_sha" 'observer generator'
require_sha "$receipt_find_sha" 'observed find'
require_sha "$receipt_sort_sha" 'observed sort'
require_sha "$receipt_manifest_sha" 'receipt manifest'
require_positive_decimal "$receipt_entries" 'receipt manifest entries'
require_positive_decimal "$receipt_bytes" 'receipt manifest bytes'
[ "$receipt_generator_sha" = e3b4be670797d3f1bc84960d1d1207e470f87ba5a7fadc6327d86b7b61a7f320 ] || fail 'observer source identity mismatch'
[ "$receipt_find_sha" = "$find_sha" ] || fail 'observed find differs from reviewed verifier tool'
[ "$receipt_sort_sha" = "$sort_sha" ] || fail 'observed sort differs from reviewed verifier tool'
[ "$receipt_manifest_sha" = "$manifest_sha" ] || fail 'receipt manifest digest mismatch'
[ "$receipt_bytes" -eq "$manifest_bytes" ] || fail 'receipt manifest byte count mismatch'
observation_canonical=$scratch/observation.canonical
printf '%s\n' \
    'schema=rar-alpha-controller-helper-closure-observation-v0' \
    'status=observed-not-reviewed-not-ready' \
    "controller_sha=$controller_sha" \
    "source_sha=$source_sha" \
    'repository=AndyTechCoder/RAR-OS' \
    'ref=refs/heads/main' \
    'event=push' \
    "run_id=$receipt_run_id" \
    "run_attempt=$receipt_run_attempt" \
    'runner_os=ubuntu24' \
    "runner_image_version=$receipt_runner_version" \
    "oci_image=$image" \
    "closure_root=$root" \
    "generator_sha256=$receipt_generator_sha" \
    "find_sha256=$receipt_find_sha" \
    "sort_sha256=$receipt_sort_sha" \
    "manifest_sha256=$receipt_manifest_sha" \
    "manifest_entries=$receipt_entries" \
    "manifest_bytes=$receipt_bytes" \
    'helper_compiled=false' \
    'helper_executed=false' \
    'target_compiled=false' \
    'readiness=false' > "$observation_canonical" || fail 'cannot reconstruct observation receipt'
"$comparator" -s -- "$observation" "$observation_canonical" || fail 'observation receipt is not canonical bytes'

candidate_paths=$scratch/candidate.paths
exec 5> "$candidate_paths" || fail 'cannot create candidate path list'
candidate_count=0
candidate_expected_bytes=0
while IFS= read -r line; do
    [ "${#line}" -le 450 ] || fail 'candidate manifest line exceeds reviewed bound'
    digest=${line%%  *}
    relative=${line#*  }
    [ "$line" = "$digest  $relative" ] || fail 'candidate manifest separator is invalid'
    require_sha "$digest" 'candidate record'
    case "$relative" in
        '' | /* | . | .. | ./* | ../* | *//* | */./* | */../* | */. | */.. | *[!A-Za-z0-9._/+:-]*)
            fail "unsafe candidate closure path: $relative"
            ;;
    esac
    [ "${#relative}" -le 384 ] || fail "oversized candidate closure path: $relative"
    printf '%s\n' "$relative" >&5 || fail 'cannot write candidate path list'
    candidate_count=$((candidate_count + 1))
    candidate_expected_bytes=$((candidate_expected_bytes + 67 + ${#relative}))
    [ "$candidate_expected_bytes" -le 1048576 ] || fail 'candidate manifest exceeds reviewed bounds'
done < "$manifest"
exec 5>&- || fail 'cannot close candidate path list'
[ "$candidate_count" -eq "$receipt_entries" ] || fail 'candidate manifest entry count mismatch'
[ "$candidate_expected_bytes" -eq "$manifest_bytes" ] || fail 'candidate manifest byte grammar mismatch'
"$sorter" -c -u "$candidate_paths" >/dev/null 2>&1 || fail 'candidate manifest paths are unordered or duplicated'

bound_snapshot() {
    bounded_file=$1
    maximum_lines=$2
    line_count=$("$counter" -l < "$bounded_file") || fail "cannot count bounded snapshot: $bounded_file"
    byte_count=$("$counter" -c < "$bounded_file") || fail "cannot size bounded snapshot: $bounded_file"
    set -- $line_count
    [ "$#" -eq 1 ] && [ "$1" -gt 0 ] && [ "$1" -le "$maximum_lines" ] || fail "snapshot line count exceeds bound: $bounded_file"
    set -- $byte_count
    [ "$#" -eq 1 ] && [ "$1" -gt 0 ] && [ "$1" -le 16777216 ] || fail "snapshot byte count exceeds bound: $bounded_file"
}

capture_pass() {
    pass=$1
    all_paths=$scratch/$pass.all-paths
    all_paths_sorted=$scratch/$pass.all-paths.sorted
    devices=$scratch/$pass.devices
    devices_sorted=$scratch/$pass.devices.sorted
    identities=$scratch/$pass.identities
    identities_sorted=$scratch/$pass.identities.sorted
    topology=$scratch/$pass.topology
    topology_sorted=$scratch/$pass.topology.sorted
    regular_paths=$scratch/$pass.regular-paths
    regular_paths_sorted=$scratch/$pass.regular-paths.sorted
    regenerated=$scratch/$pass.manifest

    unsafe=$("$finder" -P "$root" -regextype posix-extended -mindepth 1 ! -regex '/usr/local/rustup/toolchains/1\.95\.0-x86_64-unknown-linux-gnu/[A-Za-z0-9._+:/-]{1,384}' -printf x -quit) || fail 'cannot inspect closure path alphabet'
    [ -z "$unsafe" ] || fail 'closure contains a path outside the reviewed ASCII alphabet or length'
    unexpected=$("$finder" -P "$root" ! \( -type d -o -type f \) -printf x -quit) || fail 'cannot inspect closure topology'
    [ -z "$unexpected" ] || fail 'closure contains a non-directory or non-regular entry'
    linked=$("$finder" -P "$root" -type f -links +1 -printf x -quit) || fail 'cannot inspect closure link counts'
    [ -z "$linked" ] || fail 'closure contains a hardlinked regular file'
    "$finder" -P "$root" -mindepth 1 -printf '%P\n' > "$all_paths" || fail 'cannot enumerate all closure paths'
    bound_snapshot "$all_paths" 65536
    while IFS= read -r relative; do
        case "$relative" in
            '' | /* | . | .. | ./* | ../* | *//* | */./* | */../* | */. | */.. | *[!A-Za-z0-9._/+:-]*)
                fail "unsafe closure topology path: $relative"
                ;;
        esac
        [ "${#relative}" -le 384 ] || fail "oversized closure topology path: $relative"
    done < "$all_paths"
    "$sorter" "$all_paths" > "$all_paths_sorted" || fail 'cannot sort all closure paths'
    "$sorter" -c -u "$all_paths_sorted" >/dev/null 2>&1 || fail 'closure contains a path alias'
    "$finder" -P "$root" -printf '%D\n' > "$devices" || fail 'cannot enumerate closure devices'
    bound_snapshot "$devices" 65537
    "$sorter" -u "$devices" > "$devices_sorted" || fail 'cannot sort closure devices'
    device_count=$("$counter" -l < "$devices_sorted") || fail 'cannot count closure devices'
    set -- $device_count
    [ "$#" -eq 1 ] && [ "$1" -eq 1 ] || fail 'closure crosses a device boundary'
    "$finder" -P "$root" -printf '%D:%i\n' > "$identities" || fail 'cannot enumerate closure identities'
    bound_snapshot "$identities" 65537
    "$sorter" "$identities" > "$identities_sorted" || fail 'cannot sort closure identities'
    "$sorter" -c -u "$identities_sorted" >/dev/null 2>&1 || fail 'closure contains a device/inode alias'
    "$finder" -P "$root" -printf '%P|%y|%D|%i|%n|%s|%m|%T@|%C@\n' > "$topology" || fail 'cannot capture closure topology'
    bound_snapshot "$topology" 65537
    while IFS= read -r topology_line; do
        [ "${#topology_line}" -le 1023 ] || fail 'closure topology line exceeds reviewed bound'
    done < "$topology"
    "$sorter" "$topology" > "$topology_sorted" || fail 'cannot sort closure topology'
    "$finder" -P "$root" -type f -printf '%P\n' > "$regular_paths" || fail 'cannot enumerate regular closure files'
    bound_snapshot "$regular_paths" 65536
    "$sorter" "$regular_paths" > "$regular_paths_sorted" || fail 'cannot sort regular closure paths'
    "$sorter" -c -u "$regular_paths_sorted" >/dev/null 2>&1 || fail 'closure regular paths are duplicated'

    exec 6> "$regenerated" || fail 'cannot create regenerated manifest'
    pass_count=0
    pass_bytes=0
    while IFS= read -r relative; do
        file=$root/$relative
        [ -f "$file" ] && [ ! -L "$file" ] || fail "closure entry changed type: $relative"
        set -- $("$metadata" -c '%h %u' -- "$file") || fail "cannot inspect closure file: $relative"
        [ "$#" -eq 2 ] && [ "$1" -eq 1 ] || fail "closure file is hardlinked: $relative"
        digest=$(sha_file "$file")
        pass_bytes=$((pass_bytes + 67 + ${#relative}))
        [ "$pass_bytes" -le 1048576 ] || fail 'regenerated manifest exceeds reviewed bounds'
        printf '%s  %s\n' "$digest" "$relative" >&6 || fail 'cannot write regenerated manifest'
        pass_count=$((pass_count + 1))
    done < "$regular_paths_sorted"
    exec 6>&- || fail 'cannot close regenerated manifest'
    [ "$pass_count" -gt 0 ] || fail 'regenerated manifest is empty'
    regenerated_count=$("$counter" -l < "$regenerated") || fail 'cannot count regenerated manifest'
    regenerated_bytes=$("$counter" -c < "$regenerated") || fail 'cannot size regenerated manifest'
    set -- $regenerated_count
    [ "$#" -eq 1 ] && [ "$1" -eq "$pass_count" ] || fail 'regenerated manifest record count mismatch'
    set -- $regenerated_bytes
    [ "$#" -eq 1 ] && [ "$1" -eq "$pass_bytes" ] || fail 'regenerated manifest byte count mismatch'
}

capture_pass first
"$comparator" -s -- "$manifest" "$scratch/first.manifest" || fail 'candidate manifest differs from complete closure set'
capture_pass second
"$comparator" -s -- "$scratch/first.manifest" "$scratch/second.manifest" || fail 'closure manifest changed between verification passes'
"$comparator" -s -- "$scratch/first.topology.sorted" "$scratch/second.topology.sorted" || fail 'closure topology changed between verification passes'
"$comparator" -s -- "$scratch/first.identities.sorted" "$scratch/second.identities.sorted" || fail 'closure identities changed between verification passes'
"$comparator" -s -- "$manifest" "$scratch/second.manifest" || fail 'candidate manifest differs from second complete closure set'

check_nested_mounts
[ "$mountinfo_before" = "$(sha_file "$mountinfo")" ] || fail 'mount topology changed during verification'
[ "$manifest_identity_before" = "$(stat_identity "$manifest")" ] || fail 'candidate manifest identity changed during verification'
[ "$observation_identity_before" = "$(stat_identity "$observation")" ] || fail 'observation receipt identity changed during verification'
[ "$manifest_sha" = "$(sha_file "$manifest")" ] || fail 'candidate manifest bytes changed during verification'
[ "$observation_sha" = "$(sha_file "$observation")" ] || fail 'observation receipt bytes changed during verification'
[ "$script_identity_before" = "$(stat_identity "$script")" ] || fail 'verifier identity changed during verification'
[ "$verifier_sha" = "$(sha_file "$script")" ] || fail 'verifier bytes changed during verification'
recomputed_sha=$(sha_file "$scratch/first.manifest")
second_pass_sha=$(sha_file "$scratch/second.manifest")
topology_sha=$(sha_file "$scratch/first.topology.sorted")
observer_sha=$(sha_file "$observer")
[ "$recomputed_sha" = "$manifest_sha" ] && [ "$second_pass_sha" = "$manifest_sha" ] || fail 'recomputed manifest digest mismatch'
require_nonzero_sha "$verifier_sha" 'verifier output'
require_nonzero_sha "$observer_sha" 'observer output'
require_nonzero_sha "$RAR_REVIEWED_VERIFIER_TOOLS_SHA256" 'tool pins output'
require_nonzero_sha "$find_sha" 'find output'
require_nonzero_sha "$sort_sha" 'sort output'
require_nonzero_sha "$wc_sha" 'wc output'
require_nonzero_sha "$stat_sha" 'stat output'
require_nonzero_sha "$cmp_sha" 'cmp output'
require_nonzero_sha "$id_sha" 'id output'
require_nonzero_sha "$observation_sha" 'candidate receipt output'
require_nonzero_sha "$manifest_sha" 'candidate manifest output'
require_nonzero_sha "$recomputed_sha" 'recomputed manifest output'
require_nonzero_sha "$topology_sha" 'topology output'
require_nonzero_sha "$second_pass_sha" 'second pass output'

staged_receipt=$scratch/verification.receipt
exec 7> "$staged_receipt" || fail 'cannot create staged verification receipt'
printf '%s\n' \
    'schema=rar-alpha-controller-helper-closure-verification-v0' \
    'status=candidate-exact-set-verified-not-reviewed-not-ready' \
    "controller_sha=$GITHUB_SHA" \
    "source_sha=$RAR_EXPECTED_SOURCE_REVISION" \
    "repository=$GITHUB_REPOSITORY" \
    "run_id=$GITHUB_RUN_ID" \
    "run_attempt=$GITHUB_RUN_ATTEMPT" \
    "runner_os=$RAR_CI_RUNNER_IMAGE_OS" \
    "runner_image_version=$RAR_CI_RUNNER_IMAGE_VERSION" \
    "oci_image=$image" \
    "closure_root=$root" \
    "verifier_sha256=$verifier_sha" \
    "observer_sha256=$observer_sha" \
    "tool_pins_sha256=$RAR_REVIEWED_VERIFIER_TOOLS_SHA256" \
    "find_sha256=$find_sha" \
    "sort_sha256=$sort_sha" \
    "wc_sha256=$wc_sha" \
    "stat_sha256=$stat_sha" \
    "cmp_sha256=$cmp_sha" \
    "id_sha256=$id_sha" \
    "candidate_receipt_sha256=$observation_sha" \
    "candidate_manifest_sha256=$manifest_sha" \
    "recomputed_manifest_sha256=$recomputed_sha" \
    "manifest_entries=$candidate_count" \
    "manifest_bytes=$manifest_bytes" \
    "topology_sha256=$topology_sha" \
    "second_pass_sha256=$second_pass_sha" \
    'helper_compiled=false' \
    'helper_executed=false' \
    'target_compiled=false' \
    'readiness=false' >&7 || fail 'cannot write verification receipt'
exec 7>&- || fail 'cannot close verification receipt'

verification_lines=$("$counter" -l < "$staged_receipt") || fail 'cannot count verification receipt lines'
verification_bytes=$("$counter" -c < "$staged_receipt") || fail 'cannot size verification receipt'
set -- $verification_lines
[ "$#" -eq 1 ] && [ "$1" -eq 31 ] || fail 'verification receipt line count mismatch'
set -- $verification_bytes
[ "$#" -eq 1 ] && [ "$1" -gt 0 ] && [ "$1" -le 8192 ] || fail 'verification receipt exceeds reviewed bounds'
[ "$script_identity_before" = "$(stat_identity "$script")" ] || fail 'verifier identity changed before receipt publication'
[ "$verifier_sha" = "$(sha_file "$script")" ] || fail 'verifier bytes changed before receipt publication'

set -C
exec 8> "$verification_receipt" || fail 'cannot exclusively create verification receipt'
set +C
while IFS= read -r line; do
    printf '%s\n' "$line" >&8 || fail 'cannot copy validated verification receipt'
done < "$staged_receipt"
exec 8>&- || fail 'cannot close final verification receipt'
