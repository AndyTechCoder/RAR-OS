#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
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

printf '%s\n' 'Development image immutable-fixture checks passed'
