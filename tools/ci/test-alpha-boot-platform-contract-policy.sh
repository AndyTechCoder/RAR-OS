git: warning: confstr() failed with code 5: couldn't get path of DARWIN_USER_TEMP_DIR; using /tmp instead
git: error: couldn't create cache file '/tmp/xcrun_db-MJlZyMzX' (errno=Operation not permitted)
2026-08-30 20:06:20.479 xcodebuild[23927:3047154]  DVTFilePathFSEvents: Failed to start fs event stream.
2026-08-30 20:06:20.610 xcodebuild[23927:3047153] [MT] DVTDeveloperPaths: Failed to get length of DARWIN_USER_CACHE_DIR from confstr(3), error = Error Domain=NSPOSIXErrorDomain Code=5 "Input/output error". Using NSCachesDirectory instead.
git: warning: confstr() failed with code 5: couldn't get path of DARWIN_USER_TEMP_DIR; using /tmp instead
git: error: couldn't create cache file '/tmp/xcrun_db-EEVz94nj' (errno=Operation not permitted)
2026-08-30 20:06:20.987 xcodebuild[23929:3047170]  DVTFilePathFSEvents: Failed to start fs event stream.
2026-08-30 20:06:21.108 xcodebuild[23929:3047169] [MT] DVTDeveloperPaths: Failed to get length of DARWIN_USER_CACHE_DIR from confstr(3), error = Error Domain=NSPOSIXErrorDomain Code=5 "Input/output error". Using NSCachesDirectory instead.
#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'Alpha boot/platform policy mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/alpha-boot-platform.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-alpha-boot-platform-contracts.sh
source=$root/spec/alpha

reset_fixture() {
    /bin/rm -rf "$work/alpha"
    /bin/mkdir -p "$work/alpha"
    /bin/cp -R "$source/boot" "$source/platform" "$work/alpha/"
    /usr/bin/find "$work/alpha" -name '._*' -type f -exec /bin/rm -f {} \;
}

reject() {
    label=$1
    if /bin/sh "$checker" "$work/alpha" >/dev/null 2>&1; then
        printf 'unsafe boot/platform mutation unexpectedly passed: %s\n' "$label" >&2
        exit 1
    fi
}

mutate() {
    file=$1
    expression=$2
    /usr/bin/sed "$expression" "$file" > "$work/bad"
    /bin/mv "$work/bad" "$file"
}

/bin/sh "$checker" >/dev/null
reset_fixture
/bin/sh "$checker" "$work/alpha" >/dev/null

reset_fixture
mutate "$work/alpha/boot/alpha-boot-v0.fields" 's/UEFI-Loaded-Image-Protocol-only/PE-header-or-pointer/'
reject inferred-root-range

reset_fixture
mutate "$work/alpha/boot/alpha-machine-closure-v0.fields" 's/^pci_function_count=13$/pci_function_count=12/'
reject missing-pci-function

reset_fixture
mutate "$work/alpha/boot/alpha-machine-closure-v0.fields" 's/maximum-polls:100000/maximum-polls:unbounded/'
reject unbounded-ahci-wait

reset_fixture
mutate "$work/alpha/platform/alpha-platform-entry-v0.fields" '/^source_role|4|/d'
reject missing-preserved-source-role

reset_fixture
mutate "$work/alpha/platform/alpha-core-bootstrap-v0.fields" 's/no-state-read/state-read/'
reject core-readable-state

reset_fixture
mutate "$work/alpha/platform/alpha-component-bundle-v0.fields" 's/total-DAG/cycles-allowed/'
reject dependency-cycle-allowed

reset_fixture
mutate "$work/alpha/platform/alpha-identities-v0.fields" 's/RAR-ALPHA-SYSTEM-SVC-ID-V0/RAR-ALPHA-PRESERVE-SVC-ID-V0/'
reject identity-role-domain-alias

reset_fixture
mutate "$work/alpha/platform/alpha-state-image-v0.fields" 's/payload-exact-hex:616263/payload-exact-hex:00000000000000000000000000000000000000000000000000000000/'
reject obsolete-preserved-fixture

reset_fixture
mutate "$work/alpha/platform/alpha-state-slots-v0.fields" '/^transition|rebindable|redeem-matching|/d'
reject incomplete-state-transition-table

reset_fixture
mutate "$work/alpha/platform/alpha-validation-v0.fields" '/^predicate|024|/d'
reject missing-identity-precedence

reset_fixture
mutate "$work/alpha/platform/cases.v0" '/^wrong-outer-identity|/d'
reject missing-single-predicate-case

reset_fixture
mutate "$work/alpha/platform/precedence.v0" '/^pair|024|039|/d'
reject missing-sensitive-precedence-pair

reset_fixture
mutate "$work/alpha/platform/fixtures/v0/preserved-state.fixture" 's/616263/616264/'
reject stale-fixture-digest

reset_fixture
/usr/bin/printf '%s\n' unexpected > "$work/alpha/platform/fixtures/v0/unexpected.fixture"
reject extra-fixture

reset_fixture
mutate "$work/alpha/platform/contract-set-v0.manifest" 's/^r0_handoff_contract_sha256=./r0_handoff_contract_sha256=f/'
reject stale-r0-binding

reset_fixture
mutate "$work/alpha/platform/contract-set-v0.manifest" 's/status=experimental-pending-review/status=ready/'
reject overstated-readiness

reset_fixture
/bin/mv "$work/alpha/platform/alpha-identities-v0.fields" "$work/alpha/platform/identities.real"
/bin/ln -s identities.real "$work/alpha/platform/alpha-identities-v0.fields"
reject symbolic-contract

printf '%s\n' 'Alpha boot/platform contract mutation checks passed'
