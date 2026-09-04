#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'Specifications authority mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/specifications-authority.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-specifications-authority.sh
canonical=AndyTechCoder/RAR-OS

git_clean() {
    /usr/bin/env -i \
        HOME=/nonexistent-rar-specifications-test \
        PATH=/usr/bin:/bin \
        LC_ALL=C \
        LANG=C \
        GIT_CONFIG_NOSYSTEM=1 \
        GIT_OPTIONAL_LOCKS=0 \
        /usr/bin/git -c core.hooksPath=/dev/null "$@"
}

commit_all() {
    repository=$1
    message=$2
    git_clean -C "$repository" add -A
    git_clean -C "$repository" \
        -c user.name=RAR-Specifications-Test \
        -c user.email=specifications-test@invalid \
        commit -q -m "$message"
}

seed=$work/seed
/bin/mkdir -p "$seed/.github/workflows" "$seed/tools"
git_clean init -q "$seed"
/usr/bin/printf '%s\n' 'name: fixture' > "$seed/.github/workflows/specifications.yml"
/usr/bin/printf '%s\n' '#!/bin/sh' 'exit 0' > "$seed/tools/validator.sh"
/usr/bin/printf '%s\n' '*.attack' > "$seed/.gitignore"
commit_all "$seed" seed

clone_pair() {
    label=$1
    trusted=$work/$label-trusted
    source=$work/$label-source
    git_clean clone -q "$seed" "$trusted"
    git_clean clone -q "$seed" "$source"
}

run_checker() {
    trusted_root=$1
    source_root=$2
    output=$3
    controller_sha=$4
    source_sha=$5
    source_repository=$6
    canonical_repository=$7
    RAR_TRUSTED_CONTROLLER_SHA=$controller_sha \
    RAR_EXPECTED_SOURCE_REVISION=$source_sha \
    RAR_EXPECTED_SOURCE_REPOSITORY=$source_repository \
    RAR_CANONICAL_REPOSITORY=$canonical_repository \
        /bin/sh "$checker" "$trusted_root" "$source_root" "$output"
}

expect_rejected() {
    label=$1
    shift
    if run_checker "$@" >/dev/null 2>&1; then
        printf 'Specifications authority policy accepted unsafe case: %s\n' "$label" >&2
        exit 1
    fi
}

clone_pair baseline
controller_sha=$(git_clean -C "$trusted" rev-parse HEAD)
source_sha=$(git_clean -C "$source" rev-parse HEAD)
output=$work/baseline.output
run_checker "$trusted" "$source" "$output" "$controller_sha" "$source_sha" "$canonical" "$canonical" >/dev/null
[ "$(/usr/bin/sed -n '1p' "$output")" = execution=full ] || exit 1

/usr/bin/printf '%s\n' '# proposed validator change' >> "$source/tools/validator.sh"
commit_all "$source" proposed-change
source_sha=$(git_clean -C "$source" rev-parse HEAD)
output=$work/isolated-proposal.output
run_checker "$trusted" "$source" "$output" "$controller_sha" "$source_sha" "$canonical" "$canonical" >/dev/null
[ "$(/usr/bin/sed -n '1p' "$output")" = execution=isolated-proposal ] || exit 1

zeros=0000000000000000000000000000000000000000
expect_rejected malformed-controller "$trusted" "$source" "$work/malformed-controller.output" bad "$source_sha" "$canonical" "$canonical"
expect_rejected mismatched-controller "$trusted" "$source" "$work/mismatched-controller.output" "$zeros" "$source_sha" "$canonical" "$canonical"
expect_rejected mismatched-source "$trusted" "$source" "$work/mismatched-source.output" "$controller_sha" "$zeros" "$canonical" "$canonical"
expect_rejected repository-mismatch "$trusted" "$source" "$work/repository.output" "$controller_sha" "$source_sha" attacker/fork "$canonical"
expect_rejected aliased-roots "$trusted" "$trusted" "$work/aliased.output" "$controller_sha" "$controller_sha" "$canonical" "$canonical"

/bin/ln -s "$trusted" "$work/trusted-link"
expect_rejected symbolic-root "$work/trusted-link" "$source" "$work/symbolic.output" "$controller_sha" "$source_sha" "$canonical" "$canonical"
/bin/mkdir "$work/output-directory"
expect_rejected output-write-failure "$trusted" "$source" "$work/output-directory" "$controller_sha" "$source_sha" "$canonical" "$canonical"

clone_pair dirty
controller_sha=$(git_clean -C "$trusted" rev-parse HEAD)
source_sha=$(git_clean -C "$source" rev-parse HEAD)
/usr/bin/printf '%s\n' '# unstaged change' >> "$source/tools/validator.sh"
expect_rejected dirty-tracked "$trusted" "$source" "$work/dirty-tracked.output" "$controller_sha" "$source_sha" "$canonical" "$canonical"

clone_pair ignored
controller_sha=$(git_clean -C "$trusted" rev-parse HEAD)
source_sha=$(git_clean -C "$source" rev-parse HEAD)
/usr/bin/printf '%s\n' hidden > "$source/tools/ignored.attack"
expect_rejected dirty-ignored "$trusted" "$source" "$work/dirty-ignored.output" "$controller_sha" "$source_sha" "$canonical" "$canonical"

symlink_seed=$work/symlink-seed
git_clean clone -q "$seed" "$symlink_seed"
/bin/ln -s validator.sh "$symlink_seed/tools/linked-validator"
commit_all "$symlink_seed" symlink
trusted=$work/symlink-trusted
source=$work/symlink-source
git_clean clone -q "$symlink_seed" "$trusted"
git_clean clone -q "$symlink_seed" "$source"
controller_sha=$(git_clean -C "$trusted" rev-parse HEAD)
source_sha=$(git_clean -C "$source" rev-parse HEAD)
expect_rejected tracked-symlink "$trusted" "$source" "$work/tracked-symlink.output" "$controller_sha" "$source_sha" "$canonical" "$canonical"

submodule_seed=$work/submodule-seed
git_clean clone -q "$seed" "$submodule_seed"
seed_sha=$(git_clean -C "$seed" rev-parse HEAD)
git_clean -C "$submodule_seed" update-index --add --cacheinfo "160000,$seed_sha,tools/nested"
git_clean -C "$submodule_seed" \
    -c user.name=RAR-Specifications-Test \
    -c user.email=specifications-test@invalid \
    commit -q -m submodule
trusted=$work/submodule-trusted
source=$work/submodule-source
git_clean clone -q "$submodule_seed" "$trusted"
git_clean clone -q "$submodule_seed" "$source"
controller_sha=$(git_clean -C "$trusted" rev-parse HEAD)
source_sha=$(git_clean -C "$source" rev-parse HEAD)
expect_rejected tracked-submodule "$trusted" "$source" "$work/tracked-submodule.output" "$controller_sha" "$source_sha" "$canonical" "$canonical"

printf '%s\n' 'Specifications authority negative checks passed'
