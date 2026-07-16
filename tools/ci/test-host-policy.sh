#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
check="$root/tools/ci/check-host-policy.sh"
config="$root/.codex/config.toml"
rules="$root/.codex/rules/host-safety.rules"
output_root="$root/out"

if [ -L "$output_root" ]; then
    echo "host-policy output root must not be a symbolic link" >&2
    exit 1
fi

if [ ! -e "$output_root" ]; then
    mkdir "$output_root"
fi

[ -d "$output_root" ] || {
    echo "host-policy output root is not a directory" >&2
    exit 1
}

resolved_output_root=$(CDPATH= cd -- "$output_root" && pwd -P)
[ "$resolved_output_root" = "$root/out" ] || {
    echo "host-policy output root resolves outside the repository" >&2
    exit 1
}

work=$(mktemp -d "$output_root/host-policy-tests.XXXXXX")

"$check" "$config" "$rules" >/dev/null

sed 's/^sandbox_mode = /# sandbox_mode = /' "$config" > "$work/commented-sandbox.toml"
sed 's/^network_access = false$/network_access = true/' "$config" > "$work/network-enabled.toml"
sed 's/^\[sandbox_workspace_write\]$/[wrong_section]/' "$config" > "$work/wrong-section.toml"
sed 's/^\[sandbox_workspace_write\]$/["sandbox_workspace_write"]/' "$config" > "$work/quoted-section.toml"
sed 's/Deny every request that could affect anything outside the repository/Deny only selected requests outside the repository/' "$config" > "$work/weakened-auto-review.toml"

awk '
    { print }
    /^sandbox_mode = / { print "sandbox_mode = \"danger-full-access\"" }
' "$config" > "$work/conflicting-sandbox.toml"

awk '
    BEGIN {
        quote = sprintf("%c", 39)
        triple = quote quote quote
        print "attack = " triple
        print "[sandbox_workspace_write]"
        print "network_access = false"
        print triple
    }
    { print }
' "$config" > "$work/literal-multiline.toml"

awk '
    !changed && /pattern = \["sudo"\]/ {
        sub(/decision = "forbidden"/, "decision = \"prompt\"")
        changed = 1
    }
    { print }
' "$rules" > "$work/non-forbidden.rules"

awk '!/pattern = \["qemu-system-x86_64"\]/ { print }' "$rules" > "$work/missing-emulator.rules"

ln -sf "$config" "$work/config-link.toml"

expect_config_rejected() {
    fixture=$1
    if "$check" "$work/$fixture" "$rules" >/dev/null 2>&1; then
        echo "invalid host policy fixture unexpectedly passed: $fixture" >&2
        exit 1
    fi
}

for fixture in \
    commented-sandbox.toml \
    network-enabled.toml \
    wrong-section.toml \
    quoted-section.toml \
    weakened-auto-review.toml \
    conflicting-sandbox.toml \
    literal-multiline.toml \
    config-link.toml; do
    expect_config_rejected "$fixture"
done

for fixture in non-forbidden.rules missing-emulator.rules; do
    if "$check" "$config" "$work/$fixture" >/dev/null 2>&1; then
        echo "invalid host rule fixture unexpectedly passed: $fixture" >&2
        exit 1
    fi
done

echo "host policy negative checks passed"
