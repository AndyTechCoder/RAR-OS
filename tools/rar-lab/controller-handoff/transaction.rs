use std::collections::{BTreeMap, BTreeSet};

use crate::contract::{Cancellation, COPY_BUFFER_BYTES, Deadline, LAUNCH_AGGREGATE_MAXIMUM, PhasePlan, ProducerQuiesced, SourceMountKind};
use crate::{HandoffManifest, Role, Sha256, sha256};

const PRODUCER_UID: u32 = 65_532;
const PRODUCER_GID: u32 = 65_532;
const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileType { Regular, Directory, Other }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    pub device: u64,
    pub inode: u64,
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub link_count: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
    pub changed_seconds: i64,
    pub changed_nanoseconds: i64,
    pub file_type: FileType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RootIdentity {
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub uid: u32,
    pub link_count: u64,
    pub file_type: FileType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffError {
    pub code: &'static str,
    pub stage: &'static str,
}

impl HandoffError {
    pub(crate) fn new(code: &'static str, stage: &'static str) -> Self { Self { code, stage } }
}

pub type HandoffResult<T> = Result<T, HandoffError>;

/// Operations supplied only by the future reviewed Linux descriptor adapter.
///
/// Every method must be bounded controller-owned work. Opens and reads are
/// nonblocking where required. Kernel synchronization that cannot be cancelled
/// safely remains inside an outer trusted-controller process deadline: expiry
/// terminates the helper, prevents progression, and invokes journal recovery.
pub trait HandoffOps {
    type Root;
    type File;

    fn controller_uid(&self) -> u32;
    fn monotonic_nanoseconds(&self) -> HandoffResult<u64>;
    fn root_identity(&mut self, root: &mut Self::Root) -> HandoffResult<RootIdentity>;
    fn enumerate(&mut self, root: &mut Self::Root) -> HandoffResult<Vec<String>>;
    fn open_source(&mut self, root: &mut Self::Root, basename: &str) -> HandoffResult<Self::File>;
    /// Creates with O_EXCL and returns the same open file plus its verified
    /// identity. An implementation must remove and durably synchronize its own
    /// new entry before returning any post-open identity error.
    fn create_destination(&mut self, root: &mut Self::Root, basename: &str) -> HandoffResult<(Self::File, FileIdentity)>;
    fn create_manifest(&mut self, root: &mut Self::Root, basename: &str) -> HandoffResult<(Self::File, FileIdentity)>;
    fn stat(&mut self, file: &mut Self::File) -> HandoffResult<FileIdentity>;
    fn read(&mut self, file: &mut Self::File, buffer: &mut [u8]) -> HandoffResult<usize>;
    fn write(&mut self, file: &mut Self::File, buffer: &[u8]) -> HandoffResult<usize>;
    fn seek_start(&mut self, file: &mut Self::File) -> HandoffResult<()>;
    fn sync_data(&mut self, file: &mut Self::File) -> HandoffResult<()>;
    fn sync_root(&mut self, root: &mut Self::Root) -> HandoffResult<()>;
    fn remove_created(&mut self, root: &mut Self::Root, basename: &str, identity: FileIdentity) -> HandoffResult<()>;
}

pub struct SourceRoot<R> {
    pub kind: SourceMountKind,
    pub root: R,
}

pub struct HandoffRoots<R> {
    pub sources: Vec<SourceRoot<R>>,
    pub destination: R,
    pub manifests: R,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchReceipt {
    pub manifests: Vec<HandoffManifest>,
    pub manifest_sha256: Vec<[u8; 32]>,
}

#[derive(Clone)]
struct CopiedOutput {
    manifest: HandoffManifest,
}

#[derive(Clone)]
enum JournalEntry {
    Destination { basename: String, identity: FileIdentity },
    Manifest { basename: String, identity: FileIdentity },
}

fn check_boundary<O: HandoffOps>(ops: &O, deadline: Deadline, cancellation: &dyn Cancellation) -> HandoffResult<()> {
    if cancellation.is_cancelled() { return Err(HandoffError::new("cancelled", "boundary")); }
    if deadline.has_expired(ops.monotonic_nanoseconds()?) {
        return Err(HandoffError::new("deadline-expired", "boundary"));
    }
    Ok(())
}

fn canonical_set(values: Vec<String>) -> HandoffResult<BTreeSet<String>> {
    let mut set = BTreeSet::new();
    for value in values {
        if value == "." || value == ".." { continue; }
        if !set.insert(value) { return Err(HandoffError::new("duplicate-entry", "enumeration")); }
    }
    Ok(set)
}

fn validate_root(identity: RootIdentity, controller_uid: u32) -> HandoffResult<()> {
    if identity.file_type != FileType::Directory
        || identity.device == 0 || identity.inode == 0
        || identity.uid != controller_uid || identity.mode & 0o7777 != DIRECTORY_MODE
        || identity.link_count == 0
    {
        return Err(HandoffError::new("root-identity-invalid", "root"));
    }
    Ok(())
}

fn validate_source(identity: FileIdentity, maximum: u64) -> HandoffResult<()> {
    if identity.file_type != FileType::Regular
        || identity.device == 0 || identity.inode == 0
        || identity.uid != PRODUCER_UID || identity.gid != PRODUCER_GID
        || identity.mode & 0o7777 != FILE_MODE || identity.link_count != 1
        || identity.size == 0 || identity.size > maximum
    {
        return Err(HandoffError::new("source-identity-invalid", "source-stat"));
    }
    Ok(())
}

fn validate_destination(identity: FileIdentity, expected_size: u64, controller_uid: u32) -> HandoffResult<()> {
    if identity.file_type != FileType::Regular
        || identity.device == 0 || identity.inode == 0
        || identity.uid != controller_uid || identity.mode & 0o7777 != FILE_MODE
        || identity.link_count != 1 || identity.size != expected_size
    {
        return Err(HandoffError::new("destination-identity-invalid", "destination-check"));
    }
    Ok(())
}

fn write_exact<O: HandoffOps>(ops: &mut O, file: &mut O::File, bytes: &[u8], deadline: Deadline, cancellation: &dyn Cancellation) -> HandoffResult<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        check_boundary(ops, deadline, cancellation)?;
        let written = ops.write(file, &bytes[offset..])?;
        if written == 0 || written > bytes.len() - offset {
            return Err(HandoffError::new("short-write", "write"));
        }
        offset += written;
    }
    Ok(())
}

fn read_exact_hash<O: HandoffOps>(ops: &mut O, file: &mut O::File, expected_size: u64, deadline: Deadline, cancellation: &dyn Cancellation) -> HandoffResult<[u8; 32]> {
    let mut remaining = expected_size;
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    let mut hash = Sha256::new();
    while remaining != 0 {
        check_boundary(ops, deadline, cancellation)?;
        let wanted = usize::try_from(remaining.min(COPY_BUFFER_BYTES as u64))
            .map_err(|_| HandoffError::new("size-conversion", "read"))?;
        let count = ops.read(file, &mut buffer[..wanted])?;
        if count == 0 || count > wanted { return Err(HandoffError::new("short-read", "read")); }
        hash.update(&buffer[..count]);
        remaining -= u64::try_from(count).map_err(|_| HandoffError::new("size-conversion", "read"))?;
    }
    let mut extra = [0u8; 1];
    if ops.read(file, &mut extra)? != 0 { return Err(HandoffError::new("trailing-byte", "read")); }
    Ok(hash.finalize())
}

fn cleanup<O: HandoffOps>(ops: &mut O, roots: &mut HandoffRoots<O::Root>, journal: &[JournalEntry]) -> HandoffResult<()> {
    if journal.is_empty() { return Ok(()); }
    let mut uncertain = false;
    let mut destination_affected = false;
    let mut manifest_affected = false;
    for entry in journal.iter().rev() {
        let result = match entry {
            JournalEntry::Destination { basename, identity } => {
                destination_affected = true;
                ops.remove_created(&mut roots.destination, basename, *identity)
            }
            JournalEntry::Manifest { basename, identity } => {
                manifest_affected = true;
                ops.remove_created(&mut roots.manifests, basename, *identity)
            }
        };
        if result.is_err() { uncertain = true; }
    }
    if destination_affected && ops.sync_root(&mut roots.destination).is_err() { uncertain = true; }
    if manifest_affected && ops.sync_root(&mut roots.manifests).is_err() { uncertain = true; }
    if uncertain { Err(HandoffError::new("cleanup-uncertain", "cleanup")) } else { Ok(()) }
}

fn execute<O: HandoffOps>(ops: &mut O, plan: &PhasePlan, roots: &mut HandoffRoots<O::Root>, deadline: Deadline, cancellation: &dyn Cancellation, journal: &mut Vec<JournalEntry>) -> HandoffResult<BatchReceipt> {
    check_boundary(ops, deadline, cancellation)?;
    let controller_uid = ops.controller_uid();
    let destination_root_identity = ops.root_identity(&mut roots.destination)?;
    let manifest_root_identity = ops.root_identity(&mut roots.manifests)?;
    validate_root(destination_root_identity, controller_uid)?;
    validate_root(manifest_root_identity, controller_uid)?;
    if destination_root_identity.device == manifest_root_identity.device && destination_root_identity.inode == manifest_root_identity.inode {
        return Err(HandoffError::new("shared-root", "root"));
    }
    if !canonical_set(ops.enumerate(&mut roots.destination)?)?.is_empty() {
        return Err(HandoffError::new("destination-not-empty", "destination-root"));
    }

    let mut expected_by_mount: BTreeMap<SourceMountKind, BTreeSet<String>> = BTreeMap::new();
    for output in plan.outputs() {
        expected_by_mount.entry(output.source_mount).or_default().insert(output.basename.clone());
    }
    if roots.sources.len() != expected_by_mount.len() {
        return Err(HandoffError::new("source-root-set-mismatch", "source-root"));
    }
    let mut initial_roots = BTreeMap::new();
    let mut root_identities = BTreeSet::new();
    for source in &mut roots.sources {
        let expected = expected_by_mount.get(&source.kind)
            .ok_or_else(|| HandoffError::new("unexpected-source-root", "source-root"))?;
        if initial_roots.contains_key(&source.kind) {
            return Err(HandoffError::new("duplicate-source-root", "source-root"));
        }
        let identity = ops.root_identity(&mut source.root)?;
        validate_root(identity, controller_uid)?;
        if !root_identities.insert((identity.device, identity.inode)) {
            return Err(HandoffError::new("shared-source-root", "source-root"));
        }
        for other in [destination_root_identity, manifest_root_identity] {
            if identity.device == other.device && identity.inode == other.inode {
                return Err(HandoffError::new("shared-root", "source-root"));
            }
        }
        if canonical_set(ops.enumerate(&mut source.root)?)? != *expected {
            return Err(HandoffError::new("source-entry-set-mismatch", "enumeration"));
        }
        initial_roots.insert(source.kind, identity);
    }

    let mut copied = Vec::with_capacity(plan.outputs().len());
    let mut sizes = Vec::with_capacity(plan.outputs().len());
    let mut launch_aggregate = 0u64;
    for output in plan.outputs() {
        check_boundary(ops, deadline, cancellation)?;
        let source_root = roots.sources.iter_mut().find(|root| root.kind == output.source_mount)
            .ok_or_else(|| HandoffError::new("missing-source-root", "source-root"))?;
        let mut source = ops.open_source(&mut source_root.root, &output.basename)?;
        let source_identity = ops.stat(&mut source)?;
        validate_source(source_identity, output.maximum_bytes)?;
        if output.role == Role::Launch {
            launch_aggregate = launch_aggregate.checked_add(source_identity.size)
                .ok_or_else(|| HandoffError::new("aggregate-maximum", "source-stat"))?;
            if launch_aggregate > LAUNCH_AGGREGATE_MAXIMUM {
                return Err(HandoffError::new("aggregate-maximum", "source-stat"));
            }
        }
        let (mut destination, created_identity) = ops.create_destination(&mut roots.destination, &output.basename)?;
        journal.push(JournalEntry::Destination { basename: output.basename.clone(), identity: created_identity });
        validate_destination(created_identity, 0, controller_uid)?;

        let mut remaining = source_identity.size;
        let mut buffer = [0u8; COPY_BUFFER_BYTES];
        let mut copied_hash = Sha256::new();
        while remaining != 0 {
            check_boundary(ops, deadline, cancellation)?;
            let wanted = usize::try_from(remaining.min(COPY_BUFFER_BYTES as u64))
                .map_err(|_| HandoffError::new("size-conversion", "copy"))?;
            let count = ops.read(&mut source, &mut buffer[..wanted])?;
            if count == 0 || count > wanted { return Err(HandoffError::new("short-copy", "copy")); }
            write_exact(ops, &mut destination, &buffer[..count], deadline, cancellation)?;
            copied_hash.update(&buffer[..count]);
            remaining -= u64::try_from(count).map_err(|_| HandoffError::new("size-conversion", "copy"))?;
        }
        let mut extra = [0u8; 1];
        if ops.read(&mut source, &mut extra)? != 0 { return Err(HandoffError::new("source-grew", "recheck")); }
        ops.sync_data(&mut destination)?;
        check_boundary(ops, deadline, cancellation)?;
        if ops.stat(&mut source)? != source_identity { return Err(HandoffError::new("source-changed", "recheck")); }
        ops.seek_start(&mut destination)?;
        let copied_digest = copied_hash.finalize();
        if read_exact_hash(ops, &mut destination, source_identity.size, deadline, cancellation)? != copied_digest {
            return Err(HandoffError::new("destination-hash-mismatch", "destination-check"));
        }
        let destination_identity = ops.stat(&mut destination)?;
        validate_destination(destination_identity, source_identity.size, controller_uid)?;
        if destination_identity.device != created_identity.device || destination_identity.inode != created_identity.inode {
            return Err(HandoffError::new("destination-identity-changed", "destination-check"));
        }
        copied.push(CopiedOutput {
            manifest: HandoffManifest {
                phase: output.phase,
                role: output.role,
                output_kind: output.output_kind,
                output_ordinal: output.output_ordinal,
                basename: output.basename.clone(),
                output_bytes: source_identity.size,
                output_sha256: copied_digest,
                source_device: source_identity.device,
                source_inode: source_identity.inode,
                destination_device: destination_identity.device,
                destination_inode: destination_identity.inode,
            },
        });
        sizes.push(source_identity.size);
    }
    plan.validate_aggregate(&sizes).map_err(|_| HandoffError::new("aggregate-maximum", "copy"))?;
    ops.sync_root(&mut roots.destination)?;
    check_boundary(ops, deadline, cancellation)?;

    for source in &mut roots.sources {
        let expected = expected_by_mount.get(&source.kind).ok_or_else(|| HandoffError::new("missing-source-root", "enumeration"))?;
        if canonical_set(ops.enumerate(&mut source.root)?)? != *expected
            || ops.root_identity(&mut source.root)? != *initial_roots.get(&source.kind).ok_or_else(|| HandoffError::new("missing-root-identity", "enumeration"))?
        {
            return Err(HandoffError::new("source-root-changed", "enumeration"));
        }
    }
    let expected_destination: BTreeSet<String> = plan.outputs().iter().map(|output| output.basename.clone()).collect();
    if canonical_set(ops.enumerate(&mut roots.destination)?)? != expected_destination {
        return Err(HandoffError::new("destination-entry-set-mismatch", "enumeration"));
    }
    if ops.root_identity(&mut roots.destination)? != destination_root_identity {
        return Err(HandoffError::new("destination-root-changed", "enumeration"));
    }

    let mut manifest_hashes = Vec::with_capacity(copied.len());
    for output in &copied {
        let bytes = output.manifest.encode().map_err(|_| HandoffError::new("manifest-invalid", "publication"))?;
        let name = output.manifest.file_name().map_err(|_| HandoffError::new("manifest-name-invalid", "publication"))?;
        let (mut manifest_file, created_identity) = ops.create_manifest(&mut roots.manifests, &name)?;
        journal.push(JournalEntry::Manifest { basename: name, identity: created_identity });
        validate_destination(created_identity, 0, controller_uid)?;
        write_exact(ops, &mut manifest_file, &bytes, deadline, cancellation)?;
        ops.seek_start(&mut manifest_file)?;
        let mut retained = [0u8; crate::MANIFEST_BYTES];
        let mut offset = 0;
        while offset < retained.len() {
            check_boundary(ops, deadline, cancellation)?;
            let count = ops.read(&mut manifest_file, &mut retained[offset..])?;
            if count == 0 || count > retained.len() - offset {
                return Err(HandoffError::new("manifest-short-read", "publication"));
            }
            offset += count;
        }
        let mut extra = [0u8; 1];
        if ops.read(&mut manifest_file, &mut extra)? != 0 { return Err(HandoffError::new("manifest-extra-byte", "publication")); }
        let decoded = HandoffManifest::decode(&retained).map_err(|_| HandoffError::new("manifest-decode", "publication"))?;
        if decoded != output.manifest { return Err(HandoffError::new("manifest-mismatch", "publication")); }
        let final_identity = ops.stat(&mut manifest_file)?;
        validate_destination(final_identity, crate::MANIFEST_BYTES as u64, controller_uid)?;
        if final_identity.device != created_identity.device || final_identity.inode != created_identity.inode {
            return Err(HandoffError::new("manifest-identity-changed", "publication"));
        }
        ops.sync_data(&mut manifest_file)?;
        check_boundary(ops, deadline, cancellation)?;
        manifest_hashes.push(sha256(&retained));
    }
    if ops.root_identity(&mut roots.manifests)? != manifest_root_identity {
        return Err(HandoffError::new("manifest-root-changed", "publication"));
    }
    ops.sync_root(&mut roots.manifests)?;
    check_boundary(ops, deadline, cancellation)?;
    let manifests = copied.into_iter().map(|output| output.manifest).collect();
    Ok(BatchReceipt { manifests, manifest_sha256: manifest_hashes })
}

pub fn handoff_batch<O: HandoffOps>(ops: &mut O, _producer: ProducerQuiesced, plan: &PhasePlan, roots: &mut HandoffRoots<O::Root>, deadline: Deadline, cancellation: &dyn Cancellation) -> HandoffResult<BatchReceipt> {
    let mut journal = Vec::new();
    match execute(ops, plan, roots, deadline, cancellation, &mut journal) {
        Ok(receipt) => Ok(receipt),
        Err(original) => match cleanup(ops, roots, &journal) {
            Ok(()) => Err(original),
            Err(cleanup_error) => Err(cleanup_error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct FakeData { bytes: Vec<u8>, identity: FileIdentity, virtual_fill: Option<u8> }

    struct FakeRoot { identity: RootIdentity, files: BTreeMap<String, FakeData> }

    struct FakeFile { root: usize, name: String, cursor: usize, readback: bool }

    struct FakeBackend {
        controller_uid: u32,
        next_inode: u64,
        roots: BTreeMap<usize, FakeRoot>,
        corrupt_destination_readback: bool,
        cleanup_fails: bool,
        oversized_manifest_read: bool,
        replace_destination_on_write: bool,
        zero_write: bool,
        sync_calls: usize,
        monotonic_now: u64,
        expire_on_manifest_sync: bool,
        count_only_writes: bool,
        written_bytes: u64,
        fail_destination_initial_stat: bool,
        fail_manifest_initial_stat: bool,
    }

    impl FakeBackend {
        fn new() -> Self {
            let controller_uid = 501;
            let mut roots = BTreeMap::new();
            for index in 1..=4 {
                roots.insert(index, FakeRoot {
                    identity: RootIdentity {
                        device: 1, inode: 100 + index as u64, mode: DIRECTORY_MODE,
                        uid: controller_uid, link_count: 1, file_type: FileType::Directory,
                    },
                    files: BTreeMap::new(),
                });
            }
            let mut backend = Self {
                controller_uid, next_inode: 1_000, roots,
                corrupt_destination_readback: false, cleanup_fails: false,
                oversized_manifest_read: false, replace_destination_on_write: false,
                zero_write: false, sync_calls: 0,
                monotonic_now: 1, expire_on_manifest_sync: false,
                count_only_writes: false, written_bytes: 0,
                fail_destination_initial_stat: false, fail_manifest_initial_stat: false,
            };
            backend.add_source(1, "rar-os-alpha.img", b"image");
            backend.add_source(2, "comparison.bin", b"comparison");
            backend
        }

        fn identity(&mut self, bytes: usize, uid: u32, gid: u32) -> FileIdentity {
            let inode = self.next_inode;
            self.next_inode += 1;
            FileIdentity {
                device: 1, inode, size: bytes as u64, mode: FILE_MODE, uid, gid,
                link_count: 1, modified_seconds: 1, modified_nanoseconds: 2,
                changed_seconds: 3, changed_nanoseconds: 4, file_type: FileType::Regular,
            }
        }

        fn add_source(&mut self, root: usize, name: &str, bytes: &[u8]) {
            let identity = self.identity(bytes.len(), PRODUCER_UID, PRODUCER_GID);
            self.roots.get_mut(&root).unwrap().files.insert(name.into(), FakeData { bytes: bytes.to_vec(), identity, virtual_fill: None });
        }

        fn add_virtual_source(&mut self, root: usize, name: &str, size: usize) {
            let identity = self.identity(size, PRODUCER_UID, PRODUCER_GID);
            self.roots.get_mut(&root).unwrap().files.insert(name.into(), FakeData { bytes: Vec::new(), identity, virtual_fill: Some(0) });
        }

        fn roots() -> HandoffRoots<usize> {
            HandoffRoots {
                sources: vec![
                    SourceRoot { kind: SourceMountKind::BuildArtifact, root: 1 },
                    SourceRoot { kind: SourceMountKind::BuildTranscript, root: 2 },
                ],
                destination: 3,
                manifests: 4,
            }
        }

        fn launch_roots() -> HandoffRoots<usize> {
            HandoffRoots {
                sources: vec![SourceRoot { kind: SourceMountKind::Launch, root: 1 }],
                destination: 3,
                manifests: 4,
            }
        }
    }

    impl HandoffOps for FakeBackend {
        type Root = usize;
        type File = FakeFile;

        fn controller_uid(&self) -> u32 { self.controller_uid }
        fn monotonic_nanoseconds(&self) -> HandoffResult<u64> { Ok(self.monotonic_now) }
        fn root_identity(&mut self, root: &mut usize) -> HandoffResult<RootIdentity> {
            self.roots.get(root).map(|value| value.identity).ok_or_else(|| HandoffError::new("fake-root", "test"))
        }
        fn enumerate(&mut self, root: &mut usize) -> HandoffResult<Vec<String>> {
            self.roots.get(root).map(|value| value.files.keys().cloned().collect()).ok_or_else(|| HandoffError::new("fake-root", "test"))
        }
        fn open_source(&mut self, root: &mut usize, basename: &str) -> HandoffResult<FakeFile> {
            if !self.roots.get(root).is_some_and(|value| value.files.contains_key(basename)) {
                return Err(HandoffError::new("fake-missing", "test"));
            }
            Ok(FakeFile { root: *root, name: basename.into(), cursor: 0, readback: false })
        }
        fn create_destination(&mut self, root: &mut usize, basename: &str) -> HandoffResult<(FakeFile, FileIdentity)> {
            self.create_file(*root, basename, self.fail_destination_initial_stat)
        }
        fn create_manifest(&mut self, root: &mut usize, basename: &str) -> HandoffResult<(FakeFile, FileIdentity)> {
            self.create_file(*root, basename, self.fail_manifest_initial_stat)
        }
        fn stat(&mut self, file: &mut FakeFile) -> HandoffResult<FileIdentity> {
            self.roots.get(&file.root).and_then(|root| root.files.get(&file.name)).map(|data| data.identity)
                .ok_or_else(|| HandoffError::new("fake-missing", "test"))
        }
        fn read(&mut self, file: &mut FakeFile, buffer: &mut [u8]) -> HandoffResult<usize> {
            if self.oversized_manifest_read && file.root == 4 && file.readback {
                self.oversized_manifest_read = false;
                return Ok(buffer.len() + 1);
            }
            let data = self.roots.get(&file.root).and_then(|root| root.files.get(&file.name))
                .ok_or_else(|| HandoffError::new("fake-missing", "test"))?;
            let available = usize::try_from(data.identity.size).unwrap().saturating_sub(file.cursor);
            let count = buffer.len().min(available);
            if let Some(fill) = data.virtual_fill { buffer[..count].fill(fill); }
            else { buffer[..count].copy_from_slice(&data.bytes[file.cursor..file.cursor + count]); }
            if self.corrupt_destination_readback && file.root == 3 && file.readback && file.cursor == 0 && count != 0 {
                buffer[0] ^= 1;
            }
            file.cursor += count;
            Ok(count)
        }
        fn write(&mut self, file: &mut FakeFile, buffer: &[u8]) -> HandoffResult<usize> {
            if self.zero_write { return Ok(0); }
            let data = self.roots.get_mut(&file.root).and_then(|root| root.files.get_mut(&file.name))
                .ok_or_else(|| HandoffError::new("fake-missing", "test"))?;
            if data.virtual_fill.is_some() {
                file.cursor = file.cursor.checked_add(buffer.len()).ok_or_else(|| HandoffError::new("fake-overflow", "test"))?;
                data.identity.size = data.identity.size.max(file.cursor as u64);
            } else if file.cursor == data.bytes.len() {
                data.bytes.extend_from_slice(buffer);
                file.cursor += buffer.len();
                data.identity.size = data.bytes.len() as u64;
            } else {
                let end = file.cursor.checked_add(buffer.len()).ok_or_else(|| HandoffError::new("fake-overflow", "test"))?;
                if end > data.bytes.len() { data.bytes.resize(end, 0); }
                data.bytes[file.cursor..end].copy_from_slice(buffer);
                file.cursor = end;
                data.identity.size = data.bytes.len() as u64;
            }
            self.written_bytes = self.written_bytes.checked_add(buffer.len() as u64).unwrap();
            if self.replace_destination_on_write && file.root == 3 {
                data.identity.inode += 10_000;
                self.replace_destination_on_write = false;
            }
            Ok(buffer.len())
        }
        fn seek_start(&mut self, file: &mut FakeFile) -> HandoffResult<()> {
            file.cursor = 0; file.readback = true; Ok(())
        }
        fn sync_data(&mut self, _file: &mut FakeFile) -> HandoffResult<()> { Ok(()) }
        fn sync_root(&mut self, root: &mut usize) -> HandoffResult<()> {
            self.sync_calls += 1;
            if self.expire_on_manifest_sync && *root == 4 { self.monotonic_now = 100; }
            Ok(())
        }
        fn remove_created(&mut self, root: &mut usize, basename: &str, identity: FileIdentity) -> HandoffResult<()> {
            if self.cleanup_fails { return Err(HandoffError::new("fake-cleanup", "test")); }
            let stored = self.roots.get(root).and_then(|value| value.files.get(basename))
                .ok_or_else(|| HandoffError::new("fake-missing", "test"))?;
            if stored.identity.device != identity.device || stored.identity.inode != identity.inode {
                return Err(HandoffError::new("cleanup-identity", "test"));
            }
            self.roots.get_mut(root).unwrap().files.remove(basename);
            Ok(())
        }
    }

    impl FakeBackend {
        fn create_file(&mut self, root: usize, basename: &str, fail_initial_stat: bool) -> HandoffResult<(FakeFile, FileIdentity)> {
            if self.roots.get(&root).is_some_and(|value| value.files.contains_key(basename)) {
                return Err(HandoffError::new("fake-exists", "test"));
            }
            let identity = self.identity(0, self.controller_uid, self.controller_uid);
            let virtual_fill = self.count_only_writes.then_some(0);
            self.roots.get_mut(&root).ok_or_else(|| HandoffError::new("fake-root", "test"))?
                .files.insert(basename.into(), FakeData { bytes: Vec::new(), identity, virtual_fill });
            if fail_initial_stat {
                self.roots.get_mut(&root).unwrap().files.remove(basename);
                return Err(HandoffError::new("fake-initial-stat", "test"));
            }
            Ok((FakeFile { root, name: basename.into(), cursor: 0, readback: false }, identity))
        }
    }

    struct NeverCancelled;
    impl Cancellation for NeverCancelled { fn is_cancelled(&self) -> bool { false } }
    struct AlwaysCancelled;
    impl Cancellation for AlwaysCancelled { fn is_cancelled(&self) -> bool { true } }

    #[test]
    fn transaction_success_hash_failure_and_cleanup_uncertainty() {
        let deadline = Deadline::from_monotonic_nanoseconds(100);
        let plan = PhasePlan::build_one();
        let mut backend = FakeBackend::new();
        let receipt = handoff_batch(&mut backend, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap();
        assert_eq!(receipt.manifests.len(), 2);
        assert_eq!(backend.roots[&3].files.len(), 2);
        assert_eq!(backend.roots[&4].files.len(), 2);

        let mut corrupted = FakeBackend::new();
        corrupted.corrupt_destination_readback = true;
        let error = handoff_batch(&mut corrupted, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "destination-hash-mismatch");
        assert!(corrupted.roots[&3].files.is_empty());
        assert!(corrupted.roots[&4].files.is_empty());

        let mut uncertain = FakeBackend::new();
        uncertain.corrupt_destination_readback = true;
        uncertain.cleanup_fails = true;
        let error = handoff_batch(&mut uncertain, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "cleanup-uncertain");

        let mut expired = FakeBackend::new();
        let error = handoff_batch(&mut expired, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), Deadline::from_monotonic_nanoseconds(1), &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "deadline-expired");
        assert_eq!(expired.sync_calls, 0);

        let mut cancelled = FakeBackend::new();
        let error = handoff_batch(&mut cancelled, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &AlwaysCancelled).unwrap_err();
        assert_eq!(error.code, "cancelled");
        assert_eq!(cancelled.sync_calls, 0);

        let mut shared = FakeBackend::new();
        let shared_identity = shared.roots[&1].identity;
        shared.roots.get_mut(&2).unwrap().identity = shared_identity;
        let error = handoff_batch(&mut shared, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "shared-source-root");
        assert_eq!(shared.sync_calls, 0);

        let mut oversized = FakeBackend::new();
        oversized.oversized_manifest_read = true;
        let error = handoff_batch(&mut oversized, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "manifest-short-read");
        assert!(oversized.roots[&3].files.is_empty());
        assert!(oversized.roots[&4].files.is_empty());

        let mut replaced = FakeBackend::new();
        replaced.replace_destination_on_write = true;
        let error = handoff_batch(&mut replaced, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "cleanup-uncertain");

        let mut short_write = FakeBackend::new();
        short_write.zero_write = true;
        let error = handoff_batch(&mut short_write, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "short-write");
        assert!(short_write.roots[&3].files.is_empty());

        let mut destination_stat = FakeBackend::new();
        destination_stat.fail_destination_initial_stat = true;
        let error = handoff_batch(&mut destination_stat, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "fake-initial-stat");
        assert!(destination_stat.roots[&3].files.is_empty());
        assert!(destination_stat.roots[&4].files.is_empty());

        let mut manifest_stat = FakeBackend::new();
        manifest_stat.fail_manifest_initial_stat = true;
        let error = handoff_batch(&mut manifest_stat, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "fake-initial-stat");
        assert!(manifest_stat.roots[&3].files.is_empty());
        assert!(manifest_stat.roots[&4].files.is_empty());

        let mut aggregate = FakeBackend::new();
        aggregate.roots.get_mut(&1).unwrap().files.clear();
        aggregate.add_virtual_source(1, "first.bin", 40 * 1024 * 1024);
        aggregate.add_virtual_source(1, "second.bin", 40 * 1024 * 1024);
        aggregate.count_only_writes = true;
        let launch = PhasePlan::launch(&["first.bin".into(), "second.bin".into()]).unwrap();
        let error = handoff_batch(&mut aggregate, ProducerQuiesced::from_controller_observation(), &launch, &mut FakeBackend::launch_roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "aggregate-maximum");
        assert_eq!(aggregate.written_bytes, 40 * 1024 * 1024);
        assert!(aggregate.roots[&3].files.is_empty());

        let mut sync_expiry = FakeBackend::new();
        sync_expiry.expire_on_manifest_sync = true;
        let error = handoff_batch(&mut sync_expiry, ProducerQuiesced::from_controller_observation(), &plan, &mut FakeBackend::roots(), deadline, &NeverCancelled).unwrap_err();
        assert_eq!(error.code, "deadline-expired");
        assert!(sync_expiry.roots[&3].files.is_empty());
        assert!(sync_expiry.roots[&4].files.is_empty());
    }
}
