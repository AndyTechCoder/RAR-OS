#!/bin/sh
set -eu

safe_root=${1-'/Volumes/Z Slim/Andy’s folder/Codex/RAR OS Alpha'}
minimum_free_kib=${RAR_MINIMUM_SSD_FREE_KIB-10485760}
maximum_workspace_kib=${RAR_MAXIMUM_WORKSPACE_KIB-8388608}
maximum_output_kib=${RAR_MAXIMUM_OUTPUT_KIB-524288}
[ -d "$safe_root" ] && [ ! -L "$safe_root" ] || exit 1
case "$minimum_free_kib$maximum_workspace_kib$maximum_output_kib" in *[!0-9]*) exit 1 ;; esac

free_kib=$(/bin/df -Pk "$safe_root" | /usr/bin/awk 'END { print $4 }')
workspace_kib=$(/usr/bin/du -sk "$safe_root" | /usr/bin/awk 'NR == 1 { print $1 }')
case "$free_kib$workspace_kib" in *[!0-9]*) exit 1 ;; esac
[ "$free_kib" -ge "$minimum_free_kib" ] || exit 1
[ "$workspace_kib" -le "$maximum_workspace_kib" ] || exit 1

output_kib=$(find "$safe_root/repository" "$safe_root/worktrees" -type d -name out -prune -exec /usr/bin/du -sk {} \; 2>/dev/null | /usr/bin/awk '{ total += $1 } END { print total + 0 }')
case "$output_kib" in '' | *[!0-9]*) exit 1 ;; esac
[ "$output_kib" -le "$maximum_output_kib" ] || exit 1
printf 'workspace budget passed: free_kib=%s workspace_kib=%s output_kib=%s\n' "$free_kib" "$workspace_kib" "$output_kib"
