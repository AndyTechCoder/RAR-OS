use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};

use super::preauth::{DescriptorDir, canonicalize_base_oci, sha256_hex};

fn canonicalize(raw: &[u8]) -> super::preauth::Result<super::preauth::BaseOciCanonical> {
    canonicalize_base_oci(raw)
}

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

fn corrupt_first_nonempty_padding(archive: &mut [u8]) {
    let mut offset = 0usize;
    loop {
        let size_text = std::str::from_utf8(&archive[offset + 124..offset + 136]).unwrap()
            .trim_matches(char::from(0)).trim();
        let size = usize::from_str_radix(size_text, 8).unwrap();
        if size % 512 != 0 {
            archive[offset + 512 + size] = 1;
            return;
        }
        offset += 512 + size.div_ceil(512) * 512;
    }
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
    index: Option<Vec<u8>>,
    legacy: Option<Vec<u8>>,
    extra_blobs: Vec<Vec<u8>>,
    lead_members: Vec<Member>,
    extra_members: Vec<Member>,
}

impl Default for Layout {
    fn default() -> Self {
        let layers = vec![b"layer-one-bytes".to_vec(), b"layer-two-bytes".to_vec()];
        let diff_ids: Vec<String> = layers.iter().map(|layer| format!("\"sha256:{}\"", sha256_hex(layer))).collect();
        let config = format!(
            "{{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{{\"type\":\"layers\",\"diff_ids\":[{}]}}}}",
            diff_ids.join(",")).into_bytes();
        Self { config, layers, index: None, legacy: None, extra_blobs: Vec::new(),
            lead_members: Vec::new(), extra_members: Vec::new() }
    }
}

impl Layout {
    fn manifest_bytes(&self) -> Vec<u8> {
        let layers: Vec<String> = self.layers.iter().map(|layer| format!(
            "{{\"mediaType\":\"application/vnd.oci.image.layer.v1.tar\",\"digest\":\"sha256:{}\",\"size\":{}}}",
            sha256_hex(layer), layer.len())).collect();
        format!(
            "{{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"config\":{{\"mediaType\":\"application/vnd.oci.image.config.v1+json\",\"digest\":\"sha256:{}\",\"size\":{}}},\"layers\":[{}]}}",
            sha256_hex(&self.config), self.config.len(), layers.join(",")).into_bytes()
    }
    fn index_bytes(&self, manifest: &[u8]) -> Vec<u8> {
        self.index.clone().unwrap_or_else(|| format!(
            "{{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.index.v1+json\",\"manifests\":[{{\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"digest\":\"sha256:{}\",\"size\":{}}}]}}",
            sha256_hex(manifest), manifest.len()).into_bytes())
    }
    fn legacy_bytes(&self) -> Vec<u8> {
        self.legacy.clone().unwrap_or_else(|| {
            let layers: Vec<String> = self.layers.iter().map(|layer| format!("\"blobs/sha256/{}\"", sha256_hex(layer))).collect();
            let sources: Vec<String> = self.layers.iter().map(|layer| format!(
                "\"sha256:{0}\":{{\"mediaType\":\"application/vnd.oci.image.layer.v1.tar\",\"digest\":\"sha256:{0}\",\"size\":{1}}}",
                sha256_hex(layer), layer.len())).collect();
            format!("[{{\"Config\":\"blobs/sha256/{}\",\"RepoTags\":null,\"Layers\":[{}],\"LayerSources\":{{{}}}}}]",
                sha256_hex(&self.config), layers.join(","), sources.join(",")).into_bytes()
        })
    }
    fn render(&self) -> Vec<u8> {
        let manifest = self.manifest_bytes();
        let index = self.index_bytes(&manifest);
        let legacy = self.legacy_bytes();
        let mut blobs: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        blobs.insert(sha256_hex(&self.config), self.config.clone());
        blobs.insert(sha256_hex(&manifest), manifest);
        for layer in &self.layers { blobs.insert(sha256_hex(layer), layer.clone()); }
        for blob in &self.extra_blobs { blobs.insert(sha256_hex(blob), blob.clone()); }
        let mut members = self.lead_members.clone();
        members.extend([
            directory("blobs"), directory("blobs/sha256"),
            file("oci-layout", b"{\"imageLayoutVersion\":\"1.0.0\"}"),
            file("index.json", &index), file("manifest.json", &legacy),
        ]);
        for (digest, payload) in &blobs { members.push(file(&format!("blobs/sha256/{digest}"), payload)); }
        members.extend(self.extra_members.iter().cloned());
        render(&members)
    }
}

#[test]
fn base_oci_accepts_and_deterministically_canonicalizes_a_rooted_graph() {
    let layout = Layout::default();
    let raw = layout.render();
    let first = canonicalize(&raw).expect("valid layout");
    let second = canonicalize(&raw).expect("valid layout");
    assert_eq!(first, second);
    assert_eq!(first.layer_count, 2);
    assert_eq!(first.config_sha256, sha256_hex(&layout.config));
    let mut nonzero_padding = raw.clone();
    corrupt_first_nonempty_padding(&mut nonzero_padding);
    assert_eq!(canonicalize(&nonzero_padding).unwrap_err().code, "base-oci-tar-padding");
    // Canonical output depends only on validated content, not on raw framing noise.
    let mut noisy = Layout::default();
    noisy.lead_members = vec![pax(&pax_record("mtime", "1784332800.5"))];
    let noisy_raw = noisy.render();
    assert_ne!(raw, noisy_raw);
    assert_eq!(first.canonical, canonicalize(&noisy_raw).expect("valid layout").canonical);
}

#[test]
fn base_oci_rejects_one_over_member_bound_without_effects() {
    let members: Vec<_> = (0..=super::ARCHIVE_MEMBER_BOUND)
        .map(|index| directory(&format!("entry-{index:04}")))
        .collect();
    let one_over = render(&members);
    super::assert_side_effect_free_rejection("archive-member-count",
        || canonicalize(&one_over));
}

#[test]
fn base_oci_accepts_timestamp_pax_but_rejects_overrides_and_bad_values() {
    let mut timestamps = Layout::default();
    timestamps.lead_members = vec![pax(&format!("{}{}",
        pax_record("mtime", "1784332800.123"), pax_record("atime", "1784332800")))];
    canonicalize(&timestamps.render()).expect("timestamp pax is metadata only");
    let mut path_override = Layout::default();
    path_override.lead_members = vec![pax(&pax_record("path", "blobs/sha256/injected"))];
    assert_eq!(canonicalize(&path_override.render()).unwrap_err().code, "base-oci-pax-override");
    let mut comment = Layout::default();
    comment.lead_members = vec![pax(&pax_record("comment", "noise"))];
    assert_eq!(canonicalize(&comment.render()).unwrap_err().code, "base-oci-pax-override");
    let mut bad_value = Layout::default();
    bad_value.lead_members = vec![pax(&pax_record("mtime", "yesterday"))];
    assert_eq!(canonicalize(&bad_value.render()).unwrap_err().code, "base-oci-pax-value");
    // The canonical grammar admits exactly one raw encoding: leading-zero record lengths and
    // seconds, duplicate timestamp keys, and trailing-zero fractions are all noncanonical.
    let mut zero_length = Layout::default();
    zero_length.lead_members = vec![pax(&format!("0{}", pax_record("mtime", "1784332800")))];
    assert_eq!(canonicalize(&zero_length.render()).unwrap_err().code, "base-oci-pax-record");
    let mut zero_seconds = Layout::default();
    zero_seconds.lead_members = vec![pax(&pax_record("mtime", "0178433280"))];
    assert_eq!(canonicalize(&zero_seconds.render()).unwrap_err().code, "base-oci-pax-value");
    let mut duplicate_key = Layout::default();
    duplicate_key.lead_members = vec![pax(&format!("{}{}",
        pax_record("mtime", "1784332800"), pax_record("mtime", "1784332801")))];
    assert_eq!(canonicalize(&duplicate_key.render()).unwrap_err().code, "base-oci-pax-duplicate");
    let mut trailing_zero = Layout::default();
    trailing_zero.lead_members = vec![pax(&pax_record("mtime", "1784332800.50"))];
    assert_eq!(canonicalize(&trailing_zero.render()).unwrap_err().code, "base-oci-pax-value");
}

#[test]
fn base_oci_rejects_traversal_duplicate_collision_link_special_and_metadata_members() {
    let mut traversal = Layout::default();
    traversal.extra_members = vec![file("../escape", b"x")];
    assert_eq!(canonicalize(&traversal.render()).unwrap_err().code, "transaction-path");
    let mut duplicate = Layout::default();
    duplicate.extra_members = vec![file("oci-layout", b"{\"imageLayoutVersion\":\"1.0.0\"}")];
    assert_eq!(canonicalize(&duplicate.render()).unwrap_err().code, "archive-path-collision");
    let mut collision = Layout::default();
    collision.extra_members = vec![file("OCI-LAYOUT", b"x")];
    assert_eq!(canonicalize(&collision.render()).unwrap_err().code, "archive-path-collision");
    let mut link = Layout::default();
    link.extra_members = vec![Member { path: "blobs/link".into(), kind: b'2', mode: b"0000644\0", payload: Vec::new() }];
    assert_eq!(canonicalize(&link.render()).unwrap_err().code, "base-oci-member-type");
    let mut special = Layout::default();
    special.extra_members = vec![Member { path: "device".into(), kind: b'3', mode: b"0000644\0", payload: Vec::new() }];
    assert_eq!(canonicalize(&special.render()).unwrap_err().code, "base-oci-member-type");
    let mut loose_file = Layout::default();
    loose_file.extra_members = vec![Member { path: "notes".into(), kind: b'0', mode: b"0000600\0", payload: b"x".to_vec() }];
    assert_eq!(canonicalize(&loose_file.render()).unwrap_err().code, "base-oci-member-mode");
    let mut loose_directory = Layout::default();
    loose_directory.extra_members = vec![Member { path: "extra/".into(), kind: b'5', mode: b"0000700\0", payload: Vec::new() }];
    assert_eq!(canonicalize(&loose_directory.render()).unwrap_err().code, "base-oci-member-mode");
    let mut nonroot = Layout::default();
    let mut member = file("notes", b"x");
    member.mode = b"0000644\0";
    nonroot.extra_members = vec![Member { path: member.path, kind: member.kind, mode: member.mode, payload: member.payload }];
    let mut raw = nonroot.render();
    let position = raw.windows(5).position(|window| window == b"notes").expect("member");
    raw[position + 108..position + 116].copy_from_slice(b"0000001\0");
    let checksum_start = position + 148;
    raw[checksum_start..checksum_start + 8].fill(b' ');
    let sum: u64 = raw[position..position + 512].iter().map(|byte| *byte as u64).sum();
    raw[checksum_start..checksum_start + 8].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    assert_eq!(canonicalize(&raw).unwrap_err().code, "base-oci-member-owner");
}

#[test]
fn base_oci_rejects_substituted_dangling_or_unexpected_content() {
    let base = Layout::default();
    let manifest = base.manifest_bytes();
    let index_text = String::from_utf8(base.index_bytes(&manifest)).unwrap();
    let mut substituted = Layout::default();
    substituted.index = Some(index_text.replace(&sha256_hex(&manifest), &"a".repeat(64)).into_bytes());
    assert_eq!(canonicalize(&substituted.render()).unwrap_err().code, "base-oci-missing-blob");
    let mut wrong_size = Layout::default();
    wrong_size.index = Some(index_text.replace(
        &format!("\"size\":{}", manifest.len()), &format!("\"size\":{}", manifest.len() + 1)).into_bytes());
    assert_eq!(canonicalize(&wrong_size.render()).unwrap_err().code, "base-oci-descriptor-size");
    // Dangling raw-store metadata blobs are digest-verified, then excluded from canonical
    // generation per ADR 0018: the canonical archive equals the rooted graph exactly.
    let mut dangling = Layout::default();
    dangling.extra_blobs = vec![b"unreferenced".to_vec()];
    let excluded = canonicalize(&dangling.render()).expect("dangling metadata is excluded");
    assert_eq!(excluded.canonical, canonicalize(&Layout::default().render()).expect("rooted graph").canonical);
    let mut corrupt_dangling = Layout::default();
    corrupt_dangling.extra_members = vec![file(&format!("blobs/sha256/{}", "f".repeat(64)), b"corrupt")];
    assert_eq!(canonicalize(&corrupt_dangling.render()).unwrap_err().code, "base-oci-blob-digest");
    let mut unexpected = Layout::default();
    unexpected.extra_members = vec![file("blobs/extra", b"x")];
    assert_eq!(canonicalize(&unexpected.render()).unwrap_err().code, "base-oci-unexpected-member");
    let mut repositories = Layout::default();
    repositories.extra_members = vec![file("repositories", b"{}")];
    assert_eq!(canonicalize(&repositories.render()).unwrap_err().code, "base-oci-repositories-present");
}

#[test]
fn base_oci_requires_exact_digest_pull_identity() {
    let base = Layout::default();
    let manifest = base.manifest_bytes();
    let index_text = String::from_utf8(base.index_bytes(&manifest)).unwrap();
    // A digest pull carries no annotation or tag authority: any injected annotation or
    // platform object on the bare index descriptor is substituted identity.
    let mut injected_annotations = Layout::default();
    injected_annotations.index = Some(index_text.replace(
        ",\"size\":",
        ",\"annotations\":{\"io.containerd.image.name\":\"docker.io/library/evil:1.0\"},\"size\":").into_bytes());
    assert_eq!(canonicalize(&injected_annotations.render()).unwrap_err().code, "base-oci-descriptor-keys");
    let mut platform_present = Layout::default();
    platform_present.index = Some(index_text.replace(
        ",\"size\":",
        ",\"platform\":{\"architecture\":\"amd64\",\"os\":\"linux\"},\"size\":").into_bytes());
    assert_eq!(canonicalize(&platform_present.render()).unwrap_err().code, "base-oci-descriptor-keys");
    let legacy_text = String::from_utf8(base.legacy_bytes()).unwrap();
    let mut tagged = Layout::default();
    tagged.legacy = Some(legacy_text.replace("\"RepoTags\":null", "\"RepoTags\":[\"evil:latest\"]").into_bytes());
    assert_eq!(canonicalize(&tagged.render()).unwrap_err().code, "base-oci-repo-tags");
    let mut no_sources = Layout::default();
    let sources_start = legacy_text.find(",\"LayerSources\"").expect("sources");
    no_sources.legacy = Some(format!("{}}}]", &legacy_text[..sources_start]).into_bytes());
    assert_eq!(canonicalize(&no_sources.render()).unwrap_err().code, "base-oci-legacy-keys");
    let mut wrong_source = Layout::default();
    let first_layer = sha256_hex(&base.layers[0]);
    let second_layer = sha256_hex(&base.layers[1]);
    wrong_source.legacy = Some(legacy_text.replace(
        &format!("\"digest\":\"sha256:{first_layer}\",\"size\":{}", base.layers[0].len()),
        &format!("\"digest\":\"sha256:{second_layer}\",\"size\":{}", base.layers[0].len())).into_bytes());
    assert_eq!(canonicalize(&wrong_source.render()).unwrap_err().code, "base-oci-layer-sources");
}

#[test]
fn base_oci_rejects_graph_platform_and_legacy_binding_defects() {
    let missing_root = render(&[
        directory("blobs"), directory("blobs/sha256"),
        file("oci-layout", b"{\"imageLayoutVersion\":\"1.0.0\"}"),
    ]);
    assert_eq!(canonicalize(&missing_root).unwrap_err().code, "base-oci-roots");
    let mut diff_mismatch = Layout::default();
    diff_mismatch.config = format!(
        "{{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{{\"type\":\"layers\",\"diff_ids\":[\"sha256:{}\",\"sha256:{}\"]}}}}",
        "c".repeat(64), "d".repeat(64)).into_bytes();
    assert_eq!(canonicalize(&diff_mismatch.render()).unwrap_err().code, "base-oci-diff-id");
    let mut platform = Layout::default();
    platform.config = b"{\"architecture\":\"arm64\",\"os\":\"linux\",\"rootfs\":{\"type\":\"layers\",\"diff_ids\":[]}}".to_vec();
    assert_eq!(canonicalize(&platform.render()).unwrap_err().code, "base-oci-platform");
    let base = Layout::default();
    let legacy_text = String::from_utf8(base.legacy_bytes()).unwrap();
    let mut wrong_config = Layout::default();
    wrong_config.legacy = Some(legacy_text.replace(
        &format!("\"Config\":\"blobs/sha256/{}\"", sha256_hex(&base.config)),
        &format!("\"Config\":\"blobs/sha256/{}\"", "e".repeat(64))).into_bytes());
    assert_eq!(canonicalize(&wrong_config.render()).unwrap_err().code, "base-oci-legacy-config");
}

#[test]
fn base_oci_output_descriptor_survives_ancestor_replacement() {
    // The canonical output is created through a held directory descriptor: replacing the
    // pathname ancestor after the descriptor is held cannot redirect the write.
    let scratch = std::env::temp_dir().join(format!("rar-base-oci-held-{}", std::process::id()));
    let original = scratch.join("original");
    std::fs::create_dir_all(&original).expect("scratch");
    let held = DescriptorDir::open_root(&original).expect("held descriptor");
    let replaced = scratch.join("replaced");
    std::fs::rename(&original, &replaced).expect("ancestor replacement");
    std::fs::create_dir(&original).expect("attacker ancestor");
    let mut output = held.create_exclusive_file("canonical.tar").expect("descriptor create");
    use std::io::Write as _;
    output.write_all(b"held-bytes").expect("write");
    output.sync_all().expect("sync");
    assert!(!original.join("canonical.tar").exists());
    assert!(replaced.join("canonical.tar").exists());
    let mut reread = held.open_file("canonical.tar").expect("read through held descriptor");
    reread.seek(SeekFrom::Start(0)).expect("seek");
    let mut bytes = Vec::new();
    reread.read_to_end(&mut bytes).expect("read");
    assert_eq!(bytes, b"held-bytes");
    std::fs::remove_dir_all(&scratch).expect("cleanup");
}
