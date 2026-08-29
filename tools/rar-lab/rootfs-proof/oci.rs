//! Pure, non-activating OCI image-document and descriptor resolver.

use super::json::{self, Value};
use super::sha256;
use super::{EffectiveRootfs, Error as LayerError, MAX_LAYER_BYTES};

const LAYOUT_VERSION: &str = "1.0.0";
const INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";
const MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";
const UNCOMPRESSED_LAYER_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar";
const MAX_INDEX_MANIFESTS: usize = 64;
const MAX_MANIFEST_LAYERS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Descriptor {
    media_type: String,
    digest: String,
    size: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Json(json::Error),
    InvalidLayout,
    InvalidIndex,
    InvalidManifest,
    InvalidConfiguration,
    InvalidDescriptor,
    InvalidDigest,
    ManifestNotFound,
    AmbiguousManifest,
    UnsupportedManifestMediaType,
    UnsupportedConfigurationMediaType,
    UnsupportedLayerMediaType,
    TooManyIndexEntries,
    TooManyLayers,
    MissingBlob,
    BlobTooLarge,
    BlobSizeMismatch,
    BlobDigestMismatch,
    PlatformMismatch,
    DiffIdMismatch,
    Layer(LayerError),
    HashFailure,
}

/// Resolves one exact manifest from caller-supplied, content-addressed blobs.
///
/// `blob` must return the complete bytes for the requested lowercase SHA-256
/// digest. This function never accesses a filesystem, network, process, image
/// builder, or emulator.
pub fn resolve_uncompressed_image<'a, F>(
    layout_document: &[u8],
    index_document: &[u8],
    expected_manifest_digest: &str,
    expected_architecture: &str,
    expected_os: &str,
    mut blob: F,
) -> Result<EffectiveRootfs, Error>
where
    F: FnMut(&str) -> Option<&'a [u8]>,
{
    validate_digest(expected_manifest_digest)?;
    validate_layout(layout_document)?;

    let index = json::parse(index_document).map_err(Error::Json)?;
    require_schema_version(&index, Error::InvalidIndex)?;
    require_optional_media_type(&index, INDEX_MEDIA_TYPE, Error::InvalidIndex)?;
    let manifests = index
        .member("manifests")
        .and_then(Value::array)
        .ok_or(Error::InvalidIndex)?;
    if manifests.len() > MAX_INDEX_MANIFESTS {
        return Err(Error::TooManyIndexEntries);
    }

    let mut selected = None;
    for value in manifests {
        let descriptor = parse_descriptor(value)?;
        if descriptor.digest == expected_manifest_digest {
            if selected.is_some() {
                return Err(Error::AmbiguousManifest);
            }
            if descriptor.media_type != MANIFEST_MEDIA_TYPE {
                return Err(Error::UnsupportedManifestMediaType);
            }
            selected = Some(descriptor);
        }
    }
    let manifest_descriptor = selected.ok_or(Error::ManifestNotFound)?;
    let manifest_bytes = require_verified_blob(
        &manifest_descriptor,
        json::MAX_JSON_BYTES,
        &mut blob,
    )?;
    let manifest = json::parse(manifest_bytes).map_err(Error::Json)?;
    require_schema_version(&manifest, Error::InvalidManifest)?;
    require_optional_media_type(&manifest, MANIFEST_MEDIA_TYPE, Error::InvalidManifest)?;

    let config_descriptor = manifest
        .member("config")
        .ok_or(Error::InvalidManifest)
        .and_then(parse_descriptor)?;
    if config_descriptor.media_type != CONFIG_MEDIA_TYPE {
        return Err(Error::UnsupportedConfigurationMediaType);
    }
    let layer_values = manifest
        .member("layers")
        .and_then(Value::array)
        .ok_or(Error::InvalidManifest)?;
    if layer_values.is_empty() || layer_values.len() > MAX_MANIFEST_LAYERS {
        return Err(Error::TooManyLayers);
    }
    let mut layer_descriptors = Vec::with_capacity(layer_values.len());
    for value in layer_values {
        let descriptor = parse_descriptor(value)?;
        if descriptor.media_type != UNCOMPRESSED_LAYER_MEDIA_TYPE {
            return Err(Error::UnsupportedLayerMediaType);
        }
        if descriptor.size > MAX_LAYER_BYTES {
            return Err(Error::BlobTooLarge);
        }
        layer_descriptors.push(descriptor);
    }

    let config_bytes = require_verified_blob(
        &config_descriptor,
        json::MAX_JSON_BYTES,
        &mut blob,
    )?;
    let configuration = json::parse(config_bytes).map_err(Error::Json)?;
    validate_configuration(
        &configuration,
        expected_architecture,
        expected_os,
        &layer_descriptors,
    )?;

    let mut rootfs = EffectiveRootfs::default();
    for descriptor in &layer_descriptors {
        let bytes = require_verified_blob(descriptor, MAX_LAYER_BYTES, &mut blob)?;
        rootfs
            .apply_uncompressed_ustar_layer(bytes)
            .map_err(Error::Layer)?;
    }
    Ok(rootfs)
}

fn validate_layout(document: &[u8]) -> Result<(), Error> {
    let layout = json::parse(document).map_err(Error::Json)?;
    if layout
        .member("imageLayoutVersion")
        .and_then(Value::string)
        != Some(LAYOUT_VERSION)
    {
        return Err(Error::InvalidLayout);
    }
    Ok(())
}

fn require_schema_version(value: &Value, error: Error) -> Result<(), Error> {
    if value
        .member("schemaVersion")
        .and_then(Value::unsigned_integer)
        != Some(2)
    {
        return Err(error);
    }
    Ok(())
}

fn require_optional_media_type(
    value: &Value,
    expected: &str,
    error: Error,
) -> Result<(), Error> {
    if let Some(media_type) = value.member("mediaType") {
        if media_type.string() != Some(expected) {
            return Err(error);
        }
    }
    Ok(())
}

fn parse_descriptor(value: &Value) -> Result<Descriptor, Error> {
    let media_type = value
        .member("mediaType")
        .and_then(Value::string)
        .ok_or(Error::InvalidDescriptor)?;
    let digest = value
        .member("digest")
        .and_then(Value::string)
        .ok_or(Error::InvalidDescriptor)?;
    validate_digest(digest)?;
    let size = value
        .member("size")
        .and_then(Value::unsigned_integer)
        .ok_or(Error::InvalidDescriptor)
        .and_then(|size| usize::try_from(size).map_err(|_| Error::InvalidDescriptor))?;
    Ok(Descriptor {
        media_type: media_type.to_owned(),
        digest: digest.to_owned(),
        size,
    })
}

fn validate_digest(digest: &str) -> Result<(), Error> {
    let encoded = digest.strip_prefix("sha256:").ok_or(Error::InvalidDigest)?;
    if encoded.len() != 64
        || encoded
            .bytes()
            .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(Error::InvalidDigest);
    }
    Ok(())
}

fn require_verified_blob<'a, F>(
    descriptor: &Descriptor,
    maximum_bytes: usize,
    blob: &mut F,
) -> Result<&'a [u8], Error>
where
    F: FnMut(&str) -> Option<&'a [u8]>,
{
    if descriptor.size > maximum_bytes {
        return Err(Error::BlobTooLarge);
    }
    let bytes = blob(&descriptor.digest).ok_or(Error::MissingBlob)?;
    if bytes.len() != descriptor.size {
        return Err(Error::BlobSizeMismatch);
    }
    let actual = sha256::digest_string(bytes).map_err(|_| Error::HashFailure)?;
    if actual != descriptor.digest {
        return Err(Error::BlobDigestMismatch);
    }
    Ok(bytes)
}

fn validate_configuration(
    configuration: &Value,
    expected_architecture: &str,
    expected_os: &str,
    layers: &[Descriptor],
) -> Result<(), Error> {
    if expected_architecture.is_empty() || expected_os.is_empty() {
        return Err(Error::PlatformMismatch);
    }
    if configuration
        .member("architecture")
        .and_then(Value::string)
        != Some(expected_architecture)
        || configuration.member("os").and_then(Value::string) != Some(expected_os)
    {
        return Err(Error::PlatformMismatch);
    }
    let rootfs = configuration
        .member("rootfs")
        .ok_or(Error::InvalidConfiguration)?;
    if rootfs.member("type").and_then(Value::string) != Some("layers") {
        return Err(Error::InvalidConfiguration);
    }
    let diff_ids = rootfs
        .member("diff_ids")
        .and_then(Value::array)
        .ok_or(Error::InvalidConfiguration)?;
    if diff_ids.len() != layers.len() {
        return Err(Error::DiffIdMismatch);
    }
    for (value, layer) in diff_ids.iter().zip(layers) {
        let digest = value.string().ok_or(Error::InvalidConfiguration)?;
        validate_digest(digest)?;
        if digest != layer.digest {
            return Err(Error::DiffIdMismatch);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BLOCK_BYTES;
    use std::collections::BTreeMap;

    struct Fixture {
        layout: Vec<u8>,
        index: Vec<u8>,
        manifest_digest: String,
        blobs: BTreeMap<String, Vec<u8>>,
        first_layer_digest: String,
    }

    fn fixture(layer_media_type: &str, mismatched_diff_id: bool) -> Fixture {
        let first_layer = archive(&[
            ("bin/", b'5', 0o755, b""),
            ("bin/old", b'0', 0o755, b"old"),
            ("usr/lib/object.so", b'0', 0o644, b"\x7fELFobject"),
        ]);
        let second_layer = archive(&[
            ("bin/.wh.old", b'0', 0, b""),
            ("bin/new", b'0', 0o755, b"new"),
        ]);
        let first_layer_digest = sha256::digest_string(&first_layer).unwrap();
        let second_layer_digest = sha256::digest_string(&second_layer).unwrap();
        let first_diff_id = if mismatched_diff_id {
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        } else {
            &first_layer_digest
        };
        let configuration = format!(
            "{{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{{\"type\":\"layers\",\"diff_ids\":[\"{first_diff_id}\",\"{second_layer_digest}\"]}}}}"
        )
        .into_bytes();
        let config_digest = sha256::digest_string(&configuration).unwrap();
        let manifest = format!(
            "{{\"schemaVersion\":2,\"mediaType\":\"{MANIFEST_MEDIA_TYPE}\",\"config\":{{\"mediaType\":\"{CONFIG_MEDIA_TYPE}\",\"digest\":\"{config_digest}\",\"size\":{}}},\"layers\":[{{\"mediaType\":\"{layer_media_type}\",\"digest\":\"{first_layer_digest}\",\"size\":{}}},{{\"mediaType\":\"{layer_media_type}\",\"digest\":\"{second_layer_digest}\",\"size\":{}}}]}}",
            configuration.len(),
            first_layer.len(),
            second_layer.len()
        )
        .into_bytes();
        let manifest_digest = sha256::digest_string(&manifest).unwrap();
        let index = format!(
            "{{\"schemaVersion\":2,\"mediaType\":\"{INDEX_MEDIA_TYPE}\",\"manifests\":[{{\"mediaType\":\"{MANIFEST_MEDIA_TYPE}\",\"digest\":\"{manifest_digest}\",\"size\":{}}}]}}",
            manifest.len()
        )
        .into_bytes();
        let mut blobs = BTreeMap::new();
        blobs.insert(first_layer_digest.clone(), first_layer);
        blobs.insert(second_layer_digest, second_layer);
        blobs.insert(config_digest, configuration);
        blobs.insert(manifest_digest.clone(), manifest);
        Fixture {
            layout: br#"{"imageLayoutVersion":"1.0.0"}"#.to_vec(),
            index,
            manifest_digest,
            blobs,
            first_layer_digest,
        }
    }

    #[test]
    fn resolves_digest_bound_layers_and_configuration_in_order() {
        let fixture = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        let rootfs = resolve_uncompressed_image(
            &fixture.layout,
            &fixture.index,
            &fixture.manifest_digest,
            "amd64",
            "linux",
            |digest| fixture.blobs.get(digest).map(Vec::as_slice),
        )
        .unwrap();
        assert!(rootfs.get("bin/old").is_none());
        assert!(rootfs.get("bin/new").is_some());
        assert!(rootfs.get("usr/lib/object.so").unwrap().is_elf);
    }

    #[test]
    fn rejects_tampered_blobs_before_layer_parsing() {
        let mut fixture = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        fixture
            .blobs
            .get_mut(&fixture.first_layer_digest)
            .unwrap()[0] ^= 1;
        let result = resolve_uncompressed_image(
            &fixture.layout,
            &fixture.index,
            &fixture.manifest_digest,
            "amd64",
            "linux",
            |digest| fixture.blobs.get(digest).map(Vec::as_slice),
        );
        assert_eq!(result, Err(Error::BlobDigestMismatch));
    }

    #[test]
    fn rejects_compression_and_diff_id_mismatch_in_inactive_subset() {
        let compressed = fixture("application/vnd.oci.image.layer.v1.tar+gzip", false);
        let result = resolve_uncompressed_image(
            &compressed.layout,
            &compressed.index,
            &compressed.manifest_digest,
            "amd64",
            "linux",
            |digest| compressed.blobs.get(digest).map(Vec::as_slice),
        );
        assert_eq!(result, Err(Error::UnsupportedLayerMediaType));

        let mismatch = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, true);
        let result = resolve_uncompressed_image(
            &mismatch.layout,
            &mismatch.index,
            &mismatch.manifest_digest,
            "amd64",
            "linux",
            |digest| mismatch.blobs.get(digest).map(Vec::as_slice),
        );
        assert_eq!(result, Err(Error::DiffIdMismatch));
    }

    fn archive(entries: &[(&str, u8, u32, &[u8])]) -> Vec<u8> {
        let mut output = Vec::new();
        for (path, kind, mode, data) in entries {
            let mut header = [0u8; BLOCK_BYTES];
            header[..path.len()].copy_from_slice(path.as_bytes());
            write_octal(&mut header[100..108], u64::from(*mode));
            write_octal(&mut header[108..116], 0);
            write_octal(&mut header[116..124], 0);
            write_octal(&mut header[124..136], data.len() as u64);
            write_octal(&mut header[136..148], 0);
            header[148..156].fill(b' ');
            header[156] = *kind;
            header[257..263].copy_from_slice(b"ustar\0");
            header[263..265].copy_from_slice(b"00");
            let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
            let checksum = format!("{checksum:06o}");
            header[148..154].copy_from_slice(checksum.as_bytes());
            header[154] = 0;
            header[155] = b' ';
            output.extend_from_slice(&header);
            output.extend_from_slice(data);
            let padding = (BLOCK_BYTES - data.len() % BLOCK_BYTES) % BLOCK_BYTES;
            output.resize(output.len() + padding, 0);
        }
        output.resize(output.len() + 2 * BLOCK_BYTES, 0);
        output
    }

    fn write_octal(field: &mut [u8], value: u64) {
        field.fill(b'0');
        field[field.len() - 1] = 0;
        let encoded = format!("{value:o}");
        let start = field.len() - 1 - encoded.len();
        field[start..start + encoded.len()].copy_from_slice(encoded.as_bytes());
    }
}
