#![deny(unsafe_code)]
#[path="../../tools/rar-lab/preauth/src/lib.rs"] mod preauth;

use preauth::{canonical_input_bundle_header, sha256_hex};

fn hash(byte: char) -> String { std::iter::repeat_n(byte, 64).collect() }

fn header(name: &str, bytes: &[u8]) -> [u8; 512] {
    canonical_input_bundle_header(name, bytes.len() as u64).unwrap()
}

fn main() {
    let output = std::env::args().nth(1).expect("output path");
    let singleton = ["base-oci", "keyring", "inrelease", "inrelease", "inrelease", "security-inrelease",
        "package-manifest", "license-manifest", "license-archive", "producer-tools", "tool-lld", "tool-qemu",
        "firmware-code", "firmware-vars"];
    let mut logical: Vec<(String, Vec<u8>, String, [&str; 5])> = Vec::new();
    for (index, role) in singleton.iter().enumerate() {
        logical.push(((*role).into(), format!("{role}-{index}").into_bytes(), format!("origin-{index}"), ["-"; 5]));
    }
    for index in 0..36 {
        logical.push(("deb".into(), format!("deb-{index:03}").into_bytes(), format!("snapshot-{index:03}"),
            ["package", "version", "amd64", "source", "source-version"]));
    }
    let mut rows = Vec::new(); let mut files = Vec::new(); let mut aggregate = 0u64;
    for (role, bytes, origin, metadata) in logical {
        let digest = sha256_hex(&bytes); aggregate += bytes.len() as u64;
        rows.push(format!("{role}|objects/{digest}|{}|{digest}|{origin}|{}|{}|{}|{}|{}", bytes.len(),
            metadata[0], metadata[1], metadata[2], metadata[3], metadata[4]));
        files.push((format!("objects/{digest}"), bytes));
    }
    rows.sort();
    let objects = format!("{}\n", rows.join("\n")).into_bytes();
    let payload = format!(concat!(
        "schema=rar-preauth-input-bundle-v1\ninput_lock_sha256={}\nproducer_policy_sha256={}\n",
        "base_oci_index_sha256={}\ndebian_snapshot=20260630T000000Z\npackage_count=36\n",
        "object_count=50\naggregate_bytes={}\nobjects_manifest_sha256={}\n"),
        hash('a'), hash('b'), hash('c'), aggregate, sha256_hex(&objects));
    let manifest = format!("{payload}record_sha256={}\n", sha256_hex(payload.as_bytes())).into_bytes();
    files.push(("manifest.v1".into(), manifest)); files.push(("objects.v1".into(), objects)); files.sort_by(|a,b| a.0.cmp(&b.0));
    let mut archive = Vec::new();
    for (name, bytes) in files {
        archive.extend_from_slice(&header(&name, &bytes)); archive.extend_from_slice(&bytes);
        archive.resize(archive.len().div_ceil(512) * 512, 0);
    }
    archive.resize(archive.len() + 1024, 0);
    std::fs::write(output, archive).expect("write fixture");
}
