use std::collections::{BTreeMap, BTreeSet};

use super::{InputLockV4, PackageManifest, PreauthError, Result, TransactionGraphV1, sha256_hex};

pub const MAX_INPUT_OBJECTS: usize = 64;
pub const MAX_ARCHIVE_MEMBERS: usize = 4096;
pub const MAX_INPUT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const MAX_EXPANDED_BYTES: u64 = 8 * 1024 * 1024 * 1024;
pub const MAX_DEB_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_MEMBER_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_EXPANSION_RATIO: u64 = 64;
pub const MAX_PATH_BYTES: usize = 512;
pub const MAX_PATH_COMPONENTS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemberKind { File, Directory, Symlink, Hardlink }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntry {
    pub path: String,
    pub kind: MemberKind,
    pub compressed_bytes: u64,
    pub expanded_bytes: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub link_target: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchivePlan {
    pub entries: Vec<ArchiveEntry>,
    pub compressed_bytes: u64,
    pub expanded_bytes: u64,
}

fn normal_components(path: &str) -> Result<Vec<&str>> {
    if path.is_empty() || path.len() > MAX_PATH_BYTES || path.starts_with('/')
        || path.starts_with('\\') || path.contains('\\') || path.contains(':')
        || path.contains("//") || !path.is_ascii() || path.chars().any(char::is_control)
    {
        return Err(PreauthError::new("transaction-path"));
    }
    let components: Vec<_> = path.split('/').collect();
    if components.len() > MAX_PATH_COMPONENTS || components.iter().any(|component| {
        component.is_empty() || *component == "." || *component == ".."
            || component.len() > 255
    }) {
        return Err(PreauthError::new("transaction-path"));
    }
    Ok(components)
}

impl ArchivePlan {
    pub fn validate(entries: Vec<ArchiveEntry>) -> Result<Self> {
        if entries.len() > MAX_ARCHIVE_MEMBERS { return Err(PreauthError::new("archive-member-count")); }
        let mut exact = BTreeSet::new();
        let mut folded = BTreeSet::new();
        let mut kinds: BTreeMap<String, MemberKind> = BTreeMap::new();
        let mut compressed_total = 0u64;
        let mut expanded_total = 0u64;
        for entry in &entries {
            let components = normal_components(&entry.path)?;
            if !exact.insert(entry.path.clone()) || !folded.insert(entry.path.to_lowercase()) {
                return Err(PreauthError::new("archive-path-collision"));
            }
            if entry.compressed_bytes > MAX_MEMBER_BYTES || entry.expanded_bytes > MAX_MEMBER_BYTES
                || entry.expanded_bytes > entry.compressed_bytes.saturating_mul(MAX_EXPANSION_RATIO)
            {
                return Err(PreauthError::new("archive-size-bound"));
            }
            compressed_total = compressed_total.checked_add(entry.compressed_bytes)
                .ok_or_else(|| PreauthError::new("archive-size-overflow"))?;
            expanded_total = expanded_total.checked_add(entry.expanded_bytes)
                .ok_or_else(|| PreauthError::new("archive-size-overflow"))?;
            if compressed_total > MAX_INPUT_BYTES || expanded_total > MAX_EXPANDED_BYTES {
                return Err(PreauthError::new("archive-aggregate-bound"));
            }
            if entry.uid != 0 || entry.gid != 0 || entry.mode & !0o755 != 0 {
                return Err(PreauthError::new("archive-metadata"));
            }
            if matches!(entry.kind, MemberKind::Symlink | MemberKind::Hardlink) {
                let target = entry.link_target.as_deref().ok_or_else(|| PreauthError::new("archive-link"))?;
                normal_components(target)?;
            } else if entry.link_target.is_some() {
                return Err(PreauthError::new("archive-link"));
            }
            for depth in 1..components.len() {
                let prefix = components[..depth].join("/");
                if kinds.get(&prefix).is_some_and(|kind| *kind != MemberKind::Directory) {
                    return Err(PreauthError::new("archive-prefix-collision"));
                }
            }
            let child_prefix = format!("{}/", entry.path);
            if entry.kind != MemberKind::Directory && kinds.keys().any(|known| known.starts_with(&child_prefix)) {
                return Err(PreauthError::new("archive-prefix-collision"));
            }
            kinds.insert(entry.path.clone(), entry.kind);
        }
        for entry in &entries {
            if matches!(entry.kind, MemberKind::Symlink | MemberKind::Hardlink) {
                let target = entry.link_target.as_ref().expect("validated target");
                if !kinds.contains_key(target) {
                    return Err(PreauthError::new("archive-link-target"));
                }
            }
        }
        Ok(Self { entries, compressed_bytes: compressed_total, expanded_bytes: expanded_total })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedSnapshot {
    pub slot: String,
    pub digest: String,
    pub byte_len: u64,
    pub private_exclusive: bool,
    pub writable_aliases: u32,
    pub source_identity_before: String,
    pub source_identity_after: String,
    pub source_link_count: u32,
}

impl OwnedSnapshot {
    pub fn validate(&self) -> Result<()> {
        if self.slot.is_empty() || self.slot.len() > 128 || self.byte_len > MAX_MEMBER_BYTES
            || !self.private_exclusive || self.writable_aliases != 0 || self.source_link_count != 1
            || self.source_identity_before != self.source_identity_after
            || self.digest.len() != 64 || !self.digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(PreauthError::new("snapshot-not-exclusively-owned"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionPhase { Planned, Snapshotted, Constructed, Finalized, Published, Aborted, PublicationUncertain }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationBoundary { Snapshot, Construct, FileSync, DirectorySync, Rename, ParentSync }

pub trait TransactionEffects {
    fn snapshot(&mut self, slot: &str) -> Result<OwnedSnapshot>;
    fn construct_private(&mut self, snapshots: &[OwnedSnapshot]) -> Result<()>;
    fn sync_files(&mut self) -> Result<()>;
    fn sync_directory(&mut self) -> Result<()>;
    fn publish_no_replace(&mut self) -> Result<()>;
    fn sync_parent(&mut self) -> Result<()>;
    fn rollback_publication(&mut self) -> Result<()>;
    fn abort_private(&mut self);
}

#[derive(Debug)]
pub struct TransactionMachine {
    phase: TransactionPhase,
    slots: Vec<String>,
    snapshots: Vec<OwnedSnapshot>,
}

impl TransactionMachine {
    pub fn planned(slots: Vec<String>) -> Result<Self> {
        if slots.is_empty() || slots.len() > MAX_INPUT_OBJECTS { return Err(PreauthError::new("input-count")); }
        let unique: BTreeSet<_> = slots.iter().collect();
        if unique.len() != slots.len() || slots.iter().any(|slot| normal_components(slot).is_err()) {
            return Err(PreauthError::new("input-plan"));
        }
        Ok(Self { phase: TransactionPhase::Planned, slots, snapshots: Vec::new() })
    }

    pub fn phase(&self) -> TransactionPhase { self.phase }

    pub fn execute<E: TransactionEffects>(&mut self, effects: &mut E) -> Result<()> {
        if self.phase != TransactionPhase::Planned { return Err(PreauthError::new("transaction-reentry")); }
        let outcome = (|| {
            for slot in &self.slots {
                let snapshot = effects.snapshot(slot)?;
                snapshot.validate()?;
                self.snapshots.push(snapshot);
            }
            self.phase = TransactionPhase::Snapshotted;
            effects.construct_private(&self.snapshots)?;
            self.phase = TransactionPhase::Constructed;
            effects.sync_files()?;
            effects.sync_directory()?;
            self.phase = TransactionPhase::Finalized;
            effects.publish_no_replace()?;
            self.phase = TransactionPhase::Published;
            if let Err(error) = effects.sync_parent() {
                if effects.rollback_publication().is_err() {
                    self.phase = TransactionPhase::PublicationUncertain;
                    return Err(PreauthError::new("publication-uncertain"));
                }
                return Err(error);
            }
            Ok(())
        })();
        if outcome.is_err() {
            effects.abort_private();
            if self.phase != TransactionPhase::PublicationUncertain {
                self.phase = TransactionPhase::Aborted;
            }
        }
        outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebPlan {
    pub control_member: String,
    pub data_member: String,
    pub archive_bytes: u64,
    pub data_plan: ArchivePlan,
}

fn decimal(bytes: &[u8]) -> Result<u64> {
    let text = std::str::from_utf8(bytes).map_err(|_| PreauthError::new("ar-number"))?.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(PreauthError::new("ar-number"));
    }
    text.parse().map_err(|_| PreauthError::new("ar-number"))
}

pub fn plan_deb_ar(bytes: &[u8], expanded_data_tar: &[u8]) -> Result<DebPlan> {
    if bytes.len() as u64 > MAX_DEB_BYTES || !bytes.starts_with(b"!<arch>\n") {
        return Err(PreauthError::new("deb-framing"));
    }
    let mut offset = 8usize;
    let mut names = Vec::new();
    while offset < bytes.len() {
        let header_end = offset.checked_add(60).ok_or_else(|| PreauthError::new("ar-overflow"))?;
        if header_end > bytes.len() || &bytes[offset + 58..header_end] != b"`\n" {
            return Err(PreauthError::new("ar-framing"));
        }
        let raw_name = std::str::from_utf8(&bytes[offset..offset + 16])
            .map_err(|_| PreauthError::new("ar-name"))?.trim();
        let name = raw_name.strip_suffix('/').unwrap_or(raw_name);
        if name.is_empty() || name.contains('/') || name == "." || name == ".." || !name.is_ascii() {
            return Err(PreauthError::new("ar-name"));
        }
        let size = decimal(&bytes[offset + 48..offset + 58])?;
        let start = header_end;
        let end = start.checked_add(usize::try_from(size).map_err(|_| PreauthError::new("ar-overflow"))?)
            .ok_or_else(|| PreauthError::new("ar-overflow"))?;
        if end > bytes.len() { return Err(PreauthError::new("ar-truncated")); }
        names.push(name.to_owned());
        offset = end.checked_add((size & 1) as usize).ok_or_else(|| PreauthError::new("ar-overflow"))?;
    }
    if offset != bytes.len() || names.len() != 3 || names[0] != "debian-binary"
        || !names[1].starts_with("control.tar.") || !names[2].starts_with("data.tar.")
    {
        return Err(PreauthError::new("deb-member-set"));
    }
    let data_plan = plan_tar(expanded_data_tar, bytes.len() as u64)?;
    Ok(DebPlan { control_member: names[1].clone(), data_member: names[2].clone(),
        archive_bytes: bytes.len() as u64, data_plan })
}

fn tar_octal(bytes: &[u8]) -> Result<u64> {
    let end = bytes.iter().position(|byte| *byte == 0 || *byte == b' ').unwrap_or(bytes.len());
    let text = std::str::from_utf8(&bytes[..end]).map_err(|_| PreauthError::new("tar-number"))?.trim();
    if text.is_empty() { return Ok(0); }
    if !text.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) { return Err(PreauthError::new("tar-number")); }
    u64::from_str_radix(text, 8).map_err(|_| PreauthError::new("tar-number"))
}

pub fn plan_tar(bytes: &[u8], compressed_bytes: u64) -> Result<ArchivePlan> {
    if compressed_bytes > MAX_INPUT_BYTES || bytes.len() as u64 > MAX_EXPANDED_BYTES
        || bytes.len() as u64 > compressed_bytes.saturating_mul(MAX_EXPANSION_RATIO)
    {
        return Err(PreauthError::new("archive-aggregate-bound"));
    }
    let mut offset = 0usize;
    let mut entries = Vec::new();
    let mut zero_blocks = 0u8;
    while offset < bytes.len() {
        let end = offset.checked_add(512).ok_or_else(|| PreauthError::new("tar-overflow"))?;
        if end > bytes.len() { return Err(PreauthError::new("tar-truncated")); }
        let header = &bytes[offset..end];
        if header.iter().all(|byte| *byte == 0) {
            zero_blocks += 1; offset = end;
            if zero_blocks == 2 {
                if bytes[offset..].iter().any(|byte| *byte != 0) { return Err(PreauthError::new("tar-trailing-data")); }
                break;
            }
            continue;
        }
        if zero_blocks != 0 { return Err(PreauthError::new("tar-zero-block")); }
        let name_end = header[..100].iter().position(|byte| *byte == 0).unwrap_or(100);
        let name = std::str::from_utf8(&header[..name_end]).map_err(|_| PreauthError::new("tar-name"))?;
        let prefix_end = header[345..500].iter().position(|byte| *byte == 0).unwrap_or(155);
        let prefix = std::str::from_utf8(&header[345..345 + prefix_end]).map_err(|_| PreauthError::new("tar-name"))?;
        let raw_path = if prefix.is_empty() { name.to_owned() } else { format!("{prefix}/{name}") };
        let mode = tar_octal(&header[100..108])? as u32;
        let uid = tar_octal(&header[108..116])? as u32;
        let gid = tar_octal(&header[116..124])? as u32;
        let size = tar_octal(&header[124..136])?;
        let expected_checksum = tar_octal(&header[148..156])?;
        let actual_checksum: u64 = header.iter().enumerate().map(|(index, byte)| {
            if (148..156).contains(&index) { b' ' as u64 } else { *byte as u64 }
        }).sum();
        if expected_checksum != actual_checksum || &header[257..263] != b"ustar\0" {
            return Err(PreauthError::new("tar-header-integrity"));
        }
        let kind = match header[156] { 0 | b'0' => MemberKind::File, b'5' => MemberKind::Directory,
            b'2' => MemberKind::Symlink, b'1' => MemberKind::Hardlink,
            _ => return Err(PreauthError::new("tar-special-file")) };
        let path = if kind == MemberKind::Directory {
            raw_path.strip_suffix('/').unwrap_or(&raw_path).to_owned()
        } else {
            raw_path
        };
        if kind != MemberKind::File && size != 0 { return Err(PreauthError::new("tar-nonfile-size")); }
        let link_end = header[157..257].iter().position(|byte| *byte == 0).unwrap_or(100);
        let link = std::str::from_utf8(&header[157..157 + link_end]).map_err(|_| PreauthError::new("tar-link"))?;
        let payload_start = end;
        let padded = size.checked_add(511).ok_or_else(|| PreauthError::new("tar-overflow"))? / 512 * 512;
        let payload_end = payload_start.checked_add(usize::try_from(padded).map_err(|_| PreauthError::new("tar-overflow"))?)
            .ok_or_else(|| PreauthError::new("tar-overflow"))?;
        if payload_end > bytes.len() { return Err(PreauthError::new("tar-truncated")); }
        entries.push(ArchiveEntry { path, kind, compressed_bytes: size, expanded_bytes: size,
            mode, uid, gid, link_target: (!link.is_empty()).then(|| link.to_owned()) });
        offset = payload_end;
    }
    if zero_blocks != 2 { return Err(PreauthError::new("tar-end-marker")); }
    ArchivePlan::validate(entries)
}

pub fn validate_closure_inputs(
    lock: &InputLockV4, packages: &str, license_manifest: &[u8], observed: &BTreeMap<String, String>,
) -> Result<PackageManifest> {
    let package_record = PackageManifest::parse(packages)?;
    if package_record.rows.len() != 36
        || lock.fields.get("package_manifest_sha256") != Some(&sha256_hex(packages.as_bytes()))
        || lock.fields.get("license_manifest_sha256") != Some(&sha256_hex(license_manifest))
    {
        return Err(PreauthError::new("closure-manifest-mismatch"));
    }
    for key in ["base_oci_index_sha256", "debian_archive_keyring_sha256", "inrelease_sha256",
        "security_inrelease_sha256", "lld_sha256", "qemu_sha256", "ovmf_code_sha256",
        "ovmf_vars_sha256", "acquisition_policy_sha256"] {
        if observed.get(key) != lock.fields.get(key) { return Err(PreauthError::new("closure-input-mismatch")); }
    }
    Ok(package_record)
}

#[derive(Debug)]
pub struct FrozenTransactionGraph { bytes: Vec<u8>, graph: TransactionGraphV1 }

impl FrozenTransactionGraph {
    pub fn emit_once(mut nodes: BTreeMap<String, String>) -> Result<Self> {
        let mut payload = String::new();
        for name in &super::TRANSACTION_GRAPH_FIELDS[..super::TRANSACTION_GRAPH_FIELDS.len() - 1] {
            let value = nodes.remove(*name).ok_or_else(|| PreauthError::new("transaction-graph-omission"))?;
            payload.push_str(name); payload.push('='); payload.push_str(&value); payload.push('\n');
        }
        if !nodes.is_empty() { return Err(PreauthError::new("transaction-graph-extra")); }
        let bytes = format!("{payload}record_sha256={}\n", sha256_hex(payload.as_bytes())).into_bytes();
        let text = std::str::from_utf8(&bytes).map_err(|_| PreauthError::new("transaction-graph-encoding"))?;
        let graph = TransactionGraphV1::parse(text)?;
        Ok(Self { bytes, graph })
    }
    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub fn graph(&self) -> &TransactionGraphV1 { &self.graph }
}
