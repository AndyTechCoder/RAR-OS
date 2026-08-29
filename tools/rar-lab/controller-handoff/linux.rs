/*!
The sole Linux syscall boundary for the controller handoff library.

This module consumes directory descriptors that the trusted outer controller
already opened. It never resolves a root path, starts a process, accesses a
network, or launches a target. All child access is relative to a retained root
descriptor and every returned descriptor is owned and closed on drop.

# Unsafe invariants

- This module is compiled only for the pinned x86_64 Linux controller target;
  syscall 217 is getdents64 for that ABI.
- FFI pointers reference live CString, fixed buffer, or Timespec storage for
  the complete call and the kernel receives the exact allocation length.
- A nonnegative openat result is a newly owned descriptor and is converted to
  OwnedFd exactly once.
- Directory records are parsed only after checked kernel byte counts, minimum
  record lengths, bounded record ends, and an in-record NUL terminator.
- Destination and manifest roots are controller-owned mode-0700 directories.
  No other actor may mutate them during a transaction; this makes the required
  identity-check-then-unlinkat cleanup sequence safe despite Linux lacking an
  unlink-by-descriptor primitive.
*/

use std::ffi::{CStr, CString, c_char, c_int, c_long, c_uint, c_void};
use std::fs::{File, Metadata};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::fs::MetadataExt;

use crate::{FileIdentity, FileType, HandoffError, HandoffOps, HandoffResult, RootIdentity};

const O_RDONLY: c_int = 0;
const O_RDWR: c_int = 2;
const O_CREAT: c_int = 0o100;
const O_EXCL: c_int = 0o200;
const O_NONBLOCK: c_int = 0o4000;
const O_CLOEXEC: c_int = 0o2000000;
const O_NOFOLLOW: c_int = 0o400000;
const MODE_0600: c_uint = 0o600;
const CLOCK_MONOTONIC: c_int = 1;
const SYS_GETDENTS64_X86_64: c_long = 217;
const DIRENT64_HEADER_BYTES: usize = 19;
const DIRENT_BUFFER_BYTES: usize = 16 * 1024;
const MAX_ENUMERATION_ENTRIES: usize = 1_001;
const MAX_ENUMERATION_NAME_BYTES: usize = 64 * 999 + 3;
const MAX_INTERRUPTED_RETRIES: usize = 16;
const F_GETFD: c_int = 1;
const F_SETFD: c_int = 2;
const FD_CLOEXEC: c_int = 1;

#[repr(C)]
struct Timespec {
    seconds: c_long,
    nanoseconds: c_long,
}

unsafe extern "C" {
    fn openat(directory_fd: c_int, pathname: *const c_char, flags: c_int, mode: c_uint) -> c_int;
    fn unlinkat(directory_fd: c_int, pathname: *const c_char, flags: c_int) -> c_int;
    fn syscall(number: c_long, ...) -> c_long;
    fn clock_gettime(clock_id: c_int, time: *mut Timespec) -> c_int;
    fn geteuid() -> c_uint;
    fn fcntl(descriptor: c_int, command: c_int, ...) -> c_int;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LinuxRootPurpose {
    Source,
    Destination,
    Manifest,
}

/// Controller-issued proof bound to one already-open root identity.
///
/// Only the future trusted controller module may construct this token, after it
/// has created the task root without following links, stopped and removed the
/// producer where applicable, removed its mount, and retained the sole root FD.
pub(crate) struct ControllerRootAttestation {
    device: u64,
    inode: u64,
    purpose: LinuxRootPurpose,
    _sealed: (),
}

impl ControllerRootAttestation {
    pub(crate) fn from_controller_observation(
        device: u64,
        inode: u64,
        purpose: LinuxRootPurpose,
    ) -> Self {
        Self { device, inode, purpose, _sealed: () }
    }
}

pub(crate) struct LinuxRoot {
    directory: File,
    purpose: LinuxRootPurpose,
    initial_identity: RootIdentity,
}

impl LinuxRoot {
    /// Takes ownership of an attested, already-open controller root descriptor.
    ///
    /// The identity binding prevents a token for one root from authorizing a
    /// substituted descriptor. handoff_batch independently validates mode,
    /// ownership, directory type, and pairwise non-aliasing before use.
    pub(crate) fn from_verified_owned_fd(
        descriptor: OwnedFd,
        attestation: ControllerRootAttestation,
    ) -> HandoffResult<Self> {
        // SAFETY: F_GETFD takes no variadic argument and does not transfer or
        // mutate descriptor ownership.
        let descriptor_flags = unsafe { fcntl(descriptor.as_raw_fd(), F_GETFD) };
        if descriptor_flags < 0 || descriptor_flags & FD_CLOEXEC == 0 {
            return Err(failure("root-not-cloexec", "root-adopt"));
        }
        let directory = File::from(descriptor);
        let metadata = directory.metadata().map_err(|_| io_failure("root-adopt-stat"))?;
        let initial_identity = root_identity(&metadata);
        if initial_identity.device != attestation.device || initial_identity.inode != attestation.inode {
            return Err(failure("root-attestation-mismatch", "root-adopt"));
        }
        Ok(Self { directory, purpose: attestation.purpose, initial_identity })
    }
}

pub(crate) struct LinuxFile {
    file: File,
}

pub(crate) struct LinuxOps {
    controller_uid: u32,
}

impl LinuxOps {
    pub(crate) fn new() -> Self {
        // SAFETY: geteuid has no pointer arguments and no preconditions.
        let controller_uid = unsafe { geteuid() };
        Self { controller_uid }
    }
}

impl Default for LinuxOps {
    fn default() -> Self { Self::new() }
}

fn failure(code: &'static str, stage: &'static str) -> HandoffError {
    HandoffError::new(code, stage)
}

fn io_failure(stage: &'static str) -> HandoffError {
    failure("linux-io", stage)
}

fn child_name(basename: &str, stage: &'static str) -> HandoffResult<CString> {
    if basename.is_empty() || basename == "." || basename == ".." || basename.as_bytes().contains(&b'/') {
        return Err(failure("invalid-basename", stage));
    }
    CString::new(basename).map_err(|_| failure("invalid-basename", stage))
}

fn open_child(root: &LinuxRoot, basename: &str, flags: c_int, mode: c_uint, stage: &'static str) -> HandoffResult<LinuxFile> {
    let name = child_name(basename, stage)?;
    // SAFETY: name is live and NUL-terminated, the root descriptor is owned,
    // and a successful result transfers one new descriptor to this function.
    let descriptor = unsafe { openat(root.directory.as_raw_fd(), name.as_ptr(), flags, mode) };
    if descriptor < 0 { return Err(io_failure(stage)); }
    // SAFETY: a successful openat returns a new owned descriptor. This is its
    // sole conversion, and File closes it exactly once on drop.
    let owned = unsafe { OwnedFd::from_raw_fd(descriptor) };
    Ok(LinuxFile { file: File::from(owned) })
}

fn require_purpose(root: &LinuxRoot, expected: LinuxRootPurpose, stage: &'static str) -> HandoffResult<()> {
    if root.purpose != expected { return Err(failure("root-purpose-mismatch", stage)); }
    Ok(())
}

fn current_root_identity(root: &LinuxRoot, stage: &'static str) -> HandoffResult<RootIdentity> {
    root.directory.metadata().map(|metadata| root_identity(&metadata)).map_err(|_| io_failure(stage))
}

fn require_initial_root(root: &LinuxRoot, stage: &'static str) -> HandoffResult<()> {
    if current_root_identity(root, stage)? != root.initial_identity {
        return Err(failure("root-identity-changed", stage));
    }
    Ok(())
}

fn descriptor_is_cloexec(descriptor: c_int) -> HandoffResult<bool> {
    // SAFETY: F_GETFD takes no variadic argument and does not transfer or
    // mutate descriptor ownership.
    let flags = unsafe { fcntl(descriptor, F_GETFD) };
    if flags < 0 { return Err(io_failure("descriptor-flags")); }
    Ok(flags & FD_CLOEXEC != 0)
}

fn rollback_unjournaled_creation(root: &mut LinuxRoot, basename: &str) -> HandoffResult<()> {
    let name = child_name(basename, "create-rollback-name")?;
    // SAFETY: O_EXCL proved that this transaction created the basename, name
    // remains live, and the root has exclusive controller mutation authority.
    if unsafe { unlinkat(root.directory.as_raw_fd(), name.as_ptr(), 0) } != 0 {
        return Err(failure("cleanup-uncertain", "create-rollback-unlink"));
    }
    root.directory.sync_all().map_err(|_| failure("cleanup-uncertain", "create-rollback-sync"))?;
    require_initial_root(root, "create-rollback-root")
        .map_err(|_| failure("cleanup-uncertain", "create-rollback-root"))
}

fn create_verified(
    root: &mut LinuxRoot,
    expected_purpose: LinuxRootPurpose,
    basename: &str,
    stage: &'static str,
) -> HandoffResult<(LinuxFile, FileIdentity)> {
    require_purpose(root, expected_purpose, stage)?;
    require_initial_root(root, stage)?;
    let created = open_child(root, basename, O_RDWR | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW, MODE_0600, stage)?;
    let metadata = match created.file.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            drop(created);
            rollback_unjournaled_creation(root, basename)?;
            return Err(io_failure("create-initial-stat"));
        }
    };
    match descriptor_is_cloexec(created.file.as_raw_fd()) {
        Ok(true) => {},
        Ok(false) | Err(_) => {
            drop(created);
            rollback_unjournaled_creation(root, basename)?;
            return Err(failure("descriptor-not-cloexec", "create-flags"));
        }
    }
    Ok((created, file_identity(&metadata)))
}

fn file_type(metadata: &Metadata) -> FileType {
    let kind = metadata.file_type();
    if kind.is_file() { FileType::Regular }
    else if kind.is_dir() { FileType::Directory }
    else { FileType::Other }
}

fn file_identity(metadata: &Metadata) -> FileIdentity {
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.size(),
        mode: metadata.mode(),
        uid: metadata.uid(),
        gid: metadata.gid(),
        link_count: metadata.nlink(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
        file_type: file_type(metadata),
    }
}

fn root_identity(metadata: &Metadata) -> RootIdentity {
    RootIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.mode(),
        uid: metadata.uid(),
        link_count: metadata.nlink(),
        file_type: file_type(metadata),
    }
}

fn retry_interrupted<T>(mut operation: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut interrupted = 0usize;
    loop {
        match operation() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted && interrupted < MAX_INTERRUPTED_RETRIES => {
                interrupted += 1;
                continue;
            }
            result => return result,
        }
    }
}

fn retry_read(file: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    retry_interrupted(|| file.read(buffer))
}

fn retry_write(file: &mut File, buffer: &[u8]) -> io::Result<usize> {
    retry_interrupted(|| file.write(buffer))
}

#[derive(Default)]
struct EnumerationBudget {
    entries: usize,
    name_bytes: usize,
}

fn parse_directory_records(buffer: &[u8], output: &mut Vec<String>, budget: &mut EnumerationBudget) -> HandoffResult<()> {
    let mut offset = 0usize;
    while offset < buffer.len() {
        if buffer.len() - offset < DIRENT64_HEADER_BYTES {
            return Err(failure("directory-record", "enumerate-parse"));
        }
        let record_length = u16::from_ne_bytes([buffer[offset + 16], buffer[offset + 17]]) as usize;
        let end = offset.checked_add(record_length)
            .ok_or_else(|| failure("directory-record", "enumerate-parse"))?;
        if record_length < DIRENT64_HEADER_BYTES || end > buffer.len() {
            return Err(failure("directory-record", "enumerate-parse"));
        }
        let name_bytes = &buffer[offset + DIRENT64_HEADER_BYTES..end];
        let nul = name_bytes.iter().position(|byte| *byte == 0)
            .ok_or_else(|| failure("directory-name", "enumerate-parse"))?;
        if nul == 0 { return Err(failure("directory-name", "enumerate-parse")); }
        let name = CStr::from_bytes_with_nul(&name_bytes[..=nul])
            .map_err(|_| failure("directory-name", "enumerate-parse"))?
            .to_str().map_err(|_| failure("directory-name-encoding", "enumerate-parse"))?;
        let allowed_length = if name == "." || name == ".." { name.len() <= 2 } else { name.len() <= 64 };
        if !allowed_length { return Err(failure("directory-name-bound", "enumerate-parse")); }
        let entries = budget.entries.checked_add(1)
            .ok_or_else(|| failure("directory-entry-bound", "enumerate-parse"))?;
        let name_total = budget.name_bytes.checked_add(name.len())
            .ok_or_else(|| failure("directory-name-bound", "enumerate-parse"))?;
        if entries > MAX_ENUMERATION_ENTRIES { return Err(failure("directory-entry-bound", "enumerate-parse")); }
        if name_total > MAX_ENUMERATION_NAME_BYTES { return Err(failure("directory-name-bound", "enumerate-parse")); }
        budget.entries = entries;
        budget.name_bytes = name_total;
        output.push(name.to_owned());
        offset = end;
    }
    Ok(())
}

fn enumerate_directory(root: &mut LinuxRoot) -> HandoffResult<Vec<String>> {
    root.directory.seek(SeekFrom::Start(0)).map_err(|_| io_failure("enumerate-seek"))?;
    let mut output = Vec::new();
    let mut budget = EnumerationBudget::default();
    let mut buffer = [0u8; DIRENT_BUFFER_BYTES];
    let mut interrupted = 0usize;
    loop {
        // SAFETY: the syscall number is fixed for the cfg-gated x86_64 Linux
        // ABI and the writable buffer remains live for its exact supplied size.
        let count = unsafe {
            syscall(
                SYS_GETDENTS64_X86_64,
                root.directory.as_raw_fd(),
                buffer.as_mut_ptr().cast::<c_void>(),
                buffer.len(),
            )
        };
        if count < 0 {
            if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted
                && interrupted < MAX_INTERRUPTED_RETRIES
            {
                interrupted += 1;
                continue;
            }
            return Err(io_failure("enumerate-read"));
        }
        interrupted = 0;
        if count == 0 { break; }
        let count = usize::try_from(count).map_err(|_| failure("directory-count", "enumerate-read"))?;
        if count > buffer.len() { return Err(failure("directory-count", "enumerate-read")); }
        parse_directory_records(&buffer[..count], &mut output, &mut budget)?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    unsafe extern "C" {
        fn mkfifo(pathname: *const c_char, mode: c_uint) -> c_int;
    }

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("rar-handoff-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    fn adopt(path: &Path, purpose: LinuxRootPurpose) -> LinuxRoot {
        let file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        let attestation = ControllerRootAttestation::from_controller_observation(metadata.dev(), metadata.ino(), purpose);
        LinuxRoot::from_verified_owned_fd(OwnedFd::from(file), attestation).unwrap()
    }

    fn record(name: &[u8]) -> Vec<u8> {
        let length = DIRENT64_HEADER_BYTES + name.len() + 1;
        let mut bytes = vec![0u8; length];
        bytes[16..18].copy_from_slice(&(length as u16).to_ne_bytes());
        bytes[DIRENT64_HEADER_BYTES..DIRENT64_HEADER_BYTES + name.len()].copy_from_slice(name);
        bytes[length - 1] = 0;
        bytes
    }

    #[test]
    fn parses_bounded_dirents_and_rejects_malformed_records() {
        let mut output = Vec::new();
        let mut bytes = record(b".");
        bytes.extend(record(b"rar-os-alpha.img"));
        let mut budget = EnumerationBudget::default();
        parse_directory_records(&bytes, &mut output, &mut budget).unwrap();
        assert_eq!(output, [".".to_owned(), "rar-os-alpha.img".to_owned()]);

        let mut short = record(b"bad");
        short[16..18].copy_from_slice(&18u16.to_ne_bytes());
        assert_eq!(parse_directory_records(&short, &mut Vec::new(), &mut EnumerationBudget::default()).unwrap_err().code, "directory-record");

        let mut missing_nul = record(b"bad");
        *missing_nul.last_mut().unwrap() = b'x';
        assert_eq!(parse_directory_records(&missing_nul, &mut Vec::new(), &mut EnumerationBudget::default()).unwrap_err().code, "directory-name");

        let invalid_utf8 = record(&[0xff]);
        assert_eq!(parse_directory_records(&invalid_utf8, &mut Vec::new(), &mut EnumerationBudget::default()).unwrap_err().code, "directory-name-encoding");

        let mut over_count = EnumerationBudget { entries: MAX_ENUMERATION_ENTRIES, name_bytes: 0 };
        assert_eq!(parse_directory_records(&record(b"x"), &mut Vec::new(), &mut over_count).unwrap_err().code, "directory-entry-bound");
        let mut across_buffers = EnumerationBudget::default();
        for _ in 0..MAX_ENUMERATION_ENTRIES {
            parse_directory_records(&record(b"x"), &mut Vec::new(), &mut across_buffers).unwrap();
        }
        assert_eq!(parse_directory_records(&record(b"x"), &mut Vec::new(), &mut across_buffers).unwrap_err().code, "directory-entry-bound");
        let mut over_names = EnumerationBudget { entries: 0, name_bytes: MAX_ENUMERATION_NAME_BYTES };
        assert_eq!(parse_directory_records(&record(b"x"), &mut Vec::new(), &mut over_names).unwrap_err().code, "directory-name-bound");

        let mut interrupted = 0usize;
        let exhausted: io::Result<()> = retry_interrupted(|| {
            interrupted += 1;
            Err(io::Error::from(io::ErrorKind::Interrupted))
        });
        assert_eq!(exhausted.unwrap_err().kind(), io::ErrorKind::Interrupted);
        assert_eq!(interrupted, MAX_INTERRUPTED_RETRIES + 1);
    }

    #[test]
    fn enforces_linux_descriptor_authority_and_cleanup_invariants() {
        let source_path = temporary_root("source");
        let destination_path = temporary_root("destination");
        let manifest_path = temporary_root("manifest");

        let file_path = source_path.join("source.bin");
        let mut source_file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&file_path).unwrap();
        source_file.write_all(b"source").unwrap();
        drop(source_file);
        fs::hard_link(&file_path, source_path.join("source-link.bin")).unwrap();
        symlink("source.bin", source_path.join("source-symlink.bin")).unwrap();
        fs::create_dir(source_path.join("directory.bin")).unwrap();
        let fifo_path = source_path.join("source-fifo.bin");
        let fifo_name = CString::new(fifo_path.as_os_str().as_bytes()).unwrap();
        // SAFETY: the test-owned absolute path is live and NUL-terminated.
        assert_eq!(unsafe { mkfifo(fifo_name.as_ptr(), 0o600) }, 0);

        let wrong_file = File::open(&source_path).unwrap();
        let wrong_metadata = wrong_file.metadata().unwrap();
        let wrong = ControllerRootAttestation::from_controller_observation(
            wrong_metadata.dev(), wrong_metadata.ino() + 1, LinuxRootPurpose::Source,
        );
        assert_eq!(
            LinuxRoot::from_verified_owned_fd(OwnedFd::from(wrong_file), wrong).err().unwrap().code,
            "root-attestation-mismatch",
        );
        let no_cloexec_file = File::open(&source_path).unwrap();
        let no_cloexec_metadata = no_cloexec_file.metadata().unwrap();
        // SAFETY: F_SETFD with zero clears only descriptor flags and does not
        // transfer ownership of the live test descriptor.
        assert_eq!(unsafe { fcntl(no_cloexec_file.as_raw_fd(), F_SETFD, 0) }, 0);
        let no_cloexec = ControllerRootAttestation::from_controller_observation(
            no_cloexec_metadata.dev(), no_cloexec_metadata.ino(), LinuxRootPurpose::Source,
        );
        assert_eq!(
            LinuxRoot::from_verified_owned_fd(OwnedFd::from(no_cloexec_file), no_cloexec).err().unwrap().code,
            "root-not-cloexec",
        );

        let mut source = adopt(&source_path, LinuxRootPurpose::Source);
        let mut destination = adopt(&destination_path, LinuxRootPurpose::Destination);
        let mut manifests = adopt(&manifest_path, LinuxRootPurpose::Manifest);
        let mut ops = LinuxOps::new();
        let names = ops.enumerate(&mut source).unwrap();
        assert!(names.iter().any(|name| name == "source.bin"));

        let mut regular = ops.open_source(&mut source, "source.bin").unwrap();
        assert!(descriptor_is_cloexec(regular.file.as_raw_fd()).unwrap());
        assert_eq!(ops.stat(&mut regular).unwrap().link_count, 2);
        assert!(ops.open_source(&mut source, "source-symlink.bin").is_err());
        let mut directory = ops.open_source(&mut source, "directory.bin").unwrap();
        assert_eq!(ops.stat(&mut directory).unwrap().file_type, FileType::Directory);
        let mut fifo = ops.open_source(&mut source, "source-fifo.bin").unwrap();
        assert_eq!(ops.stat(&mut fifo).unwrap().file_type, FileType::Other);
        let device = File::open("/dev/null").unwrap();
        assert_eq!(file_identity(&device.metadata().unwrap()).file_type, FileType::Other);

        assert_eq!(
            ops.create_manifest(&mut destination, "wrong-root.v0").err().unwrap().code,
            "root-purpose-mismatch",
        );
        let (created, identity) = ops.create_destination(&mut destination, "copied.bin").unwrap();
        assert!(descriptor_is_cloexec(created.file.as_raw_fd()).unwrap());
        drop(created);
        assert!(ops.create_destination(&mut destination, "copied.bin").is_err());
        let mut wrong_identity = identity;
        wrong_identity.inode += 1;
        assert_eq!(
            ops.remove_created(&mut destination, "copied.bin", wrong_identity).unwrap_err().code,
            "cleanup-identity-mismatch",
        );
        ops.remove_created(&mut destination, "copied.bin", identity).unwrap();

        let (created, identity) = ops.create_destination(&mut destination, "mutated.bin").unwrap();
        drop(created);
        fs::set_permissions(&destination_path, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            ops.remove_created(&mut destination, "mutated.bin", identity).unwrap_err().code,
            "root-identity-changed",
        );
        fs::set_permissions(&destination_path, fs::Permissions::from_mode(0o700)).unwrap();
        ops.remove_created(&mut destination, "mutated.bin", identity).unwrap();

        let rollback = open_child(
            &destination, "rollback.bin", O_RDWR | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW, MODE_0600, "test-create",
        ).unwrap();
        drop(rollback);
        rollback_unjournaled_creation(&mut destination, "rollback.bin").unwrap();
        assert!(!destination_path.join("rollback.bin").exists());
        let missing = rollback_unjournaled_creation(&mut destination, "missing.bin").unwrap_err();
        assert_eq!(missing.code, "cleanup-uncertain");

        let (manifest, manifest_identity) = ops.create_manifest(&mut manifests, "handoff-p02-o001.v0").unwrap();
        drop(manifest);
        ops.remove_created(&mut manifests, "handoff-p02-o001.v0", manifest_identity).unwrap();

        drop((regular, directory, fifo, source, destination, manifests, ops));
        fs::remove_dir_all(&source_path).unwrap();
        fs::remove_dir_all(&destination_path).unwrap();
        fs::remove_dir_all(&manifest_path).unwrap();
    }
}

impl HandoffOps for LinuxOps {
    type Root = LinuxRoot;
    type File = LinuxFile;

    fn controller_uid(&self) -> u32 { self.controller_uid }

    fn monotonic_nanoseconds(&self) -> HandoffResult<u64> {
        let mut value = Timespec { seconds: 0, nanoseconds: 0 };
        // SAFETY: value is valid writable storage for one Timespec.
        if unsafe { clock_gettime(CLOCK_MONOTONIC, &mut value) } != 0 { return Err(io_failure("clock")); }
        if value.seconds < 0 || !(0..1_000_000_000).contains(&value.nanoseconds) {
            return Err(failure("clock-range", "clock"));
        }
        let seconds = u64::try_from(value.seconds).map_err(|_| failure("clock-range", "clock"))?;
        let nanoseconds = u64::try_from(value.nanoseconds).map_err(|_| failure("clock-range", "clock"))?;
        seconds.checked_mul(1_000_000_000).and_then(|base| base.checked_add(nanoseconds))
            .ok_or_else(|| failure("clock-range", "clock"))
    }

    fn root_identity(&mut self, root: &mut LinuxRoot) -> HandoffResult<RootIdentity> {
        root.directory.metadata().map(|metadata| root_identity(&metadata)).map_err(|_| io_failure("root-stat"))
    }

    fn enumerate(&mut self, root: &mut LinuxRoot) -> HandoffResult<Vec<String>> { enumerate_directory(root) }

    fn open_source(&mut self, root: &mut LinuxRoot, basename: &str) -> HandoffResult<LinuxFile> {
        require_purpose(root, LinuxRootPurpose::Source, "source-open")?;
        let opened = open_child(root, basename, O_RDONLY | O_CLOEXEC | O_NOFOLLOW | O_NONBLOCK, 0, "source-open")?;
        if !descriptor_is_cloexec(opened.file.as_raw_fd())? {
            return Err(failure("descriptor-not-cloexec", "source-flags"));
        }
        Ok(opened)
    }

    fn create_destination(&mut self, root: &mut LinuxRoot, basename: &str) -> HandoffResult<(LinuxFile, FileIdentity)> {
        create_verified(root, LinuxRootPurpose::Destination, basename, "destination-create")
    }

    fn create_manifest(&mut self, root: &mut LinuxRoot, basename: &str) -> HandoffResult<(LinuxFile, FileIdentity)> {
        create_verified(root, LinuxRootPurpose::Manifest, basename, "manifest-create")
    }

    fn stat(&mut self, file: &mut LinuxFile) -> HandoffResult<FileIdentity> {
        file.file.metadata().map(|metadata| file_identity(&metadata)).map_err(|_| io_failure("file-stat"))
    }

    fn read(&mut self, file: &mut LinuxFile, buffer: &mut [u8]) -> HandoffResult<usize> {
        retry_read(&mut file.file, buffer).map_err(|_| io_failure("read"))
    }

    fn write(&mut self, file: &mut LinuxFile, buffer: &[u8]) -> HandoffResult<usize> {
        retry_write(&mut file.file, buffer).map_err(|_| io_failure("write"))
    }

    fn seek_start(&mut self, file: &mut LinuxFile) -> HandoffResult<()> {
        file.file.seek(SeekFrom::Start(0)).map(|_| ()).map_err(|_| io_failure("seek"))
    }

    fn sync_data(&mut self, file: &mut LinuxFile) -> HandoffResult<()> {
        file.file.sync_data().map_err(|_| io_failure("file-sync"))
    }

    fn sync_root(&mut self, root: &mut LinuxRoot) -> HandoffResult<()> {
        root.directory.sync_all().map_err(|_| io_failure("root-sync"))
    }

    fn remove_created(&mut self, root: &mut LinuxRoot, basename: &str, expected: FileIdentity) -> HandoffResult<()> {
        if !matches!(root.purpose, LinuxRootPurpose::Destination | LinuxRootPurpose::Manifest) {
            return Err(failure("root-purpose-mismatch", "cleanup-open"));
        }
        require_initial_root(root, "cleanup-root-before")?;
        let current = open_child(root, basename, O_RDONLY | O_CLOEXEC | O_NOFOLLOW | O_NONBLOCK, 0, "cleanup-open")?;
        let metadata = current.file.metadata().map_err(|_| io_failure("cleanup-stat"))?;
        if metadata.dev() != expected.device
            || metadata.ino() != expected.inode
            || !metadata.file_type().is_file()
            || metadata.uid() != expected.uid
            || metadata.gid() != expected.gid
            || metadata.mode() != expected.mode
            || metadata.nlink() != expected.link_count
        {
            return Err(failure("cleanup-identity-mismatch", "cleanup-stat"));
        }
        drop(current);
        let name = child_name(basename, "cleanup-name")?;
        // SAFETY: name remains live and the controller-owned root is stable
        // under the module's exclusive-mutation invariant.
        if unsafe { unlinkat(root.directory.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            return Err(io_failure("cleanup-unlink"));
        }
        require_initial_root(root, "cleanup-root-after")?;
        Ok(())
    }
}
