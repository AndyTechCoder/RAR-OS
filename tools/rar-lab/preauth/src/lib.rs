#![deny(unsafe_code)]

use std::collections::BTreeMap;

pub const AUTHORITY_SCHEMA: &str = "rar-external-authorization-v1";
pub const CLOSURE_SCHEMA: &str = "rar-preauth-closure-v2";
pub const DISK_SCHEMA: &str = "rar-disposable-disk-v1";
pub const EXECUTION_HOST_SCHEMA: &str = "rar-execution-host-v1";
pub const BASE_OCI_INDEX_SHA256: &str =
    "f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreauthError {
    pub code: &'static str,
}

impl PreauthError {
    fn new(code: &'static str) -> Self {
        Self { code }
    }
}

pub type Result<T> = std::result::Result<T, PreauthError>;

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/' | b'@')
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchBindings {
    pub certification_sha256: String,
    pub profile_sha256: String,
    pub command_sha256: String,
    pub artifact_sha256: String,
    pub disk_sha256: String,
    pub firmware_sha256: String,
    pub closure_sha256: String,
}

impl LaunchBindings {
    pub fn validate(&self) -> Result<()> {
        if [
            &self.certification_sha256,
            &self.profile_sha256,
            &self.command_sha256,
            &self.artifact_sha256,
            &self.disk_sha256,
            &self.firmware_sha256,
            &self.closure_sha256,
        ]
        .into_iter()
        .all(|value| digest(value))
        {
            Ok(())
        } else {
            Err(PreauthError::new("invalid-binding"))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityState {
    Issued,
    Consumed,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityRecord {
    pub authorization_id: String,
    pub state: AuthorityState,
    pub bindings: LaunchBindings,
    pub nonce: String,
    pub expires_at: u64,
    pub repository: String,
    pub workflow: String,
    pub git_ref: String,
    pub environment: String,
    pub oidc_subject: String,
    pub kms_key_arn: String,
    pub kms_signature_sha256: String,
    pub cloudtrail_evidence_sha256: String,
    pub transition_version: u64,
}

impl AuthorityRecord {
    pub fn validate(&self) -> Result<()> {
        self.bindings.validate()?;
        if self.state != AuthorityState::Issued
            || self.transition_version != 1
            || !token(&self.authorization_id)
            || !token(&self.nonce)
            || self.expires_at == 0
            || self.repository != "AndyTechCoder/RAR-OS"
            || !self.git_ref.starts_with("refs/heads/codex/")
            || self.environment != "rar-r0-first-boot"
            || !self.oidc_subject.starts_with("repo:AndyTechCoder/RAR-OS:environment:")
            || !self.kms_key_arn.starts_with("arn:aws:kms:")
            || !digest(&self.kms_signature_sha256)
            || !digest(&self.cloudtrail_evidence_sha256)
        {
            return Err(PreauthError::new("invalid-authority-record"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConditionalResult {
    Committed(AuthorityRecord),
    ConditionFailed,
    Uncertain,
}

pub trait ConditionalLedger {
    fn transition(
        &mut self,
        authorization_id: &str,
        expected_state: AuthorityState,
        expected_version: u64,
        new_state: AuthorityState,
    ) -> ConditionalResult;
}

pub fn consume_once<L: ConditionalLedger>(
    ledger: &mut L,
    record: &AuthorityRecord,
    presented: &LaunchBindings,
    now: u64,
) -> Result<AuthorityRecord> {
    record.validate()?;
    presented.validate()?;
    if record.bindings != *presented {
        return Err(PreauthError::new("authority-binding-mismatch"));
    }
    if now >= record.expires_at {
        return Err(PreauthError::new("authority-expired"));
    }
    match ledger.transition(
        &record.authorization_id,
        AuthorityState::Issued,
        record.transition_version,
        AuthorityState::Consumed,
    ) {
        ConditionalResult::Committed(committed)
            if committed.state == AuthorityState::Consumed
                && committed.transition_version == record.transition_version + 1
                && committed.bindings == record.bindings => Ok(committed),
        ConditionalResult::Committed(_) => Err(PreauthError::new("authority-commit-mismatch")),
        ConditionalResult::ConditionFailed => Err(PreauthError::new("authority-replay-or-revoked")),
        ConditionalResult::Uncertain => Err(PreauthError::new("authority-commit-uncertain")),
    }
}

#[derive(Default)]
pub struct SyntheticLedger {
    records: BTreeMap<String, AuthorityRecord>,
    uncertain: bool,
}

impl SyntheticLedger {
    pub fn issue(&mut self, record: AuthorityRecord) -> Result<()> {
        record.validate()?;
        if self.records.insert(record.authorization_id.clone(), record).is_some() {
            return Err(PreauthError::new("duplicate-authorization"));
        }
        Ok(())
    }

    pub fn revoke(&mut self, authorization_id: &str) -> Result<()> {
        let record = self
            .records
            .get_mut(authorization_id)
            .ok_or_else(|| PreauthError::new("authorization-absent"))?;
        if record.state != AuthorityState::Issued {
            return Err(PreauthError::new("authority-replay-or-revoked"));
        }
        record.state = AuthorityState::Revoked;
        record.transition_version += 1;
        Ok(())
    }

    pub fn make_uncertain(&mut self) {
        self.uncertain = true;
    }
}

impl ConditionalLedger for SyntheticLedger {
    fn transition(
        &mut self,
        authorization_id: &str,
        expected_state: AuthorityState,
        expected_version: u64,
        new_state: AuthorityState,
    ) -> ConditionalResult {
        if self.uncertain {
            return ConditionalResult::Uncertain;
        }
        let Some(record) = self.records.get_mut(authorization_id) else {
            return ConditionalResult::ConditionFailed;
        };
        if record.state != expected_state || record.transition_version != expected_version {
            return ConditionalResult::ConditionFailed;
        }
        record.state = new_state;
        record.transition_version += 1;
        ConditionalResult::Committed(record.clone())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiskBinding {
    pub seed_path: String,
    pub seed_sha256: String,
    pub child_path: String,
    pub child_sha256: String,
    pub virtual_bytes: u64,
}

impl DiskBinding {
    pub fn validate(&self) -> Result<()> {
        if !self.seed_path.starts_with("out/r0/vm/x86_64/")
            || !self.child_path.starts_with("out/r0/vm/x86_64/")
            || self.seed_path == self.child_path
            || !self.seed_path.ends_with(".qcow2")
            || !self.child_path.ends_with(".qcow2")
            || !digest(&self.seed_sha256)
            || !digest(&self.child_sha256)
            || self.virtual_bytes == 0
        {
            return Err(PreauthError::new("invalid-disk-binding"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleEvent {
    Resolve,
    Spawn,
    Timeout,
    Terminate,
    Kill,
    Cleanup,
    Quarantine,
}

pub trait LifecycleBackend {
    fn event(&mut self, event: LifecycleEvent) -> Result<()>;
}

pub fn synthetic_timeout_cleanup<B: LifecycleBackend>(backend: &mut B) -> Result<()> {
    backend.event(LifecycleEvent::Timeout)?;
    if backend.event(LifecycleEvent::Terminate).is_err() {
        backend.event(LifecycleEvent::Kill)?;
    }
    if backend.event(LifecycleEvent::Cleanup).is_err() {
        backend.event(LifecycleEvent::Quarantine)?;
        return Err(PreauthError::new("cleanup-failed-quarantined"));
    }
    Ok(())
}

pub fn explicit_non_execution_evidence() -> &'static str {
    "target_execution=not-attempted\nqemu_execution=not-attempted\nemulator_execution=not-attempted\nvm_execution=not-attempted\naws_calls=not-attempted\n"
}
