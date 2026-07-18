#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Component, Path};

mod contracts;
mod json;
pub use contracts::{
    AttestationRecord, ClosureLock, ExecutionHostRecord, IdentityGraph, PackageManifest,
    PreparedCertification, StrictAuthorityRecord, sha256_hex, sha256_reader,
};
pub use json::Json;

pub const AUTHORITY_SCHEMA: &str = "rar-external-authorization-v1";
pub const CLOSURE_SCHEMA: &str = "rar-preauth-closure-v3";
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityState {
    Issued,
    Consumed,
    Revoked,
    Uncertain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionReceipt { pub state: AuthorityState, pub version: u64, pub record_sha256: String }

#[derive(Clone)]
struct LedgerItem { record: StrictAuthorityRecord, state: AuthorityState, version: u64 }

#[derive(Default)]
pub struct SyntheticLedger { records: BTreeMap<String, LedgerItem>, uncertain: bool }

impl SyntheticLedger {
    /// Validates the complete canonical, signature-bound record and identity graph before insert.
    pub fn issue(&mut self, record: StrictAuthorityRecord, graph: &IdentityGraph) -> Result<()> {
        if graph.phase != "attested" { return Err(PreauthError::new("authority-requires-attested-identity")); }
        record.validate_graph(graph)?;
        if record.state != "issued" || record.transition_version != 1 {
            return Err(PreauthError::new("invalid-authority-state"));
        }
        if self.records.contains_key(&record.authorization_id) {
            return Err(PreauthError::new("duplicate-authorization"));
        }
        self.records.insert(record.authorization_id.clone(), LedgerItem {
            version: record.transition_version, record, state: AuthorityState::Issued,
        });
        Ok(())
    }

    pub fn revoke(&mut self, authorization_id: &str) -> Result<()> {
        let item = self.records.get_mut(authorization_id).ok_or_else(|| PreauthError::new("authorization-absent"))?;
        if item.state != AuthorityState::Issued { return Err(PreauthError::new("authority-replay-or-revoked")); }
        item.state = AuthorityState::Revoked; item.version += 1; Ok(())
    }
    pub fn make_uncertain(&mut self) { self.uncertain = true; }
}

pub fn consume_once(
    ledger: &mut SyntheticLedger, record: &StrictAuthorityRecord, graph: &IdentityGraph, now: u64,
) -> Result<TransitionReceipt> {
    record.validate_graph(graph)?;
    if record.state != "issued" || now >= record.expires_at { return Err(PreauthError::new("authority-expired-or-state")); }
    if ledger.uncertain { return Err(PreauthError::new("authority-commit-uncertain")); }
    let item = ledger.records.get_mut(&record.authorization_id)
        .ok_or_else(|| PreauthError::new("authority-absent"))?;
    if item.record != *record || item.state != AuthorityState::Issued || item.version != record.transition_version {
        return Err(PreauthError::new("authority-replay-or-revoked"));
    }
    item.state = AuthorityState::Consumed; item.version += 1;
    Ok(TransitionReceipt { state: item.state, version: item.version, record_sha256: item.record.record_sha256.clone() })
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
    Authorize,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Prepared, Authorized, Resolving, Running, Terminating, Exited, Cleaning,
    Consumed, Refused, Quarantined,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleMachine { state: LifecycleState, cleanup_required: bool }

impl LifecycleMachine {
    pub fn prepared() -> Self { Self { state: LifecycleState::Prepared, cleanup_required: false } }
    pub fn state(&self) -> LifecycleState { self.state }
    pub fn apply(&mut self, event: LifecycleEvent) -> Result<LifecycleState> {
        use LifecycleEvent::*; use LifecycleState::*;
        let next = match (self.state, event) {
            (Prepared, Authorize) => Authorized,
            (Authorized, Resolve) => Resolving,
            (Resolving, Spawn) => { self.cleanup_required = true; Running }
            (Running, ExitObserved) => Exited,
            (Running, Timeout | OutputLimit | Terminate) => Terminating,
            (Terminating, Kill | ExitObserved) => Exited,
            (Running | Resolving | Terminating, Crash) => Quarantined,
            (Exited | Quarantined, Cleanup) => { self.cleanup_required = false; Cleaning }
            (Cleaning, Reconcile) if !self.cleanup_required => Consumed,
            (Prepared | Authorized | Resolving, RefuseRetry) => Refused,
            (_, Quarantine) => Quarantined,
            (Consumed | Refused | Quarantined, _) => return Err(PreauthError::new("lifecycle-terminal")),
            _ => return Err(PreauthError::new("lifecycle-order")),
        };
        self.state = next; Ok(next)
    }

    pub fn finish(&self) -> Result<()> {
        if matches!(self.state, LifecycleState::Consumed | LifecycleState::Refused)
            && !self.cleanup_required { Ok(()) }
        else { Err(PreauthError::new("lifecycle-not-terminal-clean")) }
    }
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
    pub identity_graph: IdentityGraph,
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
        if self.identity_graph.phase != "attested" { return Err(PreauthError::new("descriptor-requires-attested-identity")); }
        if [
            &self.execution_host_sha256,
            &self.resolver_sha256, &self.spawner_sha256, &self.executable_sha256,
            &self.artifact_sha256, &self.disk_sha256, &self.firmware_code_sha256,
            &self.firmware_vars_sha256,
        ].into_iter().all(|value| digest(value)) {
            for (kind, actual) in [
                ("execution_host_sha256", &self.execution_host_sha256), ("resolver_sha256", &self.resolver_sha256),
                ("spawner_sha256", &self.spawner_sha256), ("qemu_sha256", &self.executable_sha256),
                ("artifact_sha256", &self.artifact_sha256), ("disk_child_sha256", &self.disk_sha256),
                ("ovmf_code_sha256", &self.firmware_code_sha256), ("ovmf_vars_sha256", &self.firmware_vars_sha256),
            ] { self.identity_graph.require(kind, actual)?; }
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
