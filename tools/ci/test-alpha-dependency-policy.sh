#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'alpha dependency policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/alpha-deps.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$work/check.sh
/usr/bin/sed "s|^root=.*|root='$work/repo'|" "$root/tools/ci/check-alpha-dependencies.sh" > "$checker"
/bin/mkdir -p "$work/repo/local"
/usr/bin/printf '%s\n' '[workspace]' 'members = []' '[dependencies]' > "$work/repo/Cargo.toml"
/usr/bin/printf '%s\n' '[package]' 'name = "local"' 'version = "0.0.0"' > "$work/repo/local/Cargo.toml"
/bin/sh "$checker" >/dev/null
/usr/bin/printf '%s\n' 'local = { path = "local" }' >> "$work/repo/Cargo.toml"
/bin/sh "$checker" >/dev/null

expect_rejected() {
    line=$1
    /usr/bin/printf '%s\n' '[workspace]' 'members = []' '[dependencies]' "$line" > "$work/repo/Cargo.toml"
    if /bin/sh "$checker" >/dev/null 2>&1; then
        printf 'external dependency unexpectedly passed: %s\n' "$line" >&2
        exit 1
    fi
}
expect_rejected 'serde = "1"'
expect_rejected 'serde = { version = "1" }'
expect_rejected 'serde = { git = "https://invalid.example/repo" }'
expect_rejected 'serde = { path = "local", version = "1" }'
/usr/bin/printf '%s\n' '[dependencies.serde]' 'version = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' '[workspace.dependencies.serde]' 'version = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "[target.'cfg(unix)'.build-dependencies.serde]" 'version = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "[target.'cfg(unix)'.dev-dependencies]" 'serde = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' '[patch.crates-io]' 'serde = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' '[replace]' '"serde:1.0.0" = { path = "local" }' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' '[dependencies] # comment' 'serde = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' ' [dependencies]' 'serde = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' '["dependencies".serde]' 'version = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' 'dependencies.serde = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "target.'cfg(unix)'.build-dependencies.cc = \"1\"" > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' 'dependencies . serde = "1"' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "target . 'cfg(unix)' . build-dependencies . cc = \"1\"" > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' 'dependencies = { serde = "1" }' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "target = { 'cfg(unix)' = { dependencies = { serde = \"1\" } } }" > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "target = { 'cfg(unix)' = { \"dependencies\" = { serde = \"1\" } } }" > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' "target = { 'cfg(unix)' = { 'dependencies' = { serde = \"1\" } } }" > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' 'target = { '\''cfg(unix)'\'' = { "\u0064ependencies" = { serde = "1" } } }' > "$work/repo/Cargo.toml"
if /bin/sh "$checker" >/dev/null 2>&1; then exit 1; fi
/usr/bin/printf '%s\n' 'source = "registry+https://example.invalid"' > "$work/repo/Cargo.lock"
expect_rejected 'local = { path = "local" }'
/usr/bin/printf '%s\n' '  source="registry+https://example.invalid"' > "$work/repo/Cargo.lock"
expect_rejected 'local = { path = "local" }'
/usr/bin/printf '%s\n' '  "source" = "registry+https://example.invalid"' > "$work/repo/Cargo.lock"
expect_rejected 'local = { path = "local" }'
/usr/bin/printf '%s\n' '  "checksum" = "0000000000000000000000000000000000000000000000000000000000000000"' > "$work/repo/Cargo.lock"
expect_rejected 'local = { path = "local" }'
/usr/bin/printf '%s\n' "  'source' = \"registry+https://example.invalid\"" > "$work/repo/Cargo.lock"
expect_rejected 'local = { path = "local" }'
/usr/bin/printf '%s\n' '  "\u0073ource" = "registry+https://example.invalid"' > "$work/repo/Cargo.lock"
expect_rejected 'local = { path = "local" }'
printf '%s\n' 'Alpha dependency negative checks passed'
