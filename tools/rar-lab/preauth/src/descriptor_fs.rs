//! Descriptor-relative filesystem primitives for `preauth-transaction`.
//!
//! # Unsafe invariants
//!
//! The FFI calls below are the only unsafe code in the module.  Every pathname
//! passed to them is a previously validated single normal component represented by
//! a NUL-terminated `CString`; no slash, NUL, dot, or dot-dot component is allowed.
//! Every directory FD is owned by a live `File`.  Returned FDs are checked for
//! failure before `File::from_raw_fd` assumes ownership exactly once.  `openat`
//! always uses `O_NOFOLLOW|O_CLOEXEC`; directory walks additionally use
//! `O_DIRECTORY`.  Publication uses the platform's atomic no-replace primitive and
//! then fsyncs the held parent descriptor.

#![allow(unsafe_code)]

use std::ffi::{CString, c_char, c_int, c_uint};
use std::fs::{File, OpenOptions, Permissions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use super::{OwnedSnapshot, PreauthError, Result, sha256_reader};

#[cfg(target_os = "linux")]
const O_CLOEXEC: c_int = 0o2000000;
#[cfg(target_os = "linux")]
const O_DIRECTORY: c_int = 0o200000;
#[cfg(target_os = "linux")]
const O_NOFOLLOW: c_int = 0o400000;
#[cfg(target_os = "linux")]
const O_CREAT: c_int = 0o100;
#[cfg(target_os = "linux")]
const O_EXCL: c_int = 0o200;

#[cfg(target_os = "macos")]
const O_CLOEXEC: c_int = 0x01000000;
#[cfg(target_os = "macos")]
const O_DIRECTORY: c_int = 0x00100000;
#[cfg(target_os = "macos")]
const O_NOFOLLOW: c_int = 0x00000100;
#[cfg(target_os = "macos")]
const O_CREAT: c_int = 0x00000200;
#[cfg(target_os = "macos")]
const O_EXCL: c_int = 0x00000800;

const O_RDONLY: c_int = 0;
const O_RDWR: c_int = 2;

unsafe extern "C" {
    fn openat(directory: c_int, path: *const c_char, flags: c_int, mode: c_uint) -> c_int;
    fn mkdirat(directory: c_int, path: *const c_char, mode: c_uint) -> c_int;
    fn unlinkat(directory: c_int, path: *const c_char, flags: c_int) -> c_int;
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn renameat2(
        old_directory: c_int, old_path: *const c_char,
        new_directory: c_int, new_path: *const c_char, flags: c_uint,
    ) -> c_int;
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn renameatx_np(
        old_directory: c_int, old_path: *const c_char,
        new_directory: c_int, new_path: *const c_char, flags: c_uint,
    ) -> c_int;
}

#[cfg(target_os = "linux")]
const RENAME_NOREPLACE: c_uint = 1;
#[cfg(target_os = "macos")]
const RENAME_NOREPLACE: c_uint = 0x00000004;
const AT_REMOVEDIR: c_int = 0x200;

fn component(value: &str) -> io::Result<CString> {
    if value.is_empty() || value == "." || value == ".." || value.contains('/')
        || value.contains('\\') || value.contains(':') || value.len() > 255
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid path component"));
    }
    CString::new(value).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL component"))
}

fn opened(fd: c_int) -> io::Result<File> {
    if fd < 0 { Err(io::Error::last_os_error()) }
    else {
        // SAFETY: `fd` is a fresh successful return from openat and ownership is
        // transferred to this File exactly once.
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

fn open_at(directory: &File, name: &str, flags: c_int, mode: c_uint) -> io::Result<File> {
    let name = component(name)?;
    // SAFETY: directory is live and owned; name is one validated CString component.
    opened(unsafe { openat(directory.as_raw_fd(), name.as_ptr(), flags, mode) })
}

#[derive(Debug)]
pub struct DescriptorDir(File);

impl DescriptorDir {
    pub fn open_root(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().read(true)
            .custom_flags(O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC).open(path)?;
        Ok(Self(file))
    }

    pub fn open_dir(&self, name: &str) -> io::Result<Self> {
        open_at(&self.0, name, O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC, 0).map(Self)
    }

    pub fn open_file(&self, name: &str) -> io::Result<File> {
        open_at(&self.0, name, O_RDONLY | O_NOFOLLOW | O_CLOEXEC, 0)
    }

    pub fn open_relative_file(&self, path: &str) -> io::Result<File> {
        if path.is_empty() || path.starts_with('/') || path.contains('\\') || path.contains(':') {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid relative path"));
        }
        let mut parts = path.split('/').peekable();
        let first = parts.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty path"))?;
        if parts.peek().is_none() { return self.open_file(first); }
        let mut directory = self.open_dir(first)?;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() { return directory.open_file(part); }
            directory = directory.open_dir(part)?;
        }
        Err(io::Error::new(io::ErrorKind::InvalidInput, "path names a directory"))
    }

    pub fn create_private_dir(&self, name: &str) -> io::Result<Self> {
        let name_c = component(name)?;
        // SAFETY: self is a live directory and name_c is one validated component.
        if unsafe { mkdirat(self.0.as_raw_fd(), name_c.as_ptr(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        match self.open_dir(name) {
            Ok(directory) => Ok(directory),
            Err(error) => {
                // SAFETY: removes only the exact just-created empty component.
                let _ = unsafe { unlinkat(self.0.as_raw_fd(), name_c.as_ptr(), AT_REMOVEDIR) };
                Err(error)
            }
        }
    }

    pub fn create_exclusive_file(&self, name: &str) -> io::Result<File> {
        let file = open_at(
            &self.0, name, O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0o600,
        )?;
        file.set_permissions(Permissions::from_mode(0o600))?;
        Ok(file)
    }

    pub fn sync(&self) -> io::Result<()> { self.0.sync_all() }

    pub fn publish_no_replace(&self, private_name: &str, final_name: &str) -> io::Result<()> {
        let private_name = component(private_name)?;
        let final_name = component(final_name)?;
        #[cfg(target_os = "linux")]
        // SAFETY: both names are validated components beneath the same held parent.
        let status = unsafe { renameat2(
            self.0.as_raw_fd(), private_name.as_ptr(), self.0.as_raw_fd(), final_name.as_ptr(),
            RENAME_NOREPLACE,
        ) };
        #[cfg(target_os = "macos")]
        // SAFETY: both names are validated components beneath the same held parent.
        let status = unsafe { renameatx_np(
            self.0.as_raw_fd(), private_name.as_ptr(), self.0.as_raw_fd(), final_name.as_ptr(),
            RENAME_NOREPLACE,
        ) };
        if status != 0 { return Err(io::Error::last_os_error()); }
        self.sync()
    }
}

#[derive(Debug)]
pub struct HeldSnapshot {
    pub evidence: OwnedSnapshot,
    pub file: File,
}

/// Copies from an already opened no-follow source descriptor into one private
/// exclusive file.  All parsing and later consumption must use the returned file,
/// never the source pathname or source descriptor again.
pub fn snapshot_to_private(
    source: &mut File, private: &DescriptorDir, slot: &str, expected_sha256: &str,
    maximum: u64,
) -> Result<HeldSnapshot> {
    use std::os::unix::fs::MetadataExt;

    let before = source.metadata().map_err(|_| PreauthError::new("snapshot-source-stat"))?;
    if !before.is_file() || before.nlink() != 1 || before.len() > maximum {
        return Err(PreauthError::new("snapshot-source-identity"));
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| PreauthError::new("snapshot-source-seek"))?;
    let mut private_file = private.create_exclusive_file(slot)
        .map_err(|_| PreauthError::new("snapshot-private-create"))?;
    let copied = io::copy(&mut (&mut *source).take(maximum.saturating_add(1)), &mut private_file)
        .map_err(|_| PreauthError::new("snapshot-copy"))?;
    if copied > maximum || copied != before.len() {
        return Err(PreauthError::new("snapshot-size-changed"));
    }
    private_file.flush().map_err(|_| PreauthError::new("snapshot-flush"))?;
    private_file.sync_all().map_err(|_| PreauthError::new("snapshot-fsync"))?;
    let after = source.metadata().map_err(|_| PreauthError::new("snapshot-source-stat"))?;
    let before_identity = format!("{}:{}:{}:{}", before.dev(), before.ino(), before.len(), before.mtime_nsec());
    let after_identity = format!("{}:{}:{}:{}", after.dev(), after.ino(), after.len(), after.mtime_nsec());
    if before_identity != after_identity || after.nlink() != 1 {
        return Err(PreauthError::new("snapshot-source-mutated"));
    }
    private_file.seek(SeekFrom::Start(0)).map_err(|_| PreauthError::new("snapshot-private-seek"))?;
    let digest = sha256_reader(&mut private_file).map_err(|_| PreauthError::new("snapshot-private-hash"))?;
    if digest != expected_sha256 {
        return Err(PreauthError::new("snapshot-digest-mismatch"));
    }
    private_file.set_permissions(Permissions::from_mode(0o400))
        .map_err(|_| PreauthError::new("snapshot-seal"))?;
    private_file.seek(SeekFrom::Start(0)).map_err(|_| PreauthError::new("snapshot-private-seek"))?;
    Ok(HeldSnapshot {
        evidence: OwnedSnapshot {
            slot: slot.to_owned(), digest, byte_len: copied, private_exclusive: true,
            writable_aliases: 0, source_identity_before: before_identity,
            source_identity_after: after_identity, source_link_count: after.nlink() as u32,
        },
        file: private_file,
    })
}
