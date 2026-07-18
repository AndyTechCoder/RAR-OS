#![deny(unsafe_code)]

#[path = "../rar-lab/preauth/src/lib.rs"]
mod preauth;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path};

const MAX_ARCHIVE: u64 = 2 * 1024 * 1024 * 1024;
const MAX_MEMBER: u64 = 1024 * 1024 * 1024;
const MAX_MEMBERS: usize = 256;
const MAX_INLINE: u64 = 8 * 1024 * 1024;
const MAX_REPOSITORIES: u64 = 512;
const MAX_DIAGNOSTIC_MEMBERS: usize = 32;
const MAX_DIAGNOSTIC_REFS: usize = 8;
const MAX_DIAGNOSTIC_FIELDS: usize = 16;
const SOURCE_DATE_EPOCH: u64 = 1_784_332_800;

struct TarEntry {
    data: Vec<u8>,
    sha256: String,
    size: u64,
    mode: u64,
    uid: u64,
    gid: u64,
    kind: u8,
}

fn fail(message: &str) -> ! { eprintln!("{message}"); std::process::exit(73) }

fn digest_file(path: &Path) -> String {
    let metadata = fs::symlink_metadata(path).unwrap_or_else(|_| fail("missing OCI input"));
    if !metadata.file_type().is_file() || metadata.len() > MAX_ARCHIVE { fail("invalid OCI input"); }
    let mut file = File::open(path).unwrap_or_else(|_| fail("unreadable OCI input"));
    preauth::sha256_reader(&mut file).unwrap_or_else(|_| fail("unreadable OCI input"))
}

fn parse_octal(field: &[u8]) -> u64 {
    let text = std::str::from_utf8(field).unwrap_or_else(|_| fail("non-ASCII tar number"));
    let text = text.trim_matches(|c| c == '\0' || c == ' ');
    if text.is_empty() || !text.bytes().all(|b| (b'0'..=b'7').contains(&b)) { fail("invalid tar number"); }
    u64::from_str_radix(text, 8).unwrap_or_else(|_| fail("tar number overflow"))
}

fn safe_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 240 && !name.contains('\\') && !name.contains(':')
        && Path::new(name).is_relative()
        && Path::new(name).components().all(|c| matches!(c, Component::Normal(_)))
}

fn tar_entries(path: &Path, require_canonical: bool) -> BTreeMap<String, TarEntry> {
    let mut file = File::open(path).unwrap_or_else(|_| fail("unreadable OCI archive"));
    let mut entries = BTreeMap::new();
    let mut header = [0_u8; 512];
    let mut offset = 0_u64;
    let mut total = 0_u64;
    let mut previous_name: Option<String> = None;
    loop {
        file.read_exact(&mut header).unwrap_or_else(|_| fail("truncated tar header"));
        offset += 512;
        if header.iter().all(|byte| *byte == 0) {
            let mut second = [0_u8; 512];
            file.read_exact(&mut second).unwrap_or_else(|_| fail("truncated tar terminator"));
            if second.iter().any(|byte| *byte != 0) { fail("single tar terminator"); }
            let mut tail = Vec::new();
            file.read_to_end(&mut tail).unwrap_or_else(|_| fail("unreadable tar tail"));
            if tail.len() > 1024 * 1024 || tail.iter().any(|byte| *byte != 0) { fail("nonzero tar trailing data"); }
            break;
        }
        let stored = parse_octal(&header[148..156]);
        let mut checked = header;
        checked[148..156].fill(b' ');
        let actual: u64 = checked.iter().map(|byte| u64::from(*byte)).sum();
        if stored != actual { fail("tar checksum mismatch"); }
        if header[345..500].iter().any(|byte| *byte != 0) { fail("tar prefix refused"); }
        let end = header[..100].iter().position(|byte| *byte == 0).unwrap_or(100);
        let name = std::str::from_utf8(&header[..end]).unwrap_or_else(|_| fail("non-UTF8 tar path"));
        if !safe_name(name) { fail("unsafe tar path"); }
        if require_canonical && previous_name.as_deref().is_some_and(|previous| previous >= name) {
            fail("noncanonical tar member order");
        }
        previous_name = Some(name.to_owned());
        let directory = header[156] == b'5';
        if !matches!(header[156], 0 | b'0' | b'5') { fail("unsupported tar member type"); }
        let mode = parse_octal(&header[100..108]);
        let uid = parse_octal(&header[108..116]);
        let gid = parse_octal(&header[116..124]);
        let size = parse_octal(&header[124..136]);
        let mtime = parse_octal(&header[136..148]);
        if require_canonical && (mode != 0o644 || uid != 0 || gid != 0 || mtime != SOURCE_DATE_EPOCH) {
            fail("noncanonical tar member metadata");
        }
        if directory && size != 0 { fail("nonempty tar directory"); }
        if directory {
            if require_canonical { fail("directory tar member refused"); }
            let normalized = name.trim_end_matches('/');
            eprintln!(
                "oci_raw_directory path={} type=directory size={} mode={:04o} uid={} gid={}",
                normalized, size, mode, uid, gid,
            );
            if !matches!(normalized, "blobs" | "blobs/sha256")
                || mode != 0o755 || uid != 0 || gid != 0
            { fail("invalid OCI raw directory"); }
        }
        total = total.checked_add(size).unwrap_or_else(|| fail("tar total overflow"));
        if size > MAX_MEMBER || total > MAX_ARCHIVE || entries.len() >= MAX_MEMBERS { fail("tar bound exceeded"); }
        if !directory {
            let entry = if size <= MAX_INLINE {
                let size_usize = usize::try_from(size).unwrap_or_else(|_| fail("tar size overflow"));
                let mut data = vec![0_u8; size_usize];
                file.read_exact(&mut data).unwrap_or_else(|_| fail("truncated tar member"));
                TarEntry { sha256: preauth::sha256_hex(&data), data, size, mode, uid, gid, kind: header[156] }
            } else {
                let mut bounded = (&mut file).take(size);
                let sha256 = preauth::sha256_reader(&mut bounded).unwrap_or_else(|_| fail("truncated tar member"));
                if bounded.limit() != 0 { fail("truncated tar member"); }
                TarEntry { data: Vec::new(), sha256, size, mode, uid, gid, kind: header[156] }
            };
            if entries.insert(name.to_owned(), entry).is_some() { fail("duplicate tar member"); }
        }
        let padding = (512 - (size % 512)) % 512;
        file.seek(SeekFrom::Current(i64::try_from(padding).unwrap())).unwrap_or_else(|_| fail("truncated tar padding"));
        offset = offset.checked_add(size + padding).unwrap_or_else(|| fail("tar offset overflow"));
    }
    entries
}

fn print_inventory(archive: &Path, entries: &BTreeMap<String, TarEntry>) {
    eprintln!(
        "oci_member_inventory archive={} members={} reported={} cap={}",
        archive.display(), entries.len(), entries.len().min(MAX_DIAGNOSTIC_MEMBERS), MAX_DIAGNOSTIC_MEMBERS,
    );
    for (name, entry) in entries.iter().take(MAX_DIAGNOSTIC_MEMBERS) {
        let kind = if matches!(entry.kind, 0 | b'0') { "regular" } else { "unsupported" };
        eprintln!(
            "oci_member path={} type={} size={} mode={:04o} uid={} gid={} sha256={}",
            name, kind, entry.size, entry.mode, entry.uid, entry.gid, entry.sha256,
        );
    }
}

fn unexpected_member(name: &str, entry: &TarEntry) -> ! {
    eprintln!(
        "unexpected OCI archive member path={} type=regular size={} mode={:04o} uid={} gid={} sha256={}",
        name, entry.size, entry.mode, entry.uid, entry.gid, entry.sha256,
    );
    if !entry.data.is_empty() {
        match std::str::from_utf8(&entry.data) {
            Ok(text) => eprintln!("unexpected OCI archive member utf8={text:?}"),
            Err(_) => eprintln!("unexpected OCI archive member content=binary"),
        }
    } else {
        eprintln!("unexpected OCI archive member content=over-inline-limit");
    }
    fail("unexpected OCI archive member");
}

fn after<'a>(text: &'a str, marker: &str) -> &'a str {
    text.split_once(marker).map(|(_, value)| value).unwrap_or_else(|| fail("missing JSON field"))
}

fn quoted(text: &str, marker: &str) -> String {
    let rest = after(text, marker);
    let end = rest.find('"').unwrap_or_else(|| fail("malformed JSON string"));
    let value = &rest[..end];
    if value.is_empty() || value.contains('\\') { fail("noncanonical JSON string"); }
    value.to_owned()
}

fn quoted_list(text: &str, marker: &str) -> Vec<String> {
    let rest = after(text, marker);
    let end = rest.find(']').unwrap_or_else(|| fail("malformed JSON list"));
    let inner = &rest[..end];
    if inner.is_empty() { return Vec::new(); }
    inner.split(',').map(|part| {
        let part = part.trim();
        if !part.starts_with('"') || !part.ends_with('"') { fail("malformed JSON list item"); }
        part[1..part.len()-1].to_owned()
    }).collect()
}

fn quoted_values(text: &str, marker: &str) -> Vec<String> {
    let mut rest = text;
    let mut values = Vec::new();
    while let Some((_, after_marker)) = rest.split_once(marker) {
        let end = after_marker.find('"').unwrap_or_else(|| fail("malformed JSON string"));
        let value = &after_marker[..end];
        if value.is_empty() || value.contains('\\') { fail("noncanonical JSON string"); }
        values.push(value.to_owned());
        if values.len() > 128 { fail("JSON value bound exceeded"); }
        rest = &after_marker[end + 1..];
    }
    values
}

fn numeric_values(text: &str, marker: &str) -> Vec<u64> {
    let mut rest = text;
    let mut values = Vec::new();
    while let Some((_, after_marker)) = rest.split_once(marker) {
        let end = after_marker.bytes().position(|byte| !byte.is_ascii_digit())
            .unwrap_or(after_marker.len());
        if end == 0 { fail("malformed JSON number"); }
        values.push(after_marker[..end].parse::<u64>().unwrap_or_else(|_| fail("JSON number overflow")));
        if values.len() > 128 { fail("JSON value bound exceeded"); }
        rest = &after_marker[end..];
    }
    values
}

fn optional_quoted_values(text: &str, marker: &str) -> Vec<String> {
    if text.contains(marker) { quoted_values(text, marker) } else { Vec::new() }
}

fn annotation_pairs(text: &str) -> Vec<(String, String)> {
    let Some((_, rest)) = text.split_once("\"annotations\":{") else { return Vec::new() };
    let end = rest.find('}').unwrap_or_else(|| fail("malformed OCI index annotations"));
    let body = &rest[..end];
    if body.is_empty() { return Vec::new(); }
    let mut pairs = BTreeMap::new();
    for item in body.split(',') {
        let (key, value) = item.split_once(':').unwrap_or_else(|| fail("malformed OCI index annotation"));
        if key.len() < 2 || value.len() < 2 || !key.starts_with('"') || !key.ends_with('"')
            || !value.starts_with('"') || !value.ends_with('"')
        { fail("malformed OCI index annotation"); }
        let key = &key[1..key.len() - 1];
        let value = &value[1..value.len() - 1];
        if key.is_empty() || key.len() > 128 || value.len() > 512
            || key.contains('\\') || value.contains('\\')
        { fail("noncanonical OCI index annotation"); }
        if pairs.insert(key.to_owned(), value.to_owned()).is_some() { fail("duplicate OCI index annotation"); }
        if pairs.len() > MAX_DIAGNOSTIC_FIELDS { fail("OCI index annotation bound exceeded"); }
    }
    pairs.into_iter().collect()
}

fn report_index_mapping(index: &str, expected_digest: &str, expected_size: u64, revision: &str) {
    let schema = numeric_values(index, "\"schemaVersion\":");
    let media = optional_quoted_values(index, "\"mediaType\":\"");
    let digests = descriptor_digests(index);
    let sizes = numeric_values(index, "\"size\":");
    let architecture = optional_quoted_values(index, "\"architecture\":\"");
    let os = optional_quoted_values(index, "\"os\":\"");
    let variant = optional_quoted_values(index, "\"variant\":\"");
    let os_version = optional_quoted_values(index, "\"os.version\":\"");
    let os_features = if index.contains("\"os.features\":[") {
        quoted_list(index, "\"os.features\":[")
    } else { Vec::new() };
    let features = if index.contains("\"features\":[") {
        quoted_list(index, "\"features\":[")
    } else { Vec::new() };
    eprintln!(
        "oci_index_mapping actual_schema={schema:?} expected_schema=[2] expected_schema_source=oci-image-spec actual_media={media:?} expected_media=[application/vnd.oci.image.index.v1+json,application/vnd.oci.image.manifest.v1+json] expected_media_source=validated-index-and-selected-manifest",
    );
    eprintln!(
        "oci_index_descriptor count={} order=archive-index-order actual_digest={digests:?} expected_digest=[{expected_digest}] expected_digest_source=sha256-selected-manifest-bytes actual_size={sizes:?} expected_size=[{expected_size}] expected_size_source=validated-selected-manifest-byte-length",
        digests.len(),
    );
    eprintln!(
        "oci_index_platform actual_os={os:?} expected_os=[] expected_os_source=canonical-index-omission-platform-bound-by-config actual_architecture={architecture:?} expected_architecture=[] expected_architecture_source=canonical-index-omission-platform-bound-by-config actual_variant={variant:?} expected_variant=[] actual_os_version={os_version:?} expected_os_version=[] actual_os_features={os_features:?} expected_os_features=[] actual_features={features:?} expected_features=[]",
    );
    let annotations = annotation_pairs(index);
    let summaries = annotations.iter().map(|(key, value)| {
        format!("{key}:len={}:sha256={}", value.len(), preauth::sha256_hex(value.as_bytes()))
    }).collect::<Vec<_>>();
    let expected_name = format!("docker.io/library/rar-preauth:{revision}");
    let expected_summaries = [
        format!("io.containerd.image.name:len={}:sha256={}", expected_name.len(), preauth::sha256_hex(expected_name.as_bytes())),
        format!("org.opencontainers.image.ref.name:len={}:sha256={}", revision.len(), preauth::sha256_hex(revision.as_bytes())),
    ];
    eprintln!(
        "oci_index_annotations count={} keys_and_value_hashes={summaries:?} expected_keys_and_value_hashes={expected_summaries:?} expected_policy_source=adr-0018-and-validated-repositories-revision",
        annotations.len(),
    );
}

fn typed_list_digest(kind: &str, values: &[String]) -> String {
    let mut canonical = String::new();
    for (index, value) in values.iter().enumerate() {
        canonical.push_str(&format!("{kind}[{index}]=sha256:{value}\n"));
    }
    preauth::sha256_hex(canonical.as_bytes())
}

fn one_marker(text: &str, marker: &str) {
    if text.match_indices(marker).count() != 1 { fail("duplicate or missing JSON field"); }
}

fn repositories_binding(entries: &BTreeMap<String, TarEntry>, repo_tag: &str, layer: &str) -> String {
    let entry = entries.get("repositories").unwrap_or_else(|| fail("Docker repositories index absent"));
    if !matches!(entry.kind, 0 | b'0') || entry.mode != 0o644 || entry.uid != 0 || entry.gid != 0
        || entry.size == 0 || entry.size > MAX_REPOSITORIES || entry.data.len() as u64 != entry.size
    { fail("invalid Docker repositories member"); }
    let revision = repo_tag.strip_prefix("rar-preauth:").unwrap_or_else(|| fail("invalid Docker image tag"));
    if revision.len() != 40
        || !revision.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    { fail("invalid Docker image revision tag"); }
    let layer_identity = blob_digest(layer).or_else(|| layer.strip_suffix("/layer.tar"))
        .unwrap_or_else(|| fail("invalid Docker top-layer identity"));
    if layer_identity.len() != 64
        || !layer_identity.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    { fail("invalid Docker top-layer identity"); }
    let canonical = format!("{{\"rar-preauth\":{{\"{revision}\":\"{layer_identity}\"}}}}\n");
    if entry.data != canonical.as_bytes() { fail("Docker repositories binding mismatch"); }
    revision.to_owned()
}

fn metadata_digest(path: &Path, field: &str) -> String {
    let text = fs::read_to_string(path).unwrap_or_else(|_| fail("unreadable OCI metadata"));
    let compact: String = text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let value = quoted(&compact, &format!("\"{field}\":\"sha256:"));
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) { fail("invalid OCI metadata digest"); }
    value
}

fn metadata_summary(path: &Path) -> (String, String, Option<(String, String, String, String)>) {
    let text = fs::read_to_string(path).unwrap_or_else(|_| fail("unreadable OCI metadata"));
    if text.len() as u64 > MAX_INLINE { fail("oversized OCI metadata"); }
    let compact: String = text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let known_keys = [
        "buildx.build.provenance", "buildx.build.ref", "containerimage.config.digest",
        "containerimage.descriptor", "containerimage.digest", "image.name",
    ];
    let present = known_keys.into_iter().filter(|key| compact.contains(&format!("\"{key}\":")))
        .collect::<Vec<_>>();
    eprintln!("oci_buildx_metadata keys={}", present.join(","));
    let config = metadata_digest(path, "containerimage.config.digest");
    let digest = metadata_digest(path, "containerimage.digest");
    for value in [&config, &digest] {
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
            fail("invalid OCI metadata descriptor digest");
        }
    }
    let descriptor = compact.split_once("\"containerimage.descriptor\":{").map(|(_, value)| {
        let descriptor_digest = quoted(value, "\"digest\":\"sha256:");
        let media_type = quoted(value, "\"mediaType\":\"");
        let platform = after(value, "\"platform\":{");
        let architecture = quoted(platform, "\"architecture\":\"");
        let os = quoted(platform, "\"os\":\"");
        if descriptor_digest.len() != 64
            || !descriptor_digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        { fail("invalid OCI metadata descriptor digest"); }
        (descriptor_digest, media_type, architecture, os)
    });
    (config, digest, descriptor)
}

fn blob_digest(path: &str) -> Option<&str> {
    let digest = path.strip_prefix("blobs/sha256/")?;
    if digest.len() == 64
        && digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    { Some(digest) } else { None }
}

fn descriptor_digests(json: &str) -> Vec<String> {
    let compact: String = json.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let marker = "\"digest\":\"sha256:";
    let mut rest = compact.as_str();
    let mut digests = Vec::new();
    while let Some((_, after_marker)) = rest.split_once(marker) {
        if after_marker.len() < 65 || after_marker.as_bytes()[64] != b'\"' { fail("malformed OCI descriptor digest"); }
        let digest = &after_marker[..64];
        if !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) { fail("invalid OCI descriptor digest"); }
        digests.push(digest.to_owned());
        rest = &after_marker[65..];
    }
    digests
}

fn record_inbound(
    inbound: &mut BTreeMap<String, BTreeSet<String>>, digest: &str, source: &str,
) {
    inbound.entry(format!("blobs/sha256/{digest}")).or_default().insert(source.to_owned());
}

fn report_unreachable(
    entries: &BTreeMap<String, TarEntry>, names: &[String], inbound: &BTreeMap<String, BTreeSet<String>>,
    config_name: &str, layers: &[String], image_manifests: &BTreeSet<String>,
) {
    eprintln!(
        "oci_unreachable_summary count={} reported={} cap={}",
        names.len(), names.len().min(MAX_DIAGNOSTIC_MEMBERS), MAX_DIAGNOSTIC_MEMBERS,
    );
    for name in names.iter().take(MAX_DIAGNOSTIC_MEMBERS) {
        let entry = entries.get(name).unwrap_or_else(|| fail("unreachable diagnostic member absent"));
        let digest = blob_digest(name);
        let class = if name == config_name {
            "docker-config"
        } else if layers.iter().any(|layer| layer == name) {
            "docker-layer"
        } else if digest.is_some_and(|value| image_manifests.contains(value)) {
            "oci-image-manifest"
        } else if digest.is_some() && entry.data.is_empty() {
            "unreferenced-payload-blob"
        } else if digest.is_some() {
            "unreferenced-inline-blob"
        } else {
            "unrecognized-root-member"
        };
        let references = inbound.get(name).map_or_else(
            || "none".to_owned(),
            |values| {
                let mut refs = values.iter().take(MAX_DIAGNOSTIC_REFS).cloned().collect::<Vec<_>>();
                if values.len() > MAX_DIAGNOSTIC_REFS { refs.push("capped".to_owned()); }
                refs.join(",")
            },
        );
        eprintln!(
            "oci_unreachable path={} type=regular size={} mode={:04o} uid={} gid={} sha256={} class={} inbound={}",
            name, entry.size, entry.mode, entry.uid, entry.gid, entry.sha256, class, references,
        );
    }
}

fn verify_oci_layout(
    entries: &BTreeMap<String, TarEntry>, config_name: &str, layers: &[String], config_digest: &str,
    revision: &str, project_export: bool,
) -> (String, String, BTreeSet<String>) {
    let config_path_digest = blob_digest(config_name).unwrap_or_else(|| fail("invalid OCI config path"));
    if config_path_digest != config_digest { fail("OCI config name/digest mismatch"); }
    let layout = entries.get("oci-layout").unwrap_or_else(|| fail("OCI layout absent"));
    let layout_text = std::str::from_utf8(&layout.data).unwrap_or_else(|_| fail("OCI layout is not UTF-8"));
    let compact_layout: String = layout_text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    if compact_layout != "{\"imageLayoutVersion\":\"1.0.0\"}" { fail("unsupported OCI layout"); }
    let index = entries.get("index.json").unwrap_or_else(|| fail("OCI index absent"));
    let index_text = std::str::from_utf8(&index.data).unwrap_or_else(|_| fail("OCI index is not UTF-8"));
    for (name, entry) in entries {
        if let Some(digest) = blob_digest(name) {
            if digest != entry.sha256 { fail("OCI blob name/digest mismatch"); }
        } else if !matches!(name.as_str(), "index.json" | "manifest.json" | "oci-layout" | "repositories") {
            unexpected_member(name, entry);
        }
    }
    let mut inbound = BTreeMap::<String, BTreeSet<String>>::new();
    let mut pending = descriptor_digests(index_text);
    if pending.is_empty() || pending.len() > 64 { fail("invalid OCI index descriptors"); }
    for digest in &pending { record_inbound(&mut inbound, digest, "index.json"); }
    if let Some(digest) = blob_digest(config_name) { record_inbound(&mut inbound, digest, "manifest.json:config"); }
    for (index, layer) in layers.iter().enumerate() {
        if let Some(digest) = blob_digest(layer) {
            record_inbound(&mut inbound, digest, &format!("manifest.json:layer[{index}]"));
        }
    }
    if let Some(layer) = layers.last().and_then(|value| blob_digest(value)) {
        record_inbound(&mut inbound, layer, "repositories");
    }
    let mut reachable = BTreeSet::new();
    let mut image_manifests = BTreeSet::new();
    while let Some(digest) = pending.pop() {
        if !reachable.insert(digest.clone()) { continue; }
        if reachable.len() > MAX_MEMBERS { fail("OCI graph bound exceeded"); }
        let path = format!("blobs/sha256/{digest}");
        let entry = entries.get(&path).unwrap_or_else(|| fail("OCI descriptor target absent"));
        if !entry.data.is_empty() {
            let text = std::str::from_utf8(&entry.data).unwrap_or_else(|_| fail("OCI descriptor is not UTF-8"));
            let compact: String = text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
            if compact.contains("\"config\":{") && compact.contains("\"layers\":[") { image_manifests.insert(digest.clone()); }
            let children = descriptor_digests(text);
            for child in &children { record_inbound(&mut inbound, child, &path); }
            pending.extend(children);
        }
    }
    if !reachable.contains(config_path_digest) { fail("OCI config is not index-reachable"); }
    for layer in layers {
        let digest = blob_digest(layer).unwrap_or_else(|| fail("invalid OCI layer path"));
        if !reachable.contains(digest) { fail("OCI layer is not index-reachable"); }
    }
    let expected = BTreeSet::from_iter(
        ["index.json".to_owned(), "manifest.json".to_owned(), "oci-layout".to_owned(), "repositories".to_owned()].into_iter()
            .chain(reachable.iter().map(|digest| format!("blobs/sha256/{digest}"))),
    );
    let unreachable = entries.keys().filter(|name| !expected.contains(*name)).cloned().collect::<Vec<_>>();
    if !unreachable.is_empty() {
        report_unreachable(entries, &unreachable, &inbound, config_name, layers, &image_manifests);
        if project_export {
            eprintln!("oci_projection_omitted count={}", unreachable.len());
        } else {
            fail("unreachable OCI archive member");
        }
    }
    if image_manifests.len() != 1 { fail("OCI image manifest is not unique"); }
    let selected_digest = image_manifests.into_iter().next().unwrap();
    let selected_path = format!("blobs/sha256/{selected_digest}");
    let selected = entries.get(&selected_path).unwrap_or_else(|| fail("selected OCI manifest absent"));
    let selected_text = std::str::from_utf8(&selected.data).unwrap_or_else(|_| fail("selected OCI manifest is not UTF-8"));
    let selected_compact: String = selected_text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let expected_digests = std::iter::once(config_digest.to_owned())
        .chain(layers.iter().map(|layer| blob_digest(layer).unwrap().to_owned()))
        .collect::<Vec<_>>();
    if descriptor_digests(&selected_compact) != expected_digests { fail("OCI manifest descriptor mapping mismatch"); }
    let media_types = quoted_values(&selected_compact, "\"mediaType\":\"");
    if media_types.len() != layers.len() + 2
        || media_types[0] != "application/vnd.oci.image.manifest.v1+json"
        || media_types[1] != "application/vnd.oci.image.config.v1+json"
        || !media_types[2..].iter().all(|value| value == "application/vnd.oci.image.layer.v1.tar")
    { fail("OCI manifest media-type mapping mismatch"); }
    let expected_sizes = std::iter::once(entries.get(config_name).unwrap().size)
        .chain(layers.iter().map(|layer| entries.get(layer).unwrap().size))
        .collect::<Vec<_>>();
    if numeric_values(&selected_compact, "\"size\":") != expected_sizes {
        fail("OCI manifest size mapping mismatch");
    }
    let index_compact: String = index_text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let canonical_index = format!(concat!(
        "{{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.index.v1+json\",",
        "\"manifests\":[{{\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",",
        "\"digest\":\"sha256:{}\",\"size\":{},\"annotations\":{{",
        "\"io.containerd.image.name\":\"docker.io/library/rar-preauth:{}\",",
        "\"org.opencontainers.image.ref.name\":\"{}\"}}}}]}}",
    ), selected_digest, selected.size, revision, revision);
    if index.data != canonical_index.as_bytes() {
        report_index_mapping(&index_compact, &selected_digest, selected.size, revision);
        fail("OCI index descriptor mapping mismatch");
    }
    (selected_digest, index.sha256.clone(), expected)
}

fn verify_one(
    entries: &BTreeMap<String, TarEntry>, metadata: &Path, image_id_path: &Path, project_export: bool,
) -> (String, String, String, String, String, BTreeSet<String>) {
    let manifest = &entries.get("manifest.json").unwrap_or_else(|| fail("Docker manifest absent")).data;
    if manifest.is_empty() { fail("oversized Docker manifest"); }
    let manifest = std::str::from_utf8(manifest).unwrap_or_else(|_| fail("manifest is not UTF-8"));
    let compact_manifest: String = manifest.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    one_marker(&compact_manifest, "\"Config\":\"");
    one_marker(&compact_manifest, "\"RepoTags\":[");
    one_marker(&compact_manifest, "\"Layers\":[");
    let config_name = quoted(manifest, "\"Config\":\"");
    let modern_oci = blob_digest(&config_name).is_some();
    if !safe_name(&config_name) || (!modern_oci && !config_name.ends_with(".json")) { fail("invalid config path"); }
    let layers = quoted_list(manifest, "\"Layers\":[");
    if layers.is_empty() || layers.len() > 64 { fail("invalid layer count"); }
    let repo_tags = quoted_list(manifest, "\"RepoTags\":[");
    if repo_tags.len() != 1 { fail("invalid repository tag count"); }
    let revision = repositories_binding(entries, &repo_tags[0], layers.last().unwrap());
    let config_entry = entries.get(&config_name).unwrap_or_else(|| fail("config absent"));
    if config_entry.data.is_empty() { fail("oversized image config"); }
    let config = &config_entry.data;
    let config_digest = config_entry.sha256.clone();
    if !modern_oci && config_name != format!("{config_digest}.json") { fail("config name/digest mismatch"); }
    let config_text = std::str::from_utf8(config).unwrap_or_else(|_| fail("config is not UTF-8"));
    let diff_ids = quoted_list(config_text, "\"diff_ids\":[");
    if diff_ids.len() != layers.len() { fail("layer/diff-id count mismatch"); }
    let mut expected = BTreeSet::from(["manifest.json".to_owned(), "repositories".to_owned(), config_name.clone()]);
    for (layer, diff_id) in layers.iter().zip(&diff_ids) {
        if !safe_name(layer) || !diff_id.starts_with("sha256:") { fail("invalid layer identity"); }
        let entry = entries.get(layer).unwrap_or_else(|| fail("layer absent"));
        if diff_id != &format!("sha256:{}", entry.sha256) { fail("layer diff-id mismatch"); }
        expected.insert(layer.clone());
        if modern_oci {
            let digest = blob_digest(layer).unwrap_or_else(|| fail("invalid OCI layer path"));
            if digest != entry.sha256 { fail("OCI layer name/digest mismatch"); }
        } else {
            let directory = layer.strip_suffix("/layer.tar").unwrap_or_else(|| fail("noncanonical layer path"));
            if directory.len() != 64 || !directory.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) { fail("invalid layer directory"); }
            expected.insert(format!("{directory}/VERSION"));
            expected.insert(format!("{directory}/json"));
        }
    }
    let (reported_digest, index_digest, graph) = if modern_oci {
        verify_oci_layout(&entries, &config_name, &layers, &config_digest, &revision, project_export)
    } else {
        if project_export { fail("legacy Docker archive projection unsupported"); }
        if entries.keys().any(|name| !expected.contains(name)) { fail("unexpected archive member"); }
        (config_digest.clone(), String::new(), expected)
    };
    let (metadata_config, metadata_digest, descriptor) = metadata_summary(metadata);
    let selected_path = format!("blobs/sha256/{reported_digest}");
    let selected = entries.get(&selected_path).unwrap_or_else(|| fail("selected OCI manifest absent"));
    let selected_text = std::str::from_utf8(&selected.data).unwrap_or_else(|_| fail("selected OCI manifest is not UTF-8"));
    let selected_compact: String = selected_text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let selected_media_type = quoted(&selected_compact, "\"mediaType\":\"");
    if let Some((descriptor_digest, descriptor_media_type, architecture, os)) = &descriptor {
        eprintln!(
            "oci_buildx_descriptor digest=sha256:{} descriptor_digest=sha256:{} config=sha256:{} media_type={} platform={}/{} descriptor_object=present",
            metadata_digest, descriptor_digest, metadata_config, descriptor_media_type, os, architecture,
        );
    } else {
        eprintln!(
            "oci_buildx_descriptor digest=sha256:{} config=sha256:{} descriptor_object=absent",
            metadata_digest, metadata_config,
        );
    }
    eprintln!(
        "oci_selected_manifest digest=sha256:{} size={} media_type={} canonical_bytes_sha256={}",
        reported_digest, selected.size, selected_media_type, selected.sha256,
    );
    if metadata_config != config_digest
        || descriptor.is_some()
        || metadata_digest != config_digest { fail("reported image digest mismatch"); }
    let image_id = fs::read_to_string(image_id_path).unwrap_or_else(|_| fail("loaded image identity absent"));
    if image_id.trim() != format!("sha256:{config_digest}") { fail("loaded image identity mismatch"); }
    let layer_digests = layers.iter().map(|layer| blob_digest(layer).unwrap().to_owned()).collect::<Vec<_>>();
    let diff_digests = diff_ids.iter().map(|value| value.strip_prefix("sha256:").unwrap().to_owned()).collect::<Vec<_>>();
    (
        config_digest, reported_digest, index_digest, typed_list_digest("layer_descriptor", &layer_digests),
        typed_list_digest("rootfs_diff_id", &diff_digests), graph,
    )
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 6 && args[1] == "--member-list" {
        let archive = Path::new(&args[2]);
        let entries = tar_entries(archive, false);
        print_inventory(archive, &entries);
        let (_, _, _, _, _, graph) = verify_one(&entries, Path::new(&args[3]), Path::new(&args[4]), true);
        if args[5] != "-" { fail("member-list output must be stdout"); }
        for name in graph { println!("{name}"); }
        return;
    }
    if args.len() != 7 { fail("usage: preauth-verify-oci archive1 metadata1 image-id1 archive2 metadata2 image-id2"); }
    let paths: Vec<_> = args[1..].iter().map(Path::new).collect();
    let entries_one = tar_entries(paths[0], true);
    let entries_two = tar_entries(paths[3], true);
    print_inventory(paths[0], &entries_one);
    print_inventory(paths[3], &entries_two);
    let (config_one, manifest_one, index_one, layers_one, diff_ids_one, _) = verify_one(&entries_one, paths[1], paths[2], false);
    let (config_two, manifest_two, index_two, layers_two, diff_ids_two, _) = verify_one(&entries_two, paths[4], paths[5], false);
    if (config_one.clone(), manifest_one.clone(), index_one.clone(), layers_one.clone(), diff_ids_one.clone())
        != (config_two, manifest_two, index_two, layers_two, diff_ids_two)
        || digest_file(paths[0]) != digest_file(paths[3]) { fail("independent OCI builds differ"); }
    println!("derived_oci_archive_sha256={}", digest_file(paths[0]));
    println!("buildx_descriptor_kind=docker-config-id");
    println!("buildx_descriptor_sha256={config_one}");
    println!("docker_config_sha256={config_one}");
    println!("selected_oci_manifest_sha256={manifest_one}");
    println!("canonical_oci_index_sha256={index_one}");
    println!("layer_descriptor_set_sha256={layers_one}");
    println!("rootfs_diff_id_set_sha256={diff_ids_one}");
    println!("loaded_image_config_sha256={config_one}");
}
