#![allow(unsafe_code)]

//! Minimal descriptor-relative Unix filesystem operations for the R0 host boundary.
//!
//! Unsafe invariants:
//! - every C pathname is a NUL-free single component produced by `component_name`;
//! - every directory descriptor is owned by a live `File` for the full FFI call;
//! - each non-negative descriptor returned by `openat` is transferred exactly once to
//!   `File::from_raw_fd`, so Rust becomes its sole closer;
//! - `renameat`, `mkdirat`, and `unlinkat` receive valid pointers for the duration of each call;
//! - all flags below are the audited values from the supported macOS and Linux ABIs.

#[cfg(not(unix))]
compile_error!("R0 host safety currently supports Unix hosts only");

use std::ffi::CString;
use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::{c_char, c_int, c_uint};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path};

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
const O_WRONLY: c_int = 1;
const ENOENT: i32 = 2;
const EEXIST: i32 = 17;

unsafe extern "C" {
    fn openat(dirfd: c_int, path: *const c_char, flags: c_int, ...) -> c_int;
    fn mkdirat(dirfd: c_int, path: *const c_char, mode: c_uint) -> c_int;
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
                let result = unsafe { mkdirat(directory.as_raw_fd(), name.as_ptr(), 0o700) };
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
            O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC,
            0o600 as c_uint,
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
