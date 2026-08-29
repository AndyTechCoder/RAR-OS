#!/bin/sh

# Sourced by R0 host-only entry points. This file intentionally uses only POSIX shell
# builtins until an explicit bootstrap trust root has been selected.

rar_file_has_exact_line() {
    rar_expected_line=$2
    while IFS= read -r rar_line || [ -n "$rar_line" ]; do
        [ "$rar_line" = "$rar_expected_line" ] && return 0
    done < "$1"
    return 1
}

rar_validate_absolute_file() {
    rar_candidate=$1
    case "$rar_candidate" in
        /*) ;;
        *) return 1 ;;
    esac
    [ -f "$rar_candidate" ] && [ ! -L "$rar_candidate" ] || return 1
    rar_parent=${rar_candidate%/*}
    rar_name=${rar_candidate##*/}
    [ -n "$rar_parent" ] || rar_parent=/
    rar_physical_parent=$(CDPATH= cd -- "$rar_parent" && pwd -P) || return 1
    [ "$rar_physical_parent/$rar_name" = "$rar_candidate" ]
}

rar_validate_absolute_directory() {
    rar_candidate=$1
    case "$rar_candidate" in
        /*) ;;
        *) return 1 ;;
    esac
    [ -d "$rar_candidate" ] && [ ! -L "$rar_candidate" ] || return 1
    rar_physical=$(CDPATH= cd -- "$rar_candidate" && pwd -P) || return 1
    [ "$rar_physical" = "$rar_candidate" ]
}

rar_validate_digest() {
    [ "${#1}" -eq 64 ] || return 1
    case "$1" in
        *[!0-9a-f]*) return 1 ;;
        *) return 0 ;;
    esac
}

rar_select_preparser_axiom() {
    case "${RAR_CI_BOOTSTRAP_IMAGE-}" in
        '')
            rar_preparser_hasher_path=/usr/bin/shasum
            rar_preparser_hasher_kind=shasum-256
            rar_preparser_hasher_sha256=0812595f981a26f813d98dc380af14d4af427626c9339eda29eb849ae13de1e3
            rar_preparser_wc_path=/usr/bin/wc
            rar_preparser_wc_sha256=f2b1ce363ebe840a972de8968284d9fb00ab62119e809f2e676c5f5b1109ee47
            rar_preparser_grep_path=/usr/bin/grep
            rar_preparser_grep_sha256=2f74bf2aa5de6486424ccec68f46585e869a36722c07f03a5c03ef3778e663ca
            ;;
        sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3)
            rar_preparser_hasher_path=/usr/bin/sha256sum
            rar_preparser_hasher_kind=sha256sum
            rar_preparser_hasher_sha256=89f8c1d1ba3c76138f3771e1a91e2796ade6180b1c1e4258c04698ff32787c97
            rar_preparser_wc_path=/usr/bin/wc
            rar_preparser_wc_sha256=e8fe45a85ebdb0dade6dabf96f21dfd686c6414ff2a4a8980727076a5981d2af
            rar_preparser_grep_path=/usr/bin/grep
            rar_preparser_grep_sha256=bd6686bf7a650a9717fd7e73fdb07dc63b70547a1da41bce093c56df937a66eb
            ;;
        *) return 1 ;;
    esac
    rar_validate_absolute_file "$rar_preparser_hasher_path" || return 1
    rar_validate_absolute_file "$rar_preparser_wc_path" || return 1
    rar_validate_absolute_file "$rar_preparser_grep_path" || return 1
}

rar_preparser_hash_matches() {
    rar_preparser_file=$1
    rar_preparser_expected=$2
    case "$rar_preparser_hasher_kind" in
        shasum-256)
            rar_preparser_output=$(LC_ALL=C LANG=C "$rar_preparser_hasher_path" -a 256 "$rar_preparser_file") || return 1
            ;;
        sha256sum)
            rar_preparser_output=$(LC_ALL=C LANG=C "$rar_preparser_hasher_path" "$rar_preparser_file") || return 1
            ;;
        *) return 1 ;;
    esac
    rar_preparser_actual=${rar_preparser_output%% *}
    rar_validate_digest "$rar_preparser_actual" || return 1
    [ "$rar_preparser_actual" = "$rar_preparser_expected" ]
}

rar_preparser_hash_text_matches() {
    rar_preparser_text=$1
    rar_preparser_expected=$2
    case "$rar_preparser_hasher_kind" in
        shasum-256)
            rar_preparser_output=$(printf '%s' "$rar_preparser_text" | LC_ALL=C LANG=C "$rar_preparser_hasher_path" -a 256) || return 1
            ;;
        sha256sum)
            rar_preparser_output=$(printf '%s' "$rar_preparser_text" | LC_ALL=C LANG=C "$rar_preparser_hasher_path") || return 1
            ;;
        *) return 1 ;;
    esac
    rar_preparser_actual=${rar_preparser_output%% *}
    rar_validate_digest "$rar_preparser_actual" || return 1
    [ "$rar_preparser_actual" = "$rar_preparser_expected" ]
}

rar_capture_reviewed_lock() {
    rar_capture_path=$1
    rar_capture_expected=$2
    rar_validate_bounded_text_file "$rar_capture_path" 16384 512 || return 1
    rar_capture_newline='
'
    rar_reviewed_lock_snapshot=
    while :; do
        rar_capture_line=
        if IFS= read -r rar_capture_line; then
            rar_reviewed_lock_snapshot=$rar_reviewed_lock_snapshot$rar_capture_line$rar_capture_newline
        else
            [ -z "$rar_capture_line" ] || return 1
            break
        fi
    done < "$rar_capture_path"
    [ -n "$rar_reviewed_lock_snapshot" ] || return 1
    rar_preparser_hash_text_matches "$rar_reviewed_lock_snapshot" "$rar_capture_expected"
}

rar_verify_preparser_axiom() {
    rar_preparser_hash_matches "$rar_preparser_hasher_path" "$rar_preparser_hasher_sha256" || return 1
    rar_preparser_hash_matches "$rar_preparser_wc_path" "$rar_preparser_wc_sha256" || return 1
    rar_preparser_hash_matches "$rar_preparser_grep_path" "$rar_preparser_grep_sha256"
}

rar_validate_bounded_text_file() {
    rar_bounded_path=$1
    rar_bounded_maximum=$2
    rar_bounded_line_maximum=$3
    [ -f "$rar_bounded_path" ] && [ ! -L "$rar_bounded_path" ] || return 1
    rar_bounded_size=$(LC_ALL=C LANG=C "$rar_preparser_wc_path" -c < "$rar_bounded_path") || return 1
    set -f
    set -- $rar_bounded_size
    set +f
    [ "$#" -eq 1 ] || return 1
    case "$1" in '' | *[!0-9]*) return 1 ;; esac
    [ "$1" -le "$rar_bounded_maximum" ] || return 1
    rar_bounded_remaining=$((rar_bounded_line_maximum + 1))
    rar_bounded_pattern=
    while [ "$rar_bounded_remaining" -gt 255 ]; do
        rar_bounded_pattern=$rar_bounded_pattern'.\{255\}'
        rar_bounded_remaining=$((rar_bounded_remaining - 255))
    done
    rar_bounded_pattern=$rar_bounded_pattern".\\{$rar_bounded_remaining\\}"
    if LC_ALL=C LANG=C "$rar_preparser_grep_path" -q "$rar_bounded_pattern" "$rar_bounded_path"; then
        return 1
    else
        rar_bounded_grep_status=$?
        [ "$rar_bounded_grep_status" -eq 1 ] || return 1
    fi
}

rar_preflight_policy_records() {
    rar_policy_root=$1
    rar_select_preparser_axiom || return 1
    rar_verify_preparser_axiom || return 1
    rar_validate_bounded_text_file "$rar_policy_root/tools/toolchain/host-tools.lock" 16384 512 || return 1
    if [ -n "${RAR_CI_BOOTSTRAP_IMAGE-}" ]; then
        rar_validate_bounded_text_file "$rar_policy_root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" 16384 512 || return 1
    fi
    rar_validate_bounded_text_file "$rar_policy_root/docs/approval-record.md" 1048576 4096 || return 1
    rar_validate_bounded_text_file "$rar_policy_root/docs/tasks/release-0.md" 1048576 4096 || return 1
    rar_validate_bounded_text_file "$rar_policy_root/docs/host-safety.md" 1048576 4096
}

rar_validate_relative_path() {
    case "$1" in
        '' | /* | . | .. | ./* | ../* | *//* | */./* | */../* | */. | */..) return 1 ;;
        *) return 0 ;;
    esac
}

rar_load_bootstrap_lock_snapshot() {
    rar_lock_snapshot=$1
    rar_expected_platform=$2
    rar_expected_trust=$3
    rar_schema=
    rar_platform=
    bootstrap_trust=
    bootstrap_shell_path=
    bootstrap_shell_sha256=
    bootstrap_hasher_path=
    bootstrap_hasher_kind=
    bootstrap_hasher_sha256=
    bootstrap_mkdir_path=
    bootstrap_mkdir_sha256=
    bootstrap_rm_path=
    bootstrap_rm_sha256=
    bootstrap_env_path=
    bootstrap_env_sha256=
    bootstrap_closure_kind=
    bootstrap_rust_toolchain_root=
    bootstrap_rust_closure_manifest_relative=
    bootstrap_rust_closure_manifest_sha256=
    bootstrap_rustc_path=
    bootstrap_rustc_sha256=
    bootstrap_linker_path=
    bootstrap_linker_flavor=
    bootstrap_linker_sha256=
    bootstrap_sdk_path=
    bootstrap_sdk_marker_relative=
    bootstrap_sdk_settings_sha256=
    bootstrap_sdk_closure_manifest_relative=
    bootstrap_sdk_closure_manifest_sha256=
    bootstrap_cargo_path=
    bootstrap_cargo_sha256=
    bootstrap_git_path=
    bootstrap_git_version=
    bootstrap_git_sha256=
    rar_lock_newline='
'
    rar_lock_remaining=$rar_lock_snapshot
    while [ -n "$rar_lock_remaining" ]; do
        case "$rar_lock_remaining" in *"$rar_lock_newline"*) ;; *) return 1 ;; esac
        rar_lock_line=${rar_lock_remaining%%"$rar_lock_newline"*}
        rar_lock_remaining=${rar_lock_remaining#*"$rar_lock_newline"}
        case "$rar_lock_line" in *=*) ;; *) return 1 ;; esac
        rar_key=${rar_lock_line%%=*}
        rar_value=${rar_lock_line#*=}
        [ -n "$rar_key" ] && [ -n "$rar_value" ] || return 1
        case "$rar_value" in *=*) return 1 ;; esac
        case "$rar_key" in
            schema) [ -z "$rar_schema" ] || return 1; rar_schema=$rar_value ;;
            platform) [ -z "$rar_platform" ] || return 1; rar_platform=$rar_value ;;
            bootstrap_trust) [ -z "$bootstrap_trust" ] || return 1; bootstrap_trust=$rar_value ;;
            bootstrap_shell_path) [ -z "$bootstrap_shell_path" ] || return 1; bootstrap_shell_path=$rar_value ;;
            bootstrap_shell_sha256) [ -z "$bootstrap_shell_sha256" ] || return 1; bootstrap_shell_sha256=$rar_value ;;
            bootstrap_hasher_path) [ -z "$bootstrap_hasher_path" ] || return 1; bootstrap_hasher_path=$rar_value ;;
            bootstrap_hasher_kind) [ -z "$bootstrap_hasher_kind" ] || return 1; bootstrap_hasher_kind=$rar_value ;;
            bootstrap_hasher_sha256) [ -z "$bootstrap_hasher_sha256" ] || return 1; bootstrap_hasher_sha256=$rar_value ;;
            bootstrap_mkdir_path) [ -z "$bootstrap_mkdir_path" ] || return 1; bootstrap_mkdir_path=$rar_value ;;
            bootstrap_mkdir_sha256) [ -z "$bootstrap_mkdir_sha256" ] || return 1; bootstrap_mkdir_sha256=$rar_value ;;
            bootstrap_rm_path) [ -z "$bootstrap_rm_path" ] || return 1; bootstrap_rm_path=$rar_value ;;
            bootstrap_rm_sha256) [ -z "$bootstrap_rm_sha256" ] || return 1; bootstrap_rm_sha256=$rar_value ;;
            bootstrap_env_path) [ -z "$bootstrap_env_path" ] || return 1; bootstrap_env_path=$rar_value ;;
            bootstrap_env_sha256) [ -z "$bootstrap_env_sha256" ] || return 1; bootstrap_env_sha256=$rar_value ;;
            bootstrap_closure_kind) [ -z "$bootstrap_closure_kind" ] || return 1; bootstrap_closure_kind=$rar_value ;;
            rust_toolchain_root) [ -z "$bootstrap_rust_toolchain_root" ] || return 1; bootstrap_rust_toolchain_root=$rar_value ;;
            rust_toolchain_closure_manifest_relative) [ -z "$bootstrap_rust_closure_manifest_relative" ] || return 1; bootstrap_rust_closure_manifest_relative=$rar_value ;;
            rust_toolchain_closure_manifest_sha256) [ -z "$bootstrap_rust_closure_manifest_sha256" ] || return 1; bootstrap_rust_closure_manifest_sha256=$rar_value ;;
            rustc_path) [ -z "$bootstrap_rustc_path" ] || return 1; bootstrap_rustc_path=$rar_value ;;
            rustc_sha256) [ -z "$bootstrap_rustc_sha256" ] || return 1; bootstrap_rustc_sha256=$rar_value ;;
            host_linker_path) [ -z "$bootstrap_linker_path" ] || return 1; bootstrap_linker_path=$rar_value ;;
            host_linker_flavor) [ -z "$bootstrap_linker_flavor" ] || return 1; bootstrap_linker_flavor=$rar_value ;;
            host_linker_sha256) [ -z "$bootstrap_linker_sha256" ] || return 1; bootstrap_linker_sha256=$rar_value ;;
            host_sdk_path) [ -z "$bootstrap_sdk_path" ] || return 1; bootstrap_sdk_path=$rar_value ;;
            host_sdk_marker_relative) [ -z "$bootstrap_sdk_marker_relative" ] || return 1; bootstrap_sdk_marker_relative=$rar_value ;;
            host_sdk_settings_sha256) [ -z "$bootstrap_sdk_settings_sha256" ] || return 1; bootstrap_sdk_settings_sha256=$rar_value ;;
            host_sdk_closure_manifest_relative) [ -z "$bootstrap_sdk_closure_manifest_relative" ] || return 1; bootstrap_sdk_closure_manifest_relative=$rar_value ;;
            host_sdk_closure_manifest_sha256) [ -z "$bootstrap_sdk_closure_manifest_sha256" ] || return 1; bootstrap_sdk_closure_manifest_sha256=$rar_value ;;
            cargo_path) [ -z "$bootstrap_cargo_path" ] || return 1; bootstrap_cargo_path=$rar_value ;;
            cargo_sha256) [ -z "$bootstrap_cargo_sha256" ] || return 1; bootstrap_cargo_sha256=$rar_value ;;
            git_path) [ -z "$bootstrap_git_path" ] || return 1; bootstrap_git_path=$rar_value ;;
            git_version) [ -z "$bootstrap_git_version" ] || return 1; bootstrap_git_version=$rar_value ;;
            git_sha256) [ -z "$bootstrap_git_sha256" ] || return 1; bootstrap_git_sha256=$rar_value ;;
            rustc_version | rustc_commit | rustc_llvm_version | cargo_version | cargo_commit | rust_src_manifest_sha256 | aarch64_target_manifest_sha256 | thumbv8m_target_manifest_sha256 | x86_64_target_manifest_sha256 | clang_version | clang_status | lld_status | lld_version | lld_sha256 | qemu_x86_64_status | qemu_x86_64_version | qemu_x86_64_sha256 | qemu_aarch64_status | qemu_aarch64_version | qemu_aarch64_sha256 | qemu_arm_status | qemu_arm_version | qemu_arm_sha256 | firmware_x86_64_status | firmware_x86_64_id | firmware_x86_64_sha256 | firmware_aarch64_status | firmware_aarch64_id | firmware_aarch64_sha256 | certifiable | target_linked_dependencies) ;;
            *) return 1 ;;
        esac
    done
    [ "$rar_schema" = rar-host-tool-lock-v3 ] || return 1
    [ "$rar_platform" = "$rar_expected_platform" ] || return 1
    [ "$bootstrap_trust" = "$rar_expected_trust" ] || return 1
    rar_validate_absolute_file "$bootstrap_shell_path" || return 1
    rar_validate_absolute_file "$bootstrap_hasher_path" || return 1
    rar_validate_absolute_file "$bootstrap_mkdir_path" || return 1
    rar_validate_absolute_file "$bootstrap_rm_path" || return 1
    rar_validate_absolute_file "$bootstrap_env_path" || return 1
    rar_validate_absolute_directory "$bootstrap_rust_toolchain_root" || return 1
    rar_validate_absolute_file "$bootstrap_rustc_path" || return 1
    rar_validate_absolute_file "$bootstrap_linker_path" || return 1
    rar_validate_absolute_file "$bootstrap_cargo_path" || return 1
    rar_validate_absolute_file "$bootstrap_git_path" || return 1
    rar_validate_absolute_directory "$bootstrap_sdk_path" || return 1
    rar_validate_relative_path "$bootstrap_sdk_marker_relative" || return 1
    rar_validate_absolute_file "$bootstrap_sdk_path/$bootstrap_sdk_marker_relative" || return 1
    rar_validate_digest "$bootstrap_shell_sha256" || return 1
    rar_validate_digest "$bootstrap_hasher_sha256" || return 1
    rar_validate_digest "$bootstrap_mkdir_sha256" || return 1
    rar_validate_digest "$bootstrap_rm_sha256" || return 1
    rar_validate_digest "$bootstrap_env_sha256" || return 1
    rar_validate_digest "$bootstrap_rustc_sha256" || return 1
    rar_validate_digest "$bootstrap_linker_sha256" || return 1
    rar_validate_digest "$bootstrap_sdk_settings_sha256" || return 1
    rar_validate_digest "$bootstrap_cargo_sha256" || return 1
    rar_validate_digest "$bootstrap_git_sha256" || return 1
    [ -n "$bootstrap_git_version" ] || return 1
    case "$bootstrap_hasher_kind" in shasum-256 | sha256sum) ;; *) return 1 ;; esac
    case "$bootstrap_linker_flavor" in ld64.lld | gcc) ;; *) return 1 ;; esac
    case "$bootstrap_closure_kind" in
        sha256-manifests)
            [ "$rar_platform" = aarch64-apple-darwin ] || return 1
            rar_validate_relative_path "$bootstrap_rust_closure_manifest_relative" || return 1
            rar_validate_relative_path "$bootstrap_sdk_closure_manifest_relative" || return 1
            rar_validate_digest "$bootstrap_rust_closure_manifest_sha256" || return 1
            rar_validate_digest "$bootstrap_sdk_closure_manifest_sha256" || return 1
            rar_validate_absolute_file "$bootstrap_repository_root/$bootstrap_rust_closure_manifest_relative" || return 1
            rar_validate_absolute_file "$bootstrap_repository_root/$bootstrap_sdk_closure_manifest_relative" || return 1
            ;;
        oci-image)
            [ "$rar_platform" = x86_64-unknown-linux-gnu ] || return 1
            [ "$bootstrap_rust_closure_manifest_relative" = none ] || return 1
            [ "$bootstrap_rust_closure_manifest_sha256" = none ] || return 1
            [ "$bootstrap_sdk_closure_manifest_relative" = none ] || return 1
            [ "$bootstrap_sdk_closure_manifest_sha256" = none ] || return 1
            ;;
        *) return 1 ;;
    esac
    bootstrap_boundary=$bootstrap_trust
}

rar_load_selected_bootstrap_root() {
    rar_selection_root=$1
    bootstrap_repository_root=$rar_selection_root
    case "${RAR_CI_BOOTSTRAP_IMAGE-}" in
        '')
            rar_selected_lock=$rar_selection_root/tools/toolchain/host-tools.lock
            rar_expected_selected_lock_sha256=f7e9baf24aaff9eaa2a2032cf0a9919568cca817d6b5d0c7e6891bce05ec979a
            rar_capture_reviewed_lock "$rar_selected_lock" "$rar_expected_selected_lock_sha256" || return 1
            rar_load_bootstrap_lock_snapshot \
                "$rar_reviewed_lock_snapshot" \
                aarch64-apple-darwin \
                owner-approved-macos-shell-hasher-axiom-v1
            ;;
        sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3)
            rar_selected_lock=$rar_selection_root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock
            rar_expected_selected_lock_sha256=6752b1b21ac8fa93a671ff9444173e4c3bbc4cdcbe4cf5cd39820371dc79aa24
            rar_capture_reviewed_lock "$rar_selected_lock" "$rar_expected_selected_lock_sha256" || return 1
            rar_load_bootstrap_lock_snapshot \
                "$rar_reviewed_lock_snapshot" \
                x86_64-unknown-linux-gnu \
                oci-image-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
            ;;
        *) return 1 ;;
    esac
    bootstrap_lock_sha256=$rar_expected_selected_lock_sha256
}

rar_verify_selected_bootstrap_closure() {
    case "$bootstrap_closure_kind" in
        oci-image)
            [ "$bootstrap_boundary" = oci-image-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ]
            ;;
        sha256-manifests)
            rar_hash_matches "$bootstrap_repository_root/$bootstrap_rust_closure_manifest_relative" "$bootstrap_rust_closure_manifest_sha256" || return 1
            rar_hash_matches "$bootstrap_repository_root/$bootstrap_sdk_closure_manifest_relative" "$bootstrap_sdk_closure_manifest_sha256" || return 1
            (
                CDPATH= cd -- "$bootstrap_rust_toolchain_root" || exit 1
                case "$bootstrap_hasher_kind" in
                    shasum-256) LC_ALL=C LANG=C "$bootstrap_hasher_path" -a 256 -c "$bootstrap_repository_root/$bootstrap_rust_closure_manifest_relative" >/dev/null ;;
                    sha256sum) LC_ALL=C LANG=C "$bootstrap_hasher_path" -c "$bootstrap_repository_root/$bootstrap_rust_closure_manifest_relative" >/dev/null ;;
                    *) exit 1 ;;
                esac
            ) || return 1
            (
                CDPATH= cd -- "$bootstrap_sdk_path" || exit 1
                case "$bootstrap_hasher_kind" in
                    shasum-256) LC_ALL=C LANG=C "$bootstrap_hasher_path" -a 256 -c "$bootstrap_repository_root/$bootstrap_sdk_closure_manifest_relative" >/dev/null ;;
                    sha256sum) LC_ALL=C LANG=C "$bootstrap_hasher_path" -c "$bootstrap_repository_root/$bootstrap_sdk_closure_manifest_relative" >/dev/null ;;
                    *) exit 1 ;;
                esac
            )
            ;;
        *) return 1 ;;
    esac
}

rar_hash_matches() {
    rar_hash_path=$1
    rar_expected_hash=$2
    case "$bootstrap_hasher_kind" in
        shasum-256)
            rar_hash_output=$(LC_ALL=C LANG=C "$bootstrap_hasher_path" -a 256 "$rar_hash_path") || return 1
            ;;
        sha256sum)
            rar_hash_output=$(LC_ALL=C LANG=C "$bootstrap_hasher_path" "$rar_hash_path") || return 1
            ;;
        *) return 1 ;;
    esac
    rar_actual_hash=${rar_hash_output%% *}
    rar_validate_digest "$rar_actual_hash" || return 1
    [ "$rar_actual_hash" = "$rar_expected_hash" ]
}

rar_verify_selected_bootstrap_root() {
    # The selected shell plus hasher is the documented irreducible bootstrap axiom.
    # The hasher checks both axiom files for evidence, then every non-root executable
    # and SDK marker before any of those non-root inputs can execute.
    rar_hash_matches "$bootstrap_hasher_path" "$bootstrap_hasher_sha256" || return 1
    rar_hash_matches "$bootstrap_shell_path" "$bootstrap_shell_sha256" || return 1
    rar_hash_matches "$bootstrap_mkdir_path" "$bootstrap_mkdir_sha256" || return 1
    rar_hash_matches "$bootstrap_rm_path" "$bootstrap_rm_sha256" || return 1
    rar_hash_matches "$bootstrap_env_path" "$bootstrap_env_sha256" || return 1
    rar_hash_matches "$bootstrap_rustc_path" "$bootstrap_rustc_sha256" || return 1
    rar_hash_matches "$bootstrap_linker_path" "$bootstrap_linker_sha256" || return 1
    rar_hash_matches "$bootstrap_cargo_path" "$bootstrap_cargo_sha256" || return 1
    rar_hash_matches "$bootstrap_git_path" "$bootstrap_git_sha256" || return 1
    rar_hash_matches "$bootstrap_sdk_path/$bootstrap_sdk_marker_relative" "$bootstrap_sdk_settings_sha256" || return 1
    rar_verify_selected_bootstrap_closure
}

rar_validate_git_object_id() {
    case "${#1}" in 40 | 64) ;; *) return 1 ;; esac
    case "$1" in *[!0-9a-f]*) return 1 ;; *) return 0 ;; esac
}

rar_verify_read_only_ci_tool_mounts() {
    [ -f /proc/self/mountinfo ] && [ ! -L /proc/self/mountinfo ] || return 1
    rar_root_mount_seen=false
    while IFS=' ' read -r rar_mount_id rar_mount_parent rar_mount_device rar_mount_root rar_mount_point rar_mount_options rar_mount_rest; do
        case ",$rar_mount_options," in *,ro,*) rar_mount_read_only=true ;; *) rar_mount_read_only=false ;; esac
        if [ "$rar_mount_point" = / ]; then
            [ "$rar_mount_read_only" = true ] || return 1
            rar_root_mount_seen=true
        fi
        case "$rar_mount_point" in
            /bin | /bin/* | /lib | /lib/* | /lib64 | /lib64/* | /sbin | /sbin/* | /usr | /usr/*)
                [ "$rar_mount_read_only" = true ] || return 1
                ;;
        esac
    done < /proc/self/mountinfo
    [ "$rar_root_mount_seen" = true ]
}

rar_verify_ci_execution_boundary() {
    [ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || return 1
    [ "${GITHUB_ACTIONS-}" = true ] || return 1
    [ "${CI-}" = true ] || return 1
    [ "${RAR_CI_RUNNER_OS-}" = Linux ] || return 1
    [ "${RAR_CI_RUNNER_ARCH-}" = X64 ] || return 1
    [ "${RAR_CI_RUNNER_IMAGE_OS-}" = ubuntu24 ] || return 1
    [ "${RAR_CI_RUNNER_IMAGE_VERSION-}" = 20260823.283.1 ] || return 1
    [ "$bootstrap_lock_sha256" = 6752b1b21ac8fa93a671ff9444173e4c3bbc4cdcbe4cf5cd39820371dc79aa24 ] || return 1
    rar_validate_git_object_id "${RAR_EXPECTED_SOURCE_REVISION-}" || return 1
    rar_verify_read_only_ci_tool_mounts
}

rar_run_pinned_git() {
    "$bootstrap_env_path" -i \
        GIT_CONFIG_NOSYSTEM=1 \
        GIT_OPTIONAL_LOCKS=0 \
        GIT_TERMINAL_PROMPT=0 \
        HOME=/nonexistent-rar-bootstrap-home \
        LANG=C \
        LC_ALL=C \
        PATH=/usr/bin \
        XDG_CONFIG_HOME=/nonexistent-rar-bootstrap-config \
        "$bootstrap_git_path" \
        -c core.fsmonitor=false \
        -c core.untrackedCache=false \
        -c "safe.directory=$bootstrap_repository_root" \
        -C "$bootstrap_repository_root" \
        "$@"
}

rar_verify_ci_source_snapshot() {
    rar_hash_matches "$bootstrap_git_path" "$bootstrap_git_sha256" || return 1
    rar_git_root=$(rar_run_pinned_git rev-parse --show-toplevel) || return 1
    [ "$rar_git_root" = "$bootstrap_repository_root" ] || return 1
    bootstrap_source_revision=$(rar_run_pinned_git rev-parse --verify 'HEAD^{commit}') || return 1
    rar_validate_git_object_id "$bootstrap_source_revision" || return 1
    [ "$bootstrap_source_revision" = "${RAR_EXPECTED_SOURCE_REVISION-}" ] || return 1
    rar_run_pinned_git cat-file -e "$bootstrap_source_revision^{commit}" || return 1
    rar_git_status=$(rar_run_pinned_git status --porcelain=v1 --untracked-files=all --ignored=no) || return 1
    [ -z "$rar_git_status" ]
}

rar_materialize_git_sources() {
    rar_materialize_directory=$1
    shift
    [ $(( $# % 2 )) -eq 0 ] || return 1
    (
        CDPATH= cd -- "$rar_materialize_directory" || exit 1
        [ "$(pwd -P)" = "$rar_materialize_directory" ] || exit 1
        while [ "$#" -gt 0 ]; do
            rar_materialize_source=$1
            rar_materialize_name=$2
            shift 2
            rar_validate_relative_path "$rar_materialize_source" || exit 1
            case "$rar_materialize_name" in '' | -* | */* | . | ..) exit 1 ;; esac
            rar_run_pinned_git show "$bootstrap_source_revision:$rar_materialize_source" > "$rar_materialize_name" || exit 1
            [ -f "$rar_materialize_name" ] && [ ! -L "$rar_materialize_name" ] || exit 1
        done
    )
}

rar_prepare_output_parent() {
    rar_parent=$1
    case "$rar_parent" in "$rar_root/"*) rar_relative_parent=${rar_parent#"$rar_root/"} ;; *) return 1 ;; esac
    rar_validate_relative_path "$rar_relative_parent" || return 1
    rar_hash_matches "$bootstrap_mkdir_path" "$bootstrap_mkdir_sha256" || return 1
    (
        CDPATH= cd -- "$rar_root" || exit 1
        rar_walk_expected=$rar_root
        rar_walk_remaining=$rar_relative_parent
        while [ -n "$rar_walk_remaining" ]; do
            case "$rar_walk_remaining" in
                */*) rar_walk_component=${rar_walk_remaining%%/*}; rar_walk_remaining=${rar_walk_remaining#*/} ;;
                *) rar_walk_component=$rar_walk_remaining; rar_walk_remaining= ;;
            esac
            case "$rar_walk_component" in '' | . | ..) exit 1 ;; esac
            if [ ! -e "$rar_walk_component" ]; then
                "$bootstrap_mkdir_path" -m 700 -- "$rar_walk_component" || exit 1
            fi
            [ -d "$rar_walk_component" ] && [ ! -L "$rar_walk_component" ] || exit 1
            CDPATH= cd -- "$rar_walk_component" || exit 1
            rar_walk_expected=$rar_walk_expected/$rar_walk_component
            [ "$(pwd -P)" = "$rar_walk_expected" ] || exit 1
        done
    )
}

rar_allocate_private_directory() {
    rar_private_parent=$1
    rar_private_prefix=$2
    rar_private_sequence=0
    rar_hash_matches "$bootstrap_mkdir_path" "$bootstrap_mkdir_sha256" || return 1
    (
        CDPATH= cd -- "$rar_private_parent" || exit 1
        [ "$(pwd -P)" = "$rar_private_parent" ] || exit 1
        while [ "$rar_private_sequence" -lt 64 ]; do
            rar_private_name=$rar_private_prefix-$PPID-$$-$rar_private_sequence
            if "$bootstrap_mkdir_path" -m 700 -- "$rar_private_name" 2>/dev/null; then
                printf '%s\n' "$rar_private_parent/$rar_private_name"
                exit 0
            fi
            rar_private_sequence=$((rar_private_sequence + 1))
        done
        exit 1
    )
}

rar_cleanup_private_directory() {
    rar_cleanup_path=$1
    shift
    case "$rar_cleanup_path" in
        "$rar_root/out/r0/host-tools/"* | "$rar_root/out/r0/host-tests/"*) ;;
        *) return 1 ;;
    esac
    rar_hash_matches "$bootstrap_rm_path" "$bootstrap_rm_sha256" || return 1
    rar_cleanup_parent=${rar_cleanup_path%/*}
    rar_cleanup_name=${rar_cleanup_path##*/}
    (
        CDPATH= cd -- "$rar_cleanup_parent" || exit 1
        [ "$(pwd -P)" = "$rar_cleanup_parent" ] || exit 1
        [ -d "$rar_cleanup_name" ] && [ ! -L "$rar_cleanup_name" ] || exit 1
        CDPATH= cd -- "$rar_cleanup_name" || exit 1
        [ "$(pwd -P)" = "$rar_cleanup_path" ] || exit 1
        for rar_cleanup_file in "$@"; do
            case "$rar_cleanup_file" in '' | -* | */* | . | ..) exit 1 ;; esac
            if [ -e "$rar_cleanup_file" ] || [ -L "$rar_cleanup_file" ]; then
                [ -f "$rar_cleanup_file" ] && [ ! -L "$rar_cleanup_file" ] || exit 1
                "$bootstrap_rm_path" -f -- "$rar_cleanup_file" || exit 1
            fi
        done
        CDPATH= cd -- .. || exit 1
        "$bootstrap_rm_path" -d -- "$rar_cleanup_name"
    )
}

rar_execute_generated_host_binary() {
    rar_generated_binary=$1
    shift
    rar_generated_parent=${rar_generated_binary%/*}
    rar_generated_name=${rar_generated_binary##*/}
    case "$rar_generated_name" in '' | -* | */* | . | ..) return 1 ;; esac
    (
        CDPATH= cd -- "$rar_generated_parent" || exit 1
        [ "$(pwd -P)" = "$rar_generated_parent" ] || exit 1
        [ -f "$rar_generated_name" ] && [ ! -L "$rar_generated_name" ] || exit 1
        exec 9< "$rar_generated_name" || exit 1
        rar_verify_selected_bootstrap_closure || exit 1
        [ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || exit 1
        rar_generated_descriptor=/proc/self/fd/9
        CDPATH= cd -- "$rar_root" || exit 1
        [ "$(pwd -P)" = "$rar_root" ] || exit 1
        "$rar_generated_descriptor" "$@"
    )
}

rar_compile_host_rust() {
    rar_compile_directory=$1
    rar_source=$2
    rar_output=$3
    shift 3
    case "$rar_source" in '' | -* | */* | *[!A-Za-z0-9._-]*) return 1 ;; esac
    case "$rar_output" in '' | -* | */* | *[!A-Za-z0-9._-]*) return 1 ;; esac
    TMPDIR=$rar_root/out/r0/tmp
    export TMPDIR
    if [ -n "$bootstrap_sdk_path" ]; then
        SDKROOT=$bootstrap_sdk_path
        export SDKROOT
    else
        unset SDKROOT
    fi
    rar_hash_matches "$bootstrap_rustc_path" "$bootstrap_rustc_sha256" || return 1
    rar_hash_matches "$bootstrap_linker_path" "$bootstrap_linker_sha256" || return 1
    rar_hash_matches "$bootstrap_env_path" "$bootstrap_env_sha256" || return 1
    rar_hash_matches "$bootstrap_sdk_path/$bootstrap_sdk_marker_relative" "$bootstrap_sdk_settings_sha256" || return 1
    rar_verify_selected_bootstrap_closure || return 1
    if [ "$bootstrap_closure_kind" = oci-image ]; then
        rar_compiler_path=/usr/bin
    else
        rar_compiler_path=/nonexistent-rar-bootstrap-path
    fi
    (
        CDPATH= cd -- "$rar_compile_directory" || exit 1
        [ "$(pwd -P)" = "$rar_compile_directory" ] || exit 1
        [ -f "$rar_source" ] && [ ! -L "$rar_source" ] || exit 1
        "$bootstrap_env_path" -i \
            HOME="$rar_root/out/r0/tmp" \
            LANG=C \
            LC_ALL=C \
            PATH="$rar_compiler_path" \
            SDKROOT="${SDKROOT-}" \
            TMPDIR="$TMPDIR" \
            "$bootstrap_rustc_path" \
            --edition 2024 \
            "$@" \
            "$rar_source" \
            -C "linker=$bootstrap_linker_path" \
            -C "linker-flavor=$bootstrap_linker_flavor" \
            -o "$rar_output"
    )
}
