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
binding_checker=$root/tools/ci/check-controller-helper-closure-observer-run-evidence-source.sh
repo=$work/repo
reset() {
    /bin/rm -rf "$repo"
    /bin/mkdir -p "$repo/.github/workflows" "$repo/spec/alpha/lab" "$repo/tools/ci/contracts" "$repo/tools/ci/fixtures/controller-helper-closure-observer" "$repo/tools/sprint-alpha" "$repo/tools/toolchain"
    for file_path in \
        .github/workflows/controller-helper-closure-observer.yml \
        spec/alpha/lab/controller-helper-closure-observer-run-evidence-v0.fields \
        tools/ci/contracts/controller-helper-closure-observer-case-evidence-v0.fields \
        tools/ci/fixtures/controller-helper-closure-observer/run-evidence-valid.v0 \
        tools/ci/fixtures/controller-helper-closure-observer/run-evidence-malformed.v0 \
        tools/ci/fixtures/controller-helper-closure-observer/run-evidence-cases.v0 \
        tools/ci/verify-controller-helper-closure-observer-run-evidence.sh \
        tools/ci/test-controller-helper-closure-observer-run-evidence-policy.sh \
        tools/sprint-alpha/controller-helper-v0.env \
        tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock \
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
binding_check() { /bin/sh "$binding_checker" "$repo"; }

reset
check >/dev/null
binding_check >/dev/null
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
reset
/usr/bin/sed -i 's/o=size=67108864/o=size=0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-capacity check
reset
/usr/bin/sed -i 's#target=/checkout,volume-nocopy#target=/checkout#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-volume-copyup check
reset
/usr/bin/sed -i '/\/usr\/bin\/cp -a \/source\/\. \/destination\//d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-copy-missing check
reset
/usr/bin/sed -i 's/120s \/usr\/bin\/docker --config "\$docker_config" --host unix:\/\/\/var\/run\/docker.sock start/0s \/usr\/bin\/docker --config "\$docker_config" --host unix:\/\/\/var\/run\/docker.sock start/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-timeout check
reset
/usr/bin/sed -i '0,/--network none/s//--network bridge/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-network check
reset
/usr/bin/sed -i 's#target=/source,readonly#target=/source#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-source-write check
reset
/usr/bin/sed -i '/chmod -R a-w,a+rX "\$partial"/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject writer-revocation check
reset
/usr/bin/sed -i '/"\$workspace_parent"\/.rar-c2b-checkout-partial-.*)/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject partial-cleanup-identity check
reset
/usr/bin/sed -i '/remove_volume "\$volume"/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject volume-cleanup check
reset
/usr/bin/sed -i 's/--depth=1/--depth=0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject shallow-exact-fetch check
reset
/usr/bin/sed -i 's/\[\[ "\$partial_device" == "\$parent_device" \]\]/[[ "$partial_device" != "$parent_device" ]]/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject promotion-device check
reset
/usr/bin/sed -i 's/partial_inode=\$(\/usr\/bin\/stat -c %i "\$partial")/partial_inode=0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject promotion-identity check
reset
/usr/bin/sed -i 's#/usr/bin/mv -T#/usr/bin/mv#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject rename-only check
reset
/usr/bin/sed -i '0,/remove_container "\$acquisition"/s//remove_container "$acquisition" || true/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject cleanup-suppression check
reset
/usr/bin/sed -i '/\/usr\/bin\/docker --config "\$docker_config" --host unix:\/\/\/var\/run\/docker.sock container ls -a/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject container-absence-query check
reset
/usr/bin/sed -i 's/if \[\[ "\$cleanup_failed" -ne 0 \]\]; then exit 1; fi/if [[ "$cleanup_failed" -ne 0 ]]; then exit 0; fi/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject cleanup-failure check
reset
/usr/bin/sed -i 's/force_remove_created_container "\$acquisition"/remove_container "$acquisition"/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject immediate-acquisition-removal check
reset
/bin/rm "$repo/.github/workflows/controller-helper-closure-observer.yml"
binding_check >/dev/null
reset
/usr/bin/sed -i '\#^          tools/ci/verify-controller-helper-closure-observer-run-evidence\.sh \\$#d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject binding-missing binding_check
reset
/usr/bin/sed -i '\#^          tools/ci/verify-controller-helper-closure-observer-run-evidence\.sh \\$#a\          tools/ci/verify-controller-helper-closure-observer-run-evidence.sh \\' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject binding-duplicate-exact binding_check
reset
/usr/bin/printf '%s\n' '# verify-controller-helper-closure-observer-run-evidence-shadow' > "$repo/.github/workflows/foreign.yml"
reject binding-partial-foreign binding_check
reset
/usr/bin/sed -i '\#^          tools/ci/verify-controller-helper-closure-observer-run-evidence\.sh \\$#d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
/usr/bin/printf '%s\n' '          tools/ci/verify-controller-helper-closure-observer-run-evidence.sh \' > "$repo/.github/workflows/foreign.yml"
reject binding-exact-foreign binding_check
reset
/usr/bin/sed -i '\#^          tools/ci/verify-controller-helper-closure-observer-run-evidence\.sh \\$#d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
/usr/bin/sed -i '/^      - name: Run isolated harness and one candidate observation$/a\          tools/ci/verify-controller-helper-closure-observer-run-evidence.sh \\' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject binding-misplaced binding_check
reset
/usr/bin/sed -i 's/ --pull=never//g' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject workflow-pull-policy check
reset
/usr/bin/sed -i 's/ --pull=never//' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-pull-policy check
reset
/usr/bin/sed -i 's/DOCKER_HOST DOCKER_CONTEXT/DOCKER_GHOST DOCKER_CONTEXT/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject docker-endpoint-override-boundary check
reset
/usr/bin/sed -i 's/|| cleanup_failed=1/|| true/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-cleanup-suppression check
reset
/usr/bin/sed -i '/container_absent "\$container"/d' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-residual-container-proof check
reset
/usr/bin/sed -i 's/|| rc=1/|| rc=0/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-cleanup-status-preservation check
reset
/usr/bin/sed -i '/docker --config "\$docker_config" --host unix:\/\/\/var\/run\/docker.sock container ls -a --no-trunc/d' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-absence-query check
reset
/usr/bin/sed -i '0,/--host unix:\/\/\/var\/run\/docker.sock/s//--host tcp:\/\/127.0.0.1:2375/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject workflow-endpoint-binding check
reset
/usr/bin/sed -i '0,/--host unix:\/\/\/var\/run\/docker.sock/s//--host tcp:\/\/127.0.0.1:2375/' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-endpoint-binding check
reset
/usr/bin/sed -i 's/DOCKER_CERT_PATH DOCKER_CONFIG/DOCKER_CERT_PATH DOCKER_GONFIG/g' "$repo/.github/workflows/controller-helper-closure-observer.yml" "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject docker-config-selector-boundary check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --host unix:///var/run/docker.sock create --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock' >> "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject docker-socket-propagation check
reset
/usr/bin/sed -i '/ pull --quiet --platform linux\/amd64 /d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject explicit-image-acquisition check
reset
/usr/bin/sed -i 's/180s \/usr\/bin\/docker/0s \/usr\/bin\/docker/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject image-acquisition-timeout check
reset
/usr/bin/sed -i '/image inspect --format/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject image-presence-proof check
reset
/usr/bin/sed -i '/\/usr\/bin\/rmdir "\$docker_config"/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject workflow-docker-config-cleanup check
reset
/usr/bin/sed -i '/\/usr\/bin\/rmdir "\$docker_config"/d' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject wrapper-docker-config-cleanup check
reset
/usr/bin/sed -i 's#if /usr/bin/grep -Fqx -- "\$checkout_repo_digest"#if ! /usr/bin/grep -Fqx -- "$checkout_repo_digest"#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject image-inventory-branch-inversion check
reset
/usr/bin/sed -i 's/if \[\[ "\$needs_pull" -eq 1 \]\]; then/if [[ 1 -eq 1 ]]; then/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject unconditional-image-acquisition check
reset
/usr/bin/awk '
    /if \[\[ "\$needs_pull" -eq 1 \]\]; then/ { print; print "          fi"; moving=1; next }
    moving && / pull --quiet --platform linux\/amd64 / { pull=$0; next }
    moving && $0 == "          fi" { print pull; moving=0; next }
    { print }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject pull-moved-outside-guard check
reset
/usr/bin/sed -i 's/^            needs_pull=1$/            needs_pull=0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject absent-inventory-assignment check

reset
/usr/bin/sed -i 's/git -C "\$GITHUB_WORKSPACE" rev-parse HEAD/git rev-parse HEAD/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject checkout-cwd-binding check
reset
/usr/bin/sed -i 's/git -C "\$GITHUB_WORKSPACE" status/git status/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject checkout-status-cwd-binding check
reset
/usr/bin/sed -i 's#"\$GITHUB_WORKSPACE/tools/ci/run-controller-helper-closure-observer.sh"#tools/ci/run-controller-helper-closure-observer.sh#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject checkout-writability-cwd-binding check

reset
/usr/bin/sed -i 's/--workdir \/checkout/--workdir \/wrong-checkout/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-workdir check
reset
/usr/bin/sed -i 's/--workdir \/destination/--workdir \/wrong-destination/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-workdir check
reset
/usr/bin/awk '$0 != "              [ \"$(pwd -P)\" = /checkout ] || fail \"working directory\"" { print }' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-pwd-assertion check
reset
/usr/bin/awk '$0 != "              [ \"$(pwd -P)\" = /destination ] || fail \"working directory\"" { print }' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-pwd-assertion check
reset
/usr/bin/awk '
    $0 == "              [ \"$(pwd -P)\" = /checkout ] || fail \"working directory\"" { held=$0; next }
    held != "" && $0 == "              /usr/bin/git init /checkout || fail \"git init\"" { print; print held; held=""; next }
    { print }
    END { if (held != "") print held }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-pwd-order check
reset
/usr/bin/awk '
    $0 == "              [ \"$(pwd -P)\" = /destination ] || fail \"working directory\"" { held=$0; next }
    held != "" && index($0, "              actual_sha=$(/usr/bin/git -C /destination rev-parse HEAD)") == 1 { print; print held; held=""; next }
    { print }
    END { if (held != "") print held }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-pwd-order check
reset
/usr/bin/sed -i '1s#/usr/bin/bash#/bin/sh#' "$repo/tools/ci/run-controller-helper-closure-observer.sh"
reject bash-wrapper-identity check

reset
/usr/bin/sed -i 's#GIT_DIR=/checkout/.git#GIT_DIR=/wrong/.git#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-git-dir check
reset
/usr/bin/sed -i 's#GIT_DIR=/destination/.git#GIT_DIR=/wrong/.git#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-git-dir check
reset
/usr/bin/sed -i 's#GIT_WORK_TREE=/checkout#GIT_WORK_TREE=/wrong#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-git-work-tree check
reset
/usr/bin/sed -i 's#GIT_WORK_TREE=/destination#GIT_WORK_TREE=/wrong#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-git-work-tree check
reset
/usr/bin/sed -i '/\[ -d \/checkout\/\.git \].*git directory/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-git-directory-guard check
reset
/usr/bin/sed -i '/\[ -d \/destination\/\.git \].*git directory/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-git-directory-guard check
reset
/usr/bin/awk '$0 == "              export GIT_DIR GIT_WORK_TREE" { seen++; if (seen == 1) next } { print }' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-git-export check
reset
/usr/bin/awk '$0 == "              export GIT_DIR GIT_WORK_TREE" { seen++; if (seen == 2) next } { print }' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-git-export check
reset
/usr/bin/awk '
    $0 == "              export GIT_DIR GIT_WORK_TREE" && !moved { held=$0; moved=1; next }
    moved == 1 && index($0, "              /usr/bin/git -C /checkout remote add origin ") == 1 { print; print held; moved=2; next }
    { print }
    END { if (moved == 1) print held }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject acquisition-git-export-order check
reset
/usr/bin/awk '
    $0 == "              export GIT_DIR GIT_WORK_TREE" { seen++; if (seen == 2) { held=$0; next } }
    held != "" && index($0, "              actual_sha=$(/usr/bin/git -C /destination rev-parse HEAD)") == 1 { print; print held; held=""; next }
    { print }
    END { if (held != "") print held }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject transfer-git-export-order check
reset
/usr/bin/sed -i '0,/ || fail "count-objects"/s///' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject masked-git-failure check

reset
/usr/bin/sed -i 's#/usr/bin/cp -a /source/\. /destination/#/usr/bin/cp -a /source/* /destination/#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject git-metadata-copy check

reset
/usr/bin/sed -i 's/keeper_uid=65532/keeper_uid=0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-uid check
reset
/usr/bin/sed -i 's/keeper_gid=65532/keeper_gid=0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-gid check
reset
/usr/bin/sed -i 's/--name "\$keeper" --interactive/--name "$keeper"/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-interactive check
reset
/usr/bin/sed -i 's/--name "\$keeper" --interactive --read-only --network none/--name "$keeper" --interactive --read-only --network bridge/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-network check
reset
/usr/bin/sed -i 's#target=/keep,readonly,volume-nocopy#target=/keep#' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-mount check
reset
/usr/bin/sed -i '/\[ ! -r \/keep \].*volume readable/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-read-denial check
reset
/usr/bin/sed -i '/\[ ! -x \/keep \].*volume searchable/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-search-denial check
reset
/usr/bin/sed -i 's/if IFS= read -r _; then fail "unexpected input"; fi/exit 0/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-early-exit check
reset
/usr/bin/sed -i 's/start "\$keeper"/start --attach "$keeper"/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-input-attachment check
reset
/usr/bin/awk 'removed == 0 && $0 == "          keeper_policy_exact" { removed=1; next } { print }' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-lifecycle-check check
reset
/usr/bin/sed -i '/remove_container "\$keeper"/d' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-cleanup check
reset
/usr/bin/sed -i 's/true|false|0|\$keeper_uid/true|false|1|$keeper_uid/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-restart-count check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock restart "$keeper"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-restart-authority check
reset
/usr/bin/printf '%s\n' 'printf "%s\\n" "$CREDENTIAL_SECRET"' >> "$repo/tools/ci/controller-helper-closure-observer-harness.sh"
reject harness-credential-read check
reset
/usr/bin/sed -i 's/timeout-minutes: 15/timeout-minutes: 1/' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject observe-deadline check
reset
/usr/bin/sed -i 's/mode=0700/mode=0777/g' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-volume-access check

reset
/usr/bin/awk '
    $0 == "          keeper_policy_exact" { seen++; if (seen == 2) { held=$0; next } }
    held != "" && index($0, "--name \"$transfer\"") { print; print held; held=""; next }
    { print }
    END { if (held != "") print held }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-check-order check
reset
/usr/bin/sed -i '/force_remove_created_container "\$acquisition"/a\          /usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock stop "$keeper"' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-stop-between-phases check
reset
/usr/bin/sed -i '/force_remove_created_container "\$acquisition"/a\          remove_container "$keeper"' "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-remove-between-phases check
reset
/usr/bin/awk '
    $0 == "          remove_volume \"$volume\"" { after_remove=1; print; next }
    after_remove && $0 == "          container_absent \"$keeper\"" { held=$0; next }
    after_remove && held != "" && $0 == "          volume_absent \"$volume\"" { print; print held; held=""; after_remove=0; next }
    { print }
    END { if (held != "") print held }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-post-volume-absence-order check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock container stop "$keeper"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-namespaced-stop check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock container exec "$keeper" /usr/bin/true' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-namespaced-exec check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start -a "$keeper"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-short-attach check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock start --interactive "$keeper"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-long-interactive check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock container start -i "$keeper"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-namespaced-short-interactive check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock \' '  container exec "$keeper" /usr/bin/true' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject keeper-split-command check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock create --name "$identity-extra" --privileged --network host "$checkout_image" /usr/bin/true' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject extra-container-create check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock volume create "$identity-extra-volume"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject extra-volume-create check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock rm --force unrelated-container' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject extra-container-remove check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock volume rm --force unrelated-volume' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject extra-volume-remove check
reset
/usr/bin/printf '%s\n' '/usr/bin/docker --config "$docker_config" --host unix:///var/run/docker.sock image ls; /usr/bin/docker --config "$docker_config" --log-level debug --host unix:///var/run/docker.sock create --name "$identity-extra" "$checkout_image"' >> "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject double-docker-call check
reset
/usr/bin/awk '
    index($0, "Options \"type\"") {
        print "          /usr/bin/docker --config \"$docker_config\" --host unix:///var/run/docker.sock create --name \"$identity-extra\" --privileged --network host \"$checkout_image\" /usr/bin/true >/dev/null"
        next
    }
    { print }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject same-count-docker-substitution check
reset
/usr/bin/awk '
    index($0, "--name \"$acquisition\" --read-only --network bridge") {
        print
        print "            --privileged --pid=host --mount \"type=bind,source=/,target=/host\" \\"
        next
    }
    { print }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject hostile-create-continuation check
reset
/usr/bin/awk '
    index($0, "volume ls \\") {
        print
        print "              --filter \"label=never\" \\"
        next
    }
    { print }
' "$repo/.github/workflows/controller-helper-closure-observer.yml" > "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated"
/usr/bin/mv "$repo/.github/workflows/controller-helper-closure-observer.yml.mutated" "$repo/.github/workflows/controller-helper-closure-observer.yml"
reject false-volume-absence-filter check

printf '%s\n' 'controller-helper observer policy mutations passed: cases=127'
