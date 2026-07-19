//! Bounded structural validation and canonical re-serialization of the pinned base OCI export.
//!
//! The raw Docker export is untrusted delivery input. This module parses its tar framing with
//! strict bounds, re-hashes every content-addressed blob against its declared name, structurally
//! parses `oci-layout`, `index.json`, the selected OCI manifest, the image config, the ordered
//! layers, and Docker's legacy `manifest.json`, proves index→manifest→config→layer reachability
//! with no dangling or unexpected member, and then emits one deterministic canonical ustar archive
//! of the validated regular files. Raw export serialization never reaches the published bundle.

use std::collections::{BTreeMap, BTreeSet};

use super::transaction::{ArchiveEntry, ArchivePlan, MAX_INPUT_BYTES, MemberKind};
use super::{Json, PreauthError, Result, sha256_hex};

const SOURCE_DATE_EPOCH: u64 = 1_784_332_800;
const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_REPOSITORIES_BYTES: usize = 512;
const MAX_LAYERS: usize = 64;
const MAX_PAX_BYTES: u64 = 16 * 1024;
const MAX_PAX_RECORDS: usize = 64;

const INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";
const MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";
const LAYER_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseOciCanonical {
    pub canonical: Vec<u8>,
    pub config_sha256: String,
    pub manifest_sha256: String,
    pub layer_count: usize,
}

fn tar_octal(bytes: &[u8]) -> Result<u64> {
    let end = bytes.iter().position(|byte| *byte == 0 || *byte == b' ').unwrap_or(bytes.len());
    let value = std::str::from_utf8(&bytes[..end]).map_err(|_| PreauthError::new("base-oci-tar-number"))?.trim();
    if value.is_empty() { return Ok(0); }
    if !value.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) { return Err(PreauthError::new("base-oci-tar-number")); }
    u64::from_str_radix(value, 8).map_err(|_| PreauthError::new("base-oci-tar-number"))
}

/// A pax extended header may only carry timestamp/comment metadata, which the canonical
/// serialization discards. Any identity-bearing override fails closed.
fn check_pax_records(payload: &[u8]) -> Result<()> {
    if payload.len() as u64 > MAX_PAX_BYTES { return Err(PreauthError::new("base-oci-pax-bound")); }
    let mut rest = payload;
    let mut records = 0usize;
    while !rest.is_empty() {
        records += 1;
        if records > MAX_PAX_RECORDS { return Err(PreauthError::new("base-oci-pax-bound")); }
        let space = rest.iter().position(|byte| *byte == b' ').ok_or_else(|| PreauthError::new("base-oci-pax-record"))?;
        let length: usize = std::str::from_utf8(&rest[..space]).map_err(|_| PreauthError::new("base-oci-pax-record"))?
            .parse().map_err(|_| PreauthError::new("base-oci-pax-record"))?;
        if length <= space + 1 || length > rest.len() || rest[length - 1] != b'\n' {
            return Err(PreauthError::new("base-oci-pax-record"));
        }
        let record = std::str::from_utf8(&rest[space + 1..length - 1]).map_err(|_| PreauthError::new("base-oci-pax-record"))?;
        let key = record.split_once('=').map(|(key, _)| key).ok_or_else(|| PreauthError::new("base-oci-pax-record"))?;
        if !matches!(key, "mtime" | "atime" | "ctime" | "comment") {
            return Err(PreauthError::new("base-oci-pax-override"));
        }
        rest = &rest[length..];
    }
    Ok(())
}

struct RawMember<'a> { path: String, kind: MemberKind, mode: u32, uid: u32, gid: u32, payload: &'a [u8] }

fn walk_raw_export(raw: &[u8]) -> Result<Vec<RawMember<'_>>> {
    if raw.len() as u64 > MAX_INPUT_BYTES { return Err(PreauthError::new("base-oci-raw-bound")); }
    let mut offset = 0usize;
    let mut members = Vec::new();
    let mut zero_blocks = 0u8;
    let mut pending_pax = false;
    while offset < raw.len() {
        let end = offset.checked_add(512).ok_or_else(|| PreauthError::new("base-oci-tar-overflow"))?;
        if end > raw.len() { return Err(PreauthError::new("base-oci-tar-truncated")); }
        let header = &raw[offset..end];
        if header.iter().all(|byte| *byte == 0) {
            if pending_pax { return Err(PreauthError::new("base-oci-pax-orphan")); }
            zero_blocks += 1; offset = end;
            if zero_blocks == 2 {
                if raw[offset..].iter().any(|byte| *byte != 0) { return Err(PreauthError::new("base-oci-tar-trailing")); }
                break;
            }
            continue;
        }
        if zero_blocks != 0 || &header[257..263] != b"ustar\0" {
            return Err(PreauthError::new("base-oci-tar-framing"));
        }
        let expected_checksum = tar_octal(&header[148..156])?;
        let actual_checksum: u64 = header.iter().enumerate().map(|(index, byte)| {
            if (148..156).contains(&index) { b' ' as u64 } else { *byte as u64 }
        }).sum();
        if expected_checksum != actual_checksum { return Err(PreauthError::new("base-oci-tar-checksum")); }
        let size = tar_octal(&header[124..136])?;
        let payload_start = end;
        let padded = size.checked_add(511).ok_or_else(|| PreauthError::new("base-oci-tar-overflow"))? / 512 * 512;
        let payload_end = payload_start.checked_add(usize::try_from(padded).map_err(|_| PreauthError::new("base-oci-tar-overflow"))?)
            .ok_or_else(|| PreauthError::new("base-oci-tar-overflow"))?;
        let data_end = payload_start.checked_add(usize::try_from(size).map_err(|_| PreauthError::new("base-oci-tar-overflow"))?)
            .ok_or_else(|| PreauthError::new("base-oci-tar-overflow"))?;
        if payload_end > raw.len() { return Err(PreauthError::new("base-oci-tar-truncated")); }
        let kind = match header[156] {
            0 | b'0' => MemberKind::File,
            b'5' => MemberKind::Directory,
            b'x' => {
                if pending_pax { return Err(PreauthError::new("base-oci-pax-orphan")); }
                check_pax_records(&raw[payload_start..data_end])?;
                pending_pax = true;
                offset = payload_end;
                continue;
            }
            _ => return Err(PreauthError::new("base-oci-member-type")),
        };
        pending_pax = false;
        if kind == MemberKind::Directory && size != 0 { return Err(PreauthError::new("base-oci-directory-size")); }
        let name_end = header[..100].iter().position(|byte| *byte == 0).unwrap_or(100);
        let name = std::str::from_utf8(&header[..name_end]).map_err(|_| PreauthError::new("base-oci-member-name"))?;
        let prefix_end = header[345..500].iter().position(|byte| *byte == 0).unwrap_or(155);
        let prefix = std::str::from_utf8(&header[345..345 + prefix_end]).map_err(|_| PreauthError::new("base-oci-member-name"))?;
        let raw_path = if prefix.is_empty() { name.to_owned() } else { format!("{prefix}/{name}") };
        let path = if kind == MemberKind::Directory {
            raw_path.strip_suffix('/').unwrap_or(&raw_path).to_owned()
        } else { raw_path };
        members.push(RawMember {
            path, kind,
            mode: u32::try_from(tar_octal(&header[100..108])?).map_err(|_| PreauthError::new("base-oci-tar-number"))?,
            uid: u32::try_from(tar_octal(&header[108..116])?).map_err(|_| PreauthError::new("base-oci-tar-number"))?,
            gid: u32::try_from(tar_octal(&header[116..124])?).map_err(|_| PreauthError::new("base-oci-tar-number"))?,
            payload: &raw[payload_start..data_end],
        });
        offset = payload_end;
    }
    if zero_blocks != 2 { return Err(PreauthError::new("base-oci-tar-end-marker")); }
    Ok(members)
}

fn descriptor_digest(descriptor: &Json) -> Result<String> {
    let digest = descriptor.get("digest")?.string()?;
    let hex = digest.strip_prefix("sha256:").ok_or_else(|| PreauthError::new("base-oci-digest-algorithm"))?;
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(PreauthError::new("base-oci-digest-form"));
    }
    Ok(hex.to_owned())
}

fn resolve_blob<'a>(blobs: &BTreeMap<String, &'a [u8]>, descriptor: &Json, media_type: &str) -> Result<(String, &'a [u8])> {
    descriptor.exact_keys(&["mediaType", "digest", "size"], &["annotations", "platform"])
        .map_err(|_| PreauthError::new("base-oci-descriptor-keys"))?;
    if descriptor.get("mediaType")?.string()? != media_type { return Err(PreauthError::new("base-oci-media-type")); }
    let hex = descriptor_digest(descriptor)?;
    let payload = *blobs.get(&hex).ok_or_else(|| PreauthError::new("base-oci-missing-blob"))?;
    if descriptor.get("size")?.number()? != payload.len() as u64 { return Err(PreauthError::new("base-oci-descriptor-size")); }
    Ok((hex, payload))
}

fn parse_metadata(name: &str, files: &BTreeMap<String, &[u8]>) -> Result<Json> {
    let bytes = *files.get(name).ok_or_else(|| PreauthError::new("base-oci-roots"))?;
    if bytes.len() > MAX_METADATA_BYTES { return Err(PreauthError::new("base-oci-metadata-bound")); }
    Json::parse(bytes)
}

fn emit_canonical(files: &BTreeMap<String, &[u8]>) -> Vec<u8> {
    let mut archive = Vec::new();
    for (path, payload) in files {
        let mut header = [0u8; 512];
        header[..path.len()].copy_from_slice(path.as_bytes());
        header[100..108].copy_from_slice(b"0000644\0");
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        header[124..136].copy_from_slice(format!("{:011o}\0", payload.len()).as_bytes());
        header[136..148].copy_from_slice(format!("{SOURCE_DATE_EPOCH:011o}\0").as_bytes());
        header[148..156].fill(b' ');
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u64 = header.iter().map(|byte| *byte as u64).sum();
        header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
        archive.extend_from_slice(&header);
        archive.extend_from_slice(payload);
        archive.resize(archive.len().div_ceil(512) * 512, 0);
    }
    archive.resize(archive.len() + 1024, 0);
    archive
}

pub fn canonicalize_base_oci(raw: &[u8]) -> Result<BaseOciCanonical> {
    let members = walk_raw_export(raw)?;
    ArchivePlan::validate(members.iter().map(|member| ArchiveEntry {
        path: member.path.clone(), kind: member.kind, compressed_bytes: member.payload.len() as u64,
        expanded_bytes: member.payload.len() as u64, mode: member.mode, uid: member.uid,
        gid: member.gid, link_target: None,
    }).collect())?;
    let mut files: BTreeMap<String, &[u8]> = BTreeMap::new();
    let mut blobs: BTreeMap<String, &[u8]> = BTreeMap::new();
    for member in &members {
        match member.kind {
            MemberKind::Directory => {
                if member.path != "blobs" && member.path != "blobs/sha256" {
                    return Err(PreauthError::new("base-oci-unexpected-member"));
                }
            }
            MemberKind::File => {
                if let Some(hex) = member.path.strip_prefix("blobs/sha256/") {
                    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
                        return Err(PreauthError::new("base-oci-unexpected-member"));
                    }
                    if sha256_hex(member.payload) != hex { return Err(PreauthError::new("base-oci-blob-digest")); }
                    blobs.insert(hex.to_owned(), member.payload);
                } else if !matches!(member.path.as_str(), "oci-layout" | "index.json" | "manifest.json" | "repositories") {
                    return Err(PreauthError::new("base-oci-unexpected-member"));
                }
                files.insert(member.path.clone(), member.payload);
            }
            _ => return Err(PreauthError::new("base-oci-member-type")),
        }
    }

    let layout = parse_metadata("oci-layout", &files)?;
    layout.exact_keys(&["imageLayoutVersion"], &[]).map_err(|_| PreauthError::new("base-oci-layout-keys"))?;
    if layout.get("imageLayoutVersion")?.string()? != "1.0.0" { return Err(PreauthError::new("base-oci-layout")); }

    let index = parse_metadata("index.json", &files)?;
    index.exact_keys(&["schemaVersion", "manifests"], &["mediaType", "annotations"])
        .map_err(|_| PreauthError::new("base-oci-index-keys"))?;
    if index.get("schemaVersion")?.number()? != 2 { return Err(PreauthError::new("base-oci-schema-version")); }
    if let Ok(media_type) = index.get("mediaType") {
        if media_type.string()? != INDEX_MEDIA_TYPE { return Err(PreauthError::new("base-oci-media-type")); }
    }
    let descriptors = index.get("manifests")?.array()?;
    if descriptors.len() != 1 { return Err(PreauthError::new("base-oci-manifest-count")); }
    if let Ok(platform) = descriptors[0].get("platform") {
        platform.exact_keys(&["architecture", "os"], &["variant"])
            .map_err(|_| PreauthError::new("base-oci-platform-keys"))?;
        if platform.get("architecture")?.string()? != "amd64" || platform.get("os")?.string()? != "linux" {
            return Err(PreauthError::new("base-oci-platform"));
        }
    }
    let (manifest_hex, manifest_bytes) = resolve_blob(&blobs, &descriptors[0], MANIFEST_MEDIA_TYPE)?;

    if manifest_bytes.len() > MAX_METADATA_BYTES { return Err(PreauthError::new("base-oci-metadata-bound")); }
    let manifest = Json::parse(manifest_bytes)?;
    manifest.exact_keys(&["schemaVersion", "config", "layers"], &["mediaType", "annotations"])
        .map_err(|_| PreauthError::new("base-oci-manifest-keys"))?;
    if manifest.get("schemaVersion")?.number()? != 2 { return Err(PreauthError::new("base-oci-schema-version")); }
    if let Ok(media_type) = manifest.get("mediaType") {
        if media_type.string()? != MANIFEST_MEDIA_TYPE { return Err(PreauthError::new("base-oci-media-type")); }
    }
    let (config_hex, config_bytes) = resolve_blob(&blobs, manifest.get("config")?, CONFIG_MEDIA_TYPE)?;
    let layer_descriptors = manifest.get("layers")?.array()?;
    if layer_descriptors.is_empty() || layer_descriptors.len() > MAX_LAYERS {
        return Err(PreauthError::new("base-oci-layer-count"));
    }
    let mut layer_hexes = Vec::new();
    for descriptor in layer_descriptors {
        let (hex, _) = resolve_blob(&blobs, descriptor, LAYER_MEDIA_TYPE)?;
        layer_hexes.push(hex);
    }

    if config_bytes.len() > MAX_METADATA_BYTES { return Err(PreauthError::new("base-oci-metadata-bound")); }
    let config = Json::parse(config_bytes)?;
    if config.get("architecture")?.string()? != "amd64" || config.get("os")?.string()? != "linux" {
        return Err(PreauthError::new("base-oci-platform"));
    }
    let rootfs = config.get("rootfs")?;
    if rootfs.get("type")?.string()? != "layers" { return Err(PreauthError::new("base-oci-rootfs")); }
    let diff_ids = rootfs.get("diff_ids")?.array()?;
    if diff_ids.len() != layer_hexes.len() { return Err(PreauthError::new("base-oci-diff-id")); }
    for (diff_id, layer_hex) in diff_ids.iter().zip(&layer_hexes) {
        if diff_id.string()? != format!("sha256:{layer_hex}") { return Err(PreauthError::new("base-oci-diff-id")); }
    }

    let legacy = parse_metadata("manifest.json", &files)?;
    let entries = legacy.array()?;
    if entries.len() != 1 { return Err(PreauthError::new("base-oci-manifest-count")); }
    let entry = &entries[0];
    entry.exact_keys(&["Config", "Layers"], &["RepoTags", "LayerSources"])
        .map_err(|_| PreauthError::new("base-oci-legacy-keys"))?;
    if entry.get("Config")?.string()? != format!("blobs/sha256/{config_hex}") {
        return Err(PreauthError::new("base-oci-legacy-config"));
    }
    let legacy_layers = entry.get("Layers")?.array()?;
    if legacy_layers.len() != layer_hexes.len() { return Err(PreauthError::new("base-oci-legacy-layers")); }
    for (layer, hex) in legacy_layers.iter().zip(&layer_hexes) {
        if layer.string()? != format!("blobs/sha256/{hex}") { return Err(PreauthError::new("base-oci-legacy-layers")); }
    }
    if let Ok(repo_tags) = entry.get("RepoTags") {
        match repo_tags {
            Json::Null => {}
            Json::Array(values) => {
                if values.len() > 8 { return Err(PreauthError::new("base-oci-repo-tags")); }
                for value in values {
                    if value.string()?.len() > 256 { return Err(PreauthError::new("base-oci-repo-tags")); }
                }
            }
            _ => return Err(PreauthError::new("base-oci-repo-tags")),
        }
    }
    if let Ok(layer_sources) = entry.get("LayerSources") {
        let sources = layer_sources.object()?;
        if sources.len() != layer_hexes.len() { return Err(PreauthError::new("base-oci-layer-sources")); }
        for hex in &layer_hexes {
            let source = sources.get(&format!("sha256:{hex}")).ok_or_else(|| PreauthError::new("base-oci-layer-sources"))?;
            source.exact_keys(&["mediaType", "digest", "size"], &[])
                .map_err(|_| PreauthError::new("base-oci-layer-sources"))?;
            if source.get("mediaType")?.string()? != LAYER_MEDIA_TYPE
                || source.get("digest")?.string()? != format!("sha256:{hex}")
                || source.get("size")?.number()? != blobs[hex].len() as u64
            {
                return Err(PreauthError::new("base-oci-layer-sources"));
            }
        }
    }

    if let Some(repositories) = files.get("repositories") {
        if repositories.len() > MAX_REPOSITORIES_BYTES { return Err(PreauthError::new("base-oci-repositories")); }
        let parsed = Json::parse(repositories).map_err(|_| PreauthError::new("base-oci-repositories"))?;
        for value in parsed.object().map_err(|_| PreauthError::new("base-oci-repositories"))?.values() {
            for binding in value.object().map_err(|_| PreauthError::new("base-oci-repositories"))?.values() {
                binding.string().map_err(|_| PreauthError::new("base-oci-repositories"))?;
            }
        }
    }

    let mut referenced: BTreeSet<&str> = BTreeSet::new();
    referenced.insert(&manifest_hex);
    referenced.insert(&config_hex);
    for hex in &layer_hexes { referenced.insert(hex); }
    if blobs.keys().any(|hex| !referenced.contains(hex.as_str())) {
        return Err(PreauthError::new("base-oci-dangling-blob"));
    }

    Ok(BaseOciCanonical {
        canonical: emit_canonical(&files),
        config_sha256: config_hex,
        manifest_sha256: manifest_hex,
        layer_count: layer_hexes.len(),
    })
}
