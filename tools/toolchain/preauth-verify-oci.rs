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

struct TarEntry { data: Vec<u8>, sha256: String }

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

fn tar_entries(path: &Path) -> BTreeMap<String, TarEntry> {
    let mut file = File::open(path).unwrap_or_else(|_| fail("unreadable OCI archive"));
    let mut entries = BTreeMap::new();
    let mut header = [0_u8; 512];
    let mut offset = 0_u64;
    let mut total = 0_u64;
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
        let directory = header[156] == b'5';
        if !matches!(header[156], 0 | b'0' | b'5') { fail("unsupported tar member type"); }
        let size = parse_octal(&header[124..136]);
        if directory && size != 0 { fail("nonempty tar directory"); }
        total = total.checked_add(size).unwrap_or_else(|| fail("tar total overflow"));
        if size > MAX_MEMBER || total > MAX_ARCHIVE || entries.len() >= MAX_MEMBERS { fail("tar bound exceeded"); }
        if !directory {
            let entry = if size <= MAX_INLINE {
                let size_usize = usize::try_from(size).unwrap_or_else(|_| fail("tar size overflow"));
                let mut data = vec![0_u8; size_usize];
                file.read_exact(&mut data).unwrap_or_else(|_| fail("truncated tar member"));
                TarEntry { sha256: preauth::sha256_hex(&data), data }
            } else {
                let mut bounded = (&mut file).take(size);
                let sha256 = preauth::sha256_reader(&mut bounded).unwrap_or_else(|_| fail("truncated tar member"));
                if bounded.limit() != 0 { fail("truncated tar member"); }
                TarEntry { data: Vec::new(), sha256 }
            };
            if entries.insert(name.to_owned(), entry).is_some() { fail("duplicate tar member"); }
        }
        let padding = (512 - (size % 512)) % 512;
        file.seek(SeekFrom::Current(i64::try_from(padding).unwrap())).unwrap_or_else(|_| fail("truncated tar padding"));
        offset = offset.checked_add(size + padding).unwrap_or_else(|| fail("tar offset overflow"));
    }
    entries
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

fn metadata_digest(path: &Path, field: &str) -> String {
    let text = fs::read_to_string(path).unwrap_or_else(|_| fail("unreadable OCI metadata"));
    let compact: String = text.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let value = quoted(&compact, &format!("\"{field}\":\"sha256:"));
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) { fail("invalid OCI metadata digest"); }
    value
}

fn verify_one(archive: &Path, metadata: &Path, image_id_path: &Path) -> String {
    let entries = tar_entries(archive);
    let manifest = &entries.get("manifest.json").unwrap_or_else(|| fail("Docker manifest absent")).data;
    if manifest.is_empty() { fail("oversized Docker manifest"); }
    let manifest = std::str::from_utf8(manifest).unwrap_or_else(|_| fail("manifest is not UTF-8"));
    let config_name = quoted(manifest, "\"Config\":\"");
    if !config_name.ends_with(".json") || !safe_name(&config_name) { fail("invalid config path"); }
    let layers = quoted_list(manifest, "\"Layers\":[");
    if layers.is_empty() || layers.len() > 64 { fail("invalid layer count"); }
    let config_entry = entries.get(&config_name).unwrap_or_else(|| fail("config absent"));
    if config_entry.data.is_empty() { fail("oversized image config"); }
    let config = &config_entry.data;
    let config_digest = config_entry.sha256.clone();
    if config_name != format!("{config_digest}.json") { fail("config name/digest mismatch"); }
    let config_text = std::str::from_utf8(config).unwrap_or_else(|_| fail("config is not UTF-8"));
    let diff_ids = quoted_list(config_text, "\"diff_ids\":[");
    if diff_ids.len() != layers.len() { fail("layer/diff-id count mismatch"); }
    let mut expected = BTreeSet::from(["manifest.json".to_owned(), "repositories".to_owned(), config_name.clone()]);
    for (layer, diff_id) in layers.iter().zip(diff_ids) {
        if !safe_name(layer) || !diff_id.starts_with("sha256:") { fail("invalid layer identity"); }
        let entry = entries.get(layer).unwrap_or_else(|| fail("layer absent"));
        if diff_id != format!("sha256:{}", entry.sha256) { fail("layer diff-id mismatch"); }
        expected.insert(layer.clone());
        let directory = layer.strip_suffix("/layer.tar").unwrap_or_else(|| fail("noncanonical layer path"));
        if directory.len() != 64 || !directory.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
            fail("invalid layer directory");
        }
        expected.insert(format!("{directory}/VERSION"));
        expected.insert(format!("{directory}/json"));
    }
    if entries.keys().any(|name| !expected.contains(name)) { fail("unexpected archive member"); }
    if metadata_digest(metadata, "containerimage.config.digest") != config_digest
        || metadata_digest(metadata, "containerimage.digest") != config_digest { fail("reported image digest mismatch"); }
    let image_id = fs::read_to_string(image_id_path).unwrap_or_else(|_| fail("loaded image identity absent"));
    if image_id.trim() != format!("sha256:{config_digest}") { fail("loaded image identity mismatch"); }
    config_digest
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 7 { fail("usage: preauth-verify-oci archive1 metadata1 image-id1 archive2 metadata2 image-id2"); }
    let paths: Vec<_> = args[1..].iter().map(Path::new).collect();
    let digest_one = verify_one(paths[0], paths[1], paths[2]);
    let digest_two = verify_one(paths[3], paths[4], paths[5]);
    if digest_one != digest_two || digest_file(paths[0]) != digest_file(paths[3]) { fail("independent OCI builds differ"); }
    println!("derived_oci_archive_sha256={}", digest_file(paths[0]));
    println!("derived_oci_digest=sha256:{digest_one}");
    println!("loaded_image_sha256={digest_one}");
}
