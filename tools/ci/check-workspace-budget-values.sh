#!/bin/sh
set -eu

[ "$#" -eq 6 ] || exit 1
free_kib=$1
workspace_kib=$2
output_kib=$3
minimum_free_kib=$4
maximum_workspace_kib=$5
maximum_output_kib=$6
for value in "$free_kib" "$workspace_kib" "$output_kib" "$minimum_free_kib" "$maximum_workspace_kib" "$maximum_output_kib"; do
    case "$value" in '' | *[!0-9]*) exit 1 ;; esac
done
[ "$minimum_free_kib" -gt 0 ] || exit 1
[ "$maximum_workspace_kib" -gt 0 ] || exit 1
[ "$maximum_output_kib" -gt 0 ] || exit 1
[ "$free_kib" -ge "$minimum_free_kib" ] || exit 1
[ "$workspace_kib" -le "$maximum_workspace_kib" ] || exit 1
[ "$output_kib" -le "$maximum_output_kib" ] || exit 1
