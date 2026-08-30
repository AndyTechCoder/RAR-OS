#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(/bin/sh "$root/tools/ci/require-ephemeral-policy-test-root.sh")
[ "$scratch" != disabled ] || { printf '%s\n' 'Development image mutations skipped: ephemeral CI required'; exit 0; }
work=$(mktemp -d "$scratch/development-image.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
input_checker=$root/tools/ci/check-development-image-inputs.sh
source_checker=$root/tools/ci/check-development-image-sources.sh
container_checker=$root/tools/ci/check-containerfile-static-policy.sh
source=$root/tools/rar-lab/images/image-inputs-v1.env
fixtures=$root/spec/alpha/lab/fixtures/development-image-policy

/bin/sh "$input_checker" "$source" --require-decision-blocked >/dev/null
/bin/sh "$source_checker" "$root/tools/rar-lab/images" >/dev/null

for fixture in inputs-ready.env aliased-bases.env populated-output.env; do
    [ -f "$fixtures/$fixture" ] && [ ! -L "$fixtures/$fixture" ] || exit 1
    if /bin/sh "$input_checker" "$fixtures/$fixture" >/dev/null 2>&1; then
        printf 'Development image policy failed: accepted %s\n' "$fixture" >&2
        exit 1
    fi
done

for fixture in download-pipe.Containerfile download-pipe-multiline.Containerfile download-pipe-wrapper.Containerfile latest.Containerfile; do
    [ -f "$fixtures/$fixture" ] && [ ! -L "$fixtures/$fixture" ] || exit 1
    if /bin/sh "$container_checker" "$fixtures/$fixture" >/dev/null 2>&1; then
        printf 'Development image policy failed: accepted %s\n' "$fixture" >&2
        exit 1
    fi
done

launch=$root/tools/rar-lab/images/launch.Containerfile
expect_launch_rejected() {
    label=$1
    instruction=$2
    candidate=$work/$label.Containerfile
    /bin/cp "$launch" "$candidate"
    /usr/bin/printf '%s\n' "$instruction" >> "$candidate"
    if RAR_POLICY_MUTATION_TESTS=1 /bin/sh "$source_checker" "$root/tools/rar-lab/images" "$candidate" >/dev/null 2>&1; then
        printf 'Development image policy accepted extra context ingress: %s\n' "$label" >&2
        exit 1
    fi
}
expect_launch_rejected json-copy 'COPY ["tools/rar-lab/qmp-client", "/controller/tools/rar-lab/qmp-client"]'
expect_launch_rejected lowercase-copy '  copy ./tools/rar-lab/qmp-client /controller/tools/rar-lab/qmp-client'
expect_launch_rejected local-add 'ADD tools/rar-lab/qmp-client /controller/tools/rar-lab/qmp-client'
expect_launch_rejected onbuild-copy 'ONBUILD COPY tools/rar-lab/qmp-client /controller/tools/rar-lab/qmp-client'
expect_launch_rejected run-mount 'RUN --mount=type=bind,source=tools/rar-lab/qmp-client,target=/tmp/qmp cp /tmp/qmp/._hidden.rs /controller/tools/rar-lab/qmp-client/._hidden.rs'
expect_launch_rejected run-mount-second-flag 'RUN --network=none --mount=type=bind,source=tools/rar-lab/qmp-client,target=/tmp/qmp cp /tmp/qmp/._hidden.rs /controller/tools/rar-lab/qmp-client/._hidden.rs'
expect_launch_rejected run-mount-continuation 'RUN --network=none \
    --mount=type=bind,source=tools/rar-lab/qmp-client,target=/tmp/qmp cp /tmp/qmp/._hidden.rs /controller/tools/rar-lab/qmp-client/._hidden.rs'

printf '%s\n' 'Development image immutable-fixture checks passed'
