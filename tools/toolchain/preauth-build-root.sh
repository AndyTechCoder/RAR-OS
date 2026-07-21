#!/bin/sh
# Shared external build-root contract for Prompt 7A host-only Rust wrappers.
# This file is sourced; it deliberately has no command dispatcher.

preauth_build_refuse() {
    printf '%s:%s\n' "$PREAUTH_BUILD_DIAGNOSTIC" "$1" >&2
    exit 73
}

preauth_build_stat() {
    if stat -c '%u %a %d %i' "$1" >/dev/null 2>&1; then
        stat -c '%u %a %d %i' "$1"
    else
        stat -f '%u %Lp %d %i' "$1"
    fi
}

preauth_build_no_symlink_path() {
    preauth_path=$1
    preauth_walk=
    preauth_rest=${preauth_path#/}
    while [ -n "$preauth_rest" ]; do
        case "$preauth_rest" in
            */*) preauth_component=${preauth_rest%%/*}; preauth_rest=${preauth_rest#*/} ;;
            *) preauth_component=$preauth_rest; preauth_rest= ;;
        esac
        [ -n "$preauth_component" ] || preauth_build_refuse build-root-canonical
        preauth_walk=$preauth_walk/$preauth_component
        [ ! -L "$preauth_walk" ] || preauth_build_refuse build-root-symlink
        [ -d "$preauth_walk" ] || preauth_build_refuse build-root-missing
    done
}

preauth_build_mount_is_executable() {
    preauth_mount_path=$1
    if [ -r /proc/self/mountinfo ]; then
        /usr/bin/awk -v requested="$preauth_mount_path" '
            function decoded(value) {
                gsub(/\\040/, " ", value); gsub(/\\011/, "\t", value);
                gsub(/\\012/, "\n", value); gsub(/\\134/, "\\", value); return value
            }
            {
                mountpoint=decoded($5)
                if (mountpoint == "/" || requested == mountpoint || index(requested, mountpoint "/") == 1) {
                    if (length(mountpoint) > best) {
                        best=length(mountpoint); options=$6; super_options=""
                        for (index_field=7; index_field<=NF; index_field++) {
                            if ($index_field == "-" && index_field + 3 <= NF) {
                                super_options=$(index_field + 3); break
                            }
                        }
                    }
                }
            }
            END {
                if (best == 0 || options ~ /(^|,)noexec(,|$)/ || super_options ~ /(^|,)noexec(,|$)/) exit 1
            }
        ' /proc/self/mountinfo || preauth_build_refuse build-root-non-executable
    elif command -v mount >/dev/null 2>&1; then
        # Darwin exposes mount flags textually. The actual compiled-helper probe below
        # remains authoritative, but an explicitly noexec containing mount is refused
        # before mktemp as required by the root contract.
        preauth_mount_line=$(mount | /usr/bin/awk -v requested="$preauth_mount_path" '
            {
                marker=" on "; at=index($0, marker); if (at == 0) next
                rest=substr($0, at + length(marker)); flags=index(rest, " (")
                if (flags == 0) next; point=substr(rest, 1, flags - 1)
                if (point == "/" || requested == point || index(requested, point "/") == 1)
                    if (length(point) > best) { best=length(point); line=$0 }
            }
            END { if (best != 0) print line }
        ')
        [ -n "$preauth_mount_line" ] || preauth_build_refuse build-root-mount
        case "$preauth_mount_line" in *noexec*) preauth_build_refuse build-root-non-executable;; esac
    else
        preauth_build_refuse build-root-mount
    fi
}

preauth_build_pinned_rustc_path() {
    preauth_pinned_root=$1
    . "$preauth_pinned_root/tools/rarbuild/bootstrap-lib.sh" || return 1
    rar_select_preparser_axiom || return 1
    rar_load_selected_bootstrap_root "$preauth_pinned_root" || return 1
    rar_verify_selected_bootstrap_root || return 1
    printf '%s\n' "$bootstrap_rustc_path"
}

preauth_build_root_create() {
    PREAUTH_BUILD_REPOSITORY=$1
    PREAUTH_BUILD_PREFIX=$2
    PREAUTH_BUILD_DIAGNOSTIC=$3
    case "$PREAUTH_BUILD_PREFIX" in ''|*[!A-Za-z0-9-]*) preauth_build_refuse build-prefix;; esac

    preauth_selected=${RAR_PREAUTH_BUILD_ROOT:-${TMPDIR:-/tmp}}
    case "$preauth_selected" in /*) :;; *) preauth_build_refuse build-root-relative;; esac
    while [ "$preauth_selected" != / ] && [ "${preauth_selected%/}" != "$preauth_selected" ]; do
        preauth_selected=${preauth_selected%/}
    done
    [ -d "$preauth_selected" ] || {
        [ -e "$preauth_selected" ] && preauth_build_refuse build-root-not-directory
        preauth_build_refuse build-root-missing
    }
    [ ! -L "$preauth_selected" ] || preauth_build_refuse build-root-symlink
    preauth_build_no_symlink_path "$preauth_selected"
    preauth_canonical=$(CDPATH= cd -- "$preauth_selected" 2>/dev/null && pwd -P) || preauth_build_refuse build-root-canonical
    [ "$preauth_canonical" = "$preauth_selected" ] || preauth_build_refuse build-root-canonical

    case "$preauth_canonical/" in "$PREAUTH_BUILD_REPOSITORY/"*) preauth_build_refuse build-root-repository;; esac
    case "$PREAUTH_BUILD_REPOSITORY/" in "$preauth_canonical/"*) preauth_build_refuse build-root-repository-ancestor;; esac
    if find "$PREAUTH_BUILD_REPOSITORY" -type d -samefile "$preauth_canonical" -print -quit 2>/dev/null | grep -q .; then
        preauth_build_refuse build-root-repository-alias
    fi

    set -- $(preauth_build_stat "$preauth_canonical") || preauth_build_refuse build-root-stat
    [ "$#" -eq 4 ] || preauth_build_refuse build-root-stat
    preauth_owner=$1; preauth_mode=$2; PREAUTH_BUILD_ROOT_IDENTITY="$3:$4"
    preauth_uid=$(id -u) || preauth_build_refuse build-root-owner
    if [ "$preauth_owner" = "$preauth_uid" ]; then
        [ "$preauth_mode" = 700 ] || preauth_build_refuse build-root-mode
    else
        [ "$preauth_owner" = 0 ] && [ "$preauth_mode" = 1777 ] || preauth_build_refuse build-root-owner
    fi
    [ -w "$preauth_canonical" ] && [ -x "$preauth_canonical" ] || preauth_build_refuse build-root-access
    preauth_build_mount_is_executable "$preauth_canonical"

    CDPATH= cd -- "$preauth_canonical" || preauth_build_refuse build-root-canonical
    [ "$(pwd -P)" = "$preauth_canonical" ] || preauth_build_refuse build-root-canonical
    set -- $(preauth_build_stat .) || preauth_build_refuse build-root-stat
    [ "$3:$4" = "$PREAUTH_BUILD_ROOT_IDENTITY" ] || preauth_build_refuse build-root-race

    preauth_existing_leaves=$(find . -maxdepth 1 -type d -name "$PREAUTH_BUILD_PREFIX.????????" -print | LC_ALL=C sort)
    PREAUTH_BUILD_LEAF_RELATIVE=$(mktemp -d "./$PREAUTH_BUILD_PREFIX.XXXXXXXX") || preauth_build_refuse build-leaf-create
    case "$PREAUTH_BUILD_LEAF_RELATIVE" in "./$PREAUTH_BUILD_PREFIX."????????) :;; *) preauth_build_refuse build-leaf-template;; esac
    if printf '%s\n' "$preauth_existing_leaves" | grep -Fxq "$PREAUTH_BUILD_LEAF_RELATIVE"; then
        preauth_build_refuse build-leaf-collision
    fi
    [ -d "$PREAUTH_BUILD_LEAF_RELATIVE" ] && [ ! -L "$PREAUTH_BUILD_LEAF_RELATIVE" ] || preauth_build_refuse build-leaf-identity
    set -- $(preauth_build_stat "$PREAUTH_BUILD_LEAF_RELATIVE") || preauth_build_refuse build-leaf-stat
    [ "$1" = "$preauth_uid" ] && [ "$2" = 700 ] || preauth_build_refuse build-leaf-mode
    PREAUTH_BUILD_LEAF_IDENTITY="$3:$4"
    PREAUTH_BUILD_DIR=$preauth_canonical/${PREAUTH_BUILD_LEAF_RELATIVE#./}
}

preauth_build_root_probe_executable() {
    preauth_build_run_child "$1" --build-root-exec-probe >/dev/null 2>&1 \
        || preauth_build_refuse build-root-non-executable
}

preauth_build_child_pid=
preauth_build_child_registering=0
preauth_build_child_waiting=0
preauth_build_pending_signal=
preauth_build_shutdown=0

preauth_build_queue_or_signal() {
    preauth_build_requested_signal=$1
    if [ "$preauth_build_shutdown" -eq 1 ]; then
        return 0
    fi
    if [ "$preauth_build_child_registering" -eq 1 ] || [ "$preauth_build_child_waiting" -eq 1 ]; then
        [ -n "$preauth_build_pending_signal" ] || preauth_build_pending_signal=$preauth_build_requested_signal
        return 0
    fi
    preauth_build_signal "$preauth_build_requested_signal"
}

preauth_build_signal() {
    preauth_build_signal_number=$1
    [ "$preauth_build_shutdown" -eq 0 ] || return 0
    preauth_build_shutdown=1
    # The first terminating signal owns the final status. Ignore repeats while
    # shutdown is in progress instead of permitting a later signal to interrupt
    # cleanup or replace the original result.
    trap '' 1 2 15
    if [ -n "${preauth_build_child_pid-}" ]; then
        # The registered PID remains an unreaped direct child throughout this
        # sequence, so it cannot be reused between signals. Do not use kill -0:
        # an already-exited child is a harmless zombie until the final wait.
        kill -"$preauth_build_signal_number" "$preauth_build_child_pid" 2>/dev/null || :
        sleep 1
        kill -15 "$preauth_build_child_pid" 2>/dev/null || :
        sleep 1
        kill -9 "$preauth_build_child_pid" 2>/dev/null || :
        set +e
        wait "$preauth_build_child_pid" 2>/dev/null || :
        set -e
        preauth_build_child_pid=
    fi
    preauth_build_root_cleanup || :
    exit $((128 + preauth_build_signal_number))
}

preauth_build_install_traps() {
    trap 'preauth_build_root_cleanup' 0
    trap 'preauth_build_queue_or_signal 1' 1
    trap 'preauth_build_queue_or_signal 2' 2
    trap 'preauth_build_queue_or_signal 15' 15
}

preauth_build_run_child() {
    preauth_build_child_registering=1
    "$@" &
    preauth_build_child_pid=$!
    preauth_build_child_registering=0
    if [ -n "$preauth_build_pending_signal" ]; then
        preauth_build_queued_signal=$preauth_build_pending_signal
        preauth_build_pending_signal=
        preauth_build_signal "$preauth_build_queued_signal"
    fi
    preauth_build_child_waiting=1
    set +e
    wait "$preauth_build_child_pid"
    preauth_build_child_status=$?
    set -e
    preauth_build_child_waiting=0
    if [ -n "$preauth_build_pending_signal" ]; then
        preauth_build_queued_signal=$preauth_build_pending_signal
        preauth_build_pending_signal=
        # A normal child completion has already been reaped, so clear its PID
        # before honoring a signal delivered at that boundary. An interrupted
        # wait leaves the child registered for bounded escalation.
        if [ "$preauth_build_child_status" -lt 128 ]; then
            preauth_build_child_pid=
        fi
        preauth_build_signal "$preauth_build_queued_signal"
    fi
    preauth_build_child_pid=
    return "$preauth_build_child_status"
}

preauth_build_run_child_in_directory() {
    preauth_build_child_directory=$1
    shift
    preauth_build_child_registering=1
    (
        CDPATH= cd -- "$preauth_build_child_directory" || exit 73
        exec "$@"
    ) &
    preauth_build_child_pid=$!
    preauth_build_child_registering=0
    if [ -n "$preauth_build_pending_signal" ]; then
        preauth_build_queued_signal=$preauth_build_pending_signal
        preauth_build_pending_signal=
        preauth_build_signal "$preauth_build_queued_signal"
    fi
    preauth_build_child_waiting=1
    set +e
    wait "$preauth_build_child_pid"
    preauth_build_child_status=$?
    set -e
    preauth_build_child_waiting=0
    if [ -n "$preauth_build_pending_signal" ]; then
        preauth_build_queued_signal=$preauth_build_pending_signal
        preauth_build_pending_signal=
        if [ "$preauth_build_child_status" -lt 128 ]; then
            preauth_build_child_pid=
        fi
        preauth_build_signal "$preauth_build_queued_signal"
    fi
    preauth_build_child_pid=
    return "$preauth_build_child_status"
}

preauth_build_root_cleanup() {
    [ -n "${PREAUTH_BUILD_LEAF_RELATIVE-}" ] || return 0
    if [ -e "$PREAUTH_BUILD_LEAF_RELATIVE" ] || [ -L "$PREAUTH_BUILD_LEAF_RELATIVE" ]; then
        [ -d "$PREAUTH_BUILD_LEAF_RELATIVE" ] && [ ! -L "$PREAUTH_BUILD_LEAF_RELATIVE" ] || {
            printf '%s:%s\n' "$PREAUTH_BUILD_DIAGNOSTIC" build-leaf-cleanup-identity >&2
            return 1
        }
        set -- $(preauth_build_stat "$PREAUTH_BUILD_LEAF_RELATIVE") || return 1
        [ "$3:$4" = "$PREAUTH_BUILD_LEAF_IDENTITY" ] || {
            printf '%s:%s\n' "$PREAUTH_BUILD_DIAGNOSTIC" build-leaf-cleanup-identity >&2
            return 1
        }
        rm -rf -- "$PREAUTH_BUILD_LEAF_RELATIVE" || return 1
    fi
    PREAUTH_BUILD_LEAF_RELATIVE=
}
