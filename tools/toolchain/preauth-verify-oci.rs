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

fn json(input: &[u8]) -> preauth::Json {
    preauth::Json::parse(input).unwrap_or_else(|error| fail(error.code))
}

fn exact_keys(document: &str, path: &str, value: &preauth::Json, required: &[&str], optional: &[&str]) {
    match value.key_set_diagnostic(document, path, required, optional)
        .unwrap_or_else(|error| fail(error.code))
    {
        None => {}
        Some(diagnostic) => { eprintln!("{diagnostic}"); fail("json-key-set"); }
    }
}

fn validate_layer_sources(
    manifest: &preauth::Json, diff_ids: &[String], layers: &[String],
    entries: &BTreeMap<String, TarEntry>, config_digest: &str,
) {
    let sources = manifest.get("LayerSources").unwrap_or_else(|error| fail(error.code));
    let sources = sources.object().unwrap_or_else(|error| fail(error.code));
    if sources.len() != diff_ids.len() || sources.len() != layers.len() {
        fail("Docker LayerSources count mismatch");
    }
    let authoritative_layers = layers.iter().map(|layer| {
        blob_digest(layer).unwrap_or_else(|| fail("invalid OCI layer path"))
    }).collect::<BTreeSet<_>>();
    let mut source_digests = BTreeSet::new();
    for diff_id in diff_ids {
        if !diff_id.starts_with("sha256:") || diff_id.len() != 71 {
            fail("invalid Docker LayerSources key");
        }
        let descriptor = sources.get(diff_id).unwrap_or_else(|| fail("Docker LayerSources key mismatch"));
        exact_keys(
            "docker-manifest", "/0/LayerSources/*", descriptor,
            &["digest", "mediaType", "size"], &[],
        );
        let digest = sha_digest(descriptor.get("digest").unwrap());
        let media_type = descriptor.get("mediaType").and_then(preauth::Json::string)
            .unwrap_or_else(|error| fail(error.code));
        if media_type != "application/vnd.oci.image.layer.v1.tar+gzip" {
            fail("Docker LayerSources media type mismatch");
        }
        let size = descriptor.get("size").and_then(preauth::Json::number)
            .unwrap_or_else(|error| fail(error.code));
        if size == 0 || size > MAX_MEMBER { fail("Docker LayerSources size bound exceeded"); }
        if digest == config_digest || authoritative_layers.contains(digest.as_str())
            || !source_digests.insert(digest.clone())
        {
            fail("Docker LayerSources identity collision");
        }
        let source_path = format!("blobs/sha256/{digest}");
        if let Some(source) = entries.get(&source_path) {
            if source.sha256 != digest || source.size != size {
                fail("Docker LayerSources payload mismatch");
            }
        }
    }
}

fn string_array(value: &preauth::Json) -> Vec<String> {
    value.array().unwrap_or_else(|error| fail(error.code)).iter().map(|item|
        item.string().unwrap_or_else(|error| fail(error.code)).to_owned()).collect()
}

fn sha_digest(value: &preauth::Json) -> String {
    let value = value.string().unwrap_or_else(|error| fail(error.code));
    let digest = value.strip_prefix("sha256:").unwrap_or_else(|| fail("digest algorithm mismatch"));
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        fail("invalid JSON digest");
    }
    digest.to_owned()
}

fn annotation_pairs(index: &preauth::Json) -> Vec<(String, String)> {
    let descriptor = &index.get("manifests").unwrap_or_else(|error| fail(error.code))
        .array().unwrap_or_else(|error| fail(error.code))[0];
    let Some(annotations) = descriptor.object().unwrap().get("annotations") else { return Vec::new() };
    annotations.object().unwrap_or_else(|error| fail(error.code)).iter().map(|(key, value)| {
        let value = value.string().unwrap_or_else(|error| fail(error.code));
        if key.len() > 128 || value.len() > 512 { fail("OCI index annotation bound exceeded"); }
        (key.clone(), value.to_owned())
    }).collect()
}

fn report_index_mapping(index: &str, expected_digest: &str, expected_size: u64, revision: &str) {
    let parsed = json(index.as_bytes());
    let schema = vec![parsed.get("schemaVersion").and_then(preauth::Json::number).unwrap_or(0)];
    let media = parsed.get("mediaType").and_then(preauth::Json::string).map_or_else(|_| Vec::new(), |v| vec![v.to_owned()]);
    let descriptors = parsed.get("manifests").and_then(preauth::Json::array).unwrap_or(&[]);
    let digests = descriptors.iter().map(|value| sha_digest(value.get("digest").unwrap())).collect::<Vec<_>>();
    let sizes = descriptors.iter().map(|value| value.get("size").and_then(preauth::Json::number).unwrap_or(0)).collect::<Vec<_>>();
    let (architecture, os, variant, os_version, os_features, features) = (Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new());
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
    let annotations = annotation_pairs(&parsed);
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
    let parsed = json(&entry.data);
    exact_keys("docker-repositories", "/", &parsed, &["rar-preauth"], &[]);
    exact_keys("docker-repositories", "/rar-preauth", parsed.get("rar-preauth").unwrap(), &[revision], &[]);
    if entry.data != canonical.as_bytes() { fail("Docker repositories binding mismatch"); }
    revision.to_owned()
}

fn metadata_digest(path: &Path, field: &str) -> String {
    let bytes = fs::read(path).unwrap_or_else(|_| fail("unreadable OCI metadata"));
    let parsed = json(&bytes);
    sha_digest(parsed.get(field).unwrap_or_else(|error| fail(error.code)))
}

fn metadata_summary(path: &Path) -> (String, String, Option<(String, String, String, String)>) {
    let bytes = fs::read(path).unwrap_or_else(|_| fail("unreadable OCI metadata"));
    if bytes.len() as u64 > MAX_INLINE { fail("oversized OCI metadata"); }
    let parsed = json(&bytes);
    let object = parsed.object().unwrap_or_else(|error| fail(error.code));
    let known_keys = [
        "buildx.build.provenance", "buildx.build.ref", "containerimage.config.digest",
        "containerimage.descriptor", "containerimage.digest", "image.name",
    ];
    exact_keys(
        "buildx-metadata", "/", &parsed,
        &["containerimage.config.digest", "containerimage.digest"],
        &["buildx.build.provenance", "buildx.build.ref", "containerimage.descriptor", "image.name"],
    );
    let present = known_keys.into_iter().filter(|key| object.contains_key(*key))
        .collect::<Vec<_>>();
    eprintln!("oci_buildx_metadata keys={}", present.join(","));
    let config = metadata_digest(path, "containerimage.config.digest");
    let digest = metadata_digest(path, "containerimage.digest");
    for value in [&config, &digest] {
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
            fail("invalid OCI metadata descriptor digest");
        }
    }
    let descriptor = object.get("containerimage.descriptor").map(|value| {
        exact_keys("buildx-metadata", "/containerimage.descriptor", value, &["digest", "mediaType", "platform"], &[]);
        let platform = value.get("platform").unwrap();
        exact_keys("buildx-metadata", "/containerimage.descriptor/platform", platform, &["architecture", "os"], &[]);
        (sha_digest(value.get("digest").unwrap()), value.get("mediaType").and_then(preauth::Json::string).unwrap().to_owned(),
         platform.get("architecture").and_then(preauth::Json::string).unwrap().to_owned(),
         platform.get("os").and_then(preauth::Json::string).unwrap().to_owned())
    });
    (config, digest, descriptor)
}

fn blob_digest(path: &str) -> Option<&str> {
    let digest = path.strip_prefix("blobs/sha256/")?;
    if digest.len() == 64
        && digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    { Some(digest) } else { None }
}

fn descriptor_digests(value: &preauth::Json) -> Vec<String> {
    fn walk(value: &preauth::Json, output: &mut Vec<String>) {
        match value {
            preauth::Json::Object(object) => {
                if let Some(value) = object.get("digest") { output.push(sha_digest(value)); }
                for (key, child) in object { if key != "digest" { walk(child, output); } }
            }
            preauth::Json::Array(values) => for value in values { walk(value, output); },
            _ => {}
        }
    }
    let mut output = Vec::new(); walk(value, &mut output); output
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
    let layout_json = json(&layout.data);
    exact_keys("oci-layout", "/", &layout_json, &["imageLayoutVersion"], &[]);
    if layout_json.get("imageLayoutVersion").and_then(preauth::Json::string).unwrap() != "1.0.0" { fail("unsupported OCI layout"); }
    let index = entries.get("index.json").unwrap_or_else(|| fail("OCI index absent"));
    let index_json = json(&index.data);
    for (name, entry) in entries {
        if let Some(digest) = blob_digest(name) {
            if digest != entry.sha256 { fail("OCI blob name/digest mismatch"); }
        } else if !matches!(name.as_str(), "index.json" | "manifest.json" | "oci-layout" | "repositories") {
            unexpected_member(name, entry);
        }
    }
    let mut inbound = BTreeMap::<String, BTreeSet<String>>::new();
    let mut pending = descriptor_digests(&index_json);
    let leaf_digests = std::iter::once(config_path_digest.to_owned())
        .chain(layers.iter().map(|layer| blob_digest(layer).unwrap_or_else(|| fail("invalid OCI layer path")).to_owned()))
        .collect::<BTreeSet<_>>();
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
        // Config and layer payloads are terminal typed nodes. They are validated by their
        // dedicated schemas below and must never be reinterpreted as descriptor JSON merely
        // because a small fixture payload fits within the inline archive bound.
        if !leaf_digests.contains(&digest) && !entry.data.is_empty() {
            let parsed = json(&entry.data);
            let object = parsed.object().unwrap_or_else(|error| fail(error.code));
            if object.contains_key("config") && object.contains_key("layers") { image_manifests.insert(digest.clone()); }
            let children = descriptor_digests(&parsed);
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
    let selected_json = json(&selected.data);
    exact_keys("oci-manifest", "/", &selected_json, &["schemaVersion", "mediaType", "config", "layers"], &[]);
    let config_descriptor = selected_json.get("config").unwrap();
    exact_keys("oci-manifest", "/config", config_descriptor, &["mediaType", "digest", "size"], &[]);
    let layer_descriptors = selected_json.get("layers").and_then(preauth::Json::array).unwrap();
    for (index, descriptor) in layer_descriptors.iter().enumerate() {
        exact_keys("oci-manifest", &format!("/layers/{index}"), descriptor, &["mediaType", "digest", "size"], &[]);
    }
    let expected_digests = std::iter::once(config_digest.to_owned())
        .chain(layers.iter().map(|layer| blob_digest(layer).unwrap().to_owned()))
        .collect::<Vec<_>>();
    let actual_digests = std::iter::once(sha_digest(config_descriptor.get("digest").unwrap()))
        .chain(layer_descriptors.iter().map(|value| sha_digest(value.get("digest").unwrap()))).collect::<Vec<_>>();
    if actual_digests != expected_digests { fail("OCI manifest descriptor mapping mismatch"); }
    let media_types = std::iter::once(selected_json.get("mediaType").and_then(preauth::Json::string).unwrap().to_owned())
        .chain(std::iter::once(config_descriptor.get("mediaType").and_then(preauth::Json::string).unwrap().to_owned()))
        .chain(layer_descriptors.iter().map(|value| value.get("mediaType").and_then(preauth::Json::string).unwrap().to_owned())).collect::<Vec<_>>();
    if media_types.len() != layers.len() + 2
        || media_types[0] != "application/vnd.oci.image.manifest.v1+json"
        || media_types[1] != "application/vnd.oci.image.config.v1+json"
        || !media_types[2..].iter().all(|value| value == "application/vnd.oci.image.layer.v1.tar")
    { fail("OCI manifest media-type mapping mismatch"); }
    let expected_sizes = std::iter::once(entries.get(config_name).unwrap().size)
        .chain(layers.iter().map(|layer| entries.get(layer).unwrap().size))
        .collect::<Vec<_>>();
    let actual_sizes = std::iter::once(config_descriptor.get("size").and_then(preauth::Json::number).unwrap())
        .chain(layer_descriptors.iter().map(|value| value.get("size").and_then(preauth::Json::number).unwrap())).collect::<Vec<_>>();
    if actual_sizes != expected_sizes {
        fail("OCI manifest size mapping mismatch");
    }
    let canonical_index_source = format!(concat!(
        "{{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.index.v1+json\",",
        "\"manifests\":[{{\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",",
        "\"digest\":\"sha256:{}\",\"size\":{},\"annotations\":{{",
        "\"io.containerd.image.name\":\"docker.io/library/rar-preauth:{}\",",
        "\"org.opencontainers.image.ref.name\":\"{}\"}}}}]}}",
    ), selected_digest, selected.size, revision, revision);
    let canonical_index = json(canonical_index_source.as_bytes()).canonical();
    if index.data != canonical_index.as_bytes() {
        report_index_mapping(std::str::from_utf8(&index.data).unwrap_or(""), &selected_digest, selected.size, revision);
        fail("OCI index descriptor mapping mismatch");
    }
    (selected_digest, index.sha256.clone(), expected)
}

fn verify_one(
    entries: &BTreeMap<String, TarEntry>, metadata: &Path, image_id_path: &Path, project_export: bool,
) -> (String, String, String, String, String, BTreeSet<String>) {
    let manifest = &entries.get("manifest.json").unwrap_or_else(|| fail("Docker manifest absent")).data;
    if manifest.is_empty() { fail("oversized Docker manifest"); }
    let manifest_json = json(manifest);
    let manifest_items = manifest_json.array().unwrap_or_else(|error| fail(error.code));
    if manifest_items.len() != 1 { fail("invalid Docker manifest count"); }
    let manifest = &manifest_items[0];
    exact_keys(
        "docker-manifest", "/0", manifest,
        &["Config", "RepoTags", "Layers", "LayerSources"], &[],
    );
    let config_name = manifest.get("Config").and_then(preauth::Json::string).unwrap_or_else(|error| fail(error.code)).to_owned();
    let modern_oci = blob_digest(&config_name).is_some();
    if !safe_name(&config_name) || (!modern_oci && !config_name.ends_with(".json")) { fail("invalid config path"); }
    let layers = string_array(manifest.get("Layers").unwrap());
    if layers.is_empty() || layers.len() > 64 { fail("invalid layer count"); }
    let repo_tags = string_array(manifest.get("RepoTags").unwrap());
    if repo_tags.len() != 1 { fail("invalid repository tag count"); }
    let revision = repositories_binding(entries, &repo_tags[0], layers.last().unwrap());
    let config_entry = entries.get(&config_name).unwrap_or_else(|| fail("config absent"));
    if config_entry.data.is_empty() { fail("oversized image config"); }
    let config = &config_entry.data;
    let config_digest = config_entry.sha256.clone();
    if !modern_oci && config_name != format!("{config_digest}.json") { fail("config name/digest mismatch"); }
    let config_json = json(config);
    exact_keys("docker-config", "/", &config_json, &["rootfs"], &["architecture", "os", "config", "created", "history"]);
    let rootfs = config_json.get("rootfs").unwrap();
    exact_keys("docker-config", "/rootfs", rootfs, &["type", "diff_ids"], &[]);
    if rootfs.get("type").and_then(preauth::Json::string).unwrap() != "layers" { fail("invalid rootfs type"); }
    let diff_ids = string_array(rootfs.get("diff_ids").unwrap());
    if diff_ids.len() != layers.len() { fail("layer/diff-id count mismatch"); }
    validate_layer_sources(manifest, &diff_ids, &layers, entries, &config_digest);
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
    let selected_json = json(&selected.data);
    let selected_media_type = selected_json.get("mediaType").and_then(preauth::Json::string).unwrap();
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
    if args.len() == 3 && args[1] == "--canonicalize-json" {
        if !matches!(args[2].as_str(), "bare" | "line") { fail("invalid canonical JSON mode"); }
        let mut input = Vec::new();
        std::io::stdin().take(MAX_INLINE + 1).read_to_end(&mut input).unwrap_or_else(|_| fail("unreadable JSON fixture"));
        if input.len() as u64 > MAX_INLINE { fail("oversized JSON fixture"); }
        let parsed = preauth::Json::parse(&input).unwrap_or_else(|error| fail(error.code));
        print!("{}", parsed.canonical());
        if args[2] == "line" { println!(); }
        return;
    }
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
