#!/bin/sh
set -eu

fail() {
    echo "$1" >&2
    exit 1
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
requested_config=${1:-"$root/.codex/config.toml"}
requested_rules=${2:-"$root/.codex/rules/host-safety.rules"}

resolve_repository_file() {
    path=$1

    [ -f "$path" ] || fail "missing regular policy file: $path"
    [ ! -L "$path" ] || fail "policy file must not be a symbolic link: $path"

    directory=$(dirname -- "$path")
    filename=$(basename -- "$path")
    resolved_directory=$(CDPATH= cd -- "$directory" && pwd -P)
    resolved="$resolved_directory/$filename"

    case "$resolved" in
        "$root"/*) ;;
        *) fail "policy file resolves outside the repository: $path" ;;
    esac

    printf '%s\n' "$resolved"
}

sha256_file() {
    file=$1

    if command -v sha256sum >/dev/null 2>&1; then
        LC_ALL=C sha256sum "$file" | awk '{ print $1 }'
    elif command -v shasum >/dev/null 2>&1; then
        LC_ALL=C shasum -a 256 "$file" | awk '{ print $1 }'
    else
        fail "no SHA-256 host tool is available"
    fi
}

config=$(resolve_repository_file "$requested_config")
rules=$(resolve_repository_file "$requested_rules")

if [ "$config" = "$root/.codex/config.toml" ]; then
    expected_config_sha256='790e7c9631245036ed6faf812dcaa0371d0bebce0e495e31e3b96614495d0358'
    [ "$(sha256_file "$config")" = "$expected_config_sha256" ] || fail "canonical Codex configuration integrity mismatch"
fi

if [ "$rules" = "$root/.codex/rules/host-safety.rules" ]; then
    expected_rules_sha256='37e2d927a2708643c2d0badbf3fbcc1dde56a1f4faf89f818d1000257094ebf5'
    [ "$(sha256_file "$rules")" = "$expected_rules_sha256" ] || fail "canonical host safety rules integrity mismatch"
fi

if grep -Fq "'''" "$config"; then
    fail "multiline literal strings are not permitted in the security configuration"
fi

if grep -Eq '^[[:space:]]*\["' "$config"; then
    fail "quoted table headers are not permitted in the security configuration"
fi

toml_value() {
    wanted_section=$1
    wanted_key=$2

    awk -v wanted_section="$wanted_section" -v wanted_key="$wanted_key" '
        function trim(value) {
            sub(/^[[:space:]]+/, "", value)
            sub(/[[:space:]]+$/, "", value)
            return value
        }

        BEGIN {
            section = ""
            multiline = 0
        }

        {
            line = $0

            if (multiline) {
                if (index(line, "\"\"\"") > 0) {
                    multiline = 0
                }
                next
            }

            if (line ~ /^[[:space:]]*#/ || line ~ /^[[:space:]]*$/) {
                next
            }

            if (line ~ /^[[:space:]]*\[[^]]+\][[:space:]]*$/) {
                section = line
                sub(/^[[:space:]]*\[/, "", section)
                sub(/\][[:space:]]*$/, "", section)
                next
            }

            if (index(line, "\"\"\"") > 0) {
                multiline = 1
                next
            }

            sub(/[[:space:]]+#.*/, "", line)
            pattern = "^[[:space:]]*" wanted_key "[[:space:]]*="
            if (section == wanted_section && line ~ pattern) {
                sub(pattern, "", line)
                print trim(line)
            }
        }
    ' "$config"
}

assert_setting() {
    section=$1
    key=$2
    expected=$3
    values=$(toml_value "$section" "$key")
    count=$(printf '%s\n' "$values" | awk 'NF { count++ } END { print count + 0 }')

    if [ "$count" -ne 1 ]; then
        fail "expected exactly one active [$section] $key assignment in $config"
    fi

    if [ "$values" != "$expected" ]; then
        fail "unsafe [$section] $key value in $config: expected $expected"
    fi
}

assert_setting "" "sandbox_mode" '"workspace-write"'
assert_setting "" "approval_policy" '"on-request"'
assert_setting "" "approvals_reviewer" '"auto_review"'
assert_setting "" "allow_login_shell" "false"
assert_setting "sandbox_workspace_write" "writable_roots" "[]"
assert_setting "sandbox_workspace_write" "network_access" "false"
assert_setting "sandbox_workspace_write" "exclude_tmpdir_env_var" "true"
assert_setting "sandbox_workspace_write" "exclude_slash_tmp" "true"
assert_setting "agents" "max_threads" "4"
assert_setting "agents" "max_depth" "1"

auto_review_policy=$(awk '
    BEGIN {
        section = ""
        inside = 0
        found = 0
        closed = 0
    }

    /^[[:space:]]*\[[^]]+\][[:space:]]*$/ {
        if (!inside) {
            section = $0
            sub(/^[[:space:]]*\[/, "", section)
            sub(/\][[:space:]]*$/, "", section)
        }
    }

    section == "auto_review" && /^[[:space:]]*policy[[:space:]]*=[[:space:]]*"""[[:space:]]*$/ {
        if (inside || found) {
            exit 2
        }
        inside = 1
        found = 1
        next
    }

    inside && /^[[:space:]]*"""[[:space:]]*$/ {
        inside = 0
        closed = 1
        next
    }

    inside { print }

    END {
        if (inside || found != 1 || closed != 1) {
            exit 2
        }
    }
' "$config") || fail "auto-review policy must be one closed triple-double-quoted value"

required_policy_phrases='Approve when all side effects are confined to this repository
https://github.com/AndyTechCoder/RAR-OS
Deny every request that could affect anything outside the repository
direct QEMU or other target/emulator execution
network access to any destination other than the canonical GitHub repository
changing .codex/config.toml
If command parsing, destination, path ownership, or side effects are uncertain
deny the request. Do not convert a denial into a broader alternative.'

printf '%s\n' "$required_policy_phrases" | while IFS= read -r phrase; do
    printf '%s\n' "$auto_review_policy" | grep -Fq "$phrase" || fail "auto-review policy is missing required deny semantics: $phrase"
done

if ! awk '
    /^[[:space:]]*($|#)/ { next }
    $0 !~ /^prefix_rule\(pattern = \["[^"]+"\], decision = "forbidden", justification = "[^"]+"\)$/ {
        print "invalid or non-forbidden host safety rule at line " NR ": " $0 > "/dev/stderr"
        invalid = 1
    }
    END { exit invalid ? 1 : 0 }
' "$rules"; then
    exit 1
fi

required_forbidden_commands='sudo
diskutil
bless
nvram
launchctl
osascript
defaults
security
dd
asr
hdiutil
mount
umount
installer
systemextensionsctl
kmutil
kextload
kextunload
shutdown
reboot
pmset
scutil
dscl
qemu-system-x86_64
qemu-system-aarch64
qemu-system-arm
qemu-system-i386
qemu-system-riscv32
qemu-system-riscv64
qemu-kvm
qemu
VBoxManage
vmrun
utmctl
prlctl'

printf '%s\n' "$required_forbidden_commands" | while IFS= read -r command; do
    if ! awk -v command="$command" '
        {
            prefix = "prefix_rule(pattern = [\"" command "\"], decision = \"forbidden\","
            if (index($0, prefix) == 1) {
                count++
            }
        }
        END { exit count == 1 ? 0 : 1 }
    ' "$rules"; then
        fail "missing unique forbidden prefix rule for $command"
    fi
done

echo "host policy configuration passed"
