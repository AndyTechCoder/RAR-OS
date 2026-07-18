#![deny(unsafe_code)]

#[path = "../../../tools/rar-lab/preauth/src/lib.rs"]
mod preauth;


use preauth::{
    AuthorityRecord, AuthorityState, DiskBinding, LaunchBindings, LifecycleBackend,
    LifecycleEvent, PreauthError, SyntheticLedger, consume_once, explicit_non_execution_evidence,
    synthetic_timeout_cleanup,
};

fn hash(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn bindings() -> LaunchBindings {
    LaunchBindings {
        certification_sha256: hash('1'),
        profile_sha256: hash('2'),
        command_sha256: hash('3'),
        artifact_sha256: hash('4'),
        disk_sha256: hash('5'),
        firmware_sha256: hash('6'),
        closure_sha256: hash('7'),
    }
}

fn record() -> AuthorityRecord {
    AuthorityRecord {
        authorization_id: "r0-first-boot-0001".into(),
        state: AuthorityState::Issued,
        bindings: bindings(),
        nonce: "nonce-0001".into(),
        expires_at: 2_000,
        repository: "AndyTechCoder/RAR-OS".into(),
        workflow: "first-boot".into(),
        git_ref: "refs/heads/codex/r0-prompt7a-preauth".into(),
        environment: "rar-r0-first-boot".into(),
        oidc_subject: "repo:AndyTechCoder/RAR-OS:environment:rar-r0-first-boot".into(),
        kms_key_arn: "arn:aws:kms:eu-central-1:000000000000:key/synthetic".into(),
        kms_signature_sha256: hash('8'),
        cloudtrail_evidence_sha256: hash('9'),
        transition_version: 1,
    }
}

#[test]
fn exact_binding_consumes_once() {
    let candidate = record();
    let mut ledger = SyntheticLedger::default();
    ledger.issue(candidate.clone()).unwrap();
    let consumed = consume_once(&mut ledger, &candidate, &bindings(), 1_000).unwrap();
    assert_eq!(consumed.state, AuthorityState::Consumed);
    assert_eq!(
        consume_once(&mut ledger, &candidate, &bindings(), 1_000)
            .unwrap_err()
            .code,
        "authority-replay-or-revoked"
    );
}

#[test]
fn stale_substituted_revoked_and_uncertain_fail_closed() {
    let candidate = record();
    let mut changed = bindings();
    changed.disk_sha256 = hash('a');
    let mut ledger = SyntheticLedger::default();
    ledger.issue(candidate.clone()).unwrap();
    assert_eq!(
        consume_once(&mut ledger, &candidate, &changed, 1_000)
            .unwrap_err()
            .code,
        "authority-binding-mismatch"
    );
    assert_eq!(
        consume_once(&mut ledger, &candidate, &bindings(), 2_000)
            .unwrap_err()
            .code,
        "authority-expired"
    );
    ledger.revoke(&candidate.authorization_id).unwrap();
    assert_eq!(
        consume_once(&mut ledger, &candidate, &bindings(), 1_000)
            .unwrap_err()
            .code,
        "authority-replay-or-revoked"
    );

    let mut uncertain = SyntheticLedger::default();
    uncertain.issue(candidate.clone()).unwrap();
    uncertain.make_uncertain();
    assert_eq!(
        consume_once(&mut uncertain, &candidate, &bindings(), 1_000)
            .unwrap_err()
            .code,
        "authority-commit-uncertain"
    );
}

#[test]
fn disk_binding_is_content_and_path_bound() {
    let disk = DiskBinding {
        seed_path: "out/r0/vm/x86_64/seed.qcow2".into(),
        seed_sha256: hash('a'),
        child_path: "out/r0/vm/x86_64/launch-0001.qcow2".into(),
        child_sha256: hash('b'),
        virtual_bytes: 64 * 1024 * 1024,
    };
    disk.validate().unwrap();
    let mut changed = disk.clone();
    changed.child_path = "../../dev/disk0".into();
    assert_eq!(changed.validate().unwrap_err().code, "invalid-disk-binding");
}

#[derive(Default)]
struct FakeLifecycle {
    events: Vec<LifecycleEvent>,
    fail_terminate: bool,
    fail_cleanup: bool,
}

impl LifecycleBackend for FakeLifecycle {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), PreauthError> {
        self.events.push(event);
        if (event == LifecycleEvent::Terminate && self.fail_terminate)
            || (event == LifecycleEvent::Cleanup && self.fail_cleanup)
        {
            return Err(PreauthError { code: "synthetic-failure" });
        }
        Ok(())
    }
}

#[test]
fn timeout_escalates_and_cleanup_failure_quarantines() {
    let mut backend = FakeLifecycle {
        fail_terminate: true,
        fail_cleanup: true,
        ..FakeLifecycle::default()
    };
    assert_eq!(
        synthetic_timeout_cleanup(&mut backend).unwrap_err().code,
        "cleanup-failed-quarantined"
    );
    assert_eq!(
        backend.events,
        vec![
            LifecycleEvent::Timeout,
            LifecycleEvent::Terminate,
            LifecycleEvent::Kill,
            LifecycleEvent::Cleanup,
            LifecycleEvent::Quarantine,
        ]
    );
    assert!(!backend.events.contains(&LifecycleEvent::Spawn));
    assert!(!backend.events.contains(&LifecycleEvent::Resolve));
}

#[test]
fn evidence_states_every_non_execution_class() {
    let evidence = explicit_non_execution_evidence();
    for line in [
        "target_execution=not-attempted",
        "qemu_execution=not-attempted",
        "emulator_execution=not-attempted",
        "vm_execution=not-attempted",
        "aws_calls=not-attempted",
    ] {
        assert!(evidence.lines().any(|candidate| candidate == line));
    }
}
