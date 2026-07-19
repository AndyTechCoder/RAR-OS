use std::collections::BTreeMap;

use super::preauth::{canonicalize_base_oci, sha256_hex};

#[derive(Clone)]
struct Member { path: String, kind: u8, mode: &'static [u8; 8], payload: Vec<u8> }

fn file(path: &str, payload: &[u8]) -> Member {
    Member { path: path.into(), kind: b'0', mode: b"0000644\0", payload: payload.to_vec() }
}

fn directory(path: &str) -> Member {
    Member { path: format!("{path}/"), kind: b'5', mode: b"0000755\0", payload: Vec::new() }
}

fn pax(records: &str) -> Member {
    Member { path: "PaxHeaders.0/next".into(), kind: b'x', mode: b"0000644\0", payload: records.as_bytes().to_vec() }
}

fn pax_record(key: &str, value: &str) -> String {
    let bare = format!(" {key}={value}\n");
    let total = (bare.len() + 1..bare.len() + 8)
        .find(|total| total.to_string().len() + bare.len() == *total)
        .expect("representable pax length");
    format!("{total}{bare}")
}

fn render(members: &[Member]) -> Vec<u8> {
    let mut archive = Vec::new();
    for member in members {
        let mut header = [0u8; 512];
        header[..member.path.len()].copy_from_slice(member.path.as_bytes());
        header[100..108].copy_from_slice(member.mode);
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        header[124..136].copy_from_slice(format!("{:011o}\0", member.payload.len()).as_bytes());
        header[136..148].copy_from_slice(b"14000000000\0");
        header[148..156].fill(b' ');
        header[156] = member.kind;
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u64 = header.iter().map(|byte| *byte as u64).sum();
        header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
        archive.extend_from_slice(&header);
        archive.extend_from_slice(&member.payload);
        archive.resize(archive.len().div_ceil(512) * 512, 0);
    }
    archive.resize(archive.len() + 1024, 0);
    archive
}

struct Layout {
    config: Vec<u8>,
    layers: Vec<Vec<u8>>,
    manifest: Option<Vec<u8>>,
    index: Option<Vec<u8>>,
    legacy: Option<Vec<u8>>,
    extra_blobs: Vec<Vec<u8>>,
    extra_members: Vec<Member>,
}

impl Default for Layout {
    fn default() -> Self {
        let layers = vec![b"layer-one-bytes".to_vec(), b"layer-two-bytes".to_vec()];
        let diff_ids: Vec<String> = layers.iter().map(|layer| format!("\"sha256:{}\"", sha256_hex(layer))).collect();
        let config = format!(
            "{{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{{\"type\":\"layers\",\"diff_ids\":[{}]}}}}",
            diff_ids.join(",")).into_bytes();
        Self { config, layers, manifest: None, index: None, legacy: None, extra_blobs: Vec::new(), extra_members: Vec::new() }
    }
}

impl Layout {
    fn manifest_bytes(&self) -> Vec<u8> {
        self.manifest.clone().unwrap_or_else(|| {
            let layers: Vec<String> = self.layers.iter().map(|layer| format!(
                "{{\"mediaType\":\"application/vnd.oci.image.layer.v1.tar\",\"digest\":\"sha256:{}\",\"size\":{}}}",
                sha256_hex(layer), layer.len())).collect();
            format!(
                "{{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"config\":{{\"mediaType\":\"application/vnd.oci.image.config.v1+json\",\"digest\":\"sha256:{}\",\"size\":{}}},\"layers\":[{}]}}",
                sha256_hex(&self.config), self.config.len(), layers.join(",")).into_bytes()
        })
    }
    fn render(&self) -> Vec<u8> {
        let manifest = self.manifest_bytes();
        let index = self.index.clone().unwrap_or_else(|| format!(
            "{{\"schemaVersion\":2,\"manifests\":[{{\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"digest\":\"sha256:{}\",\"size\":{}}}]}}",
            sha256_hex(&manifest), manifest.len()).into_bytes());
        let legacy = self.legacy.clone().unwrap_or_else(|| {
            let layers: Vec<String> = self.layers.iter().map(|layer| format!("\"blobs/sha256/{}\"", sha256_hex(layer))).collect();
            format!("[{{\"Config\":\"blobs/sha256/{}\",\"Layers\":[{}],\"RepoTags\":null}}]",
                sha256_hex(&self.config), layers.join(",")).into_bytes()
        });
        let mut blobs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        blobs.insert(sha256_hex(&self.config), self.config.clone());
        blobs.insert(sha256_hex(&manifest), manifest);
        for layer in &self.layers { blobs.insert(sha256_hex(layer), layer.clone()); }
        for blob in &self.extra_blobs { blobs.insert(sha256_hex(blob), blob.clone()); }
        let mut members = vec![
            directory("blobs"), directory("blobs/sha256"),
            file("oci-layout", b"{\"imageLayoutVersion\":\"1.0.0\"}"),
            file("index.json", &index), file("manifest.json", &legacy),
        ];
        for (digest, payload) in &blobs { members.push(file(&format!("blobs/sha256/{digest}"), payload)); }
        members.extend(self.extra_members.iter().cloned());
        render(&members)
    }
}

#[test]
fn base_oci_accepts_and_deterministically_canonicalizes_a_rooted_graph() {
    let layout = Layout::default();
    let raw = layout.render();
    let first = canonicalize_base_oci(&raw).expect("valid layout");
    let second = canonicalize_base_oci(&raw).expect("valid layout");
    assert_eq!(first, second);
    assert_eq!(first.layer_count, 2);
    assert_eq!(first.config_sha256, sha256_hex(&layout.config));
    // Canonical output depends only on validated content, not on raw framing noise.
    let mut plain = Layout::default();
    plain.extra_members = vec![file("repositories", b"{}")];
    let mut noisy = Layout::default();
    noisy.extra_members = vec![pax(&pax_record("mtime", "1784332800.5")), file("repositories", b"{}")];
    let plain_raw = plain.render();
    let noisy_raw = noisy.render();
    assert_ne!(plain_raw, noisy_raw);
    assert_eq!(canonicalize_base_oci(&plain_raw).expect("valid layout").canonical,
        canonicalize_base_oci(&noisy_raw).expect("valid layout").canonical);
}

#[test]
fn base_oci_accepts_timestamp_pax_but_rejects_identity_overrides() {
    let mut layout = Layout::default();
    layout.extra_members = vec![pax(&pax_record("mtime", "1784332800.123")), file("repositories", b"{}")];
    canonicalize_base_oci(&layout.render()).expect("timestamp pax is metadata only");
    let mut override_layout = Layout::default();
    override_layout.extra_members = vec![pax(&pax_record("path", "blobs/sha256/injected")), file("repositories", b"{}")];
    assert_eq!(canonicalize_base_oci(&override_layout.render()).unwrap_err().code, "base-oci-pax-override");
}

#[test]
fn base_oci_rejects_traversal_duplicate_collision_link_and_special_members() {
    let mut traversal = Layout::default();
    traversal.extra_members = vec![file("../escape", b"x")];
    assert_eq!(canonicalize_base_oci(&traversal.render()).unwrap_err().code, "transaction-path");
    let mut duplicate = Layout::default();
    duplicate.extra_members = vec![file("oci-layout", b"{\"imageLayoutVersion\":\"1.0.0\"}")];
    assert_eq!(canonicalize_base_oci(&duplicate.render()).unwrap_err().code, "archive-path-collision");
    let mut collision = Layout::default();
    collision.extra_members = vec![file("OCI-LAYOUT", b"x")];
    assert_eq!(canonicalize_base_oci(&collision.render()).unwrap_err().code, "archive-path-collision");
    let mut link = Layout::default();
    link.extra_members = vec![Member { path: "blobs/link".into(), kind: b'2', mode: b"0000644\0", payload: Vec::new() }];
    assert_eq!(canonicalize_base_oci(&link.render()).unwrap_err().code, "base-oci-member-type");
    let mut special = Layout::default();
    special.extra_members = vec![Member { path: "device".into(), kind: b'3', mode: b"0000644\0", payload: Vec::new() }];
    assert_eq!(canonicalize_base_oci(&special.render()).unwrap_err().code, "base-oci-member-type");
}

#[test]
fn base_oci_rejects_substituted_dangling_or_unexpected_content() {
    let mut substituted = Layout::default();
    let manifest = substituted.manifest_bytes();
    substituted.index = Some(format!(
        "{{\"schemaVersion\":2,\"manifests\":[{{\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"digest\":\"sha256:{}\",\"size\":{}}}]}}",
        "a".repeat(64), manifest.len()).into_bytes());
    assert_eq!(canonicalize_base_oci(&substituted.render()).unwrap_err().code, "base-oci-missing-blob");
    let mut wrong_size = Layout::default();
    wrong_size.index = Some(format!(
        "{{\"schemaVersion\":2,\"manifests\":[{{\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"digest\":\"sha256:{}\",\"size\":{}}}]}}",
        sha256_hex(&manifest), manifest.len() + 1).into_bytes());
    assert_eq!(canonicalize_base_oci(&wrong_size.render()).unwrap_err().code, "base-oci-descriptor-size");
    // Dangling raw-store metadata blobs are digest-verified, then excluded from canonical
    // generation per ADR 0018: the canonical archive equals the rooted graph exactly.
    let mut dangling = Layout::default();
    dangling.extra_blobs = vec![b"unreferenced".to_vec()];
    let excluded = canonicalize_base_oci(&dangling.render()).expect("dangling metadata is excluded");
    assert_eq!(excluded.canonical, canonicalize_base_oci(&Layout::default().render()).expect("rooted graph").canonical);
    let mut corrupt_dangling = Layout::default();
    corrupt_dangling.extra_members = vec![file(&format!("blobs/sha256/{}", "f".repeat(64)), b"corrupt")];
    assert_eq!(canonicalize_base_oci(&corrupt_dangling.render()).unwrap_err().code, "base-oci-blob-digest");
    let mut renamed = Layout::default();
    renamed.extra_members = vec![file(&format!("blobs/sha256/{}", "b".repeat(64)), b"mismatch")];
    assert_eq!(canonicalize_base_oci(&renamed.render()).unwrap_err().code, "base-oci-blob-digest");
    let mut unexpected = Layout::default();
    unexpected.extra_members = vec![file("blobs/extra", b"x")];
    assert_eq!(canonicalize_base_oci(&unexpected.render()).unwrap_err().code, "base-oci-unexpected-member");
    let mut stray_root = Layout::default();
    stray_root.extra_members = vec![file("notes.txt", b"x")];
    assert_eq!(canonicalize_base_oci(&stray_root.render()).unwrap_err().code, "base-oci-unexpected-member");
}

#[test]
fn base_oci_rejects_graph_platform_and_legacy_binding_defects() {
    let missing_root = render(&[
        directory("blobs"), directory("blobs/sha256"),
        file("oci-layout", b"{\"imageLayoutVersion\":\"1.0.0\"}"),
    ]);
    assert_eq!(canonicalize_base_oci(&missing_root).unwrap_err().code, "base-oci-roots");
    let mut diff_mismatch = Layout::default();
    diff_mismatch.config = format!(
        "{{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{{\"type\":\"layers\",\"diff_ids\":[\"sha256:{}\",\"sha256:{}\"]}}}}",
        "c".repeat(64), "d".repeat(64)).into_bytes();
    assert_eq!(canonicalize_base_oci(&diff_mismatch.render()).unwrap_err().code, "base-oci-diff-id");
    let mut platform = Layout::default();
    platform.config = b"{\"architecture\":\"arm64\",\"os\":\"linux\",\"rootfs\":{\"type\":\"layers\",\"diff_ids\":[]}}".to_vec();
    assert_eq!(canonicalize_base_oci(&platform.render()).unwrap_err().code, "base-oci-platform");
    let mut legacy = Layout::default();
    legacy.legacy = Some(format!("[{{\"Config\":\"blobs/sha256/{}\",\"Layers\":[]}}]", "e".repeat(64)).into_bytes());
    assert_eq!(canonicalize_base_oci(&legacy.render()).unwrap_err().code, "base-oci-legacy-config");
    let mut sources = Layout::default();
    let manifest_default = Layout::default();
    let layer_zero = sha256_hex(&manifest_default.layers[0]);
    let layer_one = sha256_hex(&manifest_default.layers[1]);
    sources.legacy = Some(format!(
        "[{{\"Config\":\"blobs/sha256/{}\",\"Layers\":[\"blobs/sha256/{layer_zero}\",\"blobs/sha256/{layer_one}\"],\"LayerSources\":{{\"sha256:{layer_zero}\":{{\"mediaType\":\"application/vnd.oci.image.layer.v1.tar\",\"digest\":\"sha256:{layer_one}\",\"size\":15}},\"sha256:{layer_one}\":{{\"mediaType\":\"application/vnd.oci.image.layer.v1.tar\",\"digest\":\"sha256:{layer_one}\",\"size\":15}}}}}}]",
        sha256_hex(&sources.config)).into_bytes());
    assert_eq!(canonicalize_base_oci(&sources.render()).unwrap_err().code, "base-oci-layer-sources");
}
