#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
check="$root/tools/ci/check-host-policy.sh"
config="$root/.codex/config.toml"
rules="$root/.codex/rules/host-safety.rules"
permissions="$root/.codex/rar-os-ssd-user-fragment.toml"
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
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM

"$check" "$config" "$rules" "$permissions" >/dev/null

sed 's/^default_permissions = /# default_permissions = /' "$config" > "$work/commented-profile.toml"
sed 's/^enabled = false$/enabled = true/' "$permissions" > "$work/network-enabled.toml"
sed 's/^\[permissions.rar-os-ssd.filesystem\]$/[wrong_section]/' "$permissions" > "$work/wrong-section.toml"
sed 's/^\[permissions.rar-os-ssd.filesystem\]$/["permissions.rar-os-ssd.filesystem"]/' "$permissions" > "$work/quoted-section.toml"
sed 's/Deny every request that could affect anything outside the repository/Deny only selected requests outside the repository/' "$config" > "$work/weakened-auto-review.toml"

awk '
    { print }
    /^default_permissions = / { print "default_permissions = \"some-other-profile\"" }
' "$config" > "$work/conflicting-profile.toml"

awk '
    { print }
    /^default_permissions = / { print "sandbox_mode = \"workspace-write\"" }
' "$config" > "$work/legacy-sandbox.toml"

awk '
    BEGIN {
        quote = sprintf("%c", 39)
        triple = quote quote quote
        print "attack = " triple
        print "[permissions.rar-os-ssd.network]"
        print "enabled = false"
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
awk '!/pattern = \["rustc"\]/ { print }' "$rules" > "$work/missing-compiler.rules"
awk '!/pattern = \["chmod"\]/ { print }' "$rules" > "$work/missing-permission-command.rules"

ln -sf "$config" "$work/config-link.toml"

expect_config_rejected() {
    fixture=$1
    if "$check" "$work/$fixture" "$rules" "$permissions" >/dev/null 2>&1; then
        echo "invalid host policy fixture unexpectedly passed: $fixture" >&2
        exit 1
    fi
}

expect_permissions_rejected() {
    fixture=$1
    if "$check" "$config" "$rules" "$work/$fixture" >/dev/null 2>&1; then
        echo "invalid permission-profile fixture unexpectedly passed: $fixture" >&2
        exit 1
    fi
}

for fixture in \
    commented-profile.toml \
    weakened-auto-review.toml \
    conflicting-profile.toml \
    legacy-sandbox.toml \
    literal-multiline.toml \
    config-link.toml; do
    expect_config_rejected "$fixture"
done

for fixture in network-enabled.toml wrong-section.toml quoted-section.toml; do
    expect_permissions_rejected "$fixture"
done

for fixture in non-forbidden.rules missing-emulator.rules missing-compiler.rules missing-permission-command.rules; do
    if "$check" "$config" "$work/$fixture" "$permissions" >/dev/null 2>&1; then
        echo "invalid host rule fixture unexpectedly passed: $fixture" >&2
        exit 1
    fi
done

echo "host policy negative checks passed"
