#![allow(unsafe_code)]

//! Descriptor-relative disposable disk writer used only inside the pinned Linux build closure.
//!
//! Unsafe invariants: every name passed to libc is a fixed audited single component; every
//! parent fd remains owned by a live `File`; each successful openat fd is transferred exactly
//! once to `File`; `O_NOFOLLOW|O_EXCL` prevents link/collision substitution; `renameat2` uses
//! `RENAME_NOREPLACE`, so the final commit cannot replace an attacker-created destination.

#[cfg(not(target_os = "linux"))]
compile_error!("the preauthorization disk writer is confined to the pinned Linux build closure");

use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::raw::{c_char, c_int, c_uint};

const CLUSTER: u64 = 65_536;
const O_RDONLY: c_int = 0;
const O_RDWR: c_int = 2;
const O_DIRECTORY: c_int = 0o200000;
const O_NOFOLLOW: c_int = 0o400000;
const O_CLOEXEC: c_int = 0o2000000;
const O_CREAT: c_int = 0o100;
const O_EXCL: c_int = 0o200;
const RENAME_NOREPLACE: c_uint = 1;

unsafe extern "C" {
    fn openat(dirfd: c_int, path: *const c_char, flags: c_int, ...) -> c_int;
    fn mkdirat(dirfd: c_int, path: *const c_char, mode: c_uint) -> c_int;
    fn renameat2(oldfd: c_int, old: *const c_char, newfd: c_int, new: *const c_char, flags: c_uint) -> c_int;
    fn unlinkat(dirfd: c_int, path: *const c_char, flags: c_int) -> c_int;
}

fn name(value: &str) -> Result<CString, Box<dyn std::error::Error>> {
    if value.is_empty() || value == "." || value == ".." || value.contains(['/', '\\', ':']) {
        return Err("noncanonical disk component".into());
    }
    Ok(CString::new(value)?)
}

fn open_dir(parent: &File, component: &str, create: bool) -> Result<File, Box<dyn std::error::Error>> {
    let component = name(component)?;
    let mut fd = unsafe { openat(parent.as_raw_fd(), component.as_ptr(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC) };
    if fd < 0 && create && io::Error::last_os_error().raw_os_error() == Some(2) {
        if unsafe { mkdirat(parent.as_raw_fd(), component.as_ptr(), 0o700) } != 0
            && io::Error::last_os_error().raw_os_error() != Some(17) { return Err(io::Error::last_os_error().into()); }
        fd = unsafe { openat(parent.as_raw_fd(), component.as_ptr(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC) };
    }
    if fd < 0 { return Err(io::Error::last_os_error().into()); }
    let file = unsafe { File::from_raw_fd(fd) };
    if !file.metadata()?.is_dir() { return Err("disk ancestor is not a directory".into()); }
    Ok(file)
}

fn put_u32(buffer: &mut [u8], offset: usize, value: u32) { buffer[offset..offset + 4].copy_from_slice(&value.to_be_bytes()); }
fn put_u64(buffer: &mut [u8], offset: usize, value: u64) { buffer[offset..offset + 8].copy_from_slice(&value.to_be_bytes()); }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RAR_PREAUTH_BUILD_CONTAINER").as_deref() != Ok("rar-preauth-closure-v3") {
        return Err("refusing outside the pinned Prompt 7A build container".into());
    }
    let path = env::args().nth(1).ok_or("one output path is required")?;
    let prefix = "/workspace/out/r0/vm/x86_64/";
    let final_name = path.strip_prefix(prefix).ok_or("unsafe disposable disk path")?;
    if !final_name.ends_with(".qcow2") || final_name.contains('/') { return Err("unsafe disposable disk path".into()); }

    let mut directory = File::open("/workspace")?;
    for component in ["out", "r0", "vm", "x86_64"] { directory = open_dir(&directory, component, true)?; }
    let temporary_name = format!(".{final_name}.partial-{}", std::process::id());
    let temporary = name(&temporary_name)?;
    let fd = unsafe { openat(directory.as_raw_fd(), temporary.as_ptr(), O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0o600_u32) };
    if fd < 0 { return Err(io::Error::last_os_error().into()); }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut header = [0_u8; 104];
        put_u32(&mut header, 0, 0x5146_49fb); put_u32(&mut header, 4, 3); put_u32(&mut header, 20, 16);
        put_u64(&mut header, 24, 64 * 1024 * 1024); put_u32(&mut header, 36, 1);
        put_u64(&mut header, 40, 3 * CLUSTER); put_u64(&mut header, 48, CLUSTER);
        put_u32(&mut header, 56, 1); put_u32(&mut header, 96, 4); put_u32(&mut header, 100, 104);
        file.write_all(&header)?; file.seek(SeekFrom::Start(CLUSTER))?; file.write_all(&(2 * CLUSTER).to_be_bytes())?;
        file.seek(SeekFrom::Start(2 * CLUSTER))?; for _ in 0..4 { file.write_all(&1_u16.to_be_bytes())?; }
        file.set_len(4 * CLUSTER)?; file.sync_all()?; drop(file);
        let final_component = name(final_name)?;
        if unsafe { renameat2(directory.as_raw_fd(), temporary.as_ptr(), directory.as_raw_fd(), final_component.as_ptr(), RENAME_NOREPLACE) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
        directory.sync_all()?; Ok(())
    })();
    if result.is_err() { unsafe { unlinkat(directory.as_raw_fd(), temporary.as_ptr(), 0) }; }
    result
}
