//! Pure, non-activating OCI image-document and descriptor resolver.

use super::gzip;
use super::json::{self, Value};
use super::sha256;
use super::{EffectiveRootfs, Error as LayerError, MAX_LAYER_BYTES};

const LAYOUT_VERSION: &str = "1.0.0";
const INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";
const MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";
const UNCOMPRESSED_LAYER_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar";
const GZIP_LAYER_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const MAX_TOTAL_COMPRESSED_LAYER_BYTES: usize = 268_435_456;
const MAX_TOTAL_UNCOMPRESSED_LAYER_BYTES: usize = MAX_LAYER_BYTES;
const MAX_INDEX_MANIFESTS: usize = 64;
const MAX_INDEX_DOCUMENTS: usize = 64;
const MAX_INDEX_DEPTH: usize = 8;
const MAX_MANIFEST_LAYERS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Descriptor {
    media_type: String,
    digest: String,
    size: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LayerEncoding {
    Uncompressed,
    Gzip,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LayerDescriptor {
    descriptor: Descriptor,
    encoding: LayerEncoding,
}

struct LayerBudgets {
    remaining_uncompressed_bytes: usize,
}

impl LayerBudgets {
    fn reserve(layers: &[LayerDescriptor]) -> Result<Self, Error> {
        let mut compressed_bytes = 0usize;
        let mut uncompressed_bytes = 0usize;
        for layer in layers {
            let total = match layer.encoding {
                LayerEncoding::Uncompressed => &mut uncompressed_bytes,
                LayerEncoding::Gzip => &mut compressed_bytes,
            };
            *total = total
                .checked_add(layer.descriptor.size)
                .ok_or(Error::BlobTooLarge)?;
        }
        if compressed_bytes > MAX_TOTAL_COMPRESSED_LAYER_BYTES
            || uncompressed_bytes > MAX_TOTAL_UNCOMPRESSED_LAYER_BYTES
        {
            return Err(Error::BlobTooLarge);
        }
        Ok(Self {
            remaining_uncompressed_bytes: MAX_TOTAL_UNCOMPRESSED_LAYER_BYTES
                - uncompressed_bytes,
        })
    }

    fn consume_decoded(&mut self, bytes: usize) -> Result<(), Error> {
        self.remaining_uncompressed_bytes = self
            .remaining_uncompressed_bytes
            .checked_sub(bytes)
            .ok_or(Error::BlobTooLarge)?;
        Ok(())
    }
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
    TooManyIndexDocuments,
    IndexNestingTooDeep,
    TooManyLayers,
    MissingBlob,
    BlobTooLarge,
    BlobSizeMismatch,
    BlobDigestMismatch,
    PlatformMismatch,
    DiffIdMismatch,
    Gzip(gzip::Error),
    Layer(LayerError),
    HashFailure,
}

/// Supplies one content-addressed blob to a bounded consumer.
///
/// Implementations that allocate or read external storage must enforce
/// `maximum_bytes` before allocation/read and must not invoke `consume` with
/// more bytes than that ceiling. The resolver independently verifies the
/// descriptor size and SHA-256 digest before its own consumer processes bytes.
pub trait BlobSource {
    fn with_blob<T, C>(
        &mut self,
        digest: &str,
        maximum_bytes: usize,
        consume: C,
    ) -> Result<T, Error>
    where
        C: FnOnce(&[u8]) -> Result<T, Error>;
}

struct BorrowedBlobSource<F> {
    blob: F,
}

impl<'a, F> BlobSource for BorrowedBlobSource<F>
where
    F: FnMut(&str) -> Option<&'a [u8]>,
{
    fn with_blob<T, C>(
        &mut self,
        digest: &str,
        maximum_bytes: usize,
        consume: C,
    ) -> Result<T, Error>
    where
        C: FnOnce(&[u8]) -> Result<T, Error>,
    {
        let bytes = (self.blob)(digest).ok_or(Error::MissingBlob)?;
        if bytes.len() > maximum_bytes {
            return Err(Error::BlobTooLarge);
        }
        consume(bytes)
    }
}

/// Resolves one exact manifest from caller-supplied, content-addressed blobs.
///
/// `blob` must return the complete bytes for the requested lowercase SHA-256
/// digest. This function never accesses a filesystem, network, process, image
/// builder, or emulator.
#[cfg(test)]
fn resolve_uncompressed_image<'a, F>(
    layout_document: &[u8],
    index_document: &[u8],
    expected_manifest_digest: &str,
    expected_architecture: &str,
    expected_os: &str,
    blob: F,
) -> Result<EffectiveRootfs, Error>
where
    F: FnMut(&str) -> Option<&'a [u8]>,
{
    let mut source = BorrowedBlobSource { blob };
    resolve_uncompressed_image_from_source(
        layout_document,
        index_document,
        expected_manifest_digest,
        expected_architecture,
        expected_os,
        &mut source,
    )
}

/// Resolves one exact manifest through a source that receives every read bound.
pub fn resolve_uncompressed_image_from_source<S>(
    layout_document: &[u8],
    index_document: &[u8],
    expected_manifest_digest: &str,
    expected_architecture: &str,
    expected_os: &str,
    source: &mut S,
) -> Result<EffectiveRootfs, Error>
where
    S: BlobSource,
{
    validate_digest(expected_manifest_digest)?;
    validate_layout(layout_document)?;

    let index = json::parse(index_document).map_err(Error::Json)?;
    let mut traversal = IndexTraversal { documents: 1 };
    let manifest_descriptor = find_manifest_descriptor(
        &index,
        expected_manifest_digest,
        0,
        &mut traversal,
        source,
    )?
    .ok_or(Error::ManifestNotFound)?;
    let manifest = with_verified_blob(
        &manifest_descriptor,
        json::MAX_JSON_BYTES,
        source,
        |bytes| json::parse(bytes).map_err(Error::Json),
    )?;
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
        let (encoding, maximum_size) = match descriptor.media_type.as_str() {
            UNCOMPRESSED_LAYER_MEDIA_TYPE => (LayerEncoding::Uncompressed, MAX_LAYER_BYTES),
            GZIP_LAYER_MEDIA_TYPE => (LayerEncoding::Gzip, MAX_TOTAL_COMPRESSED_LAYER_BYTES),
            _ => return Err(Error::UnsupportedLayerMediaType),
        };
        if descriptor.size > maximum_size {
            return Err(Error::BlobTooLarge);
        }
        layer_descriptors.push(LayerDescriptor {
            descriptor,
            encoding,
        });
    }
    let mut layer_budgets = LayerBudgets::reserve(&layer_descriptors)?;

    let configuration = with_verified_blob(
        &config_descriptor,
        json::MAX_JSON_BYTES,
        source,
        |bytes| json::parse(bytes).map_err(Error::Json),
    )?;
    let diff_ids = validate_configuration(
        &configuration,
        expected_architecture,
        expected_os,
        &layer_descriptors,
    )?;

    let mut rootfs = EffectiveRootfs::default();
    for (layer, diff_id) in layer_descriptors.iter().zip(&diff_ids) {
        let maximum_size = match layer.encoding {
            LayerEncoding::Uncompressed => MAX_LAYER_BYTES,
            LayerEncoding::Gzip => MAX_TOTAL_COMPRESSED_LAYER_BYTES,
        };
        with_verified_blob(&layer.descriptor, maximum_size, source, |bytes| {
            let decoded;
            let layer_bytes = match layer.encoding {
                LayerEncoding::Uncompressed => bytes,
                LayerEncoding::Gzip => {
                    decoded = gzip::decode_gzip(
                        bytes,
                        layer_budgets.remaining_uncompressed_bytes,
                    )
                    .map_err(Error::Gzip)?;
                    layer_budgets.consume_decoded(decoded.len())?;
                    &decoded
                }
            };
            let actual_diff_id =
                sha256::digest_string(layer_bytes).map_err(|_| Error::HashFailure)?;
            if &actual_diff_id != diff_id {
                return Err(Error::DiffIdMismatch);
            }
            rootfs
                .apply_uncompressed_ustar_layer(layer_bytes)
                .map_err(Error::Layer)
        })?;
    }
    Ok(rootfs)
}

struct IndexTraversal {
    documents: usize,
}

fn find_manifest_descriptor<S>(
    index: &Value,
    expected_manifest_digest: &str,
    depth: usize,
    traversal: &mut IndexTraversal,
    source: &mut S,
) -> Result<Option<Descriptor>, Error>
where
    S: BlobSource,
{
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
            if descriptor.media_type != MANIFEST_MEDIA_TYPE {
                return Err(Error::UnsupportedManifestMediaType);
            }
            record_selection(&mut selected, descriptor.clone())?;
        }
        if descriptor.media_type == INDEX_MEDIA_TYPE {
            if depth >= MAX_INDEX_DEPTH {
                return Err(Error::IndexNestingTooDeep);
            }
            if traversal.documents >= MAX_INDEX_DOCUMENTS {
                return Err(Error::TooManyIndexDocuments);
            }
            // Reserve the document budget before requesting or parsing bytes.
            traversal.documents += 1;
            let child = with_verified_blob(
                &descriptor,
                json::MAX_JSON_BYTES,
                source,
                |bytes| json::parse(bytes).map_err(Error::Json),
            )?;
            if let Some(found) = find_manifest_descriptor(
                &child,
                expected_manifest_digest,
                depth + 1,
                traversal,
                source,
            )? {
                record_selection(&mut selected, found)?;
            }
        }
    }
    Ok(selected)
}

fn record_selection(
    selected: &mut Option<Descriptor>,
    candidate: Descriptor,
) -> Result<(), Error> {
    if selected.is_some() {
        return Err(Error::AmbiguousManifest);
    }
    *selected = Some(candidate);
    Ok(())
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

fn with_verified_blob<S, T, C>(
    descriptor: &Descriptor,
    maximum_bytes: usize,
    source: &mut S,
    consume: C,
) -> Result<T, Error>
where
    S: BlobSource,
    C: FnOnce(&[u8]) -> Result<T, Error>,
{
    if descriptor.size > maximum_bytes {
        return Err(Error::BlobTooLarge);
    }
    source.with_blob(&descriptor.digest, descriptor.size, |bytes| {
        if bytes.len() != descriptor.size {
            return Err(Error::BlobSizeMismatch);
        }
        let actual = sha256::digest_string(bytes).map_err(|_| Error::HashFailure)?;
        if actual != descriptor.digest {
            return Err(Error::BlobDigestMismatch);
        }
        consume(bytes)
    })
}

fn validate_configuration(
    configuration: &Value,
    expected_architecture: &str,
    expected_os: &str,
    layers: &[LayerDescriptor],
) -> Result<Vec<String>, Error> {
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
    let mut validated = Vec::with_capacity(diff_ids.len());
    for value in diff_ids {
        let digest = value.string().ok_or(Error::InvalidConfiguration)?;
        validate_digest(digest)?;
        validated.push(digest.to_owned());
    }
    Ok(validated)
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

    struct RecordingSource<'a> {
        blobs: &'a BTreeMap<String, Vec<u8>>,
        accesses: Vec<(String, usize)>,
    }

    impl BlobSource for RecordingSource<'_> {
        fn with_blob<T, C>(
            &mut self,
            digest: &str,
            maximum_bytes: usize,
            consume: C,
        ) -> Result<T, Error>
        where
            C: FnOnce(&[u8]) -> Result<T, Error>,
        {
            self.accesses.push((digest.to_owned(), maximum_bytes));
            let bytes = self.blobs.get(digest).ok_or(Error::MissingBlob)?;
            if bytes.len() > maximum_bytes {
                return Err(Error::BlobTooLarge);
            }
            consume(bytes)
        }
    }

    fn fixture(layer_media_type: &str, mismatched_diff_id: bool) -> Fixture {
        let first_layer_bytes = archive(&[
            ("bin/", b'5', 0o755, b""),
            ("bin/old", b'0', 0o755, b"old"),
            ("usr/lib/object.so", b'0', 0o644, b"\x7fELFobject"),
        ]);
        let second_layer_bytes = archive(&[
            ("bin/.wh.old", b'0', 0, b""),
            ("bin/new", b'0', 0o755, b"new"),
        ]);
        let first_diff_id = sha256::digest_string(&first_layer_bytes).unwrap();
        let second_diff_id = sha256::digest_string(&second_layer_bytes).unwrap();
        let (first_layer, second_layer) = if layer_media_type == GZIP_LAYER_MEDIA_TYPE {
            (
                gzip_stored_member(&first_layer_bytes),
                gzip_stored_member(&second_layer_bytes),
            )
        } else {
            (first_layer_bytes, second_layer_bytes)
        };
        let first_layer_digest = sha256::digest_string(&first_layer).unwrap();
        let second_layer_digest = sha256::digest_string(&second_layer).unwrap();
        let first_diff_id = if mismatched_diff_id {
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        } else {
            &first_diff_id
        };
        let configuration = format!(
            "{{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{{\"type\":\"layers\",\"diff_ids\":[\"{first_diff_id}\",\"{second_diff_id}\"]}}}}"
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
    fn blob_source_receives_explicit_read_ceilings() {
        let fixture = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        let mut source = RecordingSource {
            blobs: &fixture.blobs,
            accesses: Vec::new(),
        };
        resolve_uncompressed_image_from_source(
            &fixture.layout,
            &fixture.index,
            &fixture.manifest_digest,
            "amd64",
            "linux",
            &mut source,
        )
        .unwrap();

        assert_eq!(source.accesses.len(), 4);
        for (digest, maximum_bytes) in &source.accesses {
            assert_eq!(*maximum_bytes, fixture.blobs[digest].len());
        }
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
    fn resolves_gzip_and_rejects_unknown_compression_and_diff_id_mismatch() {
        let compressed = fixture(GZIP_LAYER_MEDIA_TYPE, false);
        let rootfs = resolve_uncompressed_image(
            &compressed.layout,
            &compressed.index,
            &compressed.manifest_digest,
            "amd64",
            "linux",
            |digest| compressed.blobs.get(digest).map(Vec::as_slice),
        )
        .unwrap();
        assert!(rootfs.get("bin/old").is_none());
        assert!(rootfs.get("bin/new").is_some());

        let unsupported = fixture("application/vnd.oci.image.layer.v1.tar+zstd", false);
        let result = resolve_uncompressed_image(
            &unsupported.layout,
            &unsupported.index,
            &unsupported.manifest_digest,
            "amd64",
            "linux",
            |digest| unsupported.blobs.get(digest).map(Vec::as_slice),
        );
        assert_eq!(result, Err(Error::UnsupportedLayerMediaType));

        let mismatch = fixture(GZIP_LAYER_MEDIA_TYPE, true);
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

    #[test]
    fn aggregate_layer_budgets_accept_exact_limits_and_reject_one_more() {
        let exact = vec![
            synthetic_layer(
                LayerEncoding::Gzip,
                MAX_TOTAL_COMPRESSED_LAYER_BYTES / 2,
            ),
            synthetic_layer(
                LayerEncoding::Gzip,
                MAX_TOTAL_COMPRESSED_LAYER_BYTES / 2,
            ),
            synthetic_layer(
                LayerEncoding::Uncompressed,
                MAX_TOTAL_UNCOMPRESSED_LAYER_BYTES / 2,
            ),
            synthetic_layer(
                LayerEncoding::Uncompressed,
                MAX_TOTAL_UNCOMPRESSED_LAYER_BYTES / 2,
            ),
        ];
        let mut budgets = LayerBudgets::reserve(&exact).unwrap();
        assert_eq!(budgets.remaining_uncompressed_bytes, 0);
        assert_eq!(budgets.consume_decoded(0), Ok(()));

        let compressed_over = vec![
            synthetic_layer(LayerEncoding::Gzip, MAX_TOTAL_COMPRESSED_LAYER_BYTES),
            synthetic_layer(LayerEncoding::Gzip, 1),
        ];
        assert!(matches!(
            LayerBudgets::reserve(&compressed_over),
            Err(Error::BlobTooLarge)
        ));

        let uncompressed_over = vec![
            synthetic_layer(
                LayerEncoding::Uncompressed,
                MAX_TOTAL_UNCOMPRESSED_LAYER_BYTES,
            ),
            synthetic_layer(LayerEncoding::Uncompressed, 1),
        ];
        assert!(matches!(
            LayerBudgets::reserve(&uncompressed_over),
            Err(Error::BlobTooLarge)
        ));

        let mut runtime = LayerBudgets {
            remaining_uncompressed_bytes: 1,
        };
        assert_eq!(runtime.consume_decoded(2), Err(Error::BlobTooLarge));
    }

    #[test]
    fn resolves_exact_manifest_through_verified_nested_index() {
        let mut fixture = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        let child = fixture.index.clone();
        fixture.index = wrap_index(&mut fixture.blobs, child);
        let rootfs = resolve_uncompressed_image(
            &fixture.layout,
            &fixture.index,
            &fixture.manifest_digest,
            "amd64",
            "linux",
            |digest| fixture.blobs.get(digest).map(Vec::as_slice),
        )
        .unwrap();
        assert!(rootfs.get("bin/new").is_some());
    }

    #[test]
    fn rejects_tampered_nested_index_before_parsing() {
        let mut fixture = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        let child = fixture.index.clone();
        let child_digest = sha256::digest_string(&child).unwrap();
        fixture.blobs.insert(child_digest.clone(), child);
        fixture.index = index_for_descriptors(&[(&child_digest, fixture.blobs[&child_digest].len())]);
        fixture.blobs.get_mut(&child_digest).unwrap()[0] ^= 1;

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
    fn rejects_ambiguous_and_overdeep_nested_indexes() {
        let mut ambiguous = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        let child = ambiguous.index.clone();
        let child_digest = sha256::digest_string(&child).unwrap();
        ambiguous.blobs.insert(child_digest.clone(), child);
        let child_size = ambiguous.blobs[&child_digest].len();
        ambiguous.index = index_for_descriptors(&[
            (&child_digest, child_size),
            (&child_digest, child_size),
        ]);
        let result = resolve_uncompressed_image(
            &ambiguous.layout,
            &ambiguous.index,
            &ambiguous.manifest_digest,
            "amd64",
            "linux",
            |digest| ambiguous.blobs.get(digest).map(Vec::as_slice),
        );
        assert_eq!(result, Err(Error::AmbiguousManifest));

        let mut deep = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        for _ in 0..=MAX_INDEX_DEPTH {
            let child = deep.index;
            deep.index = wrap_index(&mut deep.blobs, child);
        }
        let result = resolve_uncompressed_image(
            &deep.layout,
            &deep.index,
            &deep.manifest_digest,
            "amd64",
            "linux",
            |digest| deep.blobs.get(digest).map(Vec::as_slice),
        );
        assert_eq!(result, Err(Error::IndexNestingTooDeep));
    }

    #[test]
    fn rejects_index_document_budget_exhaustion() {
        let mut fixture = fixture(UNCOMPRESSED_LAYER_MEDIA_TYPE, false);
        let empty = format!(
            "{{\"schemaVersion\":2,\"mediaType\":\"{INDEX_MEDIA_TYPE}\",\"manifests\":[]}}"
        )
        .into_bytes();
        let empty_digest = sha256::digest_string(&empty).unwrap();
        let empty_size = empty.len();
        fixture.blobs.insert(empty_digest.clone(), empty);
        let descriptors = vec![(&*empty_digest, empty_size); MAX_INDEX_MANIFESTS];
        fixture.index = index_for_descriptors(&descriptors);

        let mut blob_calls = 0usize;
        let result = resolve_uncompressed_image(
            &fixture.layout,
            &fixture.index,
            &fixture.manifest_digest,
            "amd64",
            "linux",
            |digest| {
                blob_calls += 1;
                fixture.blobs.get(digest).map(Vec::as_slice)
            },
        );
        assert_eq!(result, Err(Error::TooManyIndexDocuments));
        assert_eq!(blob_calls, MAX_INDEX_DOCUMENTS - 1);
    }

    fn wrap_index(blobs: &mut BTreeMap<String, Vec<u8>>, child: Vec<u8>) -> Vec<u8> {
        let child_digest = sha256::digest_string(&child).unwrap();
        let child_size = child.len();
        blobs.insert(child_digest.clone(), child);
        index_for_descriptors(&[(&child_digest, child_size)])
    }

    fn index_for_descriptors(descriptors: &[(&str, usize)]) -> Vec<u8> {
        let descriptors = descriptors
            .iter()
            .map(|(digest, size)| {
                format!(
                    "{{\"mediaType\":\"{INDEX_MEDIA_TYPE}\",\"digest\":\"{digest}\",\"size\":{size}}}"
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schemaVersion\":2,\"mediaType\":\"{INDEX_MEDIA_TYPE}\",\"manifests\":[{descriptors}]}}"
        )
        .into_bytes()
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

    fn gzip_stored_member(bytes: &[u8]) -> Vec<u8> {
        assert!(bytes.len() <= usize::from(u16::MAX));
        let length = bytes.len() as u16;
        let mut output = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255, 1];
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(&(!length).to_le_bytes());
        output.extend_from_slice(bytes);
        output.extend_from_slice(&test_crc32(bytes).to_le_bytes());
        output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        output
    }

    fn synthetic_layer(encoding: LayerEncoding, size: usize) -> LayerDescriptor {
        LayerDescriptor {
            descriptor: Descriptor {
                media_type: String::new(),
                digest: String::new(),
                size,
            },
            encoding,
        }
    }

    fn test_crc32(bytes: &[u8]) -> u32 {
        let mut crc = u32::MAX;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0u32.wrapping_sub(crc & 1);
                crc = (crc >> 1) ^ (0xedb88320 & mask);
            }
        }
        !crc
    }

    fn write_octal(field: &mut [u8], value: u64) {
        field.fill(b'0');
        field[field.len() - 1] = 0;
        let encoded = format!("{value:o}");
        let start = field.len() - 1 - encoded.len();
        field[start..start + encoded.len()].copy_from_slice(encoded.as_bytes());
    }
}
