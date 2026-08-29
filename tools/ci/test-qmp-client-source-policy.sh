#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'QMP source mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/qmp-source.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
tree=$work/qmp-client
/bin/cp -R "$root/tools/rar-lab/qmp-client" "$tree"
find "$tree" -name '._*' -type f -exec /bin/rm -f {} \;
checker=$root/tools/ci/check-qmp-client-source.sh
/bin/sh "$checker" "$tree" >/dev/null

expect_rejected() {
    label=$1
    if /bin/sh "$checker" "$tree" >/dev/null 2>&1; then
        printf 'unsafe QMP source unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}
/usr/bin/printf '%s\n' 'unsafe fn bypass() {}' >> "$tree/main.rs"
expect_rejected unsafe-rust
/bin/cp "$root/tools/rar-lab/qmp-client/main.rs" "$tree/main.rs"
/usr/bin/printf '%s\n' extra > "$tree/extra.rs"
expect_rejected extra-source
/bin/rm -f "$tree/extra.rs"
/bin/mv "$tree/json.rs" "$tree/json.real"
/bin/ln -s json.real "$tree/json.rs"
expect_rejected symlink-source
/bin/rm -f "$tree/json.rs"
/bin/mv "$tree/json.real" "$tree/json.rs"
/usr/bin/sed '/^    deadline: Instant,$/d' "$tree/main.rs" > "$tree/main.rs.mutated"
/bin/mv "$tree/main.rs.mutated" "$tree/main.rs"
expect_rejected missing-cumulative-deadline
/bin/cp "$root/tools/rar-lab/qmp-client/main.rs" "$tree/main.rs"
/usr/bin/sed 's/^network=none$/network=enabled/' "$tree/build-plan.v1" > "$tree/plan"
/bin/mv "$tree/plan" "$tree/build-plan.v1"
expect_rejected networked-build-plan

controller=$work/controller
/bin/mkdir -p "$controller/tools/rar-lab"
/bin/cp -R "$root/tools/rar-lab/qmp-client" "$controller/tools/rar-lab/qmp-client"
contract=$root/tools/sprint-alpha/qmp-client-v1.env
/bin/sh "$root/tools/ci/check-qmp-client-contract.sh" "$contract" "$controller" >/dev/null
bad=$work/bad-contract.env
/usr/bin/sed 's/^binary_sha256=unavailable$/binary_sha256=ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff/' "$contract" > "$bad"
if /bin/sh "$root/tools/ci/check-qmp-client-contract.sh" "$bad" "$controller" >/dev/null 2>&1; then exit 1; fi
/usr/bin/sed 's/^state=source-ready$/state=ready/' "$contract" > "$bad"
if /bin/sh "$root/tools/ci/check-qmp-client-contract.sh" "$bad" "$controller" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'QMP client source negative checks passed'
