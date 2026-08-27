#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

[ "$#" -ge 1 ] || exit 1
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
images=$root/tools/rar-lab/images
fixtures=$root/spec/alpha/lab/fixtures/development-image-policy

for file in "$@"; do
    case "$file" in
        "$images/build.Containerfile"|"$images/launch-base.Containerfile"|"$images/launch.Containerfile") expected_parent=$images ;;
        "$fixtures/download-pipe.Containerfile"|"$fixtures/download-pipe-multiline.Containerfile"|"$fixtures/download-pipe-wrapper.Containerfile"|"$fixtures/latest.Containerfile") expected_parent=$fixtures ;;
        *) exit 1 ;;
    esac
    parent=$(CDPATH= cd -- "$(dirname -- "$file")" && pwd -P)
    [ "$parent" = "$expected_parent" ] || exit 1
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || exit 1
    size=$(/usr/bin/stat -f %z "$file" 2>/dev/null || /usr/bin/stat -c %s "$file")
    [ "$size" -le 65536 ] || exit 1
    /usr/bin/awk 'length($0) > 4096 { exit 1 }' "$file" || exit 1
    /usr/bin/awk '
        {
            line=$0
            sub(/[[:space:]]*\\[[:space:]]*$/, "", line)
            logical=logical " " line
            if ($0 !~ /\\[[:space:]]*$/) {
                count=split(logical, segment, ";")
                for (i=1; i<=count; i++) print segment[i]
                logical=""
            }
        }
        END {
            if (logical != "") {
                count=split(logical, segment, ";")
                for (i=1; i<=count; i++) print segment[i]
            }
        }
    ' "$file" | /usr/bin/grep -Eiq '(^|[[:space:]/])(curl|wget)([[:space:]][^|]*)?[|]' && exit 1
    ! /usr/bin/grep -Ei '(^FROM .*:latest|--privileged|--network[= ]host|apt-get[[:space:]]+upgrade|ADD[[:space:]]+https?://)' "$file" >/dev/null || exit 1
done

printf '%s\n' 'Containerfile static policy passed'
