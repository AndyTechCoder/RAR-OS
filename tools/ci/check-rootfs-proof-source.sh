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
gzip.rs
json.rs
layout.rs
lib.rs
oci.rs
sha256.rs'
actual=$(find "$tree" -mindepth 1 -maxdepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || exit 1
find "$tree" -type l -print | /usr/bin/grep -q . && exit 1

for file in README.md build-plan.v0 gzip.rs json.rs layout.rs lib.rs oci.rs sha256.rs; do
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

for file in gzip.rs json.rs layout.rs lib.rs oci.rs sha256.rs; do
    ! /usr/bin/grep -En '(unsafe[[:space:]]+(fn|impl|extern)|unsafe[[:space:]]*\{|extern[[:space:]]+crate|include!|include_bytes!|include_str!|todo!|unimplemented!|std::process::Command|Command::new|TcpStream|UdpSocket|libc::)' "$tree/$file" >/dev/null || exit 1
done
for test_name in \
    decodes_stored_fixed_and_dynamic_blocks \
    enforces_output_crc_size_and_single_member_bounds \
    rejects_bad_headers_stored_lengths_and_huffman_trees; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/gzip.rs" || exit 1
done
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
    cumulative_entry_and_path_bounds_fail_closed \
    implicit_parent_replaces_lower_file_before_child_creation \
    opaque_marker_materializes_implicit_parent_over_lower_file \
    content_hashes_bind_effective_file_bytes; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/lib.rs" || exit 1
done
/usr/bin/grep -Fq '#![forbid(unsafe_code)]' "$tree/lib.rs" || exit 1
/usr/bin/grep -Fq 'pub fn resolve_uncompressed_image_from_source' "$tree/oci.rs" || exit 1
! /usr/bin/grep -Fq 'pub fn resolve_uncompressed_image<' "$tree/oci.rs" || exit 1
for test_name in \
    reads_exact_regular_blob_through_root_handle \
    rejects_symlinked_blob_and_symlinked_root \
    rejects_intermediate_escape_and_size_mismatch_before_read \
    exact_reader_never_requests_bytes_beyond_ceiling \
    rejects_special_file_after_path_only_inspection; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/layout.rs" || exit 1
done
/usr/bin/grep -Fq 'custom_flags(O_PATH | O_NOFOLLOW | O_NONBLOCK)' "$tree/layout.rs" || exit 1
/usr/bin/grep -Fq 'read_exact(&mut bytes)' "$tree/layout.rs" || exit 1
! /usr/bin/grep -Fq '.take(' "$tree/layout.rs" || exit 1
for test_name in \
    parses_utf8_unicode_escapes_and_unsigned_integers \
    rejects_duplicates_invalid_numbers_and_trailing_data; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/json.rs" || exit 1
done
for test_name in \
    resolves_digest_bound_layers_and_configuration_in_order \
    blob_source_receives_explicit_read_ceilings \
    rejects_tampered_blobs_before_layer_parsing \
    rejects_compression_and_diff_id_mismatch_in_inactive_subset \
    resolves_exact_manifest_through_verified_nested_index \
    rejects_tampered_nested_index_before_parsing \
    rejects_ambiguous_and_overdeep_nested_indexes \
    rejects_index_document_budget_exhaustion; do
    /usr/bin/grep -Fq "fn $test_name()" "$tree/oci.rs" || exit 1
done
/usr/bin/grep -Fq 'fn official_short_sha256_vectors()' "$tree/sha256.rs" || exit 1
/usr/bin/grep -Fqx 'modules=gzip.rs,json.rs,layout.rs,oci.rs,sha256.rs' "$tree/build-plan.v0" || exit 1
for source in layer.md image-layout.md manifest.md descriptor.md config.md; do
    /usr/bin/grep -Fq "https://github.com/opencontainers/image-spec/blob/v1.1.1/$source" "$tree/README.md" || exit 1
done
/usr/bin/grep -Fq 'https://www.rfc-editor.org/rfc/rfc1951' "$tree/README.md" || exit 1
/usr/bin/grep -Fq 'https://www.rfc-editor.org/rfc/rfc1952' "$tree/README.md" || exit 1
/usr/bin/grep -Fq 'full security finding' "$tree/README.md" || exit 1
printf '%s\n' 'rootfs proof source policy passed'
