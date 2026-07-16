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
            rar_preparser_hasher_sha256=0000000000000000000000000000000000000000000000000000000000000000
            rar_preparser_wc_path=/usr/bin/wc
            rar_preparser_wc_sha256=0000000000000000000000000000000000000000000000000000000000000000
            rar_preparser_grep_path=/usr/bin/grep
            rar_preparser_grep_sha256=0000000000000000000000000000000000000000000000000000000000000000
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

rar_load_bootstrap_lock() {
    rar_lock=$1
    rar_expected_platform=$2
    rar_expected_trust=$3
    rar_validate_bounded_text_file "$rar_lock" 16384 512 || return 1
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
    while IFS='=' read -r rar_key rar_value || [ -n "$rar_key$rar_value" ]; do
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
            rustc_version | rustc_commit | rustc_llvm_version | cargo_version | cargo_commit | git_path | git_version | git_sha256 | rust_src_manifest_sha256 | aarch64_target_manifest_sha256 | thumbv8m_target_manifest_sha256 | x86_64_target_manifest_sha256 | clang_version | clang_status | lld_status | lld_version | lld_sha256 | qemu_x86_64_status | qemu_x86_64_version | qemu_x86_64_sha256 | qemu_aarch64_status | qemu_aarch64_version | qemu_aarch64_sha256 | qemu_arm_status | qemu_arm_version | qemu_arm_sha256 | firmware_x86_64_status | firmware_x86_64_id | firmware_x86_64_sha256 | firmware_aarch64_status | firmware_aarch64_id | firmware_aarch64_sha256 | certifiable | target_linked_dependencies) ;;
            *) return 1 ;;
        esac
    done < "$rar_lock"
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
            rar_load_bootstrap_lock \
                "$rar_selection_root/tools/toolchain/host-tools.lock" \
                aarch64-apple-darwin \
                owner-approved-macos-shell-hasher-axiom-v1
            ;;
        sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3)
            rar_load_bootstrap_lock \
                "$rar_selection_root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" \
                x86_64-unknown-linux-gnu \
                oci-image-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
            ;;
        *) return 1 ;;
    esac
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
    rar_hash_matches "$bootstrap_sdk_path/$bootstrap_sdk_marker_relative" "$bootstrap_sdk_settings_sha256" || return 1
    rar_verify_selected_bootstrap_closure
}

rar_prepare_output_parent() {
    rar_parent=$1
    rar_hash_matches "$bootstrap_mkdir_path" "$bootstrap_mkdir_sha256" || return 1
    "$bootstrap_mkdir_path" -p "$rar_parent"
    [ -d "$rar_parent" ] && [ ! -L "$rar_parent" ] || return 1
    rar_physical=$(CDPATH= cd -- "$rar_parent" && pwd -P) || return 1
    [ "$rar_physical" = "$rar_parent" ]
}

rar_allocate_private_directory() {
    rar_private_parent=$1
    rar_private_prefix=$2
    rar_private_sequence=0
    rar_private_directory=
    rar_hash_matches "$bootstrap_mkdir_path" "$bootstrap_mkdir_sha256" || return 1
    while [ "$rar_private_sequence" -lt 64 ]; do
        rar_private_candidate=$rar_private_parent/$rar_private_prefix-$PPID-$$-$rar_private_sequence
        if "$bootstrap_mkdir_path" -m 700 "$rar_private_candidate" 2>/dev/null; then
            rar_private_directory=$rar_private_candidate
            return 0
        fi
        rar_private_sequence=$((rar_private_sequence + 1))
    done
    return 1
}

rar_cleanup_private_directory() {
    rar_cleanup_path=$1
    case "$rar_cleanup_path" in
        "$rar_root/out/r0/host-tools/"* | "$rar_root/out/r0/host-tests/"*) ;;
        *) return 1 ;;
    esac
    rar_hash_matches "$bootstrap_rm_path" "$bootstrap_rm_sha256" || return 1
    "$bootstrap_rm_path" -rf -- "$rar_cleanup_path"
}

rar_execute_generated_host_binary() {
    rar_generated_binary=$1
    shift
    [ -f "$rar_generated_binary" ] && [ ! -L "$rar_generated_binary" ] || return 1
    rar_verify_selected_bootstrap_closure || return 1
    exec 9< "$rar_generated_binary" || return 1
    [ "${RAR_CI_BOOTSTRAP_IMAGE-}" = sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 ] || return 1
    rar_generated_descriptor=/proc/self/fd/9
    if "$rar_generated_descriptor" "$@"; then
        rar_generated_status=0
    else
        rar_generated_status=$?
    fi
    exec 9<&-
    return "$rar_generated_status"
}

rar_compile_host_rust() {
    rar_source=$1
    rar_output=$2
    shift 2
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
    "$bootstrap_env_path" -i \
        HOME="$rar_root/out/r0/tmp" \
        LANG=C \
        LC_ALL=C \
        PATH=/nonexistent-rar-bootstrap-path \
        SDKROOT="${SDKROOT-}" \
        TMPDIR="$TMPDIR" \
        "$bootstrap_rustc_path" \
        --edition 2024 \
        "$@" \
        "$rar_source" \
        -C "linker=$bootstrap_linker_path" \
        -C "linker-flavor=$bootstrap_linker_flavor" \
        -o "$rar_output"
}
