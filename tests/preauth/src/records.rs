#![deny(unsafe_code)]

#[path = "../../../tools/rar-lab/preauth/src/lib.rs"]
mod preauth;

use preauth::{AttestationRecord, ClosureLock, ExecutionHostRecord, IdentityGraph,
    PackageManifest, PreparedCertification, sha256_hex};

fn attested_graph() -> String {
    let input = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.identity")
        .replacen("phase=prepared", "phase=attested", 1);
    let split = input.rfind("record_sha256=").unwrap();
    let payload = &input[..split];
    format!("{payload}record_sha256={}\n", sha256_hex(payload.as_bytes()))
}

fn attestation(head: &str, event: &str, run: u64, graph: &IdentityGraph) -> String {
    let node = |name: &str| graph.nodes.get(name).unwrap();
    let payload = format!(concat!(
        "schema=rar-preauth-ci-attestation-v2\nphase=attested\n",
        "attested_identity_graph_sha256={}\nsource_revision={}\nevent={}\nrun_id={}\n",
        "archive_sha256={}\nbuildx_descriptor_kind=docker-config-id\n",
        "buildx_descriptor_sha256={}\ndocker_config_sha256={}\nselected_oci_manifest_sha256={}\ncanonical_oci_index_sha256={}\n",
        "layer_descriptor_set_sha256={}\nrootfs_diff_id_set_sha256={}\nloaded_image_config_sha256={}\n",
        "package_manifest_sha256={}\nprofile_sha256={}\nartifact_sha256={}\n",
        "disk_sha256={}\nclosure_sha256={}\n"), graph.document_sha256, head, event, run,
        node("canonical_oci_archive_sha256"), node("docker_config_sha256"), node("docker_config_sha256"),
        node("selected_oci_manifest_sha256"), node("canonical_oci_index_sha256"),
        node("compressed_layer_descriptor_set_sha256"), node("rootfs_diff_id_set_sha256"), node("docker_config_sha256"),
        node("package_manifest_sha256"), node("profile_sha256"), node("artifact_sha256"),
        node("disk_child_sha256"), node("closure_sha256"));
    format!("{}record_sha256={}\n", payload, sha256_hex(payload.as_bytes()))
}

fn verify_self_hash(record: &str) -> String {
    let marker = "record_sha256=";
    let split = record.rfind(marker).expect("record hash field");
    let payload = &record[..split];
    let digest = record[split + marker.len()..].trim_end();
    assert_eq!(sha256_hex(payload.as_bytes()), digest);
    digest.to_owned()
}

fn main() {
    let packages = include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-packages.v2");
    let closure = include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-v3.lock");
    let host = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.host");
    let certification = include_str!("../../../spec/lab/vm-profile/prepared/r0-x86_64-preauth-v1.cert");
    let identity = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.identity");
    let authority_identity = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.authority-identity");
    let consumption_key = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.consumption-key");
    let source_tree = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.source-tree");

    let package_record = PackageManifest::parse(packages).expect("strict 36-package closure");
    assert_eq!(package_record.rows.len(), 36);
    assert!(package_record.rows.iter().all(|row| !row.source_name.is_empty() && !row.source_version.is_empty()));
    let closure_record = ClosureLock::parse(closure, packages).expect("strict closure lock");
    assert!(ClosureLock::parse(&closure.replacen("rar-preauth-closure-v3", "rar-preauth-closure-v2", 1), packages).is_err());
    let cross_type = closure
        .replace("canonical_oci_index_sha256=cb3d83784aa9576405feb9026dc8ed1aaa2633c350bf917079f2543ef544a298",
                 "canonical_oci_index_sha256=4d59826fb248130555b99aa6bc034f17db7df4a6acbbe1ebc0a8175492476531");
    assert!(ClosureLock::parse(&cross_type, packages).is_err(), "config ID substituted for OCI index");
    let host_record = ExecutionHostRecord::parse(host).expect("strict execution-host leaf");
    let cert_record = PreparedCertification::parse(certification).expect("strict prepared certification");
    let graph = IdentityGraph::parse(identity).expect("complete acyclic identity graph");
    assert_eq!(cert_record.closure_sha256, sha256_hex(closure.as_bytes()));
    assert_eq!(cert_record.execution_host_sha256, host_record.record_sha256);
    let authority_identity_sha = verify_self_hash(authority_identity);
    let consumption_key_sha = verify_self_hash(consumption_key);
    assert!(authority_identity.contains("state=unissued\n"));
    assert!(authority_identity.contains(preauth::AUTHORITY_WORKFLOW));
    assert!(authority_identity.contains(preauth::OIDC_ISSUER));
    assert!(authority_identity.contains(preauth::OIDC_AUDIENCE));
    assert!(authority_identity.contains(preauth::OIDC_SUBJECT));
    assert!(authority_identity.contains(preauth::KMS_KEY_ARN));
    assert!(consumption_key.contains(&format!("certification_sha256={}\n", cert_record.record_sha256)));
    assert!(consumption_key.contains(&format!("authorization_identity_sha256={authority_identity_sha}\n")));
    assert_eq!(graph.nodes.len(), 29);
    for required in [&closure_record.package_manifest_sha256, &closure_record.license_manifest_sha256,
        &closure_record.lld_sha256, &closure_record.qemu_sha256, &closure_record.ovmf_code_sha256,
        &closure_record.ovmf_vars_sha256, &host_record.record_sha256, &cert_record.record_sha256,
        &authority_identity_sha, &consumption_key_sha, &sha256_hex(source_tree.as_bytes())] {
        assert!(graph.nodes.values().any(|value| value == required), "identity edge omitted: {required}");
    }

    // Every binary/source/signature/license row is transitively fixed by the package-manifest
    // digest in the closure. A one-byte valid-looking substitution cannot be accepted by the lock.
    for field in 1..10 {
        let mut lines: Vec<String> = packages.lines().map(str::to_owned).collect();
        let mut fields: Vec<String> = lines[1].split('|').map(str::to_owned).collect();
        fields[field].push('0');
        lines[1] = fields.join("|");
        let mutated = format!("{}\n", lines.join("\n"));
        assert!(ClosureLock::parse(closure, &mutated).is_err(), "unbound package field {field}");
    }

    let head = "0123456789abcdef0123456789abcdef01234567";
    let attested = attested_graph();
    let attested_graph = IdentityGraph::parse(&attested).unwrap();
    let exact = attestation(head, "push", 42, &attested_graph);
    AttestationRecord::parse(&exact, head, "push", 42).expect("exact CI attestation").validate_graph(&attested_graph).expect("attestation binds every prepared edge");
    assert!(AttestationRecord::parse(&exact, &"f".repeat(40), "push", 42).is_err());
    assert!(AttestationRecord::parse(&exact, head, "pull_request", 42).is_err());
    assert!(AttestationRecord::parse(&exact, head, "push", 43).is_err());
    let stale_phase = exact.replacen("phase=attested", "phase=prepared", 1);
    assert!(AttestationRecord::parse(&stale_phase, head, "push", 42).is_err());
    let substituted_image = exact.replacen(
        exact.lines().find(|line| line.starts_with("loaded_image_config_sha256=")).unwrap(),
        &format!("loaded_image_config_sha256={}", "f".repeat(64)), 1);
    assert!(AttestationRecord::parse(&substituted_image, head, "push", 42).is_err());
    let mut typed_substitutions = vec![("buildx_descriptor_kind=docker-config-id".to_owned(), "buildx_descriptor_kind=oci-manifest".to_owned())];
    for name in ["buildx_descriptor_sha256", "selected_oci_manifest_sha256", "canonical_oci_index_sha256", "rootfs_diff_id_set_sha256"] {
        let field = exact.lines().find(|line| line.starts_with(&format!("{name}="))).unwrap().to_owned();
        typed_substitutions.push((field, format!("{name}={}", "f".repeat(64))));
    }
    for (field, replacement) in typed_substitutions {
        assert!(AttestationRecord::parse(&exact.replacen(&field, &replacement, 1), head, "push", 42).is_err());
    }

    for record in [closure, packages, host, certification, identity] {
        assert!(!record.contains("source_revision="));
        assert!(!record.contains("run_id="));
        assert!(!record.lines().any(|line| line.starts_with("archive_sha256=")));
    }
}
