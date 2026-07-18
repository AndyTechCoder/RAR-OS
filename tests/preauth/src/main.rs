#![deny(unsafe_code)]

#[path = "../../../tools/rar-lab/preauth/src/lib.rs"]
mod preauth;


use preauth::{
    AuthorityState, DescriptorBinding, DiskBinding, IdentityGraph,
    Json, LifecycleBackend, LifecycleEvent, LifecycleMachine, LifecycleState, PreauthError, SyntheticLedger, bind_resolver_to_spawner,
    consume_once, explicit_non_execution_evidence, synthetic_crash_recovery,
    synthetic_output_limit_cleanup, synthetic_timeout_cleanup, valid_repository_relative,
    StrictAuthorityRecord, sha256_hex,
    InputLockV4, TransactionGraphV1, TRANSACTION_GRAPH_FIELDS,
};

fn hash(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

#[test]
fn transaction_contract_v4_rejects_legacy_and_source_dependent_lock_fields() {
    let lock = include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-input-v4.lock");
    InputLockV4::parse(lock).expect("v4 input lock");
    assert_eq!(
        InputLockV4::parse(&lock.replacen("rar-preauth-closure-input-lock-v4", "rar-preauth-closure-v3", 1))
            .unwrap_err().code,
        "invalid-input-lock-v4"
    );
    let injected = lock.replace(
        "launch_authority=none\n",
        &format!("canonical_oci_archive_sha256={}\nlaunch_authority=none\n", hash('a')),
    );
    assert_eq!(InputLockV4::parse(&injected).unwrap_err().code, "transaction-field-count");
}

#[test]
fn transaction_graph_is_once_hashed_typed_and_complete() {
    let mut payload = String::new();
    for name in &TRANSACTION_GRAPH_FIELDS[..TRANSACTION_GRAPH_FIELDS.len() - 1] {
        let value = match *name {
            "schema" => "rar-preauth-transaction-graph-v1".to_owned(),
            "source_revision" => "a".repeat(40),
            "raw_to_canonical_index_relation" =>
                "strict-json-parse+canonical-serialize-v1".to_owned(),
            _ => hash('b'),
        };
        payload.push_str(name); payload.push('='); payload.push_str(&value); payload.push('\n');
    }
    let graph = format!("{payload}record_sha256={}\n", sha256_hex(payload.as_bytes()));
    let parsed = TransactionGraphV1::parse(&graph).expect("transaction graph v1");
    assert_eq!(parsed.source_revision, "a".repeat(40));
    for required in ["disk_seed_sha256", "disk_initial_sha256", "ovmf_code_sha256",
        "ovmf_vars_sha256", "supervisor_sha256", "publication_receipt_sha256"] {
        assert!(parsed.nodes.contains_key(required), "missing typed edge: {required}");
    }
    let old = graph.replacen("rar-preauth-transaction-graph-v1", "rar-preauth-identity-graph-v2", 1);
    assert_eq!(TransactionGraphV1::parse(&old).unwrap_err().code, "invalid-transaction-graph-v1");
    let changed = graph.replacen(&hash('b'), &hash('c'), 1);
    assert_eq!(TransactionGraphV1::parse(&changed).unwrap_err().code, "transaction-graph-integrity");
}

fn graph() -> IdentityGraph {
    let input = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.identity")
        .replacen("phase=prepared", "phase=attested", 1);
    let split = input.rfind("record_sha256=").unwrap();
    let payload = &input[..split];
    IdentityGraph::parse(&format!("{payload}record_sha256={}\n", sha256_hex(payload.as_bytes()))).unwrap()
}

fn strict_record(graph: &IdentityGraph) -> StrictAuthorityRecord {
    let node = |name: &str| graph.nodes.get(name).unwrap();
    let payload = format!(concat!(
        "schema=rar-external-authorization-v1\nauthorization_id=r0-first-boot-0001\nstate=issued\n",
        "certification_sha256={}\nidentity_graph_sha256={}\nprofile_sha256={}\ncommand_sha256={}\n",
        "artifact_sha256={}\ndisk_sha256={}\nfirmware_code_sha256={}\nfirmware_vars_sha256={}\n",
        "closure_sha256={}\nexecution_host_sha256={}\nresolver_sha256={}\nspawner_sha256={}\n",
        "consumption_key_sha256={}\nnonce=nonce-0001\nissued_at=2026-07-18T12:00:00Z\nexpires_at=2000\n",
        "repository={}\nworkflow={}\nref={}\nenvironment={}\noidc_issuer={}\noidc_audience={}\n",
        "oidc_subject={}\nkms_key_arn={}\nkms_algorithm={}\nkms_context_sha256={}\n",
        "cloudtrail_previous_sha256={}\ncloudtrail_evidence_sha256={}\ntransition_version=1\n"),
        node("prepared_certification_sha256"), graph.document_sha256, node("profile_sha256"), node("command_sha256"),
        node("artifact_sha256"), node("disk_child_sha256"), node("ovmf_code_sha256"), node("ovmf_vars_sha256"),
        node("closure_sha256"), node("execution_host_sha256"), node("resolver_sha256"), node("spawner_sha256"),
        node("consumption_key_sha256"), preauth::AUTHORITY_REPOSITORY, preauth::AUTHORITY_WORKFLOW,
        preauth::AUTHORITY_REF, preauth::AUTHORITY_ENVIRONMENT, preauth::OIDC_ISSUER, preauth::OIDC_AUDIENCE,
        preauth::OIDC_SUBJECT, preauth::KMS_KEY_ARN, preauth::KMS_ALGORITHM, hash('7'), hash('8'), hash('9'));
    let record_hash = sha256_hex(payload.as_bytes());
    StrictAuthorityRecord::parse(&format!("{payload}record_sha256={record_hash}\nkms_signature_sha256={}\n", hash('a'))).unwrap()
}

#[test]
fn exact_binding_consumes_once() {
    let graph = graph(); let candidate = strict_record(&graph);
    let mut ledger = SyntheticLedger::default();
    ledger.issue(candidate.clone(), &graph).unwrap();
    let consumed = consume_once(&mut ledger, &candidate, &graph, 1_000).unwrap();
    assert_eq!(consumed.state, AuthorityState::Consumed);
    assert_eq!(
        consume_once(&mut ledger, &candidate, &graph, 1_000)
            .unwrap_err()
            .code,
        "authority-replay-or-revoked"
    );
}

#[test]
fn rejected_issue_has_no_ledger_side_effect() {
    let graph = graph(); let valid = strict_record(&graph); let mut ledger = SyntheticLedger::default();
    let mut changed = graph.clone(); changed.nodes.insert("artifact_sha256".into(), hash('f'));
    assert_eq!(ledger.issue(valid.clone(), &changed).unwrap_err().code, "identity-edge-mismatch");
    ledger.issue(valid, &graph).unwrap();
}

#[test]
fn stale_substituted_revoked_and_uncertain_fail_closed() {
    let graph = graph(); let candidate = strict_record(&graph);
    let mut changed = graph.clone(); changed.nodes.insert("disk_child_sha256".into(), hash('a'));
    let mut ledger = SyntheticLedger::default();
    ledger.issue(candidate.clone(), &graph).unwrap();
    assert_eq!(
        consume_once(&mut ledger, &candidate, &changed, 1_000)
            .unwrap_err()
            .code,
        "identity-edge-mismatch"
    );
    assert_eq!(
        consume_once(&mut ledger, &candidate, &graph, 2_000)
            .unwrap_err()
            .code,
        "authority-expired-or-state"
    );
    ledger.revoke(&candidate.authorization_id).unwrap();
    assert_eq!(
        consume_once(&mut ledger, &candidate, &graph, 1_000)
            .unwrap_err()
            .code,
        "authority-replay-or-revoked"
    );

    let mut uncertain = SyntheticLedger::default();
    uncertain.issue(candidate.clone(), &graph).unwrap();
    uncertain.make_uncertain();
    assert_eq!(
        consume_once(&mut uncertain, &candidate, &graph, 1_000)
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
        identity_graph: graph(),
        execution_host_sha256: graph().nodes["execution_host_sha256"].clone(),
        resolver_sha256: graph().nodes["resolver_sha256"].clone(), spawner_sha256: graph().nodes["spawner_sha256"].clone(),
        executable_sha256: graph().nodes["qemu_sha256"].clone(), artifact_sha256: graph().nodes["artifact_sha256"].clone(),
        disk_sha256: graph().nodes["disk_child_sha256"].clone(), firmware_code_sha256: graph().nodes["ovmf_code_sha256"].clone(),
        firmware_vars_sha256: graph().nodes["ovmf_vars_sha256"].clone(),
    };
    bind_resolver_to_spawner(&descriptor, &descriptor).unwrap();
    let mut substituted = descriptor.clone();
    substituted.disk_sha256 = hash('b');
    assert_eq!(bind_resolver_to_spawner(&descriptor, &substituted).unwrap_err().code,
        "identity-edge-mismatch");
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

#[test]
fn bounded_json_rejects_structural_spoofs_and_canonicalizes() {
    let parsed = Json::parse(br#" { "b" : [1,"\u0061"], "a" : {"x":true} } "#).unwrap();
    assert_eq!(parsed.canonical(), r#"{"a":{"x":true},"b":[1,"a"]}"#);
    let spoof = Json::parse(br#"{"outer":"\\\"digest\\\":\\\"sha256:00\\\"","dige\u0073t":"lookalike-decoded"}"#).unwrap();
    assert!(spoof.exact_keys(&["digest"], &[]).is_err(), "key text inside a string gained authority");
    for rejected in [
        br#"{"a":1,"a":2}"#.as_slice(), br#"{"key":"\ud800"}"#,
        br#"{"key":1} trailing"#, br#"{"key":01}"#,
    ] { assert!(Json::parse(rejected).is_err()); }
    let deep = format!("{}0{}", "[".repeat(34), "]".repeat(34));
    assert_eq!(Json::parse(deep.as_bytes()).unwrap_err().code, "json-depth-limit");

    let keys = Json::parse(br#"{"z/control\n":1,"a":2,"\u2603":3}"#).unwrap();
    let diagnostic = keys.key_set_diagnostic("test-document", "/outer~name", &["required"], &["a"])
        .unwrap().unwrap();
    assert_eq!(diagnostic, concat!(
        "json_key_set document=test-document path=/outer~0name actual_count=3 actual_reported=3 ",
        "actual=[\"a\", \"z\\\\x2fcontrol\\\\x0a\", \"\\\\xe2\\\\x98\\\\x83\"] allowed_count=2 allowed_reported=2 ",
        "allowed=[\"a\", \"required\"] missing_count=1 missing_reported=1 missing=[\"required\"] ",
        "unknown_count=2 unknown_reported=2 unknown=[\"z\\\\x2fcontrol\\\\x0a\", \"\\\\xe2\\\\x98\\\\x83\"] key_cap=32 key_byte_cap=96",
    ));
    let many = format!("{{{}}}", (0..40).map(|index| format!("\"key{index:02}\":0")).collect::<Vec<_>>().join(","));
    let first = Json::parse(many.as_bytes()).unwrap().key_set_diagnostic("test-document", "/", &[], &[]).unwrap().unwrap();
    let second = Json::parse(many.as_bytes()).unwrap().key_set_diagnostic("test-document", "/", &[], &[]).unwrap().unwrap();
    assert_eq!(first, second);
    assert!(first.contains("actual_count=40 actual_reported=32"));
    assert!(first.contains("unknown_count=40 unknown_reported=32"));
    let oversized_key = format!("{{\"{}\":0}}", "é".repeat(60));
    let bounded = Json::parse(oversized_key.as_bytes()).unwrap()
        .key_set_diagnostic("test-document", "/", &[], &[]).unwrap().unwrap();
    assert!(bounded.len() < 2_000 && bounded.contains("..."));
}

#[test]
fn lifecycle_is_stateful_terminal_and_cleanup_enforcing() {
    let mut machine = LifecycleMachine::prepared();
    for event in [LifecycleEvent::Authorize, LifecycleEvent::Resolve, LifecycleEvent::Spawn,
        LifecycleEvent::Timeout, LifecycleEvent::Kill, LifecycleEvent::Cleanup, LifecycleEvent::Reconcile] {
        machine.apply(event).unwrap();
    }
    assert_eq!(machine.state(), LifecycleState::Consumed);
    machine.finish().unwrap();
    assert_eq!(machine.apply(LifecycleEvent::Spawn).unwrap_err().code, "lifecycle-terminal");
    let mut illegal = LifecycleMachine::prepared();
    assert_eq!(illegal.apply(LifecycleEvent::Spawn).unwrap_err().code, "lifecycle-order");
    assert_eq!(illegal.finish().unwrap_err().code, "lifecycle-not-terminal-clean");
}

#[test]
fn disposable_disk_effect_path_is_descriptor_relative_and_noreplacing() {
    let source = include_str!("../../../tools/rar-lab/preauth/src/disk.rs");
    for required in ["O_NOFOLLOW", "O_EXCL", "renameat2", "RENAME_NOREPLACE", "openat", "mkdirat"] {
        assert!(source.contains(required), "missing descriptor invariant: {required}");
    }
    for forbidden in ["create_dir_all", "OpenOptions", ".starts_with("] {
        assert!(!source.contains(forbidden), "path-based disk effect remains: {forbidden}");
    }
}

#[test]
fn acquisition_plans_and_pins_everything_before_extraction_or_use() {
    let source = include_str!("../../../tools/toolchain/acquire-preauth-closure.sh");
    let closure_compare = source.find("observed package closure differs").unwrap();
    let lock_validation = source.find("assert_lock acquisition_policy_sha256").unwrap();
    let first_extract = source.find("/usr/bin/dpkg-deb -x").unwrap();
    let first_import = source.find("/usr/bin/cp -a \"$private_stage/rootfs/.\"").unwrap();
    assert!(closure_compare < lock_validation && lock_validation < first_extract && first_extract < first_import);
    for required in ["archive.plan", "content-addressed", "immutable staged bytes", "ovmf_code_sha256", "destination collision"] {
        assert!(source.contains(required), "missing complete-plan invariant: {required}");
    }
}
