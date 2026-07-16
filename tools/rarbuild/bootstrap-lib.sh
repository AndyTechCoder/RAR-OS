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

rar_load_local_bootstrap_root() {
    rar_lock=$1/tools/toolchain/host-tools.lock
    [ -f "$rar_lock" ] && [ ! -L "$rar_lock" ] || return 1
    rar_schema=
    bootstrap_shell_path=
    bootstrap_shell_sha256=
    bootstrap_mkdir_path=
    bootstrap_mkdir_sha256=
    bootstrap_rustc_path=
    bootstrap_rustc_sha256=
    bootstrap_linker_path=
    bootstrap_linker_flavor=
    bootstrap_linker_sha256=
    bootstrap_sdk_path=
    bootstrap_sdk_settings_sha256=
    while IFS='=' read -r rar_key rar_value || [ -n "$rar_key$rar_value" ]; do
        [ -n "$rar_key" ] && [ -n "$rar_value" ] || return 1
        case "$rar_value" in *=*) return 1 ;; esac
        case "$rar_key" in
            schema) [ -z "$rar_schema" ] || return 1; rar_schema=$rar_value ;;
            bootstrap_shell_path) [ -z "$bootstrap_shell_path" ] || return 1; bootstrap_shell_path=$rar_value ;;
            bootstrap_shell_sha256) [ -z "$bootstrap_shell_sha256" ] || return 1; bootstrap_shell_sha256=$rar_value ;;
            bootstrap_mkdir_path) [ -z "$bootstrap_mkdir_path" ] || return 1; bootstrap_mkdir_path=$rar_value ;;
            bootstrap_mkdir_sha256) [ -z "$bootstrap_mkdir_sha256" ] || return 1; bootstrap_mkdir_sha256=$rar_value ;;
            rustc_path) [ -z "$bootstrap_rustc_path" ] || return 1; bootstrap_rustc_path=$rar_value ;;
            rustc_sha256) [ -z "$bootstrap_rustc_sha256" ] || return 1; bootstrap_rustc_sha256=$rar_value ;;
            host_linker_path) [ -z "$bootstrap_linker_path" ] || return 1; bootstrap_linker_path=$rar_value ;;
            host_linker_flavor) [ -z "$bootstrap_linker_flavor" ] || return 1; bootstrap_linker_flavor=$rar_value ;;
            host_linker_sha256) [ -z "$bootstrap_linker_sha256" ] || return 1; bootstrap_linker_sha256=$rar_value ;;
            host_sdk_path) [ -z "$bootstrap_sdk_path" ] || return 1; bootstrap_sdk_path=$rar_value ;;
            host_sdk_settings_sha256) [ -z "$bootstrap_sdk_settings_sha256" ] || return 1; bootstrap_sdk_settings_sha256=$rar_value ;;
        esac
    done < "$rar_lock"
    [ "$rar_schema" = rar-host-tool-lock-v2 ] || return 1
    rar_validate_absolute_file "$bootstrap_shell_path" || return 1
    rar_validate_absolute_file "$bootstrap_mkdir_path" || return 1
    rar_validate_absolute_file "$bootstrap_rustc_path" || return 1
    rar_validate_absolute_file "$bootstrap_linker_path" || return 1
    rar_validate_absolute_directory "$bootstrap_sdk_path" || return 1
    rar_validate_absolute_file "$bootstrap_sdk_path/SDKSettings.json" || return 1
    rar_validate_digest "$bootstrap_shell_sha256" || return 1
    rar_validate_digest "$bootstrap_mkdir_sha256" || return 1
    rar_validate_digest "$bootstrap_rustc_sha256" || return 1
    rar_validate_digest "$bootstrap_linker_sha256" || return 1
    rar_validate_digest "$bootstrap_sdk_settings_sha256" || return 1
    [ "$bootstrap_linker_flavor" = ld64.lld ] || return 1
    bootstrap_boundary=owner-reviewed-local-path-and-sha256-record
}

rar_load_test_bootstrap_root() {
    if [ "${RAR_CI_BOOTSTRAP_IMAGE-}" = "sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3" ]; then
        bootstrap_shell_path=/usr/bin/dash
        bootstrap_mkdir_path=/usr/bin/mkdir
        bootstrap_rustc_path=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc
        bootstrap_linker_path=/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld
        bootstrap_linker_flavor=gnu-lld
        bootstrap_sdk_path=
        bootstrap_boundary=oci-image-sha256-f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3
        rar_validate_absolute_file "$bootstrap_shell_path" || return 1
        rar_validate_absolute_file "$bootstrap_mkdir_path" || return 1
        rar_validate_absolute_file "$bootstrap_rustc_path" || return 1
        rar_validate_absolute_file "$bootstrap_linker_path" || return 1
    else
        rar_load_local_bootstrap_root "$1"
    fi
}

rar_prepare_output_parent() {
    rar_parent=$1
    "$bootstrap_mkdir_path" -p "$rar_parent"
    [ -d "$rar_parent" ] && [ ! -L "$rar_parent" ] || return 1
    rar_physical=$(CDPATH= cd -- "$rar_parent" && pwd -P) || return 1
    [ "$rar_physical" = "$rar_parent" ]
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
    "$bootstrap_rustc_path" \
        --edition 2024 \
        "$@" \
        "$rar_source" \
        -C "linker=$bootstrap_linker_path" \
        -C "linker-flavor=$bootstrap_linker_flavor" \
        -o "$rar_output"
}
