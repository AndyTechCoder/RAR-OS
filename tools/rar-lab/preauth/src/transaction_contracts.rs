use std::collections::BTreeMap;

use super::{PreauthError, Result, sha256_hex};

pub const INPUT_LOCK_FIELDS: &[&str] = &[
    "schema", "record_encoding", "package_schema", "transaction_graph_schema",
    "transaction_bundle_schema", "digest_algorithm", "source_revision_algorithm",
    "archive_policy", "json_policy", "path_policy", "publication_policy",
    "base_oci_index_sha256", "debian_archive", "debian_snapshot", "debian_suite",
    "debian_security_archive", "debian_security_snapshot", "debian_security_suite",
    "debian_archive_keyring_sha256", "inrelease_sha256", "security_inrelease_sha256",
    "package_manifest_sha256", "license_manifest_sha256", "lld_path", "lld_sha256",
    "qemu_path", "qemu_sha256", "ovmf_code_path", "ovmf_code_sha256",
    "ovmf_vars_path", "ovmf_vars_sha256", "acquisition_policy_sha256",
    "target_linked_dependencies", "launch_authority",
];

pub const TRANSACTION_GRAPH_FIELDS: &[&str] = &[
    "schema", "source_revision", "source_tree_sha256", "input_lock_sha256",
    "package_manifest_sha256", "license_manifest_sha256", "debian_archive_keyring_sha256",
    "inrelease_sha256", "security_inrelease_sha256", "base_oci_index_sha256", "lld_sha256",
    "qemu_sha256", "ovmf_code_sha256", "ovmf_vars_sha256", "transaction_tool_sha256",
    "raw_oci_archive_sha256", "raw_oci_index_sha256", "canonical_oci_index_sha256",
    "raw_to_canonical_index_relation", "selected_oci_manifest_sha256", "docker_config_sha256",
    "buildx_config_sha256", "loaded_image_config_sha256",
    "compressed_layer_descriptor_set_sha256", "rootfs_diff_id_set_sha256",
    "canonical_oci_archive_sha256", "artifact_first_sha256", "artifact_second_sha256",
    "artifact_sha256", "disk_seed_sha256", "disk_initial_sha256", "disk_record_sha256",
    "profile_sha256", "command_sha256", "execution_host_sha256", "supervisor_sha256",
    "resolver_sha256", "spawner_sha256", "wrapper_sha256", "resource_controller_sha256",
    "bundle_manifest_sha256", "publication_receipt_sha256", "record_sha256",
];

fn digest(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn source_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn strict_values<'a>(input: &'a str, names: &[&str], maximum: usize) -> Result<Vec<&'a str>> {
    if input.len() > maximum || !input.ends_with('\n') || input.contains('\r')
        || !input.bytes().all(|byte| byte == b'\n' || (0x20..=0x7e).contains(&byte))
    {
        return Err(PreauthError::new("noncanonical-transaction-record"));
    }
    let lines: Vec<_> = input[..input.len() - 1].split('\n').collect();
    if lines.len() != names.len() {
        return Err(PreauthError::new("transaction-field-count"));
    }
    let mut values = Vec::with_capacity(names.len());
    for (line, name) in lines.into_iter().zip(names) {
        let prefix = format!("{name}=");
        let Some(value) = line.strip_prefix(&prefix) else {
            return Err(PreauthError::new("transaction-field-order"));
        };
        if value.is_empty() || value.contains('=') {
            return Err(PreauthError::new("transaction-field-value"));
        }
        values.push(value);
    }
    Ok(values)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputLockV4 {
    pub document_sha256: String,
    pub fields: BTreeMap<String, String>,
}

impl InputLockV4 {
    pub fn parse(input: &str) -> Result<Self> {
        let values = strict_values(input, INPUT_LOCK_FIELDS, 32 * 1024)?;
        let exact = [
            (0, "rar-preauth-closure-input-lock-v4"),
            (1, "rar-kv-canonical-v1"),
            (2, "rar-preauth-packages-v2"),
            (3, "rar-preauth-transaction-graph-v1"),
            (4, "rar-preauth-transaction-bundle-v1"),
            (5, "sha256"),
            (6, "git-object-id"),
            (7, "rar-bounded-archive-plan-v1"),
            (8, "rar-strict-json-v1"),
            (9, "descriptor-relative-no-follow-v1"),
            (10, "atomic-no-replace-fsync-v1"),
            (32, "none"),
            (33, "none"),
        ];
        if exact.iter().any(|(index, expected)| values[*index] != *expected)
            || ![11, 18, 19, 20, 21, 22, 24, 26, 28, 30, 31]
                .iter().all(|index| digest(values[*index]))
        {
            return Err(PreauthError::new("invalid-input-lock-v4"));
        }
        let fields = INPUT_LOCK_FIELDS.iter().zip(values).map(|(name, value)| {
            ((*name).to_owned(), value.to_owned())
        }).collect();
        Ok(Self { document_sha256: sha256_hex(input.as_bytes()), fields })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionGraphV1 {
    pub source_revision: String,
    pub document_sha256: String,
    pub record_sha256: String,
    pub nodes: BTreeMap<String, String>,
}

impl TransactionGraphV1 {
    pub fn parse(input: &str) -> Result<Self> {
        let values = strict_values(input, TRANSACTION_GRAPH_FIELDS, 1024 * 1024)?;
        if values[0] != "rar-preauth-transaction-graph-v1"
            || !source_revision(values[1])
            || values[18] != "strict-json-parse+canonical-serialize-v1"
            || !values[2..18].iter().all(|value| digest(value))
            || !values[19..43].iter().all(|value| digest(value))
            || values[20] != values[21] || values[20] != values[22]
            || values[26] != values[27] || values[26] != values[28]
        {
            return Err(PreauthError::new("invalid-transaction-graph-v1"));
        }
        let mut payload = String::new();
        for index in 0..42 {
            payload.push_str(TRANSACTION_GRAPH_FIELDS[index]);
            payload.push('=');
            payload.push_str(values[index]);
            payload.push('\n');
        }
        if sha256_hex(payload.as_bytes()) != values[42] {
            return Err(PreauthError::new("transaction-graph-integrity"));
        }
        let nodes = TRANSACTION_GRAPH_FIELDS[2..42].iter().zip(&values[2..42])
            .filter(|(name, _)| **name != "raw_to_canonical_index_relation")
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())).collect();
        Ok(Self {
            source_revision: values[1].to_owned(), document_sha256: sha256_hex(input.as_bytes()),
            record_sha256: values[42].to_owned(), nodes,
        })
    }
}
