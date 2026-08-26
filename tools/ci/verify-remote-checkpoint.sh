#!/bin/sh
set -eu

[ "$#" -eq 3 ] || {
    printf '%s\n' 'checkpoint verification blocked: expected tag, head, and remote records' >&2
    exit 1
}
tag=$1
head=$2
records=$3

case "$tag" in '' | *[!A-Za-z0-9._/-]*) exit 1 ;; esac
case "$tag" in sprint-alpha-rebaseline/v1 | sprint-alpha-0.1/[A-G]) ;; *) exit 1 ;; esac
case "$head" in *[!0-9a-f]*) exit 1 ;; esac
[ "${#head}" -eq 40 ] || [ "${#head}" -eq 64 ] || exit 1

printf '%s\n' "$records" | /usr/bin/awk -v tag="$tag" -v head="$head" '
    BEGIN {
        direct = "refs/tags/" tag
        peeled_ref = direct "^{}"
        width = length(head)
    }
    NF != 2 { bad = 1; next }
    length($1) != width || $1 !~ /^[0-9a-f]+$/ { bad = 1; next }
    $2 == direct { direct_count++; tag_object = $1; next }
    $2 == peeled_ref { peeled_count++; peeled = $1; next }
    { bad = 1 }
    END {
        if (bad || direct_count != 1 || peeled_count != 1 ||
            tag_object == head || peeled != head) exit 1
    }
' || exit 1

printf 'immutable checkpoint verified: tag=%s head=%s\n' "$tag" "$head"
