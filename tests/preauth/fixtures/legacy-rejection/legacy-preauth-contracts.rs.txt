use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Read;

use super::{
    AUTHORITY_ENVIRONMENT, AUTHORITY_REF, AUTHORITY_REPOSITORY, AUTHORITY_WORKFLOW,
    KMS_ALGORITHM, KMS_KEY_ARN, OIDC_AUDIENCE, OIDC_ISSUER, OIDC_SUBJECT, PreauthError,
    Result, digest, token,
};

fn package_token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(byte, b'.' | b'_' | b'-' | b':' | b'+' | b'~' | b'%')
    })
}

const IDENTITY_FIELDS: &[&str] = &[
    "schema", "phase", "closure_sha256", "package_manifest_sha256",
    "source_manifest_sha256", "signature_manifest_sha256", "license_manifest_sha256",
    "canonical_oci_archive_sha256", "canonical_oci_index_sha256", "selected_oci_manifest_sha256",
    "docker_config_sha256", "compressed_layer_descriptor_set_sha256", "rootfs_diff_id_set_sha256",
    "lld_sha256", "qemu_sha256", "ovmf_code_sha256", "ovmf_vars_sha256",
    "artifact_sha256", "disk_seed_sha256", "disk_child_sha256", "disk_record_sha256",
    "profile_sha256", "command_sha256", "execution_host_sha256", "authority_policy_sha256",
    "authorization_identity_sha256", "authorization_state", "prepared_certification_sha256",
    "resolver_sha256", "spawner_sha256", "consumption_key_sha256", "source_tree_sha256",
    "record_sha256",
];

const ATTESTATION_FIELDS: &[&str] = &[
    "schema", "phase", "attested_identity_graph_sha256", "source_revision", "event",
    "run_id", "archive_sha256", "buildx_descriptor_kind", "buildx_descriptor_sha256",
    "docker_config_sha256", "selected_oci_manifest_sha256", "canonical_oci_index_sha256", "layer_descriptor_set_sha256",
    "rootfs_diff_id_set_sha256", "loaded_image_config_sha256", "package_manifest_sha256",
    "profile_sha256", "artifact_sha256", "disk_sha256", "closure_sha256", "record_sha256",
];

const EXECUTION_HOST_FIELDS: &[&str] = &[
    "schema", "phase", "host_id", "os_class", "architecture", "kernel_class",
    "runtime_id", "runtime_sha256", "closure_sha256", "resolver_sha256",
    "spawner_sha256", "wrapper_sha256", "environment_sha256",
    "resource_controller_sha256", "timeout_seconds", "termination_grace_seconds",
    "output_bytes", "network", "devices", "direct_launch", "cleanup_policy",
    "record_sha256",
];

const AUTHORITY_FIELDS: &[&str] = &[
    "schema", "authorization_id", "state", "certification_sha256", "identity_graph_sha256",
    "profile_sha256", "command_sha256", "artifact_sha256", "disk_sha256",
    "firmware_code_sha256", "firmware_vars_sha256", "closure_sha256",
    "execution_host_sha256", "resolver_sha256", "spawner_sha256", "consumption_key_sha256",
    "nonce", "issued_at", "expires_at", "repository", "workflow", "ref", "environment",
    "oidc_issuer", "oidc_audience", "oidc_subject", "kms_key_arn", "kms_algorithm",
    "kms_context_sha256", "cloudtrail_previous_sha256", "cloudtrail_evidence_sha256",
    "transition_version", "record_sha256", "kms_signature_sha256",
];

const CLOSURE_FIELDS: &[&str] = &[
    "schema", "base_oci_index_sha256", "debian_archive", "debian_snapshot", "debian_suite",
    "debian_security_archive", "debian_security_snapshot", "debian_security_suite",
    "debian_archive_keyring_sha256", "inrelease_sha256", "security_inrelease_sha256",
    "package_manifest_sha256", "license_manifest_sha256", "canonical_oci_archive_sha256",
    "canonical_oci_index_sha256", "selected_oci_manifest_sha256", "docker_config_sha256",
    "buildx_config_sha256", "loaded_image_config_sha256", "compressed_layer_descriptor_set_sha256",
    "rootfs_diff_id_set_sha256", "lld_path", "lld_sha256", "qemu_path", "qemu_sha256", "ovmf_code_path",
    "ovmf_code_sha256", "ovmf_vars_path", "ovmf_vars_sha256", "acquisition_policy_sha256",
    "target_linked_dependencies", "certifiable",
];

const PREPARED_CERT_FIELDS: &[&str] = &[
    "schema", "phase", "profile_id", "profile_sha256", "command_sha256", "closure_sha256",
    "execution_host_sha256", "artifact_sha256", "disk_record_sha256", "firmware_code_sha256",
    "firmware_vars_sha256", "review_state", "record_sha256",
];

fn strict_fields<'a>(input: &'a str, names: &[&str], maximum: usize) -> Result<Vec<&'a str>> {
    if input.len() > maximum
        || !input.ends_with('\n')
        || input.contains('\r')
        || !input.bytes().all(|byte| byte == b'\n' || (0x20..=0x7e).contains(&byte))
    {
        return Err(PreauthError::new("noncanonical-record"));
    }
    let lines: Vec<_> = input[..input.len() - 1].split('\n').collect();
    if lines.len() != names.len() {
        return Err(PreauthError::new("record-field-count"));
    }
    let mut values = Vec::with_capacity(names.len());
    for (line, name) in lines.into_iter().zip(names) {
        let expected = format!("{name}=");
        let Some(value) = line.strip_prefix(&expected) else {
            return Err(PreauthError::new("record-field-order"));
        };
        if value.is_empty() || value.contains('=') {
            return Err(PreauthError::new("record-field-value"));
        }
        values.push(value);
    }
    Ok(values)
}

fn canonical_without_last(names: &[&str], values: &[&str], count: usize) -> String {
    let mut output = String::new();
    for index in 0..count {
        output.push_str(names[index]);
        output.push('=');
        output.push_str(values[index]);
        output.push('\n');
    }
    output
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityGraph {
    pub phase: String,
    pub document_sha256: String,
    pub record_sha256: String,
    pub nodes: BTreeMap<String, String>,
}

impl IdentityGraph {
    pub fn parse(input: &str) -> Result<Self> {
        let values = strict_fields(input, IDENTITY_FIELDS, 8192)?;
        if values[0] != "rar-preauth-identity-graph-v2"
            || !matches!(values[1], "prepared" | "attested")
            || values[26] != "unissued"
            || !values[2..26].iter().all(|value| digest(value))
            || !values[27..32].iter().all(|value| digest(value))
            || !digest(values[32])
        {
            return Err(PreauthError::new("invalid-identity-graph"));
        }
        let payload = canonical_without_last(IDENTITY_FIELDS, &values, 32);
        if sha256_hex(payload.as_bytes()) != values[32] {
            return Err(PreauthError::new("identity-graph-integrity"));
        }
        let nodes = IDENTITY_FIELDS[2..26].iter().zip(&values[2..26])
            .chain(IDENTITY_FIELDS[27..32].iter().zip(&values[27..32]))
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())).collect();
        Ok(Self { phase: values[1].to_owned(), document_sha256: sha256_hex(input.as_bytes()), record_sha256: values[32].to_owned(), nodes })
    }

    pub fn require(&self, kind: &str, digest_value: &str) -> Result<()> {
        if self.nodes.get(kind).is_some_and(|value| value == digest_value) { Ok(()) }
        else { Err(PreauthError::new("identity-edge-mismatch")) }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestationRecord {
    pub attested_identity_graph_sha256: String,
    pub source_revision: String,
    pub event: String,
    pub run_id: u64,
    pub archive_sha256: String,
    pub buildx_descriptor_sha256: String,
    pub docker_config_sha256: String,
    pub selected_oci_manifest_sha256: String,
    pub canonical_oci_index_sha256: String,
    pub layer_descriptor_set_sha256: String,
    pub rootfs_diff_id_set_sha256: String,
    pub loaded_image_config_sha256: String,
    pub package_manifest_sha256: String,
    pub profile_sha256: String,
    pub artifact_sha256: String,
    pub disk_sha256: String,
    pub closure_sha256: String,
    pub record_sha256: String,
}

impl AttestationRecord {
    pub fn parse(input: &str, expected_head: &str, expected_event: &str, expected_run: u64) -> Result<Self> {
        let values = strict_fields(input, ATTESTATION_FIELDS, 4096)?;
        let run_id = values[5].parse::<u64>().map_err(|_| PreauthError::new("invalid-run-id"))?;
        if values[0] != "rar-preauth-ci-attestation-v2"
            || values[1] != "attested"
            || !digest(values[2])
            || values[3] != expected_head
            || values[4] != expected_event
            || run_id != expected_run
            || !digest(values[6])
            || values[7] != "docker-config-id"
            || !values[8..20].iter().all(|value| digest(value))
            || values[8] != values[9]
            || values[8] != values[14]
            || !digest(values[20])
        {
            return Err(PreauthError::new("invalid-ci-attestation"));
        }
        let payload = canonical_without_last(ATTESTATION_FIELDS, &values, 20);
        if sha256_hex(payload.as_bytes()) != values[20] {
            return Err(PreauthError::new("ci-attestation-integrity"));
        }
        Ok(Self {
            attested_identity_graph_sha256: values[2].to_owned(),
            source_revision: values[3].to_owned(),
            event: values[4].to_owned(),
            run_id,
            archive_sha256: values[6].to_owned(),
            buildx_descriptor_sha256: values[8].to_owned(),
            docker_config_sha256: values[9].to_owned(),
            selected_oci_manifest_sha256: values[10].to_owned(),
            canonical_oci_index_sha256: values[11].to_owned(),
            layer_descriptor_set_sha256: values[12].to_owned(),
            rootfs_diff_id_set_sha256: values[13].to_owned(),
            loaded_image_config_sha256: values[14].to_owned(),
            package_manifest_sha256: values[15].to_owned(), profile_sha256: values[16].to_owned(),
            artifact_sha256: values[17].to_owned(), disk_sha256: values[18].to_owned(),
            closure_sha256: values[19].to_owned(),
            record_sha256: values[20].to_owned(),
        })
    }

    pub fn validate_graph(&self, graph: &IdentityGraph) -> Result<()> {
        if graph.phase != "attested" || self.attested_identity_graph_sha256 != graph.document_sha256 {
            return Err(PreauthError::new("attestation-identity-mismatch"));
        }
        for (kind, value) in [
            ("package_manifest_sha256", graph.nodes.get("package_manifest_sha256")),
            ("profile_sha256", graph.nodes.get("profile_sha256")),
            ("artifact_sha256", graph.nodes.get("artifact_sha256")),
            ("closure_sha256", graph.nodes.get("closure_sha256")),
        ] {
            let actual = match kind {
                "package_manifest_sha256" => &self.package_manifest_sha256,
                "profile_sha256" => &self.profile_sha256,
                "artifact_sha256" => &self.artifact_sha256,
                _ => &self.closure_sha256,
            };
            if value != Some(actual) { return Err(PreauthError::new("attestation-edge-mismatch")); }
        }
        for (kind, actual) in [
            ("canonical_oci_archive_sha256", &self.archive_sha256),
            ("canonical_oci_index_sha256", &self.canonical_oci_index_sha256),
            ("selected_oci_manifest_sha256", &self.selected_oci_manifest_sha256),
            ("docker_config_sha256", &self.docker_config_sha256),
            ("compressed_layer_descriptor_set_sha256", &self.layer_descriptor_set_sha256),
            ("rootfs_diff_id_set_sha256", &self.rootfs_diff_id_set_sha256),
        ] { graph.require(kind, actual)?; }
        Ok(())
    }
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionHostRecord {
    pub record_sha256: String,
    pub resolver_sha256: String,
    pub spawner_sha256: String,
    pub timeout_seconds: u64,
    pub output_bytes: u64,
}

impl ExecutionHostRecord {
    pub fn parse(input: &str) -> Result<Self> {
        let values = strict_fields(input, EXECUTION_HOST_FIELDS, 4096)?;
        let timeout_seconds = values[14].parse::<u64>().map_err(|_| PreauthError::new("invalid-host-limit"))?;
        let termination_grace = values[15].parse::<u64>().map_err(|_| PreauthError::new("invalid-host-limit"))?;
        let output_bytes = values[16].parse::<u64>().map_err(|_| PreauthError::new("invalid-host-limit"))?;
        if values[0] != "rar-execution-host-v1"
            || values[1] != "prepared"
            || values[2] != "unprovisioned-r0-x86_64"
            || values[3] != "unprovisioned"
            || values[4] != "x86_64"
            || values[5] != "unprovisioned"
            || !values[7..14].iter().all(|value| digest(value))
            || !(1..=300).contains(&timeout_seconds)
            || !(1..=30).contains(&termination_grace)
            || !(1024..=16 * 1024 * 1024).contains(&output_bytes)
            || values[17] != "off"
            || values[18] != "none"
            || values[19] != "refused"
            || values[20] != "quarantine-on-uncertainty"
            || !digest(values[21])
        {
            return Err(PreauthError::new("invalid-execution-host"));
        }
        let payload = canonical_without_last(EXECUTION_HOST_FIELDS, &values, 21);
        if sha256_hex(payload.as_bytes()) != values[21] {
            return Err(PreauthError::new("execution-host-integrity"));
        }
        Ok(Self {
            record_sha256: values[21].to_owned(),
            resolver_sha256: values[9].to_owned(),
            spawner_sha256: values[10].to_owned(),
            timeout_seconds,
            output_bytes,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictAuthorityRecord {
    values: Vec<String>,
    pub authorization_id: String,
    pub state: String,
    pub transition_version: u64,
    pub expires_at: u64,
    pub record_sha256: String,
}

impl StrictAuthorityRecord {
    pub fn parse(input: &str) -> Result<Self> {
        let values = strict_fields(input, AUTHORITY_FIELDS, 8192)?;
        let transition_version = values[31].parse::<u64>().map_err(|_| PreauthError::new("invalid-transition-version"))?;
        let expires = values[18].parse::<u64>().map_err(|_| PreauthError::new("invalid-authority-time"))?;
        if values[0] != "rar-external-authorization-v1"
            || !token(values[1])
            || !matches!(values[2], "issued" | "consumed" | "revoked" | "uncertain")
            || !values[3..16].iter().all(|value| digest(value))
            || !token(values[16])
            || !valid_timestamp(values[17])
            || expires == 0
            || values[19] != AUTHORITY_REPOSITORY
            || values[20] != AUTHORITY_WORKFLOW
            || values[21] != AUTHORITY_REF
            || values[22] != AUTHORITY_ENVIRONMENT
            || values[23] != OIDC_ISSUER
            || values[24] != OIDC_AUDIENCE
            || values[25] != OIDC_SUBJECT
            || values[26] != KMS_KEY_ARN
            || values[27] != KMS_ALGORITHM
            || !values[28..31].iter().all(|value| digest(value))
            || transition_version == 0
            || !digest(values[32])
            || !digest(values[33])
        {
            return Err(PreauthError::new("invalid-authority-record"));
        }
        let payload = canonical_without_last(AUTHORITY_FIELDS, &values, 32);
        if sha256_hex(payload.as_bytes()) != values[32] {
            return Err(PreauthError::new("authority-record-integrity"));
        }
        Ok(Self {
            values: values.iter().map(|value| (*value).to_owned()).collect(),
            authorization_id: values[1].to_owned(),
            state: values[2].to_owned(),
            transition_version,
            expires_at: expires,
            record_sha256: values[32].to_owned(),
        })
    }

    pub fn signature_input(&self) -> String {
        let refs: Vec<_> = self.values.iter().map(String::as_str).collect();
        canonical_without_last(AUTHORITY_FIELDS, &refs, 33)
    }

    pub fn field(&self, name: &str) -> Result<&str> {
        AUTHORITY_FIELDS.iter().position(|candidate| *candidate == name)
            .map(|index| self.values[index].as_str())
            .ok_or_else(|| PreauthError::new("unknown-authority-field"))
    }

    pub fn validate_graph(&self, graph: &IdentityGraph) -> Result<()> {
        if self.field("identity_graph_sha256")? != graph.document_sha256 {
            return Err(PreauthError::new("authority-identity-mismatch"));
        }
        for (authority, identity) in [
            ("certification_sha256", "prepared_certification_sha256"),
            ("profile_sha256", "profile_sha256"), ("command_sha256", "command_sha256"),
            ("artifact_sha256", "artifact_sha256"), ("disk_sha256", "disk_child_sha256"),
            ("firmware_code_sha256", "ovmf_code_sha256"), ("firmware_vars_sha256", "ovmf_vars_sha256"),
            ("closure_sha256", "closure_sha256"), ("execution_host_sha256", "execution_host_sha256"),
            ("resolver_sha256", "resolver_sha256"), ("spawner_sha256", "spawner_sha256"),
            ("consumption_key_sha256", "consumption_key_sha256"),
        ] {
            graph.require(identity, self.field(authority)?)?;
        }
        Ok(())
    }
}

fn valid_timestamp(value: &str) -> bool {
    value.len() == 20
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value.as_bytes()[10] == b'T'
        && value.as_bytes()[13] == b':'
        && value.as_bytes()[16] == b':'
        && value.ends_with('Z')
        && value.bytes().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRow {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub filename: String,
    pub size: u64,
    pub sha256: String,
    pub license_sha256: String,
    pub source_name: String,
    pub source_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageManifest {
    pub rows: Vec<PackageRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureLock {
    pub package_manifest_sha256: String,
    pub license_manifest_sha256: String,
    pub canonical_archive_sha256: String,
    pub canonical_index_sha256: String,
    pub selected_manifest_sha256: String,
    pub docker_config_sha256: String,
    pub compressed_layers_sha256: String,
    pub rootfs_diff_ids_sha256: String,
    pub lld_sha256: String,
    pub qemu_sha256: String,
    pub ovmf_code_sha256: String,
    pub ovmf_vars_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCertification {
    pub profile_sha256: String,
    pub command_sha256: String,
    pub closure_sha256: String,
    pub execution_host_sha256: String,
    pub artifact_sha256: String,
    pub disk_record_sha256: String,
    pub record_sha256: String,
}

impl PreparedCertification {
    pub fn parse(input: &str) -> Result<Self> {
        let values = strict_fields(input, PREPARED_CERT_FIELDS, 4096)?;
        if values[0] != "rar-preauth-prepared-certification-v1"
            || values[1] != "prepared"
            || values[2] != "r0-x86_64-preauth-v1"
            || !values[3..11].iter().all(|value| digest(value))
            || values[11] != "pending-independent-review"
            || !digest(values[12])
        {
            return Err(PreauthError::new("invalid-prepared-certification"));
        }
        let payload = canonical_without_last(PREPARED_CERT_FIELDS, &values, 12);
        if sha256_hex(payload.as_bytes()) != values[12] {
            return Err(PreauthError::new("prepared-certification-integrity"));
        }
        Ok(Self {
            profile_sha256: values[3].to_owned(), command_sha256: values[4].to_owned(),
            closure_sha256: values[5].to_owned(), execution_host_sha256: values[6].to_owned(),
            artifact_sha256: values[7].to_owned(), disk_record_sha256: values[8].to_owned(),
            record_sha256: values[12].to_owned(),
        })
    }
}

impl ClosureLock {
    pub fn parse(input: &str, package_manifest: &str) -> Result<Self> {
        let values = strict_fields(input, CLOSURE_FIELDS, 8192)?;
        if values[0] != "rar-preauth-closure-v3"
            || values[1] != super::BASE_OCI_INDEX_SHA256
            || values[2] != "snapshot.debian.org/archive/debian"
            || values[3] != "20260630T000000Z"
            || values[4] != "trixie"
            || values[5] != "snapshot.debian.org/archive/debian-security"
            || values[6] != "20260630T000000Z"
            || values[7] != "trixie-security"
            || !values[8..21].iter().all(|value| digest(value))
            || values[11] != sha256_hex(package_manifest.as_bytes())
            || values[16] != values[17] || values[16] != values[18]
            || BTreeSet::from([values[13], values[14], values[15], values[16], values[19], values[20]]).len() != 6
            || values[21] != "/usr/bin/ld.lld-19" || !digest(values[22])
            || values[23] != "/usr/bin/qemu-system-x86_64" || !digest(values[24])
            || values[25] != "/usr/share/OVMF/OVMF_CODE_4M.fd" || !digest(values[26])
            || values[27] != "/usr/share/OVMF/OVMF_VARS_4M.fd"
            || !values[28..30].iter().all(|value| digest(value))
            || values[30] != "none" || values[31] != "true"
        {
            return Err(PreauthError::new("invalid-closure-lock"));
        }
        Ok(Self {
            package_manifest_sha256: values[11].to_owned(),
            license_manifest_sha256: values[12].to_owned(),
            canonical_archive_sha256: values[13].to_owned(), canonical_index_sha256: values[14].to_owned(),
            selected_manifest_sha256: values[15].to_owned(), docker_config_sha256: values[16].to_owned(),
            compressed_layers_sha256: values[19].to_owned(), rootfs_diff_ids_sha256: values[20].to_owned(),
            lld_sha256: values[22].to_owned(), qemu_sha256: values[24].to_owned(),
            ovmf_code_sha256: values[26].to_owned(), ovmf_vars_sha256: values[28].to_owned(),
        })
    }
}

impl PackageManifest {
    pub fn parse(input: &str) -> Result<Self> {
        if input.len() > 64 * 1024
            || !input.ends_with('\n')
            || input.contains('\r')
            || !input.starts_with("schema=rar-preauth-package-manifest-v2\n")
        {
            return Err(PreauthError::new("invalid-package-manifest"));
        }
        let mut rows = Vec::new();
        let mut names = BTreeSet::new();
        let mut filenames = BTreeSet::new();
        let mut previous = None::<String>;
        for line in input.lines().skip(1) {
            let fields: Vec<_> = line.split('|').collect();
            if fields.len() != 10
                || fields[0] != "package"
                || !package_token(fields[1])
                || !package_token(fields[2])
                || !matches!(fields[3], "amd64" | "all")
                || !package_token(fields[4])
                || !fields[4].ends_with(".deb")
                || !digest(fields[6])
                || !digest(fields[7])
                || !package_token(fields[8])
                || !package_token(fields[9])
            {
                return Err(PreauthError::new("invalid-package-row"));
            }
            let size = fields[5].parse::<u64>().map_err(|_| PreauthError::new("invalid-package-size"))?;
            if size == 0 || size > 128 * 1024 * 1024 {
                return Err(PreauthError::new("invalid-package-size"));
            }
            if previous.as_deref().is_some_and(|name| name >= fields[1])
                || !names.insert(fields[1].to_owned())
                || !filenames.insert(fields[4].to_owned())
            {
                return Err(PreauthError::new("noncanonical-package-order"));
            }
            previous = Some(fields[1].to_owned());
            rows.push(PackageRow {
                name: fields[1].to_owned(), version: fields[2].to_owned(),
                architecture: fields[3].to_owned(), filename: fields[4].to_owned(), size,
                sha256: fields[6].to_owned(), license_sha256: fields[7].to_owned(),
                source_name: fields[8].to_owned(), source_version: fields[9].to_owned(),
            });
        }
        if rows.len() != 36 {
            return Err(PreauthError::new("package-count-mismatch"));
        }
        Ok(Self { rows })
    }
}

pub fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finish_hex()
}

pub fn sha256_reader<R: Read>(reader: &mut R) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finish_hex())
}

struct Sha256 { state: [u32; 8], block: [u8; 64], block_len: usize, total_bytes: u64 }

impl Sha256 {
    const INITIAL: [u32; 8] = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
    const K: [u32;64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
    fn new()->Self{Self{state:Self::INITIAL,block:[0;64],block_len:0,total_bytes:0}}
    fn update(&mut self, mut input:&[u8]){self.total_bytes=self.total_bytes.wrapping_add(input.len() as u64);if self.block_len!=0{let count=(64-self.block_len).min(input.len());self.block[self.block_len..self.block_len+count].copy_from_slice(&input[..count]);self.block_len+=count;input=&input[count..];if self.block_len==64{let block=self.block;self.compress(&block);self.block_len=0;}}while input.len()>=64{self.compress(&input[..64]);input=&input[64..];}if !input.is_empty(){self.block[..input.len()].copy_from_slice(input);self.block_len=input.len();}}
    fn compress(&mut self, block:&[u8]){let mut w=[0u32;64];for(i,b)in block.chunks_exact(4).enumerate(){w[i]=u32::from_be_bytes([b[0],b[1],b[2],b[3]]);}for i in 16..64{let s0=w[i-15].rotate_right(7)^w[i-15].rotate_right(18)^(w[i-15]>>3);let s1=w[i-2].rotate_right(17)^w[i-2].rotate_right(19)^(w[i-2]>>10);w[i]=w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);}let[mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h]=self.state;for i in 0..64{let s1=e.rotate_right(6)^e.rotate_right(11)^e.rotate_right(25);let ch=(e&f)^((!e)&g);let t1=h.wrapping_add(s1).wrapping_add(ch).wrapping_add(Self::K[i]).wrapping_add(w[i]);let s0=a.rotate_right(2)^a.rotate_right(13)^a.rotate_right(22);let maj=(a&b)^(a&c)^(b&c);let t2=s0.wrapping_add(maj);h=g;g=f;f=e;e=d.wrapping_add(t1);d=c;c=b;b=a;a=t1.wrapping_add(t2);}[a,b,c,d,e,f,g,h].into_iter().enumerate().for_each(|(i,v)|self.state[i]=self.state[i].wrapping_add(v));}
    fn finish_hex(mut self)->String{let bits=self.total_bytes.wrapping_mul(8);self.block[self.block_len]=0x80;self.block_len+=1;if self.block_len>56{self.block[self.block_len..].fill(0);let block=self.block;self.compress(&block);self.block=[0;64];self.block_len=0;}self.block[self.block_len..56].fill(0);self.block[56..64].copy_from_slice(&bits.to_be_bytes());let block=self.block;self.compress(&block);let mut out=String::with_capacity(64);for word in self.state{use fmt::Write as _;write!(&mut out,"{word:08x}").unwrap();}out}
}
