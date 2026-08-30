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
const O_PATH: i32 = 0o10000000;

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
        // Inspect through O_PATH first: opening a hostile device or FIFO for
        // ordinary reads can itself have side effects or block.
        let path_handle = OpenOptions::new()
            .read(true)
            .custom_flags(O_PATH | O_NOFOLLOW | O_NONBLOCK)
            .open(proc_relative)
            .map_err(|_| oci::Error::MissingBlob)?;
        let metadata = path_handle
            .metadata()
            .map_err(|_| oci::Error::MissingBlob)?;
        if !metadata.is_file() {
            return Err(oci::Error::MissingBlob);
        }
        let expected_path = self.canonical_root.join(relative);
        if canonicalize_fd(path_handle.as_raw_fd()).as_deref() != Some(expected_path.as_path()) {
            return Err(oci::Error::MissingBlob);
        }
        let metadata_bytes = usize::try_from(metadata.len()).map_err(|_| oci::Error::BlobTooLarge)?;
        if metadata_bytes > maximum_bytes {
            return Err(oci::Error::BlobTooLarge);
        }
        if exact_bytes.is_some_and(|expected| metadata_bytes != expected) {
            return Err(oci::Error::BlobSizeMismatch);
        }

        // Reopen only the already-verified O_PATH object. This proc path is
        // derived solely from our descriptor, never from layout-controlled text.
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(O_NONBLOCK)
            .open(format!("/proc/self/fd/{}", path_handle.as_raw_fd()))
            .map_err(|_| oci::Error::MissingBlob)?;
        let read_metadata = file.metadata().map_err(|_| oci::Error::MissingBlob)?;
        if !read_metadata.is_file()
            || read_metadata.dev() != metadata.dev()
            || read_metadata.ino() != metadata.ino()
            || canonicalize_fd(file.as_raw_fd()).as_deref() != Some(expected_path.as_path())
        {
            return Err(oci::Error::MissingBlob);
        }
        let bytes = read_exact_bytes(&mut file, metadata_bytes)?;
        let final_metadata = file.metadata().map_err(|_| oci::Error::MissingBlob)?;
        if final_metadata.dev() != metadata.dev()
            || final_metadata.ino() != metadata.ino()
            || final_metadata.len() != metadata.len()
        {
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

fn read_exact_bytes<R: Read>(reader: &mut R, exact_bytes: usize) -> Result<Vec<u8>, oci::Error> {
    let mut bytes = vec![0_u8; exact_bytes];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| oci::Error::BlobSizeMismatch)?;
    Ok(bytes)
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
    use std::io;
    use std::os::unix::net::UnixListener;
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
    fn resolves_zstd_oci_layout_through_confined_reader() {
        let root = fixture_root("zstd-oci");
        let layer_bytes = archive("opt/rar-lab/qemu", 0o755, b"bounded-qemu");
        let diff_id = sha256::digest_string(&layer_bytes).unwrap();
        let layer = zstd_raw_frame(&layer_bytes);
        let layer_digest = sha256::digest_string(&layer).unwrap();

        let configuration = format!(
            r#"{{"architecture":"amd64","os":"linux","rootfs":{{"type":"layers","diff_ids":["{diff_id}"]}}}}"#
        )
        .into_bytes();
        let configuration_digest = sha256::digest_string(&configuration).unwrap();
        let manifest = format!(
            r#"{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{{"mediaType":"application/vnd.oci.image.config.v1+json","digest":"{configuration_digest}","size":{}}},"layers":[{{"mediaType":"application/vnd.oci.image.layer.v1.tar+zstd","digest":"{layer_digest}","size":{}}}]}}"#,
            configuration.len(),
            layer.len()
        )
        .into_bytes();
        let manifest_digest = sha256::digest_string(&manifest).unwrap();
        let index = format!(
            r#"{{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"{manifest_digest}","size":{}}}]}}"#,
            manifest.len()
        );

        fs::write(
            root.join("oci-layout"),
            br#"{"imageLayoutVersion":"1.0.0"}"#,
        )
        .unwrap();
        fs::write(root.join("index.json"), index).unwrap();
        write_blob(&root, &configuration_digest, &configuration);
        write_blob(&root, &manifest_digest, &manifest);
        write_blob(&root, &layer_digest, &layer);

        let rootfs =
            resolve_uncompressed_layout(&root, &manifest_digest, "amd64", "linux").unwrap();
        let qemu = rootfs.get("opt/rar-lab/qemu").unwrap();
        assert!(qemu.is_executable_or_loadable());
        assert_eq!(qemu.bytes, 12);
        assert_eq!(
            qemu.content_digest.as_deref(),
            Some(sha256::digest_string(b"bounded-qemu").unwrap().as_str())
        );
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

    #[test]
    fn exact_reader_never_requests_bytes_beyond_ceiling() {
        struct CountingReader {
            bytes: Vec<u8>,
            offset: usize,
            returned: usize,
        }

        impl Read for CountingReader {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                let remaining = &self.bytes[self.offset..];
                let count = remaining.len().min(output.len());
                output[..count].copy_from_slice(&remaining[..count]);
                self.offset += count;
                self.returned += count;
                Ok(count)
            }
        }

        let mut reader = CountingReader {
            bytes: b"four".to_vec(),
            offset: 0,
            returned: 0,
        };
        assert_eq!(read_exact_bytes(&mut reader, 3).unwrap(), b"fou");
        assert_eq!(reader.returned, 3);
    }

    #[test]
    fn rejects_special_file_after_path_only_inspection() {
        // Keep the complete socket path below Linux sockaddr_un::sun_path.
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("rl{}x{serial}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let digest = sha256::digest_string(b"socket").unwrap();
        let path = blob_path(&root, &digest);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let _socket = UnixListener::bind(&path).unwrap();
        let mut layout = LayoutDirectory::open(&root).unwrap();
        assert_eq!(
            layout.with_blob(&digest, 6, |_| Ok(())),
            Err(oci::Error::MissingBlob)
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

    fn archive(path: &str, mode: u32, data: &[u8]) -> Vec<u8> {
        const BLOCK_BYTES: usize = 512;
        let mut header = [0u8; BLOCK_BYTES];
        header[..path.len()].copy_from_slice(path.as_bytes());
        write_octal(&mut header[100..108], u64::from(mode));
        write_octal(&mut header[108..116], 0);
        write_octal(&mut header[116..124], 0);
        write_octal(&mut header[124..136], data.len() as u64);
        write_octal(&mut header[136..148], 0);
        header[148..156].fill(b' ');
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
        let checksum = format!("{checksum:06o}");
        header[148..154].copy_from_slice(checksum.as_bytes());
        header[154] = 0;
        header[155] = b' ';

        let mut output = header.to_vec();
        output.extend_from_slice(data);
        let padding = (BLOCK_BYTES - data.len() % BLOCK_BYTES) % BLOCK_BYTES;
        output.resize(output.len() + padding + 2 * BLOCK_BYTES, 0);
        output
    }

    fn write_octal(field: &mut [u8], value: u64) {
        field.fill(b'0');
        field[field.len() - 1] = 0;
        let encoded = format!("{value:o}");
        let start = field.len() - 1 - encoded.len();
        field[start..start + encoded.len()].copy_from_slice(encoded.as_bytes());
    }

    fn zstd_raw_frame(bytes: &[u8]) -> Vec<u8> {
        assert!((256..=65_791).contains(&bytes.len()));
        let mut output = vec![0x28, 0xb5, 0x2f, 0xfd, 0x60];
        output.extend_from_slice(&((bytes.len() - 256) as u16).to_le_bytes());
        let header = ((bytes.len() as u32) << 3) | 1;
        output.extend_from_slice(&header.to_le_bytes()[..3]);
        output.extend_from_slice(bytes);
        output
    }
}
