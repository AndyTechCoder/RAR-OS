use std::collections::{BTreeMap, BTreeSet};

use super::{PreauthError, Result, sha256_hex};

const MAX_RECORD: usize = 1024 * 1024;
const MAX_OBJECTS: usize = super::transaction::MAX_INPUT_OBJECTS;
const MAX_OBJECT: u64 = 2 * 1024 * 1024 * 1024;
const MAX_AGGREGATE: u64 = 4 * 1024 * 1024 * 1024;
const SOURCE_DATE_EPOCH: u64 = 1_784_332_800;

const MANIFEST_FIELDS: &[&str] = &[
    "schema", "input_lock_sha256", "producer_policy_sha256", "base_oci_index_sha256",
    "debian_snapshot", "package_count", "object_count", "aggregate_bytes",
    "objects_manifest_sha256", "record_sha256",
];

fn digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_atom(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum
        && value.bytes().all(|byte| (0x21..=0x7e).contains(&byte) && byte != b'|' && byte != b'=')
}

fn decimal(value: &str) -> Result<u64> {
    if value.is_empty() || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(PreauthError::new("input-bundle-number"));
    }
    value.parse().map_err(|_| PreauthError::new("input-bundle-number"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputObjectV1 {
    pub role: String,
    pub path: String,
    pub size: u64,
    pub sha256: String,
    pub origin: String,
    pub package: String,
    pub version: String,
    pub architecture: String,
    pub source_package: String,
    pub source_version: String,
}

impl InputObjectV1 {
    fn parse(line: &str) -> Result<Self> {
        let fields: Vec<_> = line.split('|').collect();
        if fields.len() != 10 || !canonical_atom(fields[0], 64) || !canonical_atom(fields[1], 80)
            || !canonical_atom(fields[4], 512) || !fields[5..].iter().all(|value| *value == "-" || canonical_atom(value, 256))
        {
            return Err(PreauthError::new("input-object-fields"));
        }
        let size = decimal(fields[2])?;
        if size > MAX_OBJECT || !digest(fields[3]) || fields[1] != format!("objects/{}", fields[3]) {
            return Err(PreauthError::new("input-object-identity"));
        }
        Ok(Self {
            role: fields[0].into(), path: fields[1].into(), size, sha256: fields[3].into(),
            origin: fields[4].into(), package: fields[5].into(), version: fields[6].into(),
            architecture: fields[7].into(), source_package: fields[8].into(),
            source_version: fields[9].into(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputBundleV1 {
    pub input_lock_sha256: String,
    pub producer_policy_sha256: String,
    pub base_oci_index_sha256: String,
    pub package_count: usize,
    pub aggregate_bytes: u64,
    pub archive_sha256: String,
    pub objects: Vec<InputObjectV1>,
}

fn parse_manifest(bytes: &[u8], object_bytes: &[u8]) -> Result<(InputBundleV1, usize)> {
    let text = std::str::from_utf8(bytes).map_err(|_| PreauthError::new("input-bundle-utf8"))?;
    if text.len() > MAX_RECORD || !text.ends_with('\n') || text.contains('\r')
        || !text.bytes().all(|byte| byte == b'\n' || (0x20..=0x7e).contains(&byte))
    {
        return Err(PreauthError::new("input-bundle-canonical"));
    }
    let lines: Vec<_> = text[..text.len() - 1].split('\n').collect();
    if lines.len() != MANIFEST_FIELDS.len() { return Err(PreauthError::new("input-bundle-field-count")); }
    let mut values = Vec::new();
    for (line, field) in lines.iter().zip(MANIFEST_FIELDS) {
        let value = line.strip_prefix(&format!("{field}=")).ok_or_else(|| PreauthError::new("input-bundle-field-order"))?;
        if !canonical_atom(value, 512) { return Err(PreauthError::new("input-bundle-field-value")); }
        values.push(value);
    }
    if values[0] != "rar-preauth-input-bundle-v1" || values[4] != "20260630T000000Z"
        || ![1usize, 2, 3, 8, 9].iter().all(|index| digest(values[*index]))
    {
        return Err(PreauthError::new("input-bundle-header"));
    }
    let mut payload = String::new();
    for index in 0..9 { payload.push_str(MANIFEST_FIELDS[index]); payload.push('='); payload.push_str(values[index]); payload.push('\n'); }
    if sha256_hex(payload.as_bytes()) != values[9] || sha256_hex(object_bytes) != values[8] {
        return Err(PreauthError::new("input-bundle-record-integrity"));
    }
    let package_count = usize::try_from(decimal(values[5])?).map_err(|_| PreauthError::new("input-bundle-number"))?;
    let object_count = usize::try_from(decimal(values[6])?).map_err(|_| PreauthError::new("input-bundle-number"))?;
    let aggregate_bytes = decimal(values[7])?;
    if package_count == 0 || object_count > MAX_OBJECTS || aggregate_bytes > MAX_AGGREGATE {
        return Err(PreauthError::new("input-bundle-bound"));
    }
    Ok((InputBundleV1 {
        input_lock_sha256: values[1].into(), producer_policy_sha256: values[2].into(),
        base_oci_index_sha256: values[3].into(), package_count, aggregate_bytes,
        archive_sha256: String::new(), objects: Vec::new(),
    }, object_count))
}

fn octal(bytes: &[u8]) -> Result<u64> {
    let end = bytes.iter().position(|byte| *byte == 0 || *byte == b' ').unwrap_or(bytes.len());
    let value = std::str::from_utf8(&bytes[..end]).map_err(|_| PreauthError::new("input-bundle-tar-number"))?.trim();
    if value.is_empty() { return Ok(0); }
    if !value.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) { return Err(PreauthError::new("input-bundle-tar-number")); }
    u64::from_str_radix(value, 8).map_err(|_| PreauthError::new("input-bundle-tar-number"))
}

fn input_bundle_member_name(name: &str) -> bool {
    name == "manifest.v1" || name == "objects.v1"
        || (name.len() == 72 && name.starts_with("objects/") && digest(&name[8..]))
}

pub(crate) fn canonical_input_bundle_header(name: &str, size: u64) -> Result<[u8; 512]> {
    if !input_bundle_member_name(name) || size > MAX_OBJECT {
        return Err(PreauthError::new("input-bundle-tar-name"));
    }
    let mut header = [0u8; 512];
    header[..name.len()].copy_from_slice(name.as_bytes());
    header[100..108].copy_from_slice(b"0000644\0");
    header[108..116].copy_from_slice(b"0000000\0");
    header[116..124].copy_from_slice(b"0000000\0");
    header[124..136].copy_from_slice(format!("{size:011o}\0").as_bytes());
    header[136..148].copy_from_slice(b"15226541000\0");
    header[148..156].fill(b' ');
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum: u64 = header.iter().map(|byte| *byte as u64).sum();
    header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
    Ok(header)
}

fn validate_input_bundle_header(header: &[u8]) -> Result<(String, u64)> {
    if &header[257..263] != b"ustar\0" || header[156] != b'0' {
        return Err(PreauthError::new("input-bundle-tar-type"));
    }
    let expected_checksum = octal(&header[148..156])?;
    let actual_checksum: u64 = header.iter().enumerate()
        .map(|(index, byte)| if (148..156).contains(&index) { b' ' as u64 } else { *byte as u64 })
        .sum();
    if expected_checksum != actual_checksum || octal(&header[100..108])? != 0o644
        || octal(&header[108..116])? != 0 || octal(&header[116..124])? != 0
        || octal(&header[136..148])? != SOURCE_DATE_EPOCH
    {
        return Err(PreauthError::new("input-bundle-tar-metadata"));
    }
    let name_end = header[..100].iter().position(|byte| *byte == 0).unwrap_or(100);
    let name = std::str::from_utf8(&header[..name_end])
        .map_err(|_| PreauthError::new("input-bundle-tar-name"))?;
    if !input_bundle_member_name(name) {
        return Err(PreauthError::new("input-bundle-tar-name"));
    }
    let size = octal(&header[124..136])?;
    if size > MAX_OBJECT {
        return Err(PreauthError::new("input-bundle-object-bound"));
    }
    if header != canonical_input_bundle_header(name, size)?.as_slice() {
        return Err(PreauthError::new("input-bundle-tar-metadata"));
    }
    Ok((name.to_owned(), size))
}

pub fn parse_input_bundle_v1(bytes: &[u8]) -> Result<InputBundleV1> {
    if bytes.len() as u64 > MAX_AGGREGATE.saturating_add(16 * 1024 * 1024) {
        return Err(PreauthError::new("input-bundle-archive-bound"));
    }
    let mut offset = 0usize;
    let mut names = BTreeSet::new();
    let mut payloads: BTreeMap<String, &[u8]> = BTreeMap::new();
    let mut previous_name: Option<String> = None;
    let mut zero_blocks = 0u8;
    while offset < bytes.len() {
        let end = offset.checked_add(512).ok_or_else(|| PreauthError::new("input-bundle-overflow"))?;
        if end > bytes.len() { return Err(PreauthError::new("input-bundle-truncated")); }
        let header = &bytes[offset..end];
        if header.iter().all(|byte| *byte == 0) {
            zero_blocks += 1; offset = end;
            if zero_blocks == 2 {
                if bytes[offset..].iter().any(|byte| *byte != 0) { return Err(PreauthError::new("input-bundle-trailing")); }
                break;
            }
            continue;
        }
        if zero_blocks != 0 {
            return Err(PreauthError::new("input-bundle-tar-type"));
        }
        let (name, size) = validate_input_bundle_header(header)?;
        if previous_name.as_deref().is_some_and(|previous| previous >= name.as_str()) {
            return Err(PreauthError::new("input-bundle-tar-order"));
        }
        if !names.insert(name.to_owned()) { return Err(PreauthError::new("input-bundle-duplicate")); }
        previous_name = Some(name.to_owned());
        let payload_start = end;
        let padded = size.checked_add(511).ok_or_else(|| PreauthError::new("input-bundle-overflow"))? / 512 * 512;
        let payload_end = payload_start.checked_add(usize::try_from(padded).map_err(|_| PreauthError::new("input-bundle-overflow"))?).ok_or_else(|| PreauthError::new("input-bundle-overflow"))?;
        let data_end = payload_start.checked_add(usize::try_from(size).map_err(|_| PreauthError::new("input-bundle-overflow"))?).ok_or_else(|| PreauthError::new("input-bundle-overflow"))?;
        if payload_end > bytes.len() { return Err(PreauthError::new("input-bundle-truncated")); }
        if bytes[data_end..payload_end].iter().any(|byte| *byte != 0) {
            return Err(PreauthError::new("input-bundle-tar-padding"));
        }
        payloads.insert(name.to_owned(), &bytes[payload_start..data_end]);
        offset = payload_end;
    }
    if zero_blocks != 2 { return Err(PreauthError::new("input-bundle-end-marker")); }
    let manifest_bytes = payloads.get("manifest.v1").ok_or_else(|| PreauthError::new("input-bundle-manifest-missing"))?;
    let object_manifest = payloads.get("objects.v1").ok_or_else(|| PreauthError::new("input-bundle-objects-missing"))?;
    if object_manifest.len() > MAX_RECORD || !object_manifest.ends_with(b"\n") { return Err(PreauthError::new("input-objects-canonical")); }
    let object_text = std::str::from_utf8(object_manifest).map_err(|_| PreauthError::new("input-objects-utf8"))?;
    let mut previous = None;
    let mut objects = Vec::new();
    let mut aggregate = 0u64;
    for line in object_text[..object_text.len() - 1].split('\n') {
        if previous.is_some_and(|value: &str| value >= line) { return Err(PreauthError::new("input-objects-order")); }
        let object = InputObjectV1::parse(line)?;
        aggregate = aggregate.checked_add(object.size).ok_or_else(|| PreauthError::new("input-bundle-overflow"))?;
        let payload = payloads.get(&object.path).ok_or_else(|| PreauthError::new("input-object-missing"))?;
        if payload.len() as u64 != object.size || sha256_hex(payload) != object.sha256 { return Err(PreauthError::new("input-object-content")); }
        previous = Some(line); objects.push(object);
    }
    let (mut bundle, expected_objects) = parse_manifest(manifest_bytes, object_manifest)?;
    let role_count = |role: &str| objects.iter().filter(|object| object.role == role).count();
    let allowed = ["base-oci", "keyring", "inrelease", "security-inrelease", "package-manifest",
        "license-manifest", "license-archive", "producer-tools", "deb", "tool-lld", "tool-qemu", "firmware-code", "firmware-vars"];
    if objects.len() != expected_objects || payloads.len() != expected_objects + 2 || aggregate != bundle.aggregate_bytes
        || role_count("deb") != bundle.package_count || role_count("base-oci") != 1 || role_count("keyring") != 1
        || role_count("inrelease") != 3 || role_count("security-inrelease") != 1
        || ["package-manifest", "license-manifest", "license-archive", "producer-tools", "tool-lld", "tool-qemu", "firmware-code", "firmware-vars"]
            .iter().any(|role| role_count(role) != 1)
        || objects.iter().any(|object| !allowed.contains(&object.role.as_str()))
    {
        return Err(PreauthError::new("input-bundle-inventory"));
    }
    bundle.archive_sha256 = sha256_hex(bytes);
    bundle.objects = objects;
    Ok(bundle)
}
