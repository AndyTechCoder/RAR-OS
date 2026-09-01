#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'controller-helper observer policy mutations skipped: external read-only-source CI required'; exit 0; }
work=$(mktemp -d "$scratch/controller-helper-observer-policy.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-controller-helper-closure-observer-policy.sh
repo=$work/repo
reset() {
    /bin/rm -rf "$repo"
    /bin/mkdir -p "$repo/.github/workflows" "$repo/tools/ci/fixtures/controller-helper-closure-observer"
    for file_path in \
        .github/workflows/controller-helper-closure-observer.yml \
        tools/ci/run-controller-helper-closure-observer.sh \
        tools/ci/controller-helper-closure-observer-harness.sh \
        tools/ci/observe-controller-helper-closure.sh \
        tools/ci/fixtures/controller-helper-closure-observer/cases.v0 \
        tools/ci/fixtures/controller-helper-closure-observer/base-closure.v0 \
        tools/ci/fixtures/controller-helper-closure-observer/tool-pins.v0 \
        tools/ci/fixtures/controller-helper-closure-observer/expected-observation.receipt.v0; do
        /bin/cp "$root/$file_path" "$repo/$file_path"
    done
}
reject() { label=$1; shift; if "$@" >/dev/null 2>&1; then printf 'unsafe C2B policy passed: %s\n' "$label" >&2; exit 1; fi; }
check() { /bin/sh "$checker" "$repo"; }

reset
check >/dev/null
reset
/usr/bin/sed -i '/^on:/a\  workflow_dispatch:' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject workflow-dispatch check
reset
/usr/bin/sed -i 's/      - main/      - dev/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject non-main check
reset
/usr/bin/sed -i 's/contents: read/contents: write/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject write-permission check
reset
/usr/bin/printf '%s\n' '          GITHUB_TOKEN: ${{ github.token }}' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject token-bearing-checkout check
reset
/usr/bin/sed -i 's/f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3/0000000000000000000000000000000000000000000000000000000000000000/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject checkout-image-drift check
reset
/usr/bin/sed -i 's#https://github.com/AndyTechCoder/RAR-OS.git#https://example.invalid/RAR-OS.git#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject checkout-origin-drift check
reset
/usr/bin/sed -i 's/--network none/--network bridge/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject network check
reset
/usr/bin/sed -i 's/--read-only //' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject writable-root check
reset
/usr/bin/sed -i 's/target=\/workspace,readonly/target=\/workspace/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject writable-source check
reset
/usr/bin/sed -i 's/--cap-drop ALL/--cap-add SYS_ADMIN/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject capability check
reset
/usr/bin/sed -i 's/--cpus 1/--cpus 8/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject resource-vector check
reset
/usr/bin/sed -i 's/overwrite: false/overwrite: true/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject overwrite check
reset
/usr/bin/sed -i '/uses: actions\/upload-artifact@/i\        if: always()' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject upload-always check
reset
/usr/bin/sed -i 's/case_count=21/case_count=20/' "$repo/tools/ci/fixtures/controller-helper-closure-observer/cases.v0"
reject missing-case check
reset
/usr/bin/sed -i 's#/usr/bin/dash "$subject" || fail '"'"'production observer failed'"'"'#/usr/bin/dash "$subject" || fail '"'"'production observer failed'"'"'\n/usr/bin/dash "$subject" || fail '"'"'production observer failed'"'"'#' "$repo/tools/ci/controller-helper-closure-observer-harness.sh"
reject duplicate-observer check
reset
/usr/bin/sed -i 's/retention-days: 14/retention-days: 90/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject retention check
reset
/usr/bin/sed -i 's/noexec,nosuid,nodev/exec,suid,dev/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject executable-scratch check
reset
/usr/bin/sed -i '/--cap-drop ALL/a\    --privileged \\' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject privileged check
reset
/usr/bin/printf '%s\n' 'printf "%s\\n" "$GITHUB_TOKEN"' >> "$repo/tools/ci/controller-helper-closure-observer-harness.sh"
reject credential check
reset
/usr/bin/sed -i 's/target_compiled=false/target_compiled=true/' "$repo/tools/ci/fixtures/controller-helper-closure-observer/expected-observation.receipt.v0"
reject readiness-substitution check
reset
/usr/bin/sed -i '/actual_sha.*GITHUB_SHA/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject checkout-identity check
reset
/usr/bin/sed -i 's#/usr/bin/env -i#/usr/bin/env#' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject inherited-environment check
reset
/usr/bin/sed -i '/controller-helper-closure-observer-run-evidence.v0/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject artifact-set check
reset
/usr/bin/sed -i 's/done < "$subject" > "$case_subject"/done < "$fixture" > "$case_subject"/' "$repo/tools/ci/controller-helper-closure-observer-harness.sh"
reject generated-subject-binding check
reset
/usr/bin/sed -i '/hardlinked=.*cannot inspect closure hardlinks/d' "$repo/tools/ci/observe-controller-helper-closure.sh"
reject hardlink-predicate check
reset
/usr/bin/sed -i 's/RAR-C2B-BEGIN:manifest/RAR-C2B-BEGIN:wrong/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject stream-marker check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker cp "$container:/evidence/." "$evidence"' >> "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject stopped-tmpfs-copy check
reset
/usr/bin/sed -i '/pipe_status=("${PIPESTATUS\[@\]}")/d' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject stream-status check
reset
/usr/bin/sed -i 's/emit_file cases "$case_file"/emit_file receipt "$evidence\/controller-helper-closure.receipt"/' "$repo/tools/ci/controller-helper-closure-observer-harness.sh"
reject stream-order check
printf '%s\n' 'controller-helper observer policy mutations passed: cases=28'
