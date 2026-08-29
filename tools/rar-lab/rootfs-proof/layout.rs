//! Linux-only, handle-anchored reader for a bounded OCI image layout.

use super::json;
use super::oci::{self, BlobSource};
use super::EffectiveRootfs;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

// Linux values from the stable open(2) ABI. OpenOptionsExt applies them without
// an unsafe call or a third-party libc dependency.
const O_NONBLOCK: i32 = 0o004000;
const O_DIRECTORY: i32 = 0o200000;
const O_NOFOLLOW: i32 = 0o400000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidRoot,
    InvalidLayoutDocument,
    InvalidIndexDocument,
    Resolver(oci::Error),
}

/// Resolves one exact image from a local OCI layout without executing it.
pub fn resolve_uncompressed_layout(
    root: &Path,
    expected_manifest_digest: &str,
    expected_architecture: &str,
    expected_os: &str,
) -> Result<EffectiveRootfs, Error> {
    let mut layout = LayoutDirectory::open(root)?;
    let layout_document = layout
        .read_document(Path::new("oci-layout"))
        .map_err(|_| Error::InvalidLayoutDocument)?;
    let index_document = layout
        .read_document(Path::new("index.json"))
        .map_err(|_| Error::InvalidIndexDocument)?;
    oci::resolve_uncompressed_image_from_source(
        &layout_document,
        &index_document,
        expected_manifest_digest,
        expected_architecture,
        expected_os,
        &mut layout,
    )
    .map_err(Error::Resolver)
}

/// A root-directory handle used to confine all subsequent layout reads.
pub struct LayoutDirectory {
    root: File,
    canonical_root: PathBuf,
}

impl LayoutDirectory {
    pub fn open(root: &Path) -> Result<Self, Error> {
        let lexical_metadata = fs::symlink_metadata(root).map_err(|_| Error::InvalidRoot)?;
        if lexical_metadata.file_type().is_symlink() || !lexical_metadata.is_dir() {
            return Err(Error::InvalidRoot);
        }
        let canonical_root = fs::canonicalize(root).map_err(|_| Error::InvalidRoot)?;
        let handle = OpenOptions::new()
            .read(true)
            .custom_flags(O_DIRECTORY | O_NOFOLLOW | O_NONBLOCK)
            .open(root)
            .map_err(|_| Error::InvalidRoot)?;
        let opened_metadata = handle.metadata().map_err(|_| Error::InvalidRoot)?;
        if !opened_metadata.is_dir()
            || opened_metadata.dev() != lexical_metadata.dev()
            || opened_metadata.ino() != lexical_metadata.ino()
            || canonicalize_fd(handle.as_raw_fd()).as_deref() != Some(canonical_root.as_path())
        {
            return Err(Error::InvalidRoot);
        }
        Ok(Self {
            root: handle,
            canonical_root,
        })
    }

    fn read_document(&self, relative: &Path) -> Result<Vec<u8>, oci::Error> {
        self.read_bounded(relative, json::MAX_JSON_BYTES, None)
    }

    fn read_bounded(
        &self,
        relative: &Path,
        maximum_bytes: usize,
        exact_bytes: Option<usize>,
    ) -> Result<Vec<u8>, oci::Error> {
        if !is_canonical_relative(relative) {
            return Err(oci::Error::MissingBlob);
        }
        let proc_relative = PathBuf::from(format!("/proc/self/fd/{}", self.root.as_raw_fd()))
            .join(relative);
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(O_NOFOLLOW | O_NONBLOCK)
            .open(proc_relative)
            .map_err(|_| oci::Error::MissingBlob)?;
        let metadata = file.metadata().map_err(|_| oci::Error::MissingBlob)?;
        if !metadata.is_file() {
            return Err(oci::Error::MissingBlob);
        }
        let expected_path = self.canonical_root.join(relative);
        if canonicalize_fd(file.as_raw_fd()).as_deref() != Some(expected_path.as_path()) {
            return Err(oci::Error::MissingBlob);
        }
        let metadata_bytes = usize::try_from(metadata.len()).map_err(|_| oci::Error::BlobTooLarge)?;
        if metadata_bytes > maximum_bytes {
            return Err(oci::Error::BlobTooLarge);
        }
        if exact_bytes.is_some_and(|expected| metadata_bytes != expected) {
            return Err(oci::Error::BlobSizeMismatch);
        }

        let mut bytes = Vec::with_capacity(metadata_bytes);
        file.take((maximum_bytes as u64).saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| oci::Error::MissingBlob)?;
        if bytes.len() > maximum_bytes {
            return Err(oci::Error::BlobTooLarge);
        }
        if exact_bytes.is_some_and(|expected| bytes.len() != expected) {
            return Err(oci::Error::BlobSizeMismatch);
        }
        Ok(bytes)
    }
}

impl BlobSource for LayoutDirectory {
    fn with_blob<T, C>(
        &mut self,
        digest: &str,
        maximum_bytes: usize,
        consume: C,
    ) -> Result<T, oci::Error>
    where
        C: FnOnce(&[u8]) -> Result<T, oci::Error>,
    {
        let encoded = digest
            .strip_prefix("sha256:")
            .filter(|value| {
                value.len() == 64
                    && value
                        .bytes()
                        .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
            })
            .ok_or(oci::Error::InvalidDigest)?;
        let relative = Path::new("blobs").join("sha256").join(encoded);
        let bytes = self.read_bounded(&relative, maximum_bytes, Some(maximum_bytes))?;
        consume(&bytes)
    }
}

fn canonicalize_fd(fd: RawFd) -> Option<PathBuf> {
    fs::canonicalize(format!("/proc/self/fd/{fd}")).ok()
}

fn is_canonical_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sha256;
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn reads_exact_regular_blob_through_root_handle() {
        let root = fixture_root("regular");
        let bytes = b"bounded blob";
        let digest = sha256::digest_string(bytes).unwrap();
        write_blob(&root, &digest, bytes);

        let mut layout = LayoutDirectory::open(&root).unwrap();
        let returned = layout
            .with_blob(&digest, bytes.len(), |blob| Ok(blob.to_vec()))
            .unwrap();
        assert_eq!(returned, bytes);
    }

    #[test]
    fn rejects_symlinked_blob_and_symlinked_root() {
        let root = fixture_root("symlink-blob");
        let bytes = b"outside";
        let digest = sha256::digest_string(bytes).unwrap();
        let target = root.join("outside");
        fs::write(&target, bytes).unwrap();
        let blob = blob_path(&root, &digest);
        fs::create_dir_all(blob.parent().unwrap()).unwrap();
        symlink(&target, &blob).unwrap();
        let mut layout = LayoutDirectory::open(&root).unwrap();
        assert_eq!(
            layout.with_blob(&digest, bytes.len(), |_| Ok(())),
            Err(oci::Error::MissingBlob)
        );

        let alias = root.with_extension("alias");
        symlink(&root, &alias).unwrap();
        assert!(matches!(LayoutDirectory::open(&alias), Err(Error::InvalidRoot)));
    }

    #[test]
    fn rejects_intermediate_escape_and_size_mismatch_before_read() {
        let root = fixture_root("escape");
        let outside = fixture_root("outside");
        let bytes = b"oversized";
        let digest = sha256::digest_string(bytes).unwrap();
        write_blob(&outside, &digest, bytes);
        symlink(outside.join("blobs"), root.join("blobs")).unwrap();
        let mut layout = LayoutDirectory::open(&root).unwrap();
        assert_eq!(
            layout.with_blob(&digest, bytes.len(), |_| Ok(())),
            Err(oci::Error::MissingBlob)
        );

        let regular = fixture_root("size");
        write_blob(&regular, &digest, bytes);
        let mut layout = LayoutDirectory::open(&regular).unwrap();
        assert_eq!(
            layout.with_blob(&digest, bytes.len() - 1, |_| Ok(())),
            Err(oci::Error::BlobTooLarge)
        );
    }

    fn fixture_root(label: &str) -> PathBuf {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rar-rootfs-layout-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        root
    }

    fn write_blob(root: &Path, digest: &str, bytes: &[u8]) {
        let path = blob_path(root, digest);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn blob_path(root: &Path, digest: &str) -> PathBuf {
        root.join("blobs")
            .join("sha256")
            .join(digest.strip_prefix("sha256:").unwrap())
    }
}
