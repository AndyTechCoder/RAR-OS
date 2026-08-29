//! Non-activating OCI layer-resolution foundation for RAR Lab image proofs.
//!
//! This host-only library deliberately accepts only bounded, uncompressed
//! POSIX ustar layers. It does not activate a Development Lab profile and it is
//! not linked into RAR OS.

use std::collections::{BTreeMap, BTreeSet};

mod json;
pub mod oci;
mod sha256;

pub const BLOCK_BYTES: usize = 512;
pub const MAX_LAYER_BYTES: usize = 1_073_741_824;
pub const MAX_LAYER_ENTRIES: usize = 65_536;
pub const MAX_PATH_BYTES: usize = 4_096;
pub const MAX_FILE_BYTES: u64 = 1_073_741_824;
pub const MAX_LAYER_PATH_BYTES: usize = 16_777_216;
pub const MAX_EFFECTIVE_ENTRIES: usize = 262_144;
pub const MAX_EFFECTIVE_PATH_BYTES: usize = 67_108_864;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeKind {
    File,
    Directory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub path: String,
    pub kind: NodeKind,
    pub mode: u32,
    pub bytes: u64,
    pub is_elf: bool,
}

impl Node {
    pub fn is_executable_or_loadable(&self) -> bool {
        self.kind == NodeKind::File && (self.mode & 0o111 != 0 || self.is_elf)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectiveRootfs {
    nodes: BTreeMap<String, Node>,
}

impl EffectiveRootfs {
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn executable_or_loadable_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes
            .values()
            .filter(|node| node.is_executable_or_loadable())
    }

    pub fn get(&self, path: &str) -> Option<&Node> {
        self.nodes.get(path)
    }

    pub fn apply_uncompressed_ustar_layer(&mut self, archive: &[u8]) -> Result<(), Error> {
        let layer = parse_layer(archive)?;
        let mut candidate = self.nodes.clone();

        for parent in &layer.opaque_directories {
            remove_descendants(&mut candidate, parent);
        }

        for target in &layer.whiteouts {
            remove_path_and_descendants(&mut candidate, target);
        }

        for node in layer.additions.values() {
            for ancestor in ancestors(&node.path) {
                if let Some(existing) = candidate.get(ancestor) {
                    if existing.kind != NodeKind::Directory {
                        return Err(Error::ParentIsNotDirectory);
                    }
                }
            }

            if node.kind == NodeKind::File {
                remove_descendants(&mut candidate, &node.path);
            }
            candidate.insert(node.path.clone(), node.clone());
        }

        validate_effective_bounds(
            &candidate,
            MAX_EFFECTIVE_ENTRIES,
            MAX_EFFECTIVE_PATH_BYTES,
        )?;
        self.nodes = candidate;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    ArchiveTooLarge,
    ArchiveNotBlockAligned,
    MissingEndMarker,
    NonZeroAfterEndMarker,
    TruncatedHeader,
    TruncatedEntry,
    InvalidHeaderChecksum,
    UnsupportedTarFormat,
    InvalidNumericField,
    InvalidStringField,
    FileTooLarge,
    TooManyEntries,
    LayerPathsTooLarge,
    TooManyEffectiveEntries,
    EffectivePathsTooLarge,
    InvalidUtf8Path,
    InvalidPath,
    PathTooLong,
    UnsupportedEntryType,
    UnexpectedLinkName,
    NonEmptyDirectory,
    SetIdMode,
    InvalidWhiteout,
    DuplicatePath,
    DuplicateWhiteout,
    ParentIsNotDirectory,
}

#[derive(Debug, Default)]
struct Layer {
    additions: BTreeMap<String, Node>,
    whiteouts: BTreeSet<String>,
    opaque_directories: BTreeSet<String>,
}

fn parse_layer(archive: &[u8]) -> Result<Layer, Error> {
    if archive.len() > MAX_LAYER_BYTES {
        return Err(Error::ArchiveTooLarge);
    }
    if archive.len() % BLOCK_BYTES != 0 {
        return Err(Error::ArchiveNotBlockAligned);
    }

    let mut layer = Layer::default();
    let mut offset = 0usize;
    let mut entry_count = 0usize;
    let mut layer_path_bytes = 0usize;
    let mut found_end = false;

    while offset < archive.len() {
        let header_end = offset.checked_add(BLOCK_BYTES).ok_or(Error::TruncatedHeader)?;
        let header = archive.get(offset..header_end).ok_or(Error::TruncatedHeader)?;
        if header.iter().all(|byte| *byte == 0) {
            let second_end = header_end
                .checked_add(BLOCK_BYTES)
                .ok_or(Error::MissingEndMarker)?;
            let second = archive
                .get(header_end..second_end)
                .ok_or(Error::MissingEndMarker)?;
            if !second.iter().all(|byte| *byte == 0) {
                return Err(Error::MissingEndMarker);
            }
            if archive[second_end..].iter().any(|byte| *byte != 0) {
                return Err(Error::NonZeroAfterEndMarker);
            }
            found_end = true;
            break;
        }

        entry_count += 1;
        if entry_count > MAX_LAYER_ENTRIES {
            return Err(Error::TooManyEntries);
        }
        validate_header(header)?;

        let mode = parse_octal(&header[100..108])?;
        let _uid = parse_octal(&header[108..116])?;
        let _gid = parse_octal(&header[116..124])?;
        let _mtime = parse_octal(&header[136..148])?;
        if mode > 0o7777 {
            return Err(Error::InvalidNumericField);
        }
        let mode = mode as u32;
        if mode & 0o6000 != 0 {
            return Err(Error::SetIdMode);
        }
        let size = parse_octal(&header[124..136])?;
        if size > MAX_FILE_BYTES {
            return Err(Error::FileTooLarge);
        }

        let path = header_path(header)?;
        layer_path_bytes = layer_path_bytes
            .checked_add(path.len())
            .ok_or(Error::LayerPathsTooLarge)?;
        if layer_path_bytes > MAX_LAYER_PATH_BYTES {
            return Err(Error::LayerPathsTooLarge);
        }
        let type_flag = header[156];
        let kind = match type_flag {
            0 | b'0' => NodeKind::File,
            b'5' => NodeKind::Directory,
            _ => return Err(Error::UnsupportedEntryType),
        };
        if kind == NodeKind::Directory && size != 0 {
            return Err(Error::NonEmptyDirectory);
        }
        if parse_string_field(&header[157..257])?.is_some() {
            return Err(Error::UnexpectedLinkName);
        }

        let data_start = header_end;
        let size_usize = usize::try_from(size).map_err(|_| Error::FileTooLarge)?;
        let data_end = data_start
            .checked_add(size_usize)
            .ok_or(Error::TruncatedEntry)?;
        let data = archive.get(data_start..data_end).ok_or(Error::TruncatedEntry)?;
        let padded = size_usize
            .checked_add(BLOCK_BYTES - 1)
            .ok_or(Error::TruncatedEntry)?
            / BLOCK_BYTES
            * BLOCK_BYTES;
        offset = data_start.checked_add(padded).ok_or(Error::TruncatedEntry)?;
        if offset > archive.len() {
            return Err(Error::TruncatedEntry);
        }

        let (parent, basename) = split_parent(&path);
        if basename == ".wh..wh..opq" {
            validate_whiteout(kind, size)?;
            if !layer.opaque_directories.insert(parent.to_owned()) {
                return Err(Error::DuplicateWhiteout);
            }
            continue;
        }
        if let Some(removed_name) = basename.strip_prefix(".wh.") {
            validate_whiteout(kind, size)?;
            if removed_name.is_empty() {
                return Err(Error::InvalidWhiteout);
            }
            let target = if parent.is_empty() {
                removed_name.to_owned()
            } else {
                format!("{parent}/{removed_name}")
            };
            validate_canonical_path(&target, false)?;
            if !layer.whiteouts.insert(target) {
                return Err(Error::DuplicateWhiteout);
            }
            continue;
        }

        let is_elf = kind == NodeKind::File && data.starts_with(b"\x7fELF");
        let node = Node {
            path: path.clone(),
            kind,
            mode,
            bytes: size,
            is_elf,
        };
        if layer.additions.insert(path, node).is_some() {
            return Err(Error::DuplicatePath);
        }
    }

    if !found_end {
        return Err(Error::MissingEndMarker);
    }
    Ok(layer)
}

fn validate_header(header: &[u8]) -> Result<(), Error> {
    if header.len() != BLOCK_BYTES {
        return Err(Error::TruncatedHeader);
    }
    if &header[257..263] != b"ustar\0" || &header[263..265] != b"00" {
        return Err(Error::UnsupportedTarFormat);
    }
    let expected = parse_octal(&header[148..156])?;
    let actual: u64 = header
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if (148..156).contains(&index) {
                u64::from(b' ')
            } else {
                u64::from(*byte)
            }
        })
        .sum();
    if expected != actual {
        return Err(Error::InvalidHeaderChecksum);
    }
    Ok(())
}

fn parse_octal(field: &[u8]) -> Result<u64, Error> {
    if field.first().is_some_and(|byte| byte & 0x80 != 0) {
        return Err(Error::InvalidNumericField);
    }
    let start = field.iter().position(|byte| *byte != b' ').unwrap_or(field.len());
    let rest = &field[start..];
    let digit_count = rest
        .iter()
        .take_while(|byte| matches!(**byte, b'0'..=b'7'))
        .count();
    if digit_count == 0 {
        return Err(Error::InvalidNumericField);
    }
    if rest[digit_count..]
        .iter()
        .any(|byte| *byte != b' ' && *byte != 0)
    {
        return Err(Error::InvalidNumericField);
    }
    rest[..digit_count].iter().copied().try_fold(0u64, |value, byte| {
        value
            .checked_mul(8)
            .and_then(|value| value.checked_add(u64::from(byte - b'0')))
            .ok_or(Error::InvalidNumericField)
    })
}

fn header_path(header: &[u8]) -> Result<String, Error> {
    let name = parse_string_field(&header[0..100])?.ok_or(Error::InvalidPath)?;
    let prefix = parse_string_field(&header[345..500])?;
    let name = std::str::from_utf8(name).map_err(|_| Error::InvalidUtf8Path)?;
    let path = if let Some(prefix) = prefix {
        let prefix = std::str::from_utf8(prefix).map_err(|_| Error::InvalidUtf8Path)?;
        format!("{prefix}/{name}")
    } else {
        name.to_owned()
    };
    validate_canonical_path(&path, header[156] == b'5')
}

fn parse_string_field(field: &[u8]) -> Result<Option<&[u8]>, Error> {
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(field.len());
    if field[end..].iter().any(|byte| *byte != 0) {
        return Err(Error::InvalidStringField);
    }
    Ok((end != 0).then_some(&field[..end]))
}

fn validate_canonical_path(path: &str, allow_directory_slash: bool) -> Result<String, Error> {
    if path.len() > MAX_PATH_BYTES {
        return Err(Error::PathTooLong);
    }
    if path.is_empty() || path.starts_with('/') || path.contains('\0') {
        return Err(Error::InvalidPath);
    }
    let path = if allow_directory_slash {
        path.strip_suffix('/').unwrap_or(path)
    } else {
        if path.ends_with('/') {
            return Err(Error::InvalidPath);
        }
        path
    };
    if path.is_empty()
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || path.chars().any(char::is_control)
    {
        return Err(Error::InvalidPath);
    }
    Ok(path.to_owned())
}

fn validate_whiteout(kind: NodeKind, size: u64) -> Result<(), Error> {
    if kind != NodeKind::File || size != 0 {
        return Err(Error::InvalidWhiteout);
    }
    Ok(())
}

fn split_parent(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

fn remove_descendants(nodes: &mut BTreeMap<String, Node>, parent: &str) {
    if parent.is_empty() {
        nodes.clear();
        return;
    }
    let prefix = format!("{parent}/");
    let descendants: Vec<String> = nodes
        .range(prefix.clone()..)
        .take_while(|(path, _)| path.starts_with(&prefix))
        .map(|(path, _)| path.clone())
        .collect();
    for path in descendants {
        nodes.remove(&path);
    }
}

fn remove_path_and_descendants(nodes: &mut BTreeMap<String, Node>, target: &str) {
    nodes.remove(target);
    remove_descendants(nodes, target);
}

fn validate_effective_bounds(
    nodes: &BTreeMap<String, Node>,
    max_entries: usize,
    max_path_bytes: usize,
) -> Result<(), Error> {
    if nodes.len() > max_entries {
        return Err(Error::TooManyEffectiveEntries);
    }
    let path_bytes = nodes.keys().try_fold(0usize, |total, path| {
        total.checked_add(path.len()).ok_or(Error::EffectivePathsTooLarge)
    })?;
    if path_bytes > max_path_bytes {
        return Err(Error::EffectivePathsTooLarge);
    }
    Ok(())
}

fn ancestors(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/').map(|(index, _)| &path[..index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Entry<'a> {
        path: &'a str,
        kind: u8,
        mode: u32,
        data: &'a [u8],
    }

    fn archive(entries: &[Entry<'_>]) -> Vec<u8> {
        let mut output = Vec::new();
        for entry in entries {
            let mut header = [0u8; BLOCK_BYTES];
            assert!(entry.path.len() <= 100);
            header[..entry.path.len()].copy_from_slice(entry.path.as_bytes());
            write_octal(&mut header[100..108], u64::from(entry.mode));
            write_octal(&mut header[108..116], 0);
            write_octal(&mut header[116..124], 0);
            write_octal(&mut header[124..136], entry.data.len() as u64);
            write_octal(&mut header[136..148], 0);
            header[148..156].fill(b' ');
            header[156] = entry.kind;
            header[257..263].copy_from_slice(b"ustar\0");
            header[263..265].copy_from_slice(b"00");
            let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
            write_checksum(&mut header[148..156], checksum);
            output.extend_from_slice(&header);
            output.extend_from_slice(entry.data);
            let padding = (BLOCK_BYTES - entry.data.len() % BLOCK_BYTES) % BLOCK_BYTES;
            output.resize(output.len() + padding, 0);
        }
        output.resize(output.len() + 2 * BLOCK_BYTES, 0);
        output
    }

    fn write_octal(field: &mut [u8], value: u64) {
        field.fill(b'0');
        field[field.len() - 1] = 0;
        let encoded = format!("{value:o}");
        let start = field.len() - 1 - encoded.len();
        field[start..start + encoded.len()].copy_from_slice(encoded.as_bytes());
    }

    fn write_checksum(field: &mut [u8], value: u64) {
        let encoded = format!("{value:06o}");
        field[..6].copy_from_slice(encoded.as_bytes());
        field[6] = 0;
        field[7] = b' ';
    }

    fn file<'a>(path: &'a str, mode: u32, data: &'a [u8]) -> Entry<'a> {
        Entry { path, kind: b'0', mode, data }
    }

    fn directory(path: &str) -> Entry<'_> {
        Entry { path, kind: b'5', mode: 0o755, data: b"" }
    }

    #[test]
    fn whiteouts_remove_only_lower_layer_entries_regardless_of_archive_order() {
        let mut rootfs = EffectiveRootfs::default();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                directory("opt/"),
                file("opt/old", 0o755, b"old"),
                file("keep", 0o644, b"keep"),
            ]))
            .unwrap();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                file("opt/old", 0o644, b"replacement"),
                file("opt/.wh.old", 0, b""),
            ]))
            .unwrap();

        assert_eq!(rootfs.get("opt/old").unwrap().bytes, 11);
        assert!(rootfs.get("keep").is_some());
    }

    #[test]
    fn opaque_whiteout_removes_lower_descendants_but_keeps_same_layer_additions() {
        let mut rootfs = EffectiveRootfs::default();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                directory("bin/"),
                file("bin/old", 0o755, b"old"),
                file("outside", 0o644, b"outside"),
            ]))
            .unwrap();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                file("bin/new", 0o755, b"new"),
                file("bin/.wh..wh..opq", 0, b""),
            ]))
            .unwrap();

        assert!(rootfs.get("bin/old").is_none());
        assert!(rootfs.get("bin/new").is_some());
        assert!(rootfs.get("outside").is_some());
    }

    #[test]
    fn executable_and_elf_objects_are_found_outside_role_roots() {
        let mut rootfs = EffectiveRootfs::default();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                directory("usr/"),
                directory("usr/lib/"),
                file("usr/lib/hidden.so", 0o644, b"\x7fELFpayload"),
                directory("sbin/"),
                file("sbin/tool", 0o700, b"script"),
                file("notice", 0o644, b"text"),
            ]))
            .unwrap();

        let paths: Vec<&str> = rootfs
            .executable_or_loadable_nodes()
            .map(|node| node.path.as_str())
            .collect();
        assert_eq!(paths, vec!["sbin/tool", "usr/lib/hidden.so"]);
    }

    #[test]
    fn traversal_absolute_and_noncanonical_paths_fail_closed() {
        for path in ["../escape", "/absolute", "a//b", "a/./b", "a/../b"] {
            let result = EffectiveRootfs::default()
                .apply_uncompressed_ustar_layer(&archive(&[file(path, 0o644, b"x")]));
            assert_eq!(result, Err(Error::InvalidPath), "accepted {path}");
        }
    }

    #[test]
    fn links_devices_and_setid_files_fail_closed() {
        for kind in [b'1', b'2', b'3', b'4', b'6'] {
            let result = EffectiveRootfs::default()
                .apply_uncompressed_ustar_layer(&archive(&[Entry {
                    path: "bad",
                    kind,
                    mode: 0o644,
                    data: b"",
                }]));
            assert_eq!(result, Err(Error::UnsupportedEntryType));
        }
        let result = EffectiveRootfs::default()
            .apply_uncompressed_ustar_layer(&archive(&[file("setuid", 0o4755, b"x")]));
        assert_eq!(result, Err(Error::SetIdMode));
    }

    #[test]
    fn malformed_or_truncated_archives_fail_closed() {
        let mut bad_checksum = archive(&[file("file", 0o644, b"x")]);
        bad_checksum[0] ^= 1;
        assert_eq!(
            EffectiveRootfs::default().apply_uncompressed_ustar_layer(&bad_checksum),
            Err(Error::InvalidHeaderChecksum)
        );

        let mut missing_end = archive(&[file("file", 0o644, b"x")]);
        missing_end.truncate(missing_end.len() - 2 * BLOCK_BYTES);
        assert_eq!(
            EffectiveRootfs::default().apply_uncompressed_ustar_layer(&missing_end),
            Err(Error::MissingEndMarker)
        );
    }

    #[test]
    fn file_replacing_directory_removes_lower_descendants() {
        let mut rootfs = EffectiveRootfs::default();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                directory("tree/"),
                file("tree/leaf", 0o644, b"leaf"),
            ]))
            .unwrap();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[file("tree", 0o644, b"file")]))
            .unwrap();
        assert!(rootfs.get("tree/leaf").is_none());
        assert_eq!(rootfs.get("tree").unwrap().kind, NodeKind::File);
    }

    #[test]
    fn prefix_removal_preserves_lexical_neighbors() {
        let mut rootfs = EffectiveRootfs::default();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[
                directory("a/"),
                file("a/child", 0o644, b"child"),
                file("aa", 0o644, b"neighbor"),
                file("b", 0o644, b"other"),
            ]))
            .unwrap();
        rootfs
            .apply_uncompressed_ustar_layer(&archive(&[file(".wh.a", 0, b"")]))
            .unwrap();
        assert!(rootfs.get("a").is_none());
        assert!(rootfs.get("a/child").is_none());
        assert!(rootfs.get("aa").is_some());
        assert!(rootfs.get("b").is_some());
    }

    #[test]
    fn cumulative_entry_and_path_bounds_fail_closed() {
        let nodes = BTreeMap::from([
            (
                "one".to_owned(),
                Node {
                    path: "one".to_owned(),
                    kind: NodeKind::File,
                    mode: 0o644,
                    bytes: 1,
                    is_elf: false,
                },
            ),
            (
                "two".to_owned(),
                Node {
                    path: "two".to_owned(),
                    kind: NodeKind::File,
                    mode: 0o644,
                    bytes: 1,
                    is_elf: false,
                },
            ),
        ]);
        assert_eq!(
            validate_effective_bounds(&nodes, 1, usize::MAX),
            Err(Error::TooManyEffectiveEntries)
        );
        assert_eq!(
            validate_effective_bounds(&nodes, usize::MAX, 5),
            Err(Error::EffectivePathsTooLarge)
        );
    }
}
