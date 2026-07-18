#![deny(unsafe_code)]

#[path = "../../../tools/rar-lab/preauth/src/lib.rs"]
mod preauth;


use preauth::{
    AuthorityRecord, AuthorityState, DescriptorBinding, DiskBinding, LaunchBindings,
    LifecycleBackend, LifecycleEvent, PreauthError, SyntheticLedger, bind_resolver_to_spawner,
    consume_once, explicit_non_execution_evidence, synthetic_crash_recovery,
    synthetic_output_limit_cleanup, synthetic_timeout_cleanup, valid_repository_relative,
    StrictAuthorityRecord, sha256_hex,
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
        package_manifest_sha256: hash('a'),
        source_manifest_sha256: hash('b'),
        signature_manifest_sha256: hash('c'),
        license_manifest_sha256: hash('d'),
        disk_record_sha256: hash('e'),
        firmware_vars_sha256: hash('f'),
        execution_host_sha256: hash('0'),
        resolver_sha256: hash('a'),
        spawner_sha256: hash('b'),
        identity_graph_sha256: hash('c'),
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
        workflow: preauth::AUTHORITY_WORKFLOW.into(),
        git_ref: preauth::AUTHORITY_REF.into(),
        environment: "rar-r0-first-boot".into(),
        oidc_subject: "repo:AndyTechCoder/RAR-OS:environment:rar-r0-first-boot".into(),
        oidc_issuer: preauth::OIDC_ISSUER.into(),
        oidc_audience: preauth::OIDC_AUDIENCE.into(),
        kms_key_arn: preauth::KMS_KEY_ARN.into(),
        kms_algorithm: preauth::KMS_ALGORITHM.into(),
        kms_context_sha256: hash('7'),
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
fn rejected_issue_has_no_ledger_side_effect() {
    let mut ledger = SyntheticLedger::default();
    let mut invalid = record();
    invalid.bindings.artifact_sha256 = "invalid".into();
    assert_eq!(ledger.issue(invalid).unwrap_err().code, "invalid-binding");
    ledger.issue(record()).unwrap();
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
            LifecycleEvent::ExitObserved,
            LifecycleEvent::Cleanup,
            LifecycleEvent::Quarantine,
        ]
    );
    assert!(!backend.events.contains(&LifecycleEvent::Spawn));
    assert!(!backend.events.contains(&LifecycleEvent::Resolve));
}


#[test]
fn paths_and_descriptor_binding_fail_closed() {
    for rejected in ["", "/tmp/x", "../x", "out/./x", "out//x", "out\\x", "C:x"] {
        assert!(!valid_repository_relative(rejected, "out/r0", ".qcow2"));
    }
    let descriptor = DescriptorBinding {
        identity_graph_sha256: hash('1'), execution_host_sha256: hash('2'),
        resolver_sha256: hash('3'), spawner_sha256: hash('4'), executable_sha256: hash('5'),
        artifact_sha256: hash('6'), disk_sha256: hash('7'), firmware_code_sha256: hash('8'),
        firmware_vars_sha256: hash('9'),
    };
    bind_resolver_to_spawner(&descriptor, &descriptor).unwrap();
    let mut substituted = descriptor.clone();
    substituted.disk_sha256 = hash('b');
    assert_eq!(bind_resolver_to_spawner(&descriptor, &substituted).unwrap_err().code,
        "resolver-spawner-binding-mismatch");
}

#[test]
fn crash_recovery_quarantines_and_refuses_retry_without_spawn() {
    let mut backend = FakeLifecycle::default();
    synthetic_crash_recovery(&mut backend).unwrap();
    assert_eq!(backend.events, vec![LifecycleEvent::Crash, LifecycleEvent::Quarantine,
        LifecycleEvent::Reconcile, LifecycleEvent::RefuseRetry, LifecycleEvent::Cleanup]);
    assert!(!backend.events.contains(&LifecycleEvent::Spawn));
}

#[test]
fn output_bound_terminates_and_cleans_without_spawn() {
    let mut backend = FakeLifecycle::default();
    synthetic_output_limit_cleanup(&mut backend).unwrap();
    assert_eq!(backend.events, vec![LifecycleEvent::OutputLimit, LifecycleEvent::Terminate,
        LifecycleEvent::ExitObserved, LifecycleEvent::Cleanup]);
    assert!(!backend.events.contains(&LifecycleEvent::Spawn));
}

#[test]
fn canonical_external_authority_claims_and_signature_input_are_exact() {
    let payload = format!(concat!(
        "schema=rar-external-authorization-v1\nauthorization_id=r0-first-boot-0001\nstate=issued\n",
        "certification_sha256={0}\nidentity_graph_sha256={0}\nprofile_sha256={0}\ncommand_sha256={0}\n",
        "artifact_sha256={0}\ndisk_sha256={0}\nfirmware_code_sha256={0}\nfirmware_vars_sha256={0}\n",
        "closure_sha256={0}\nexecution_host_sha256={0}\nresolver_sha256={0}\nspawner_sha256={0}\n",
        "consumption_key_sha256={0}\nnonce=nonce-0001\nissued_at=2026-07-18T12:00:00Z\nexpires_at=2000\n",
        "repository={1}\nworkflow={2}\nref={3}\nenvironment={4}\noidc_issuer={5}\noidc_audience={6}\n",
        "oidc_subject={7}\nkms_key_arn={8}\nkms_algorithm={9}\nkms_context_sha256={0}\n",
        "cloudtrail_previous_sha256={0}\ncloudtrail_evidence_sha256={0}\ntransition_version=1\n"),
        hash('a'), preauth::AUTHORITY_REPOSITORY, preauth::AUTHORITY_WORKFLOW,
        preauth::AUTHORITY_REF, preauth::AUTHORITY_ENVIRONMENT, preauth::OIDC_ISSUER,
        preauth::OIDC_AUDIENCE, preauth::OIDC_SUBJECT, preauth::KMS_KEY_ARN,
        preauth::KMS_ALGORITHM);
    let record_hash = sha256_hex(payload.as_bytes());
    let input = format!("{payload}record_sha256={record_hash}\nkms_signature_sha256={}\n", hash('b'));
    let parsed = StrictAuthorityRecord::parse(&input).unwrap();
    assert!(parsed.signature_input().ends_with(&format!("record_sha256={record_hash}\n")));
    let deputy = input.replacen(preauth::OIDC_AUDIENCE, "wrong-audience", 1);
    assert_eq!(StrictAuthorityRecord::parse(&deputy).unwrap_err().code, "invalid-authority-record");
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
