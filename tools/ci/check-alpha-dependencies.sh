#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)

find "$root" -path "$root/.git" -prune -o -path "$root/out" -prune -o -name Cargo.toml -type f -print | while IFS= read -r manifest; do
    ! /usr/bin/grep -Fq '\' "$manifest" || {
        printf 'Alpha dependency policy blocked noncanonical escape: %s\n' "$manifest" >&2
        exit 1
    }
    /usr/bin/awk '
        {
            compact = $0
            gsub(/[[:space:]]/, "", compact)
            if (compact ~ /^((workspace|target\.[^.]+)\.)?(dev-|build-)?dependencies\./ ||
                compact ~ /^(patch|replace)\./ ||
                compact ~ /(dev-|build-)?dependencies=\{/ ||
                compact ~ /["\047](workspace|target|patch|replace|dependencies|dev-dependencies|build-dependencies)["\047][.=]/ ||
                compact ~ /(^|[,{])["\047][^"\047]*["\047]=/) exit 1
        }
        /^[[:space:]]*\[/ {
            if ($0 !~ /^\[[A-Za-z0-9_.()'"'"'=-]+\]$/) exit 1
            if ($0 ~ /^\[(workspace\.)?(dev-|build-)?dependencies\.[^]]+\]$/ ||
                $0 ~ /^\[target\..*\.(dev-|build-)?dependencies\.[^]]+\]$/ ||
                $0 ~ /^\[patch\.[^]]+\]$/ || $0 == "[replace]") exit 1
            in_deps = ($0 ~ /^\[(workspace\.)?(dev-|build-)?dependencies\]$/ ||
                       $0 ~ /^\[target\..*\.(dev-|build-)?dependencies\]$/)
            next
        }
        /^[[:space:]]*"/ { exit 1 }
        /^[[:space:]]*((workspace|target\.[^.]+)\.)?(dev-|build-)?dependencies\./ { exit 1 }
        /^[[:space:]]*(patch|replace)\./ { exit 1 }
        in_deps && $0 !~ /^[[:space:]]*(#|$)/ {
            if ($0 !~ /^[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*\{[^}]*path[[:space:]]*=[[:space:]]*"[^"]+"[^}]*\}[[:space:]]*$/ ||
                $0 ~ /(version|git|registry|package)[[:space:]]*=/) exit 1
        }
    ' "$manifest" || {
        printf 'Alpha dependency policy blocked manifest: %s\n' "$manifest" >&2
        exit 1
    }
done

find "$root" -path "$root/.git" -prune -o -path "$root/out" -prune -o -name Cargo.toml -type f -print | while IFS= read -r manifest; do
    /usr/bin/sed -n 's/.*path[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$manifest" | while IFS= read -r relative; do
        case "$relative" in '' | /*) exit 1 ;; esac
        current=$(dirname -- "$manifest")
        old_ifs=$IFS
        IFS=/
        set -- $relative
        IFS=$old_ifs
        for component in "$@"; do
            case "$component" in '' | .) continue ;; ..) current=$(dirname -- "$current") ;; *) current=$current/$component ;; esac
            [ ! -L "$current" ] || exit 1
        done
        dependency=$(CDPATH= cd -- "$(dirname -- "$manifest")/$relative" 2>/dev/null && pwd -P) || exit 1
        case "$dependency" in "$root" | "$root"/*) ;; *) exit 1 ;; esac
        [ -f "$dependency/Cargo.toml" ] && [ ! -L "$dependency/Cargo.toml" ] || exit 1
    done || {
        printf 'Alpha dependency path escapes or is incomplete: %s\n' "$manifest" >&2
        exit 1
    }
done

find "$root" -path "$root/.git" -prune -o -path "$root/out" -prune -o -name Cargo.lock -type f -print | while IFS= read -r lock; do
    ! /usr/bin/grep -Fq '\' "$lock" || {
        printf 'Alpha dependency policy blocked noncanonical lock escape: %s\n' "$lock" >&2
        exit 1
    }
    ! /usr/bin/grep -Eq "^[[:space:]]*[\"']?(source|checksum)[\"']?[[:space:]]*=" "$lock" || {
        printf 'Alpha dependency policy blocked external lock source: %s\n' "$lock" >&2
        exit 1
    }
done

printf '%s\n' 'Alpha dependency policy passed'
