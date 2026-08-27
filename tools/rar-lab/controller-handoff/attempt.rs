//! Deterministic, side-effect-free structural codec for controller attempts.
//!
//! This module contains no filesystem, process, clock, environment, network, or
//! cloud operations. The trusted outer controller must supply already observed
//! identities and durable ordering; these types only validate and encode them.

use crate::{Role, sha256};

pub const ACTIVE_HEADER_BYTES: usize = 512;
pub const ROOT_RECORD_BYTES: usize = 128;
pub const EXPECTED_ENTRY_BYTES: usize = 256;
pub const TRANSITION_BYTES: usize = 512;
pub const RECOVERY_HEADER_BYTES: usize = 256;
pub const RECOVERY_ENTRY_BYTES: usize = 192;
pub const ACTIVE_MAXIMUM_BYTES: usize = 256_768;
pub const RECOVERY_MAXIMUM_BYTES: usize = 383_872;
pub const EXIT_UNOBSERVED: i32 = i32::MIN;

const ACTIVE_MAGIC: [u8; 8] = *b"RARACTV0";
const TRANSITION_MAGIC: [u8; 8] = *b"RARTRNV0";
const RECOVERY_MAGIC: [u8; 8] = *b"RARRCVV0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptError {
    Size, Magic, Version, Bounds, Reserved, Hash, Identity, Canonical,
    State, Chain, Session, Recovery, Field,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum RootKind { Source = 1, Destination = 2, Manifest = 3 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ExpectedKind { Artifact = 1, Transcript = 2, ComparisonEvidence = 3, LaunchEvidence = 4 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum AttemptState {
    Prepared = 1, StartAuthorized = 2, RunningObserved = 3,
    ExitedSuccess = 4, ExitedFailure = 5, StopRequested = 6,
    StoppedObserved = 7, RecoveryRequired = 8,
    RecoveryInventoryDurable = 9, OutputsValidated = 10,
    Committed = 11, Discarded = 12, Blocked = 13,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AttemptCause {
    None = 0, Spawn = 1, Timeout = 2, Cancelled = 3, ExitNonzero = 4,
    Journal = 5, Validation = 6, Recovery = 7, Identity = 8,
    Sync = 9, Policy = 10,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptRoot {
    pub kind: RootKind,
    pub index: u16,
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub initial_entry_table_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptExpected {
    pub source_root_index: u16,
    pub kind: ExpectedKind,
    pub ordinal: u16,
    pub maximum_bytes: u64,
    pub source_basename: String,
    pub destination_basename: String,
    pub manifest_basename: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptActive {
    pub phase: u16,
    pub role: Role,
    pub milestone: u16,
    pub attempt_ordinal: u16,
    pub watchdog_seconds: u32,
    pub termination_grace_seconds: u32,
    pub total_controller_seconds: u32,
    pub helper_unavailable: bool,
    pub task_nonce: [u8; 32],
    pub attempt_nonce: [u8; 32],
    pub controller_session_nonce: [u8; 32],
    pub task_binding_sha256: [u8; 32],
    pub controller_source_sha256: [u8; 32],
    pub helper_sha256: [u8; 32],
    pub expected_table_sha256: [u8; 32],
    pub journal_device: u64,
    pub journal_inode: u64,
    pub active_device: u64,
    pub active_inode: u64,
    pub roots: Vec<AttemptRoot>,
    pub expected: Vec<AttemptExpected>,
}

impl AttemptActive {
    pub fn encode(&self) -> Result<Vec<u8>, AttemptError> {
        self.validate()?;
        let total = ACTIVE_HEADER_BYTES + self.roots.len() * ROOT_RECORD_BYTES
            + self.expected.len() * EXPECTED_ENTRY_BYTES;
        let mut out = vec![0u8; total];
        out[..8].copy_from_slice(&ACTIVE_MAGIC);
        put_u32(&mut out, 12, ACTIVE_HEADER_BYTES as u32);
        put_u32(&mut out, 16, total as u32);
        put_u16(&mut out, 20, self.phase);
        put_u16(&mut out, 22, self.role as u16);
        put_u16(&mut out, 24, self.milestone);
        put_u16(&mut out, 26, self.attempt_ordinal);
        put_u16(&mut out, 28, self.roots.len() as u16);
        put_u16(&mut out, 30, self.expected.len() as u16);
        put_u32(&mut out, 32, self.watchdog_seconds);
        put_u32(&mut out, 36, self.termination_grace_seconds);
        put_u32(&mut out, 40, self.total_controller_seconds);
        put_u32(&mut out, 44, u32::from(self.helper_unavailable));
        put_digest(&mut out, 48, self.task_nonce);
        put_digest(&mut out, 80, self.attempt_nonce);
        put_digest(&mut out, 112, self.controller_session_nonce);
        put_digest(&mut out, 144, self.task_binding_sha256);
        put_digest(&mut out, 176, self.controller_source_sha256);
        put_digest(&mut out, 208, self.helper_sha256);
        put_digest(&mut out, 240, self.expected_table_sha256);
        put_u64(&mut out, 272, self.journal_device);
        put_u64(&mut out, 280, self.journal_inode);
        put_u64(&mut out, 288, self.active_device);
        put_u64(&mut out, 296, self.active_inode);
        let mut offset = ACTIVE_HEADER_BYTES;
        for root in &self.roots {
            encode_root(root, &mut out[offset..offset + ROOT_RECORD_BYTES]);
            offset += ROOT_RECORD_BYTES;
        }
        for expected in &self.expected {
            encode_expected(expected, &mut out[offset..offset + EXPECTED_ENTRY_BYTES]);
            offset += EXPECTED_ENTRY_BYTES;
        }
        let digest = hash_with_zeroed(&out, 304, 32);
        put_digest(&mut out, 304, digest);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, AttemptError> {
        if bytes.len() < ACTIVE_HEADER_BYTES || bytes.len() > ACTIVE_MAXIMUM_BYTES { return Err(AttemptError::Size); }
        if bytes[..8] != ACTIVE_MAGIC { return Err(AttemptError::Magic); }
        if get_u16(bytes, 8) != 0 || get_u16(bytes, 10) != 0 { return Err(AttemptError::Version); }
        if get_u32(bytes, 12) as usize != ACTIVE_HEADER_BYTES || get_u32(bytes, 16) as usize != bytes.len() { return Err(AttemptError::Size); }
        if get_u32(bytes, 44) & !1 != 0 || bytes[336..512].iter().any(|byte| *byte != 0) { return Err(AttemptError::Reserved); }
        if digest_at(bytes, 304) != hash_with_zeroed(bytes, 304, 32) { return Err(AttemptError::Hash); }
        let root_count = get_u16(bytes, 28) as usize;
        let expected_count = get_u16(bytes, 30) as usize;
        let wanted = ACTIVE_HEADER_BYTES.checked_add(root_count.checked_mul(ROOT_RECORD_BYTES).ok_or(AttemptError::Bounds)?)
            .and_then(|v| v.checked_add(expected_count.checked_mul(EXPECTED_ENTRY_BYTES)?)).ok_or(AttemptError::Bounds)?;
        if wanted != bytes.len() { return Err(AttemptError::Size); }
        let mut roots = Vec::with_capacity(root_count);
        let mut offset = ACTIVE_HEADER_BYTES;
        for _ in 0..root_count {
            roots.push(decode_root(&bytes[offset..offset + ROOT_RECORD_BYTES])?);
            offset += ROOT_RECORD_BYTES;
        }
        let mut expected = Vec::with_capacity(expected_count);
        for _ in 0..expected_count {
            expected.push(decode_expected(&bytes[offset..offset + EXPECTED_ENTRY_BYTES])?);
            offset += EXPECTED_ENTRY_BYTES;
        }
        let value = Self {
            phase: get_u16(bytes, 20), role: decode_role(get_u16(bytes, 22))?, milestone: get_u16(bytes, 24),
            attempt_ordinal: get_u16(bytes, 26), watchdog_seconds: get_u32(bytes, 32),
            termination_grace_seconds: get_u32(bytes, 36), total_controller_seconds: get_u32(bytes, 40),
            helper_unavailable: get_u32(bytes, 44) == 1, task_nonce: digest_at(bytes, 48),
            attempt_nonce: digest_at(bytes, 80), controller_session_nonce: digest_at(bytes, 112),
            task_binding_sha256: digest_at(bytes, 144), controller_source_sha256: digest_at(bytes, 176),
            helper_sha256: digest_at(bytes, 208), expected_table_sha256: digest_at(bytes, 240),
            journal_device: get_u64(bytes, 272), journal_inode: get_u64(bytes, 280),
            active_device: get_u64(bytes, 288), active_inode: get_u64(bytes, 296), roots, expected,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), AttemptError> {
        if !(1..=3).contains(&self.attempt_ordinal) || !(1..=1200).contains(&self.watchdog_seconds)
            || !(1..=30).contains(&self.termination_grace_seconds) || !(1..=3600).contains(&self.total_controller_seconds)
            || !(3..=4).contains(&self.roots.len()) || !(1..=999).contains(&self.expected.len()) { return Err(AttemptError::Bounds); }
        let nonzero = |digest: &[u8; 32]| digest.iter().any(|byte| *byte != 0);
        if !nonzero(&self.task_nonce) || !nonzero(&self.attempt_nonce) || !nonzero(&self.controller_session_nonce)
            || self.task_nonce == self.attempt_nonce || self.task_nonce == self.controller_session_nonce
            || self.attempt_nonce == self.controller_session_nonce || !nonzero(&self.task_binding_sha256)
            || !nonzero(&self.controller_source_sha256) || !nonzero(&self.expected_table_sha256)
            || (!self.helper_unavailable && !nonzero(&self.helper_sha256))
            || (self.helper_unavailable && nonzero(&self.helper_sha256))
            || [self.journal_device, self.journal_inode, self.active_device, self.active_inode].contains(&0) { return Err(AttemptError::Identity); }
        let sources = self.roots.iter().take_while(|root| root.kind == RootKind::Source).count();
        if !(1..=2).contains(&sources) || self.roots.len() != sources + 2 { return Err(AttemptError::Canonical); }
        for (position, root) in self.roots.iter().enumerate() {
            let expected_kind = if position < sources { RootKind::Source } else if position == sources { RootKind::Destination } else { RootKind::Manifest };
            if root.kind != expected_kind || root.index as usize != position + 1 || root.device == 0 || root.inode == 0 || root.mode != 0o700
                || root.initial_entry_table_sha256 == [0; 32] { return Err(AttemptError::Canonical); }
            if self.roots[..position].iter().any(|prior| prior.device == root.device && prior.inode == root.inode) { return Err(AttemptError::Identity); }
        }
        let mut source_names = std::collections::BTreeSet::new();
        let mut destination_names = std::collections::BTreeSet::new();
        let mut manifest_names = std::collections::BTreeSet::new();
        for (position, entry) in self.expected.iter().enumerate() {
            if entry.source_root_index == 0 || entry.source_root_index as usize > sources || entry.ordinal == 0 || entry.ordinal > 999
                || entry.maximum_bytes == 0 || !canonical_basename(&entry.source_basename)
                || !canonical_basename(&entry.destination_basename) || !canonical_basename(&entry.manifest_basename)
                || !source_names.insert((entry.source_root_index, entry.source_basename.as_str()))
                || !destination_names.insert(entry.destination_basename.as_str()) || !manifest_names.insert(entry.manifest_basename.as_str()) { return Err(AttemptError::Canonical); }
            if position != 0 && expected_key(&self.expected[position - 1]) >= expected_key(entry) { return Err(AttemptError::Canonical); }
        }
        let mut table = vec![0u8; self.expected.len() * EXPECTED_ENTRY_BYTES];
        for (index, entry) in self.expected.iter().enumerate() {
            encode_expected(entry, &mut table[index * EXPECTED_ENTRY_BYTES..(index + 1) * EXPECTED_ENTRY_BYTES]);
        }
        if sha256(&table) != self.expected_table_sha256 { return Err(AttemptError::Hash); }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptTransition {
    pub sequence: u32, pub state: AttemptState, pub phase: u16, pub role: Role,
    pub recovery_ordinal: u16, pub exit_status: i32, pub cause: AttemptCause,
    pub monotonic_nanoseconds: u64, pub active_sha256: [u8; 32],
    pub previous_sha256: [u8; 32], pub receipt_sha256: [u8; 32],
    pub inventory_sha256: [u8; 32], pub helper_sha256: [u8; 32],
    pub active_device: u64, pub active_inode: u64,
    pub transition_device: u64, pub transition_inode: u64,
    pub controller_session_nonce: [u8; 32],
}

impl AttemptTransition {
    pub fn encode(&self) -> Result<[u8; TRANSITION_BYTES], AttemptError> {
        self.validate_fields()?;
        let mut out = [0u8; TRANSITION_BYTES];
        out[..8].copy_from_slice(&TRANSITION_MAGIC);
        put_u32(&mut out, 12, TRANSITION_BYTES as u32); put_u32(&mut out, 16, self.sequence);
        put_u16(&mut out, 20, self.state as u16); put_u16(&mut out, 22, self.phase);
        put_u16(&mut out, 24, self.role as u16); put_u16(&mut out, 26, self.recovery_ordinal);
        put_i32(&mut out, 28, self.exit_status); put_u32(&mut out, 32, self.cause as u32);
        put_u64(&mut out, 40, self.monotonic_nanoseconds); put_digest(&mut out, 48, self.active_sha256);
        put_digest(&mut out, 80, self.previous_sha256); put_digest(&mut out, 112, self.receipt_sha256);
        put_digest(&mut out, 144, self.inventory_sha256); put_digest(&mut out, 176, self.helper_sha256);
        put_u64(&mut out, 208, self.active_device); put_u64(&mut out, 216, self.active_inode);
        put_u64(&mut out, 224, self.transition_device); put_u64(&mut out, 232, self.transition_inode);
        put_digest(&mut out, 272, self.controller_session_nonce);
        let digest = hash_with_zeroed(&out, 240, 32); put_digest(&mut out, 240, digest);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, AttemptError> {
        if bytes.len() != TRANSITION_BYTES { return Err(AttemptError::Size); }
        if bytes[..8] != TRANSITION_MAGIC { return Err(AttemptError::Magic); }
        if get_u16(bytes, 8) != 0 || get_u16(bytes, 10) != 0 { return Err(AttemptError::Version); }
        if get_u32(bytes, 12) as usize != TRANSITION_BYTES || get_u32(bytes, 36) != 0 || bytes[304..].iter().any(|b| *b != 0) { return Err(AttemptError::Reserved); }
        if digest_at(bytes, 240) != hash_with_zeroed(bytes, 240, 32) { return Err(AttemptError::Hash); }
        let value = Self {
            sequence: get_u32(bytes, 16), state: decode_state(get_u16(bytes, 20))?, phase: get_u16(bytes, 22), role: decode_role(get_u16(bytes, 24))?,
            recovery_ordinal: get_u16(bytes, 26), exit_status: get_i32(bytes, 28), cause: decode_cause(get_u32(bytes, 32))?,
            monotonic_nanoseconds: get_u64(bytes, 40), active_sha256: digest_at(bytes, 48), previous_sha256: digest_at(bytes, 80),
            receipt_sha256: digest_at(bytes, 112), inventory_sha256: digest_at(bytes, 144), helper_sha256: digest_at(bytes, 176),
            active_device: get_u64(bytes, 208), active_inode: get_u64(bytes, 216), transition_device: get_u64(bytes, 224),
            transition_inode: get_u64(bytes, 232), controller_session_nonce: digest_at(bytes, 272),
        };
        value.validate_fields()?; Ok(value)
    }

    pub fn digest(&self) -> Result<[u8; 32], AttemptError> { Ok(digest_at(&self.encode()?, 240)) }

    fn validate_fields(&self) -> Result<(), AttemptError> {
        if self.sequence == 0 || self.sequence > 4096 || self.recovery_ordinal > 3
            || [self.active_device, self.active_inode, self.transition_device, self.transition_inode].contains(&0)
            || self.active_sha256 == [0; 32] || self.previous_sha256 == [0; 32]
            || self.controller_session_nonce == [0; 32] { return Err(AttemptError::Bounds); }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryEntry {
    pub root_kind: RootKind, pub root_index: u16, pub basename: String,
    pub mode: u32, pub uid: u32, pub gid: u32, pub link_count: u64,
    pub size: u64, pub device: u64, pub inode: u64, pub sha256: [u8; 32],
    pub modified_seconds: i64, pub modified_nanoseconds: i64,
    pub changed_seconds: i64, pub changed_nanoseconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryInventory {
    pub origin_recovery_ordinal: u16, pub sequence: u32,
    pub active_sha256: [u8; 32], pub previous_transition_sha256: [u8; 32],
    pub expected_table_sha256: [u8; 32], pub origin_controller_session_nonce: [u8; 32],
    pub entries: Vec<RecoveryEntry>,
}

impl RecoveryInventory {
    pub fn encode(&self) -> Result<Vec<u8>, AttemptError> {
        self.validate()?;
        let total = RECOVERY_HEADER_BYTES + self.entries.len() * RECOVERY_ENTRY_BYTES;
        let mut out = vec![0u8; total]; out[..8].copy_from_slice(&RECOVERY_MAGIC);
        put_u32(&mut out, 12, RECOVERY_HEADER_BYTES as u32); put_u32(&mut out, 16, total as u32);
        put_u16(&mut out, 20, self.entries.len() as u16); put_u16(&mut out, 22, self.origin_recovery_ordinal);
        put_u32(&mut out, 24, self.sequence); put_digest(&mut out, 32, self.active_sha256);
        put_digest(&mut out, 64, self.previous_transition_sha256); put_digest(&mut out, 96, self.expected_table_sha256);
        put_digest(&mut out, 160, self.origin_controller_session_nonce);
        for (index, entry) in self.entries.iter().enumerate() { encode_recovery_entry(entry, &mut out[RECOVERY_HEADER_BYTES + index * RECOVERY_ENTRY_BYTES..RECOVERY_HEADER_BYTES + (index + 1) * RECOVERY_ENTRY_BYTES]); }
        let digest = hash_with_zeroed(&out, 128, 32); put_digest(&mut out, 128, digest); Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, AttemptError> {
        if bytes.len() < RECOVERY_HEADER_BYTES || bytes.len() > RECOVERY_MAXIMUM_BYTES { return Err(AttemptError::Size); }
        if bytes[..8] != RECOVERY_MAGIC { return Err(AttemptError::Magic); }
        if get_u16(bytes, 8) != 0 || get_u16(bytes, 10) != 0 { return Err(AttemptError::Version); }
        if get_u32(bytes, 12) as usize != RECOVERY_HEADER_BYTES || get_u32(bytes, 16) as usize != bytes.len()
            || get_u32(bytes, 28) != 0 || bytes[192..256].iter().any(|b| *b != 0) { return Err(AttemptError::Reserved); }
        if digest_at(bytes, 128) != hash_with_zeroed(bytes, 128, 32) { return Err(AttemptError::Hash); }
        let count = get_u16(bytes, 20) as usize;
        if RECOVERY_HEADER_BYTES + count.checked_mul(RECOVERY_ENTRY_BYTES).ok_or(AttemptError::Bounds)? != bytes.len() { return Err(AttemptError::Size); }
        let mut entries = Vec::with_capacity(count);
        for index in 0..count { entries.push(decode_recovery_entry(&bytes[RECOVERY_HEADER_BYTES + index * RECOVERY_ENTRY_BYTES..RECOVERY_HEADER_BYTES + (index + 1) * RECOVERY_ENTRY_BYTES])?); }
        let value = Self { origin_recovery_ordinal: get_u16(bytes, 22), sequence: get_u32(bytes, 24), active_sha256: digest_at(bytes, 32), previous_transition_sha256: digest_at(bytes, 64), expected_table_sha256: digest_at(bytes, 96), origin_controller_session_nonce: digest_at(bytes, 160), entries };
        value.validate()?; Ok(value)
    }

    pub fn digest(&self) -> Result<[u8; 32], AttemptError> { Ok(digest_at(&self.encode()?, 128)) }

    fn validate(&self) -> Result<(), AttemptError> {
        if !(1..=3).contains(&self.origin_recovery_ordinal) || self.sequence == 0 || self.sequence > 4096 || self.entries.len() > 1998
            || self.active_sha256 == [0; 32] || self.previous_transition_sha256 == [0; 32]
            || self.expected_table_sha256 == [0; 32] || self.origin_controller_session_nonce == [0; 32] { return Err(AttemptError::Bounds); }
        for (index, entry) in self.entries.iter().enumerate() {
            if !matches!(entry.root_kind, RootKind::Destination | RootKind::Manifest) || entry.root_index == 0 || !canonical_basename(&entry.basename)
                || entry.mode != 0o600 || entry.link_count != 1 || entry.device == 0 || entry.inode == 0 || entry.sha256 == [0; 32]
                || !(0..1_000_000_000).contains(&entry.modified_nanoseconds) || !(0..1_000_000_000).contains(&entry.changed_nanoseconds) { return Err(AttemptError::Canonical); }
            if index != 0 && recovery_key(&self.entries[index - 1]) >= recovery_key(entry) { return Err(AttemptError::Canonical); }
        }
        Ok(())
    }
}

fn encode_root(value: &AttemptRoot, out: &mut [u8]) {
    put_u16(out, 0, value.kind as u16); put_u16(out, 2, value.index); put_u64(out, 8, value.device);
    put_u64(out, 16, value.inode); put_u32(out, 24, value.uid); put_u32(out, 28, value.gid);
    put_u32(out, 32, value.mode); put_u16(out, 36, 2); put_digest(out, 40, value.initial_entry_table_sha256);
}

fn decode_root(bytes: &[u8]) -> Result<AttemptRoot, AttemptError> {
    if get_u32(bytes, 4) != 0 || get_u16(bytes, 36) != 2 || get_u16(bytes, 38) != 0 || bytes[72..].iter().any(|b| *b != 0) { return Err(AttemptError::Reserved); }
    Ok(AttemptRoot { kind: decode_root_kind(get_u16(bytes, 0))?, index: get_u16(bytes, 2), device: get_u64(bytes, 8), inode: get_u64(bytes, 16), uid: get_u32(bytes, 24), gid: get_u32(bytes, 28), mode: get_u32(bytes, 32), initial_entry_table_sha256: digest_at(bytes, 40) })
}

fn encode_expected(value: &AttemptExpected, out: &mut [u8]) {
    put_u16(out, 0, value.source_root_index); put_u16(out, 2, value.kind as u16); put_u16(out, 4, value.ordinal);
    put_u64(out, 8, value.maximum_bytes); put_u16(out, 20, value.source_basename.len() as u16);
    put_u16(out, 22, value.destination_basename.len() as u16); put_u16(out, 24, value.manifest_basename.len() as u16);
    put_name(out, 32, &value.source_basename); put_name(out, 96, &value.destination_basename); put_name(out, 160, &value.manifest_basename);
}

fn decode_expected(bytes: &[u8]) -> Result<AttemptExpected, AttemptError> {
    if get_u16(bytes, 6) != 0 || get_u32(bytes, 16) != 0 || bytes[26..32].iter().any(|b| *b != 0) || bytes[224..].iter().any(|b| *b != 0) { return Err(AttemptError::Reserved); }
    Ok(AttemptExpected { source_root_index: get_u16(bytes, 0), kind: decode_expected_kind(get_u16(bytes, 2))?, ordinal: get_u16(bytes, 4), maximum_bytes: get_u64(bytes, 8), source_basename: get_name(bytes, 20, 32)?, destination_basename: get_name(bytes, 22, 96)?, manifest_basename: get_name(bytes, 24, 160)? })
}

fn encode_recovery_entry(value: &RecoveryEntry, out: &mut [u8]) {
    put_u16(out, 0, value.root_kind as u16); put_u16(out, 2, value.root_index); put_u16(out, 4, 1);
    put_u16(out, 6, value.basename.len() as u16); put_u32(out, 8, value.mode); put_u32(out, 12, value.uid);
    put_u32(out, 16, value.gid); put_u64(out, 24, value.link_count); put_u64(out, 32, value.size);
    put_u64(out, 40, value.device); put_u64(out, 48, value.inode); put_digest(out, 56, value.sha256);
    put_name(out, 88, &value.basename); put_i64(out, 152, value.modified_seconds); put_i64(out, 160, value.modified_nanoseconds);
    put_i64(out, 168, value.changed_seconds); put_i64(out, 176, value.changed_nanoseconds);
}

fn decode_recovery_entry(bytes: &[u8]) -> Result<RecoveryEntry, AttemptError> {
    if get_u16(bytes, 4) != 1 || get_u32(bytes, 20) != 0 || bytes[184..].iter().any(|b| *b != 0) { return Err(AttemptError::Reserved); }
    Ok(RecoveryEntry { root_kind: decode_root_kind(get_u16(bytes, 0))?, root_index: get_u16(bytes, 2), basename: get_name(bytes, 6, 88)?, mode: get_u32(bytes, 8), uid: get_u32(bytes, 12), gid: get_u32(bytes, 16), link_count: get_u64(bytes, 24), size: get_u64(bytes, 32), device: get_u64(bytes, 40), inode: get_u64(bytes, 48), sha256: digest_at(bytes, 56), modified_seconds: get_i64(bytes, 152), modified_nanoseconds: get_i64(bytes, 160), changed_seconds: get_i64(bytes, 168), changed_nanoseconds: get_i64(bytes, 176) })
}

fn decode_role(value: u16) -> Result<Role, AttemptError> { match value { 1 => Ok(Role::Build), 2 => Ok(Role::Reference), 3 => Ok(Role::Launch), _ => Err(AttemptError::Field) } }
fn decode_root_kind(value: u16) -> Result<RootKind, AttemptError> { match value { 1 => Ok(RootKind::Source), 2 => Ok(RootKind::Destination), 3 => Ok(RootKind::Manifest), _ => Err(AttemptError::Field) } }
fn decode_expected_kind(value: u16) -> Result<ExpectedKind, AttemptError> { match value { 1 => Ok(ExpectedKind::Artifact), 2 => Ok(ExpectedKind::Transcript), 3 => Ok(ExpectedKind::ComparisonEvidence), 4 => Ok(ExpectedKind::LaunchEvidence), _ => Err(AttemptError::Field) } }
fn decode_state(value: u16) -> Result<AttemptState, AttemptError> { match value { 1 => Ok(AttemptState::Prepared), 2 => Ok(AttemptState::StartAuthorized), 3 => Ok(AttemptState::RunningObserved), 4 => Ok(AttemptState::ExitedSuccess), 5 => Ok(AttemptState::ExitedFailure), 6 => Ok(AttemptState::StopRequested), 7 => Ok(AttemptState::StoppedObserved), 8 => Ok(AttemptState::RecoveryRequired), 9 => Ok(AttemptState::RecoveryInventoryDurable), 10 => Ok(AttemptState::OutputsValidated), 11 => Ok(AttemptState::Committed), 12 => Ok(AttemptState::Discarded), 13 => Ok(AttemptState::Blocked), _ => Err(AttemptError::State) } }
fn decode_cause(value: u32) -> Result<AttemptCause, AttemptError> { match value { 0 => Ok(AttemptCause::None), 1 => Ok(AttemptCause::Spawn), 2 => Ok(AttemptCause::Timeout), 3 => Ok(AttemptCause::Cancelled), 4 => Ok(AttemptCause::ExitNonzero), 5 => Ok(AttemptCause::Journal), 6 => Ok(AttemptCause::Validation), 7 => Ok(AttemptCause::Recovery), 8 => Ok(AttemptCause::Identity), 9 => Ok(AttemptCause::Sync), 10 => Ok(AttemptCause::Policy), _ => Err(AttemptError::Field) } }

fn canonical_basename(value: &str) -> bool { let b = value.as_bytes(); (1..=64).contains(&b.len()) && value != "." && value != ".." && b.iter().all(|v| v.is_ascii_lowercase() || v.is_ascii_digit() || matches!(v, b'.' | b'-')) }
fn expected_key(value: &AttemptExpected) -> (u16, u16, u16, &str) { (value.source_root_index, value.kind as u16, value.ordinal, &value.source_basename) }
fn recovery_key(value: &RecoveryEntry) -> (u16, u16, &str) { (value.root_kind as u16, value.root_index, &value.basename) }
fn hash_with_zeroed(bytes: &[u8], offset: usize, width: usize) -> [u8; 32] { let mut copy = bytes.to_vec(); copy[offset..offset + width].fill(0); sha256(&copy) }
fn put_name(out: &mut [u8], offset: usize, value: &str) { out[offset..offset + value.len()].copy_from_slice(value.as_bytes()); }
fn get_name(input: &[u8], length_offset: usize, value_offset: usize) -> Result<String, AttemptError> { let len = get_u16(input, length_offset) as usize; if !(1..=64).contains(&len) || input[value_offset + len..value_offset + 64].iter().any(|b| *b != 0) { return Err(AttemptError::Canonical); } let value = core::str::from_utf8(&input[value_offset..value_offset + len]).map_err(|_| AttemptError::Canonical)?.to_owned(); if !canonical_basename(&value) { return Err(AttemptError::Canonical); } Ok(value) }
fn put_digest(out: &mut [u8], offset: usize, value: [u8; 32]) { out[offset..offset + 32].copy_from_slice(&value); }
fn digest_at(input: &[u8], offset: usize) -> [u8; 32] { input[offset..offset + 32].try_into().expect("bounded digest") }
fn put_u16(out: &mut [u8], offset: usize, value: u16) { out[offset..offset + 2].copy_from_slice(&value.to_le_bytes()); }
fn put_u32(out: &mut [u8], offset: usize, value: u32) { out[offset..offset + 4].copy_from_slice(&value.to_le_bytes()); }
fn put_i32(out: &mut [u8], offset: usize, value: i32) { out[offset..offset + 4].copy_from_slice(&value.to_le_bytes()); }
fn put_u64(out: &mut [u8], offset: usize, value: u64) { out[offset..offset + 8].copy_from_slice(&value.to_le_bytes()); }
fn put_i64(out: &mut [u8], offset: usize, value: i64) { out[offset..offset + 8].copy_from_slice(&value.to_le_bytes()); }
fn get_u16(input: &[u8], offset: usize) -> u16 { u16::from_le_bytes(input[offset..offset + 2].try_into().expect("bounded u16")) }
fn get_u32(input: &[u8], offset: usize) -> u32 { u32::from_le_bytes(input[offset..offset + 4].try_into().expect("bounded u32")) }
fn get_i32(input: &[u8], offset: usize) -> i32 { i32::from_le_bytes(input[offset..offset + 4].try_into().expect("bounded i32")) }
fn get_u64(input: &[u8], offset: usize) -> u64 { u64::from_le_bytes(input[offset..offset + 8].try_into().expect("bounded u64")) }
fn get_i64(input: &[u8], offset: usize) -> i64 { i64::from_le_bytes(input[offset..offset + 8].try_into().expect("bounded i64")) }

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_expected() -> AttemptExpected {
        AttemptExpected { source_root_index: 1, kind: ExpectedKind::Artifact, ordinal: 1, maximum_bytes: 4096,
            source_basename: "rar-os-alpha.img".into(), destination_basename: "rar-os-alpha.img".into(),
            manifest_basename: "handoff-p02-o001.v0".into() }
    }

    fn canonical_active() -> AttemptActive {
        let expected = canonical_expected();
        let mut expected_bytes = [0u8; EXPECTED_ENTRY_BYTES];
        encode_expected(&expected, &mut expected_bytes);
        AttemptActive { phase: 2, role: Role::Build, milestone: 1, attempt_ordinal: 1,
            watchdog_seconds: 60, termination_grace_seconds: 5, total_controller_seconds: 300,
            helper_unavailable: false, task_nonce: [1; 32], attempt_nonce: [2; 32], controller_session_nonce: [3; 32],
            task_binding_sha256: [4; 32], controller_source_sha256: [5; 32], helper_sha256: [6; 32],
            expected_table_sha256: sha256(&expected_bytes), journal_device: 10, journal_inode: 11,
            active_device: 10, active_inode: 12,
            roots: vec![
                AttemptRoot { kind: RootKind::Source, index: 1, device: 20, inode: 21, uid: 1000, gid: 1000, mode: 0o700, initial_entry_table_sha256: [7; 32] },
                AttemptRoot { kind: RootKind::Destination, index: 2, device: 20, inode: 22, uid: 1000, gid: 1000, mode: 0o700, initial_entry_table_sha256: [8; 32] },
                AttemptRoot { kind: RootKind::Manifest, index: 3, device: 20, inode: 23, uid: 1000, gid: 1000, mode: 0o700, initial_entry_table_sha256: [9; 32] },
            ], expected: vec![expected] }
    }

    #[test]
    fn active_transition_and_inventory_round_trip() {
        let active = canonical_active();
        let active_bytes = active.encode().unwrap();
        assert_eq!(AttemptActive::decode(&active_bytes).unwrap(), active);
        let active_digest = sha256(&active_bytes);
        let transition = AttemptTransition { sequence: 1, state: AttemptState::Prepared, phase: 2, role: Role::Build,
            recovery_ordinal: 0, exit_status: EXIT_UNOBSERVED, cause: AttemptCause::None, monotonic_nanoseconds: 1,
            active_sha256: active_digest, previous_sha256: active_digest, receipt_sha256: [0; 32], inventory_sha256: [0; 32],
            helper_sha256: [6; 32], active_device: 10, active_inode: 12, transition_device: 10, transition_inode: 13,
            controller_session_nonce: [3; 32] };
        let transition_bytes = transition.encode().unwrap();
        assert_eq!(AttemptTransition::decode(&transition_bytes).unwrap(), transition);
        let inventory = RecoveryInventory { origin_recovery_ordinal: 1, sequence: 8, active_sha256: active_digest,
            previous_transition_sha256: transition.digest().unwrap(), expected_table_sha256: active.expected_table_sha256,
            origin_controller_session_nonce: [3; 32], entries: Vec::new() };
        let inventory_bytes = inventory.encode().unwrap();
        assert_eq!(RecoveryInventory::decode(&inventory_bytes).unwrap(), inventory);
    }

    #[test]
    fn golden_prehash_headers_are_exact_and_reserved_zero() {
        for (text, bytes, magic) in [
            (include_str!("fixtures/active-header-prehash.v0.hex"), ACTIVE_HEADER_BYTES, ACTIVE_MAGIC),
            (include_str!("fixtures/transition-prehash.v0.hex"), TRANSITION_BYTES, TRANSITION_MAGIC),
        ] {
            let text = text.trim(); assert_eq!(text.len(), bytes * 2); assert_eq!(&text[..16], hex8(magic));
            assert!(text[16..].bytes().all(|byte| byte == b'0'));
        }
        let recovery = include_str!("fixtures/recovery-header-prehash.v0.hex").trim();
        assert_eq!(recovery.len(), RECOVERY_HEADER_BYTES * 2); assert_eq!(&recovery[..16], hex8(RECOVERY_MAGIC));
        assert!(recovery[16..].bytes().all(|byte| byte == b'0'));
    }

    fn hex8(bytes: [u8; 8]) -> &'static str {
        match bytes {
            ACTIVE_MAGIC => "5241524143545630",
            TRANSITION_MAGIC => "52415254524e5630",
            RECOVERY_MAGIC => "5241525243565630",
            _ => unreachable!(),
        }
    }

}
