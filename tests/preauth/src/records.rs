#![deny(unsafe_code)]

#[path = "../../../tools/rar-lab/preauth/src/lib.rs"]
mod preauth;

use preauth::{AttestationRecord, ClosureLock, ExecutionHostRecord, IdentityGraph,
    PackageManifest, PreparedCertification, sha256_hex};

fn attestation(head: &str, event: &str, run: u64, archive: char, image: char) -> String {
    let graph = include_str!("../../../spec/lab/preauth/prepared/r0-x86_64-preauth-v1.identity");
    let graph_sha = sha256_hex(graph.as_bytes());
    let payload = format!(concat!(
        "schema=rar-preauth-ci-attestation-v1\nphase=attested\n",
        "prepared_identity_graph_sha256={}\nsource_revision={}\nevent={}\nrun_id={}\n",
        "archive_sha256={}\nbuildx_descriptor_kind=docker-config-id\n",
        "buildx_descriptor_sha256={}\ndocker_config_sha256={}\nselected_oci_manifest_sha256={}\ncanonical_oci_index_sha256={}\n",
        "layer_descriptor_set_sha256={}\nrootfs_diff_id_set_sha256={}\nloaded_image_config_sha256={}\n",
        "package_manifest_sha256={}\nprofile_sha256={}\nartifact_sha256={}\n",
        "disk_sha256={}\nclosure_sha256={}\n"), graph_sha, head, event, run,
        archive.to_string().repeat(64), image.to_string().repeat(64), image.to_string().repeat(64),
        "c".repeat(64), "1".repeat(64), "d".repeat(64), "e".repeat(64), image.to_string().repeat(64),
        "a39ba029b4107d9c52d91ae90f36751b7dbb30ffff385e3e7209b266f8747fd5",
        "8e7bc38fa513700556b7ea493ffd42b6df6b4adcaf0a4719a0c7fe11f7eb165f",
        "96b7705f1dd987060c34ac049afd5a0d20fa58d8aff6586ce9090dbdf8a989ea",
        "141d4f9b5756451e4d5874ac2d68c5c59052b82e52494d29ef8624fa3402e766",
        "6fce29dad39d01bc08d134c6c3bbbad7201f9fa6c2b3448349d8010be477185a");
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
    let closure = include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-v2.lock");
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
    assert_eq!(graph.digests.len(), 23);
    for required in [&closure_record.package_manifest_sha256, &closure_record.license_manifest_sha256,
        &closure_record.lld_sha256, &closure_record.qemu_sha256, &closure_record.ovmf_code_sha256,
        &closure_record.ovmf_vars_sha256, &host_record.record_sha256, &cert_record.record_sha256,
        &authority_identity_sha, &consumption_key_sha, &sha256_hex(source_tree.as_bytes())] {
        assert!(graph.digests.contains(required), "identity edge omitted: {required}");
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
    let exact = attestation(head, "push", 42, 'a', 'b');
    AttestationRecord::parse(&exact, head, "push", 42).expect("exact CI attestation");
    assert!(AttestationRecord::parse(&exact, &"f".repeat(40), "push", 42).is_err());
    assert!(AttestationRecord::parse(&exact, head, "pull_request", 42).is_err());
    assert!(AttestationRecord::parse(&exact, head, "push", 43).is_err());
    let stale_phase = exact.replacen("phase=attested", "phase=prepared", 1);
    assert!(AttestationRecord::parse(&stale_phase, head, "push", 42).is_err());
    let substituted_image = exact.replacen(&format!("loaded_image_config_sha256={}", "b".repeat(64)),
        &format!("loaded_image_config_sha256={}", "c".repeat(64)), 1);
    assert!(AttestationRecord::parse(&substituted_image, head, "push", 42).is_err());
    let typed_substitutions = vec![
        ("buildx_descriptor_kind=docker-config-id".to_owned(), "buildx_descriptor_kind=oci-manifest".to_owned()),
        (format!("buildx_descriptor_sha256={}", "b".repeat(64)), format!("buildx_descriptor_sha256={}", "c".repeat(64))),
        (format!("selected_oci_manifest_sha256={}", "c".repeat(64)), format!("selected_oci_manifest_sha256={}", "e".repeat(64))),
        (format!("canonical_oci_index_sha256={}", "1".repeat(64)), format!("canonical_oci_index_sha256={}", "2".repeat(64))),
        (format!("rootfs_diff_id_set_sha256={}", "e".repeat(64)), format!("rootfs_diff_id_set_sha256={}", "f".repeat(64))),
    ];
    for (field, replacement) in typed_substitutions {
        assert!(AttestationRecord::parse(&exact.replacen(&field, &replacement, 1), head, "push", 42).is_err());
    }

    for record in [closure, packages, host, certification, identity] {
        assert!(!record.contains("source_revision="));
        assert!(!record.contains("run_id="));
        assert!(!record.contains("archive_sha256="));
    }
}
