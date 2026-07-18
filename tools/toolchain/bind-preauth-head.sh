#!/bin/sh
set -eu

[ "$#" -eq 4 ] || {
    echo "usage: bind-preauth-head.sh <event> <event-sha> <pr-head-or-dash> <checked-out-sha>" >&2
    exit 64
}

event=$1
event_sha=$2
pr_head=$3
checked_out=$4

case "$event" in
    push) expected=$event_sha ;;
    pull_request) expected=$pr_head ;;
    *) echo "unsupported source event" >&2; exit 73 ;;
esac

case "$expected:$checked_out" in
    *[!0-9a-f:]* | :* | *:) echo "invalid source revision" >&2; exit 73 ;;
esac
[ "${#expected}" -eq 40 ] && [ "${#checked_out}" -eq 40 ] || {
    echo "invalid source revision length" >&2
    exit 73
}
[ "$expected" = "$checked_out" ] || {
    echo "checked-out source revision mismatch" >&2
    exit 73
}

printf 'source_revision=%s\n' "$checked_out"
