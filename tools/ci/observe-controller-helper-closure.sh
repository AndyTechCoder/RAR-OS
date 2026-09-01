repository=$GITHUB_REPOSITORY" \
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
