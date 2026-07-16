#![allow(unsafe_code)]

//! Minimal descriptor-relative Unix filesystem operations for the R0 host boundary.
//!
//! Unsafe invariants:
//! - every C pathname is a NUL-free single component produced by `component_name`;
//! - every directory descriptor is owned by a live `File` for the full FFI call;
//! - each non-negative descriptor returned by `openat` is transferred exactly once to
//!   `File::from_raw_fd`, so Rust becomes its sole closer;
//! - `renameat`, `mkdirat`, and `unlinkat` receive valid pointers for the duration of each call;
//! - fixed `mkdirat` mode values use each platform's `mode_t` width (Darwin `u16`, Linux
//!   `c_uint`); variadic `openat` receives Darwin's required default promotion to `c_int` and
//!   Linux's unpromoted `c_uint`;
//! - all flags and mode types below are the audited values from the supported macOS and Linux
//!   ABIs.

#[cfg(not(unix))]
compile_error!("R0 host safety currently supports Unix hosts only");

use std::ffi::CString;
use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::{c_char, c_int};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path};

#[cfg(target_os = "linux")]
use std::os::raw::c_uint;

use super::{SafetyError, SafetyResult};

#[cfg(target_os = "linux")]
const O_DIRECTORY: c_int = 0o200000;
#[cfg(target_os = "linux")]
const O_NOFOLLOW: c_int = 0o400000;
#[cfg(target_os = "linux")]
const O_CLOEXEC: c_int = 0o2000000;
#[cfg(target_os = "linux")]
const O_CREAT: c_int = 0o100;
#[cfg(target_os = "linux")]
const O_EXCL: c_int = 0o200;

#[cfg(target_os = "macos")]
const O_DIRECTORY: c_int = 0x0010_0000;
#[cfg(target_os = "macos")]
const O_NOFOLLOW: c_int = 0x0000_0100;
#[cfg(target_os = "macos")]
const O_CLOEXEC: c_int = 0x0100_0000;
#[cfg(target_os = "macos")]
const O_CREAT: c_int = 0x0000_0200;
#[cfg(target_os = "macos")]
const O_EXCL: c_int = 0x0000_0800;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("R0 descriptor-relative host I/O is implemented only for macOS and Linux");

const O_RDONLY: c_int = 0;
const O_RDWR: c_int = 2;
const ENOENT: i32 = 2;
const EEXIST: i32 = 17;

#[cfg(target_os = "macos")]
type ModeT = u16;
#[cfg(target_os = "linux")]
type ModeT = c_uint;
#[cfg(target_os = "macos")]
type OpenModeArg = c_int;
#[cfg(target_os = "linux")]
type OpenModeArg = c_uint;

#[cfg(target_os = "macos")]
const _: ModeT = 0_u16;
#[cfg(target_os = "linux")]
const _: ModeT = 0_u32;
#[cfg(target_os = "macos")]
const _: OpenModeArg = 0_i32;
#[cfg(target_os = "linux")]
const _: OpenModeArg = 0_u32;
#[cfg(target_os = "macos")]
const _: [(); std::mem::size_of::<ModeT>()] = [(); std::mem::size_of::<u16>()];
#[cfg(target_os = "linux")]
const _: [(); std::mem::size_of::<ModeT>()] = [(); std::mem::size_of::<c_uint>()];
#[cfg(target_os = "macos")]
const _: [(); std::mem::size_of::<OpenModeArg>()] = [(); std::mem::size_of::<c_int>()];
#[cfg(target_os = "linux")]
const _: [(); std::mem::size_of::<OpenModeArg>()] = [(); std::mem::size_of::<c_uint>()];

unsafe extern "C" {
    fn openat(dirfd: c_int, path: *const c_char, flags: c_int, ...) -> c_int;
    fn mkdirat(dirfd: c_int, path: *const c_char, mode: ModeT) -> c_int;
    fn renameat(
        olddirfd: c_int,
        oldpath: *const c_char,
        newdirfd: c_int,
        newpath: *const c_char,
    ) -> c_int;
    fn unlinkat(dirfd: c_int, path: *const c_char, flags: c_int) -> c_int;
}

fn error(code: &'static str, action: &str, source: io::Error) -> SafetyError {
    SafetyError::new(code, format!("{action}: {source}"))
}

fn component_name(component: &std::ffi::OsStr) -> SafetyResult<CString> {
    let bytes = component.as_bytes();
    if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
        return Err(SafetyError::new(
            "unsafe-path-component",
            "descriptor-relative path component is not canonical",
        ));
    }
    CString::new(bytes).map_err(|_| {
        SafetyError::new(
            "unsafe-path-component",
            "descriptor-relative path component contains NUL",
        )
    })
}

fn file_from_fd(fd: RawFd, action: &str) -> SafetyResult<File> {
    if fd < 0 {
        return Err(error(
            "descriptor-open-failed",
            action,
            io::Error::last_os_error(),
        ));
    }
    // SAFETY: `openat` returned a new owned descriptor and this is its only ownership transfer.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn open_directory_at_raw(parent: &File, name: &CString) -> io::Result<File> {
    // SAFETY: the parent descriptor and NUL-terminated name remain valid for this call.
    let fd = unsafe {
        openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new owned descriptor and this is its only ownership transfer.
    let directory = unsafe { File::from_raw_fd(fd) };
    let metadata = directory.metadata()?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "opened descriptor is not a directory",
        ));
    }
    Ok(directory)
}

fn open_directory_at(parent: &File, name: &CString) -> SafetyResult<File> {
    open_directory_at_raw(parent, name).map_err(|source| {
        error(
            "descriptor-open-failed",
            "open directory without following links",
            source,
        )
    })
}

fn open_root() -> SafetyResult<File> {
    let root = File::open("/")
        .map_err(|source| error("descriptor-open-failed", "open filesystem root", source))?;
    if !root
        .metadata()
        .map_err(|source| {
            error(
                "descriptor-metadata-failed",
                "inspect filesystem root",
                source,
            )
        })?
        .is_dir()
    {
        return Err(SafetyError::new(
            "descriptor-not-directory",
            "filesystem root descriptor is not a directory",
        ));
    }
    Ok(root)
}

pub fn open_absolute_directory_nofollow(path: &Path) -> SafetyResult<File> {
    if !path.is_absolute() {
        return Err(SafetyError::new(
            "descriptor-path-not-absolute",
            "descriptor traversal requires an absolute path",
        ));
    }
    let mut directory = open_root()?;
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                directory = open_directory_at(&directory, &component_name(name)?)?;
            }
            _ => {
                return Err(SafetyError::new(
                    "unsafe-path-component",
                    "absolute path contains a noncanonical component",
                ));
            }
        }
    }
    Ok(directory)
}

pub fn open_absolute_regular_nofollow(path: &Path) -> SafetyResult<File> {
    if !path.is_absolute() {
        return Err(SafetyError::new(
            "descriptor-path-not-absolute",
            "regular-file open requires an absolute path",
        ));
    }
    let parent = path.parent().ok_or_else(|| {
        SafetyError::new("unsafe-path", "regular-file path has no parent directory")
    })?;
    let name = path.file_name().ok_or_else(|| {
        SafetyError::new("unsafe-path", "regular-file path has no final component")
    })?;
    let directory = open_absolute_directory_nofollow(parent)?;
    let name = component_name(name)?;
    // SAFETY: the directory descriptor and NUL-terminated name remain valid for this call.
    let fd = unsafe {
        openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            O_RDONLY | O_NOFOLLOW | O_CLOEXEC,
        )
    };
    let file = file_from_fd(fd, "open regular file without following links")?;
    let metadata = file
        .metadata()
        .map_err(|source| error("descriptor-metadata-failed", "inspect regular file", source))?;
    if !metadata.is_file() {
        return Err(SafetyError::new(
            "descriptor-not-regular",
            "opened descriptor is not a regular file",
        ));
    }
    Ok(file)
}

pub fn open_or_create_relative_directory(root: &Path, relative: &Path) -> SafetyResult<File> {
    let mut directory = open_absolute_directory_nofollow(root)?;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(SafetyError::new(
                "unsafe-path-component",
                "output directory contains a noncanonical component",
            ));
        };
        let name = component_name(name)?;
        match open_directory_at_raw(&directory, &name) {
            Ok(child) => directory = child,
            Err(open_error) if open_error.raw_os_error() == Some(ENOENT) => {
                // SAFETY: the directory descriptor and NUL-terminated name remain valid.
                let result =
                    unsafe { mkdirat(directory.as_raw_fd(), name.as_ptr(), 0o700 as ModeT) };
                let created = result == 0;
                if result != 0 {
                    let source = io::Error::last_os_error();
                    if source.raw_os_error() != Some(EEXIST) {
                        return Err(error(
                            "descriptor-mkdir-failed",
                            "create output directory",
                            source,
                        ));
                    }
                }
                if created {
                    directory.sync_all().map_err(|source| {
                        error(
                            "descriptor-directory-sync-failed",
                            "synchronize newly created directory entry",
                            source,
                        )
                    })?;
                }
                directory = open_directory_at(&directory, &name)?;
            }
            Err(open_error) => {
                return Err(error(
                    "descriptor-open-failed",
                    "open output directory without following links",
                    open_error,
                ));
            }
        }
    }
    Ok(directory)
}

pub fn create_new_file_at(directory: &File, name: &str) -> SafetyResult<File> {
    let name = component_name(std::ffi::OsStr::new(name))?;
    // SAFETY: the directory descriptor and NUL-terminated name remain valid for this call.
    let fd = unsafe {
        openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC,
            0o600 as OpenModeArg,
        )
    };
    if fd < 0 {
        let source = io::Error::last_os_error();
        let code = if source.raw_os_error() == Some(EEXIST) {
            "descriptor-file-exists"
        } else {
            "descriptor-open-failed"
        };
        return Err(error(
            code,
            "create exclusive output temporary file",
            source,
        ));
    }
    // SAFETY: `openat` returned a new owned descriptor and this is its only ownership transfer.
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub fn verify_open_directory_binding(directory: &File, path: &Path) -> SafetyResult<()> {
    let rebound = open_absolute_directory_nofollow(path)?;
    let opened_metadata = directory.metadata().map_err(|source| {
        error(
            "descriptor-metadata-failed",
            "inspect opened output directory",
            source,
        )
    })?;
    let rebound_metadata = rebound.metadata().map_err(|source| {
        error(
            "descriptor-metadata-failed",
            "inspect rebound output directory",
            source,
        )
    })?;
    if opened_metadata.dev() != rebound_metadata.dev()
        || opened_metadata.ino() != rebound_metadata.ino()
    {
        return Err(SafetyError::new(
            "descriptor-directory-replaced",
            "output directory pathname no longer identifies the opened directory",
        ));
    }
    Ok(())
}

pub fn rename_at(directory: &File, old_name: &str, new_name: &str) -> SafetyResult<()> {
    let old_name = component_name(std::ffi::OsStr::new(old_name))?;
    let new_name = component_name(std::ffi::OsStr::new(new_name))?;
    // SAFETY: both names and the directory descriptor remain valid for this call.
    let result = unsafe {
        renameat(
            directory.as_raw_fd(),
            old_name.as_ptr(),
            directory.as_raw_fd(),
            new_name.as_ptr(),
        )
    };
    if result != 0 {
        return Err(error(
            "descriptor-rename-failed",
            "atomically commit output file",
            io::Error::last_os_error(),
        ));
    }
    Ok(())
}

pub fn unlink_at(directory: &File, name: &str) -> SafetyResult<()> {
    let name = component_name(std::ffi::OsStr::new(name))?;
    // SAFETY: the directory descriptor and NUL-terminated name remain valid for this call.
    let result = unsafe { unlinkat(directory.as_raw_fd(), name.as_ptr(), 0) };
    if result != 0 {
        return Err(error(
            "descriptor-unlink-failed",
            "remove temporary output file",
            io::Error::last_os_error(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static FFI_TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_mode_types_match_fixed_and_variadic_c_abi() {
        assert_eq!(std::mem::size_of::<ModeT>(), std::mem::size_of::<u16>());
        assert_eq!(
            std::mem::size_of::<OpenModeArg>(),
            std::mem::size_of::<c_int>()
        );
        assert_eq!(
            std::mem::align_of::<OpenModeArg>(),
            std::mem::align_of::<c_int>()
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_mode_types_match_fixed_and_variadic_c_abi() {
        assert_eq!(std::mem::size_of::<ModeT>(), std::mem::size_of::<c_uint>());
        assert_eq!(
            std::mem::size_of::<OpenModeArg>(),
            std::mem::size_of::<c_uint>()
        );
        assert_eq!(
            std::mem::align_of::<OpenModeArg>(),
            std::mem::align_of::<c_uint>()
        );
    }

    #[test]
    fn ffi_modes_and_failure_paths_are_enforced() {
        let root = PathBuf::from(
            std::env::var("RAR_REPO_ROOT").expect("RAR_REPO_ROOT must be set for FFI tests"),
        );
        let sequence = FFI_TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let relative = PathBuf::from(format!(
            "out/r0/test-state/ffi-mode-{}-{sequence}",
            std::process::id()
        ));
        let directory_path = root.join(&relative);
        let directory = open_or_create_relative_directory(&root, &relative)
            .expect("create descriptor-relative FFI test directory");
        assert_eq!(
            fs::metadata(&directory_path)
                .expect("inspect FFI test directory")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );

        let file = create_new_file_at(&directory, "mode-probe")
            .expect("create descriptor-relative FFI test file");
        assert_eq!(
            file.metadata()
                .expect("inspect FFI test file")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            create_new_file_at(&directory, "mode-probe")
                .expect_err("exclusive openat unexpectedly replaced an existing file")
                .code,
            "descriptor-file-exists"
        );
        assert_eq!(
            rename_at(&directory, "missing-source", "unused-destination")
                .expect_err("renameat unexpectedly accepted a missing source")
                .code,
            "descriptor-rename-failed"
        );
        assert_eq!(
            unlink_at(&directory, "missing-source")
                .expect_err("unlinkat unexpectedly accepted a missing source")
                .code,
            "descriptor-unlink-failed"
        );

        fs::write(directory_path.join("not-a-directory"), b"regular file\n")
            .expect("write non-directory FFI fixture");
        assert_eq!(
            open_or_create_relative_directory(&root, &relative.join("not-a-directory/child"))
                .expect_err("directory open unexpectedly accepted a regular file")
                .code,
            "descriptor-open-failed"
        );

        drop(file);
        fs::remove_file(directory_path.join("mode-probe")).expect("remove FFI mode probe");
        fs::remove_file(directory_path.join("not-a-directory"))
            .expect("remove non-directory FFI fixture");
        fs::remove_dir(directory_path).expect("remove FFI test directory");
    }
}
