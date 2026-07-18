#![deny(unsafe_code)]

#[path = "../../../tools/rar-lab/preauth/src/lib.rs"]
mod preauth;

#[path = "../../../tools/rar-lab/safety/src/lib.rs"]
mod safety;

use safety::{CertificationRecord, CommandPlan, VmProfile, sha256_hex};

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

#[test]
fn immutable_candidate_records_match_the_verified_inputs() {
    let profile_text = include_str!(
        "../../../spec/lab/vm-profile/examples/x86_64-preauth.profile"
    );
    let command_text = include_str!(
        "../../../spec/lab/vm-profile/examples/x86_64-preauth.command"
    );
    let closure_text = include_str!(
        "../../../spec/lab/preauth/locks/r0-x86_64-preauth-v2.lock"
    );
    let packages_text = include_str!(
        "../../../spec/lab/preauth/locks/r0-x86_64-preauth-packages.v2"
    );
    let disk_text = include_str!(
        "../../../spec/lab/preauth/locks/r0-x86_64-preauth-disk.v1"
    );
    let certification_text = include_str!(
        "../../../spec/lab/vm-profile/prepared/r0-x86_64-preauth-v1.cert"
    );

    let profile = VmProfile::parse(profile_text).expect("canonical preauth profile");
    assert_eq!(profile.sha256(), "6844406817b6d43643e4ff60737fbc08a84846e29f78b2914cadd5de2ec6ab9a");
    let command = CommandPlan::from_profile(&profile);
    assert_eq!(command.canonical(), command_text);
    assert_eq!(command.sha256(), "7d8e5f500c35b5da4de0d3f2a6d9b667563bb6e3ff7ed6503192ee8e69e0550d");
    assert_eq!(sha256_hex(closure_text.as_bytes()), "b4ad190e4254fdc2a8e60e77b49781ad4ed94a5746c1a58ae342629631215a21");
    assert!(closure_text.contains("derived_oci_index_sha256=4d59826fb248130555b99aa6bc034f17db7df4a6acbbe1ebc0a8175492476531\n"));
    assert!(closure_text.contains("certifiable=true\n"));
    assert_eq!(sha256_hex(packages_text.as_bytes()), "5f73693e6202969af6ca958e28bb4ce189741f4130f40e2eb5bb593e2e07519a");
    assert_eq!(packages_text.lines().filter(|line| line.starts_with("package|")).count(), 36);
    assert!(disk_text.contains("seed_sha256=141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766\n"));
    assert!(disk_text.contains("child_sha256=141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766\n"));
    assert!(disk_text.ends_with("record_sha256=89e160c117154dded20d7daeaf75576dd082a9d83d5ade47f9254a6e35371826\n"));

    let certification = CertificationRecord::parse(certification_text)
        .expect("integrity-bound prepared certification");
    assert_eq!(certification.profile_sha256, profile.sha256());
    assert_eq!(certification.command_sha256, command.sha256());
    assert_eq!(certification.tool_lock_sha256, sha256_hex(closure_text.as_bytes()));
    assert_eq!(certification.reviewer, "pending-independent-review");
}
