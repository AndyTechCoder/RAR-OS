#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
tree=${1-$root/tools/rar-lab/rootfs-proof}
[ -d "$tree" ] && [ ! -L "$tree" ] || exit 1
expected='README.md
build-plan.v0
lib.rs'
actual=$(find "$tree" -mindepth 1 -maxdepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || exit 1
find "$tree" -type l -print | /usr/bin/grep -q . && exit 1

for file in README.md build-plan.v0 lib.rs; do
    [ -s "$tree/$file" ] && [ ! -L "$tree/$file" ] || exit 1
done

for line in \
    'schema=rar-rootfs-proof-build-plan-v0' \
    'status=experimental-non-activating' \
    'dependency_policy=rust-std-only' \
    'network=none' \
    'source_mount=readonly' \
    'activation=forbidden'; do
    [ "$(/usr/bin/grep -Fxc "$line" "$tree/build-plan.v0")" -eq 1 ] || exit 1
done

! /usr/bin/grep -En '(unsafe[[:space:]]+(fn|impl|extern)|unsafe[[:space:]]*\{|extern[[:space:]]+crate|include!|include_bytes!|include_str!|todo!|unimplemented!|std::process::Command|Command::new|TcpStream|UdpSocket|libc::)' "$tree/lib.rs" >/dev/null || exit 1
for test_name in \
    whiteouts_remove_only_lower_layer_entries_regardless_of_archive_order \
    opaque_whiteout_removes_lower_descendants_but_keeps_same_layer_additions \
    executable_and_elf_objects_are_found_outside_role_roots \
    traversal_absolute_and_noncanonical_paths_fail_closed \
    links_devices_and_setid_files_fail_closed \
    malformed_or_truncated_archives_fail_closed; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/lib.rs" || exit 1
done
for test_name in \
    prefix_removal_preserves_lexical_neighbors \
    cumulative_entry_and_path_bounds_fail_closed; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/lib.rs" || exit 1
done
for source in layer.md image-layout.md manifest.md; do
    /usr/bin/grep -Fq "https://github.com/opencontainers/image-spec/blob/v1.1.1/$source" "$tree/README.md" || exit 1
done
/usr/bin/grep -Fq 'does not resolve the full security finding' "$tree/README.md" || exit 1
printf '%s\n' 'rootfs proof source policy passed'
