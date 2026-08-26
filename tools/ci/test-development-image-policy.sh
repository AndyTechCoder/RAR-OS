#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
/bin/mkdir -p "$root/out"
work=$(mktemp -d "$root/out/development-images.XXXXXX")
trap '/bin/rm -rf "$work"' EXIT HUP INT TERM
checker=$root/tools/ci/check-development-image-inputs.sh
source=$root/tools/rar-lab/images/image-inputs-v1.env

/bin/sh "$checker" "$source" --require-decision-blocked >/dev/null
if /usr/bin/sed 's/^state=decision-blocked$/state=inputs-ready/' "$source" > "$work/bad" && /bin/sh "$checker" "$work/bad" >/dev/null 2>&1; then exit 1; fi
input_build_base=$(/usr/bin/sed -n 's/^build_base=//p' "$source")
/usr/bin/sed "s|^launch_base=.*$|launch_base=$input_build_base|" "$source" > "$work/bad"
if /bin/sh "$checker" "$work/bad" >/dev/null 2>&1; then exit 1; fi
/usr/bin/sed 's|^build_image=unavailable$|build_image=ghcr.io/andytechcoder/rar-os-build@sha256:2222222222222222222222222222222222222222222222222222222222222222|' "$source" > "$work/bad"
if /bin/sh "$checker" "$work/bad" >/dev/null 2>&1; then exit 1; fi

images=$work/images
/bin/cp -R "$root/tools/rar-lab/images" "$images"
find "$images" -name '._*' -type f -exec /bin/rm -f {} \;
/usr/bin/sed 's/FROM ${BUILD_BASE}/FROM rust:latest/' "$images/build.Containerfile" > "$images/bad"
/bin/mv "$images/bad" "$images/build.Containerfile"
if /bin/sh "$root/tools/ci/check-development-image-sources.sh" "$images" >/dev/null 2>&1; then exit 1; fi
printf '%s\n' 'Development image negative checks passed'
