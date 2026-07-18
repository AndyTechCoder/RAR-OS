#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Component, Path};

mod contracts;
pub use contracts::{
    AttestationRecord, ClosureLock, ExecutionHostRecord, IdentityGraph, PackageManifest,
    PreparedCertification, StrictAuthorityRecord, sha256_hex, sha256_reader,
};

pub const AUTHORITY_SCHEMA: &str = "rar-external-authorization-v1";
pub const CLOSURE_SCHEMA: &str = "rar-preauth-closure-v2";
pub const DISK_SCHEMA: &str = "rar-disposable-disk-v1";
pub const EXECUTION_HOST_SCHEMA: &str = "rar-execution-host-v1";
pub const BASE_OCI_INDEX_SHA256: &str =
    "f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3";
pub const AUTHORITY_REPOSITORY: &str = "AndyTechCoder/RAR-OS";
pub const AUTHORITY_REF: &str = "refs/heads/codex/r0-prompt7a-preauth";
pub const AUTHORITY_ENVIRONMENT: &str = "rar-r0-first-boot";
pub const AUTHORITY_WORKFLOW: &str =
    "AndyTechCoder/RAR-OS/.github/workflows/first-boot.yml@refs/heads/main";
pub const OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";
pub const OIDC_AUDIENCE: &str = "sts.amazonaws.com";
pub const OIDC_SUBJECT: &str = "repo:AndyTechCoder/RAR-OS:environment:rar-r0-first-boot";
pub const KMS_KEY_ARN: &str =
    "arn:aws:kms:eu-central-1:000000000000:key/rar-r0-first-boot-synthetic";
pub const KMS_ALGORITHM: &str = "RSASSA_PSS_SHA_256";

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
    pub package_manifest_sha256: String,
    pub source_manifest_sha256: String,
    pub signature_manifest_sha256: String,
    pub license_manifest_sha256: String,
    pub disk_record_sha256: String,
    pub firmware_vars_sha256: String,
    pub execution_host_sha256: String,
    pub resolver_sha256: String,
    pub spawner_sha256: String,
    pub identity_graph_sha256: String,
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
            &self.package_manifest_sha256,
            &self.source_manifest_sha256,
            &self.signature_manifest_sha256,
            &self.license_manifest_sha256,
            &self.disk_record_sha256,
            &self.firmware_vars_sha256,
            &self.execution_host_sha256,
            &self.resolver_sha256,
            &self.spawner_sha256,
            &self.identity_graph_sha256,
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
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub kms_key_arn: String,
    pub kms_algorithm: String,
    pub kms_context_sha256: String,
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
            || self.workflow != AUTHORITY_WORKFLOW
            || self.git_ref != AUTHORITY_REF
            || self.environment != AUTHORITY_ENVIRONMENT
            || self.oidc_issuer != OIDC_ISSUER
            || self.oidc_audience != OIDC_AUDIENCE
            || self.oidc_subject != OIDC_SUBJECT
            || self.kms_key_arn != KMS_KEY_ARN
            || self.kms_algorithm != KMS_ALGORITHM
            || !digest(&self.kms_context_sha256)
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
        if self.records.contains_key(&record.authorization_id) {
            return Err(PreauthError::new("duplicate-authorization"));
        }
        self.records.insert(record.authorization_id.clone(), record);
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
        if !valid_repository_relative(&self.seed_path, "out/r0/vm/x86_64", ".qcow2")
            || !valid_repository_relative(&self.child_path, "out/r0/vm/x86_64", ".qcow2")
            || self.seed_path == self.child_path
            || !digest(&self.seed_sha256)
            || !digest(&self.child_sha256)
            || self.virtual_bytes == 0
        {
            return Err(PreauthError::new("invalid-disk-binding"));
        }
        Ok(())
    }
}

pub fn valid_repository_relative(value: &str, prefix: &str, suffix: &str) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.contains("//")
        || value.contains('\\')
        || value.bytes().any(|byte| byte == 0 || byte == b':')
        || !value.ends_with(suffix)
    {
        return false;
    }
    let path = Path::new(value);
    if !path.is_relative()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return false;
    }
    path.starts_with(prefix) && path.components().count() > Path::new(prefix).components().count()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleEvent {
    Resolve,
    Spawn,
    OutputLimit,
    Timeout,
    Terminate,
    ExitObserved,
    Kill,
    Crash,
    Reconcile,
    RefuseRetry,
    Cleanup,
    Quarantine,
}

pub trait LifecycleBackend {
    fn event(&mut self, event: LifecycleEvent) -> Result<()>;
}

pub fn synthetic_timeout_cleanup<B: LifecycleBackend>(backend: &mut B) -> Result<()> {
    backend.event(LifecycleEvent::Timeout)?;
    synthetic_forced_cleanup(backend)
}

pub fn synthetic_output_limit_cleanup<B: LifecycleBackend>(backend: &mut B) -> Result<()> {
    backend.event(LifecycleEvent::OutputLimit)?;
    synthetic_forced_cleanup(backend)
}

fn synthetic_forced_cleanup<B: LifecycleBackend>(backend: &mut B) -> Result<()> {
    let terminate_failed = backend.event(LifecycleEvent::Terminate).is_err();
    if terminate_failed || backend.event(LifecycleEvent::ExitObserved).is_err() {
        if backend.event(LifecycleEvent::Kill).is_err() {
            let _ = backend.event(LifecycleEvent::Quarantine);
            let _ = backend.event(LifecycleEvent::Cleanup);
            return Err(PreauthError::new("kill-failed-quarantined"));
        }
        backend.event(LifecycleEvent::ExitObserved).map_err(|_| {
            let _ = backend.event(LifecycleEvent::Quarantine);
            let _ = backend.event(LifecycleEvent::Cleanup);
            PreauthError::new("exit-unobserved-quarantined")
        })?;
    }
    if backend.event(LifecycleEvent::Cleanup).is_err() {
        backend.event(LifecycleEvent::Quarantine)?;
        return Err(PreauthError::new("cleanup-failed-quarantined"));
    }
    Ok(())
}

pub fn synthetic_crash_recovery<B: LifecycleBackend>(backend: &mut B) -> Result<()> {
    backend.event(LifecycleEvent::Crash)?;
    backend.event(LifecycleEvent::Quarantine)?;
    backend.event(LifecycleEvent::Reconcile)?;
    backend.event(LifecycleEvent::RefuseRetry)?;
    if backend.event(LifecycleEvent::Cleanup).is_err() {
        return Err(PreauthError::new("crash-cleanup-failed-quarantined"));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DescriptorBinding {
    pub identity_graph_sha256: String,
    pub execution_host_sha256: String,
    pub resolver_sha256: String,
    pub spawner_sha256: String,
    pub executable_sha256: String,
    pub artifact_sha256: String,
    pub disk_sha256: String,
    pub firmware_code_sha256: String,
    pub firmware_vars_sha256: String,
}

impl DescriptorBinding {
    pub fn validate(&self) -> Result<()> {
        if [
            &self.identity_graph_sha256, &self.execution_host_sha256,
            &self.resolver_sha256, &self.spawner_sha256, &self.executable_sha256,
            &self.artifact_sha256, &self.disk_sha256, &self.firmware_code_sha256,
            &self.firmware_vars_sha256,
        ].into_iter().all(|value| digest(value)) {
            Ok(())
        } else {
            Err(PreauthError::new("invalid-descriptor-binding"))
        }
    }
}

pub fn bind_resolver_to_spawner(
    resolved: &DescriptorBinding,
    presented: &DescriptorBinding,
) -> Result<()> {
    resolved.validate()?;
    presented.validate()?;
    if resolved != presented {
        return Err(PreauthError::new("resolver-spawner-binding-mismatch"));
    }
    Ok(())
}

pub fn explicit_non_execution_evidence() -> &'static str {
    "target_execution=not-attempted\nqemu_execution=not-attempted\nemulator_execution=not-attempted\nvm_execution=not-attempted\naws_calls=not-attempted\n"
}
