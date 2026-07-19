//! Bounded structural validation and canonical re-serialization of the pinned base OCI export.
//!
//! The raw Docker export is untrusted delivery input. This module parses its tar framing with
//! strict bounds and exact member metadata, permits pax extended headers only as a parsed
//! canonical timestamp grammar, re-hashes every content-addressed blob against its declared
//! name, structurally parses `oci-layout`, `index.json`, the selected OCI manifest, the image
//! config, the ordered layers, and Docker's legacy `manifest.json`, requires the exact
//! digest-pull identity policy (two pinned index-descriptor annotations, no platform object,
//! `RepoTags` exactly null, one exact `LayerSources` binding per diff ID, and no tag-bearing
//! `repositories` root), proves index→manifest→config→layer reachability, digest-verifies then
//! excludes edge-free store metadata per ADR 0018, and emits one deterministic canonical ustar
//! archive that equals the rooted graph exactly.

use std::collections::{BTreeMap, BTreeSet};

use super::transaction::{ArchiveEntry, ArchivePlan, MAX_INPUT_BYTES, MemberKind};
use super::{Json, PreauthError, Result, sha256_hex};

const SOURCE_DATE_EPOCH: u64 = 1_784_332_800;
const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_LAYERS: usize = 64;
const MAX_PAX_BYTES: u64 = 16 * 1024;
const MAX_PAX_RECORDS: usize = 64;

const INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";
const MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";
const LAYER_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar";
const IMAGE_NAME_ANNOTATION: &str = "io.containerd.image.name";
const REF_NAME_ANNOTATION: &str = "org.opencontainers.image.ref.name";

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

/// A pax extended header may only carry canonical timestamps, which the canonical serialization
/// discards. Any other key, and any non-timestamp value, fails closed.
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
        let (key, value) = record.split_once('=').ok_or_else(|| PreauthError::new("base-oci-pax-record"))?;
        if !matches!(key, "mtime" | "atime" | "ctime") {
            return Err(PreauthError::new("base-oci-pax-override"));
        }
        let (seconds, fraction) = value.split_once('.').unwrap_or((value, ""));
        if seconds.is_empty() || seconds.len() > 20 || !seconds.bytes().all(|byte| byte.is_ascii_digit())
            || fraction.len() > 9 || !fraction.bytes().all(|byte| byte.is_ascii_digit())
            || (value.contains('.') && fraction.is_empty())
        {
            return Err(PreauthError::new("base-oci-pax-value"));
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
        let mode = u32::try_from(tar_octal(&header[100..108])?).map_err(|_| PreauthError::new("base-oci-tar-number"))?;
        let uid = u32::try_from(tar_octal(&header[108..116])?).map_err(|_| PreauthError::new("base-oci-tar-number"))?;
        let gid = u32::try_from(tar_octal(&header[116..124])?).map_err(|_| PreauthError::new("base-oci-tar-number"))?;
        if uid != 0 || gid != 0 { return Err(PreauthError::new("base-oci-member-owner")); }
        let exact_mode = if kind == MemberKind::Directory { 0o755 } else { 0o644 };
        if mode != exact_mode { return Err(PreauthError::new("base-oci-member-mode")); }
        members.push(RawMember { path, kind, mode, uid, gid, payload: &raw[payload_start..data_end] });
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

/// `expected_image_name` and `expected_ref_name` are the exact pinned digest-pull identity the
/// export's index descriptor annotations must carry; a digest pull has no tag authority, so
/// `RepoTags` must be exactly null and no `repositories` root may exist.
pub fn canonicalize_base_oci(raw: &[u8], expected_image_name: &str, expected_ref_name: &str) -> Result<BaseOciCanonical> {
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
                } else if member.path == "repositories" {
                    return Err(PreauthError::new("base-oci-repositories-present"));
                } else if !matches!(member.path.as_str(), "oci-layout" | "index.json" | "manifest.json") {
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
    index.exact_keys(&["schemaVersion", "mediaType", "manifests"], &[])
        .map_err(|_| PreauthError::new("base-oci-index-keys"))?;
    if index.get("schemaVersion")?.number()? != 2 { return Err(PreauthError::new("base-oci-schema-version")); }
    if index.get("mediaType")?.string()? != INDEX_MEDIA_TYPE { return Err(PreauthError::new("base-oci-media-type")); }
    let descriptors = index.get("manifests")?.array()?;
    if descriptors.len() != 1 { return Err(PreauthError::new("base-oci-manifest-count")); }
    descriptors[0].exact_keys(&["mediaType", "digest", "size", "annotations"], &[])
        .map_err(|_| PreauthError::new("base-oci-descriptor-keys"))?;
    let annotations = descriptors[0].get("annotations")?;
    annotations.exact_keys(&[IMAGE_NAME_ANNOTATION, REF_NAME_ANNOTATION], &[])
        .map_err(|_| PreauthError::new("base-oci-annotation-keys"))?;
    if annotations.get(IMAGE_NAME_ANNOTATION)?.string()? != expected_image_name
        || annotations.get(REF_NAME_ANNOTATION)?.string()? != expected_ref_name
    {
        return Err(PreauthError::new("base-oci-annotations"));
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
    let config_descriptor = manifest.get("config")?;
    config_descriptor.exact_keys(&["mediaType", "digest", "size"], &[])
        .map_err(|_| PreauthError::new("base-oci-descriptor-keys"))?;
    let (config_hex, config_bytes) = resolve_blob(&blobs, config_descriptor, CONFIG_MEDIA_TYPE)?;
    let layer_descriptors = manifest.get("layers")?.array()?;
    if layer_descriptors.is_empty() || layer_descriptors.len() > MAX_LAYERS {
        return Err(PreauthError::new("base-oci-layer-count"));
    }
    let mut layer_hexes = Vec::new();
    for descriptor in layer_descriptors {
        descriptor.exact_keys(&["mediaType", "digest", "size"], &[])
            .map_err(|_| PreauthError::new("base-oci-descriptor-keys"))?;
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
    entry.exact_keys(&["Config", "RepoTags", "Layers", "LayerSources"], &[])
        .map_err(|_| PreauthError::new("base-oci-legacy-keys"))?;
    if entry.get("Config")?.string()? != format!("blobs/sha256/{config_hex}") {
        return Err(PreauthError::new("base-oci-legacy-config"));
    }
    if entry.get("RepoTags")? != &Json::Null { return Err(PreauthError::new("base-oci-repo-tags")); }
    let legacy_layers = entry.get("Layers")?.array()?;
    if legacy_layers.len() != layer_hexes.len() { return Err(PreauthError::new("base-oci-legacy-layers")); }
    for (layer, hex) in legacy_layers.iter().zip(&layer_hexes) {
        if layer.string()? != format!("blobs/sha256/{hex}") { return Err(PreauthError::new("base-oci-legacy-layers")); }
    }
    let sources = entry.get("LayerSources")?.object()?;
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

    // ADR 0018: content-addressed store metadata with no inbound graph edge is omitted from
    // canonical generation and can never enter evidence. Every blob was still digest-verified
    // above; only the rooted graph reaches the canonical archive.
    let mut referenced: BTreeSet<&str> = BTreeSet::new();
    referenced.insert(&manifest_hex);
    referenced.insert(&config_hex);
    for hex in &layer_hexes { referenced.insert(hex); }
    files.retain(|path, _| match path.strip_prefix("blobs/sha256/") {
        Some(hex) => referenced.contains(hex),
        None => true,
    });

    Ok(BaseOciCanonical {
        canonical: emit_canonical(&files),
        config_sha256: config_hex,
        manifest_sha256: manifest_hex,
        layer_count: layer_hexes.len(),
    })
}

fn escaped(bytes: &[u8], cap: usize) -> String {
    let mut output = String::new();
    for byte in bytes.iter().take(cap) {
        if byte.is_ascii_graphic() || *byte == b' ' { output.push(char::from(*byte)); }
        else { output.push_str(&format!("\\x{byte:02x}")); }
    }
    if bytes.len() > cap { output.push_str("..."); }
    output
}

/// Bounded, value-redacting-where-large, tolerant description of a raw export's observed shape.
/// Used only for failure diagnostics; it grants nothing and validates nothing.
pub fn describe_base_oci(raw: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut offset = 0usize;
    let mut metadata: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    while offset + 512 <= raw.len() && lines.len() < 96 {
        let header = &raw[offset..offset + 512];
        if header.iter().all(|byte| *byte == 0) { offset += 512; continue; }
        let name_end = header[..100].iter().position(|byte| *byte == 0).unwrap_or(100);
        let name = String::from_utf8_lossy(&header[..name_end]).into_owned();
        let size = tar_octal(&header[124..136]).unwrap_or(u64::MAX);
        if size == u64::MAX { lines.push(format!("member path={} size=unparseable", escaped(name.as_bytes(), 120))); break; }
        let mode = tar_octal(&header[100..108]).unwrap_or(0);
        let uid = tar_octal(&header[108..116]).unwrap_or(u64::MAX);
        let gid = tar_octal(&header[116..124]).unwrap_or(u64::MAX);
        let kind = header[156];
        lines.push(format!("member path={} type={} mode={:04o} uid={} gid={} size={}",
            escaped(name.as_bytes(), 120), escaped(&[kind.max(b'0')], 4), mode, uid, gid, size));
        let payload_start = offset + 512;
        let padded = (size as usize).div_ceil(512) * 512;
        if payload_start + padded > raw.len() { lines.push("truncated payload".into()); break; }
        if kind == b'x' || ((kind == 0 || kind == b'0') && size <= 4096 && !name.starts_with("blobs/")) {
            metadata.insert(name, raw[payload_start..payload_start + size as usize].to_vec());
        }
        offset = payload_start + padded;
    }
    for (name, bytes) in &metadata {
        lines.push(format!("payload {}={}", escaped(name.as_bytes(), 120), escaped(bytes, 640)));
    }
    lines.truncate(120);
    lines
}
